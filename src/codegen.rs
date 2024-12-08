use std::collections::HashMap;

use crate::grammar_ast::*;

pub fn gen_code(region: Region) -> String {
    let mut ctx = CodegenCtx::default();

    // add Buffer builtin type
    ctx.builtin_types.insert(
        "Buffer".to_string(),
        vec![
            "free".to_string(),
            "borrow".to_string(),
            "borrowMut".to_string(),
        ],
    );

    ctx.no_py_functions.push("append".to_string());

    ctx.builtin_fns.push("allocate".to_string());
    ctx.builtin_fns.push("free".to_string());

    let code = region.gen_code(&mut ctx);

    // add template header and body
    format!("{HEADER}\n\n{code}\n\n{FOOTER}")
}

#[derive(Debug, Clone, Default)]
struct CodegenCtx {
    /// list of function parameter names in the current function
    fn_params: Vec<String>,
    /// list of functions that don't require the `py` parameter
    no_py_functions: Vec<String>,
    /// list of builtin fns, that are defined in the hardcoded footer code
    builtin_fns: Vec<String>,
    /// Buffer => free/borrow/etc
    builtin_types: HashMap<String, Vec<String>>,
}

trait CodeGen {
    fn gen_code(&self, ctx: &mut CodegenCtx) -> String;
}

impl CodeGen for Region {
    fn gen_code(&self, ctx: &mut CodegenCtx) -> String {
        // separate region into functions and statements
        let (mut functions, statements) =
            self.body.iter().partition::<Vec<_>, _>(|item| match item {
                RegionItem::Function(_) => true,
                RegionItem::Statement(_) => false,
            });

        // if we have statements, put them in a dummy function
        let func;
        if !statements.is_empty() {
            let f = Function {
                name: "<Identifier, main>".to_string(),
                params: vec![],
                body: statements
                    .iter()
                    .map(|item| match item {
                        RegionItem::Function(_) => unreachable!(),
                        RegionItem::Statement(stmt) => stmt.clone(),
                    })
                    .collect(),
            };

            func = RegionItem::Function(f);
            functions.push(&func);
        }

        // render functions
        let functions_str = functions
            .iter()
            .map(|item| item.gen_code(ctx))
            .collect::<Vec<String>>()
            .join("\n");

        // render python module setup code
        let functions_registrations = functions
            .iter()
            .map(|item| match item {
                RegionItem::Function(func) => func.name.clone(),
                RegionItem::Statement(_) => unreachable!(),
            })
            .map(|name| format!("m.add_function(wrap_pyfunction!({}, m)?)?;", id(name)))
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "#[pymodule]
        fn {name}(m: &Bound<'_, PyModule>) -> PyResult<()> {{
            m.add_class::<Buffer>()?;

            {functions_registrations}

            Ok(())
        }}
        
        {functions_str}
        ",
            name = id(&self.name)
        )
    }
}

impl CodeGen for RegionItem {
    fn gen_code(&self, ctx: &mut CodegenCtx) -> String {
        match self {
            RegionItem::Function(func) => func.gen_code(ctx),
            RegionItem::Statement(statement) => statement.gen_code(ctx),
        }
    }
}

impl CodeGen for Function {
    fn gen_code(&self, ctx: &mut CodegenCtx) -> String {
        let param_names = self
            .params
            .iter()
            .map(|param| id(&param.name))
            .collect::<Vec<String>>();
        ctx.fn_params = param_names.clone();

        let params_str = param_names
            .into_iter()
            .map(|name| name + ": Py<PyAny>")
            .collect::<Vec<String>>()
            .join(", ");
        let body_str = self
            .body
            .iter()
            .map(|stmt| stmt.gen_code(ctx))
            .collect::<Vec<String>>()
            .join("\n");

        format!(
            "
            #[pyfunction]
fn {name}(py: Python<'_>, {params_str}) -> Py<PyAny> {{
    {body_str}
    return py.None();
}}",
            name = id(&self.name)
        )
    }
}

impl CodeGen for Statement {
    fn gen_code(&self, ctx: &mut CodegenCtx) -> String {
        match self {
            Statement::Noop => "".to_string(),
            Statement::Let(name, expr) => {
                let expr_str = expr.gen_code(ctx);
                format!("let mut {} = {};", id(name), any(expr_str))
            }
            Statement::Return(expr) => {
                let expr_str = expr.gen_code(ctx);
                format!("return {};", any(expr_str))
            }
            Statement::Expression(expr) => {
                let expr_str = expr.gen_code(ctx);
                format!("{};", expr_str)
            }
            Statement::Call(name, args) => {
                let args_str = format_args(id(name), args, ctx);

                format!("{}({});", id(name), args_str)
            }
            Statement::If(cond, body) => {
                let cond_str = cond.gen_code(ctx);
                let body_str = body
                    .iter()
                    .map(|stmt| stmt.gen_code(ctx))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    "if ({}).is_truthy(py).unwrap()
                 {{\n{}}}",
                    any(cond_str),
                    body_str
                )
            }
            Statement::IfElse(cond, if_body, else_body) => {
                let cond_str = cond.gen_code(ctx);
                let if_body_str = if_body
                    .iter()
                    .map(|stmt| stmt.gen_code(ctx))
                    .collect::<Vec<String>>()
                    .join("\n");
                let else_body_str = else_body
                    .iter()
                    .map(|stmt| stmt.gen_code(ctx))
                    .collect::<Vec<String>>()
                    .join("\n");
                format!(
                    "if {}.is_truthy(py).unwrap()
                     {{\n{}}} else {{\n{}}}",
                    any(cond_str),
                    if_body_str,
                    else_body_str
                )
            }
            Statement::ForLoop(init, cond, update, body) => {
                let init_str = init.gen_code(ctx);
                let cond_str = cond.gen_code(ctx);
                let update_str = update.gen_code(ctx);
                let body_str = body
                    .iter()
                    .map(|stmt| stmt.gen_code(ctx))
                    .collect::<Vec<String>>()
                    .join("\n");

                format!(
                    "{init_str}
                    while {cond_str} {{
                        {body_str}
                        {update_str}
                    }}
                    "
                )
            }
            Statement::Assignment(name, expr) => {
                let expr_str = expr.gen_code(ctx);
                format!("{} = {};", id(name), any(expr_str))
            }
        }
    }
}

impl CodeGen for Expr {
    fn gen_code(&self, ctx: &mut CodegenCtx) -> String {
        match self {
            Expr::Uninitialized => "py.None()".to_string(),
            Expr::Number(n) => n.to_string(),
            Expr::StringLiteral(s) => format!("\"{}\"", s),
            Expr::Variable(v) => id(v),
            Expr::Call(name, args) => {
                let args_str = format_args(id(name), args, ctx);

                format!("{}({})", id(name), args_str)
            }
            Expr::Array(elements) => {
                // only support empty arrays for now
                if !elements.is_empty() {
                    panic!("Array elements not supported yet");
                }

                "PyList::new(py, Vec::<Buffer>::new()).unwrap().unbind()".to_string()
            }
            Expr::Binary(lhs, op, rhs) => {
                let mut lhs_str = lhs.gen_code(ctx);
                let op_str = op.gen_code(ctx);
                let mut rhs_str = rhs.gen_code(ctx);

                // edge case: lhs and rhs need to be numbers.
                // if either of them are py types, cast them to usize.
                lhs_str = extract(&lhs_str, "usize");
                rhs_str = extract(&rhs_str, "usize");

                format!("{} {} {}", lhs_str, op_str, rhs_str)
            }
            Expr::MethodCall(obj, method_name, args) => {
                let method_name = id(method_name);
                let mut obj_str = obj.gen_code(ctx);
                let args_str = format_args(method_name.clone(), args, ctx);

                // determine whether this method is one on a builtin rust class e.g. Buffer
                let is_builtin = ctx
                    .builtin_types
                    .iter()
                    .any(|(_, methods)| methods.contains(&method_name));

                if is_builtin {
                    obj_str = extract(&obj_str, "Buffer");
                    format!("{}.{}({})", obj_str, method_name, args_str)
                } else {
                    // assume it's a python class
                    format!(
                        "{}.call_method(py, \"{}\", ({},), None).unwrap()",
                        obj_str, method_name, args_str
                    )
                }
            }
        }
    }
}

impl CodeGen for BinaryOp {
    fn gen_code(&self, _ctx: &mut CodegenCtx) -> String {
        match self {
            BinaryOp::Add => "+".to_string(),
            BinaryOp::LessThan => "<".to_string(),
        }
    }
}

/// utility function to extract identifier name from <Identifier, name>
fn id(v: impl AsRef<str>) -> String {
    // <Identifier, name> => "name"

    // get "name>"
    let name_rangle = v.as_ref().split(' ').nth(1).unwrap();

    // get "name";
    name_rangle[0..name_rangle.len() - 1].to_string()
}

/// utility function to extract a given type from a Py<PyAny>
fn extract(var: impl AsRef<str>, ty: impl AsRef<str>) -> String {
    format!("(({}).extract::<{}>(py).unwrap())", any(var), ty.as_ref())
}

/// utility function to convert type into PyAny
fn any(var: impl AsRef<str>) -> String {
    format!("({}).clone1(py).into_py_any(py).unwrap()", var.as_ref())
}

/// utility function to format function arguments when calling a function
fn format_args(fn_name: String, args: &[Box<Expr>], ctx: &mut CodegenCtx) -> String {
    let mut args = args
        .iter()
        .map(|arg| arg.gen_code(ctx))
        .collect::<Vec<String>>();

    for arg in &mut args {
        *arg = any(&arg);
    }

    if !ctx.no_py_functions.contains(&fn_name) {
        args.insert(0, "py".to_string());
    }

    args.join(", ")
}

/// the hard-coded bit of code at the top and bottom of the generated code
const HEADER: &str = "use std::sync::{Arc, RwLock};

use pyo3::exceptions::PyIndexError;
use pyo3::types::PyList;

use pyo3::{prelude::*, IntoPyObjectExt};";

const FOOTER: &str = r#"

// ====================

fn allocate(py: Python<'_>, size: Py<PyAny>) -> Buffer {
    let size = size.extract::<usize>(py).unwrap();
    Buffer::new(size)
}

fn free(py: Python<'_>, buffer: Py<PyAny>) {
    (buffer.extract::<Buffer>(py).unwrap()).free();
}

#[pyclass]
#[derive(Clone)]
struct Buffer {
    data: Arc<RwLock<Option<Vec<Byte>>>>,
}

impl Buffer {
    fn new(size: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(Some(vec![Byte::new(0); size]))),
        }
    }

    fn free(&mut self) {
        let mut data = self
            .data
            .try_write()
            .expect("Can't free buffer while borrowed");

        let Some(data_vec) = data.as_mut() else {
            // already freed, do nothing
            return;
        };

        // check no bytes are borrowed
        for byte in data_vec.iter() {
            if byte.borrowed {
                panic!("Can't free buffer while borrowed");
            }
        }

        *data = None;
    }

    fn borrow(&self, py: Python<'_>, size: Py<PyAny>, index: Py<PyAny>) -> Py<PyList> {
        let size = size.extract::<usize>(py).unwrap();
        let index = index.extract::<usize>(py).unwrap();

        let mut data = self.data.write().unwrap();
        let data = data.as_mut().unwrap();

        let mut borrowed_data = vec![];
        for i in index..index + size {
            data[i].borrow();
            borrowed_data.push(data[i].data);
        }

        PyList::new(py, borrowed_data).unwrap().into()
    }

    fn borrowMut(&self, py: Python<'_>, size: Py<PyAny>, index: Py<PyAny>) -> Py<PyList> {
        let size = size.extract::<usize>(py).unwrap();
        let index = index.extract::<usize>(py).unwrap();

        let mut data = self.data.write().unwrap();
        let data = data.as_mut().unwrap();

        let mut borrowed_data = vec![];
        for i in index..index + size {
            if data[i].borrowed {
                panic!("Can't borrow mutably while borrowed");
            }

            data[i].borrow();
            borrowed_data.push(data[i].data);
        }

        PyList::new(py, borrowed_data).unwrap().into()
    }
}

#[pymethods]
impl Buffer {
    fn __getitem__(&self, index: isize) -> PyResult<u8> {
        let guard = self.data.read().unwrap();
        let data = guard.as_ref().unwrap();

        if index < 0 || index as usize >= data.len() {
            return Err(PyErr::new::<PyIndexError, _>("Index out of bounds"));
        }

        Ok(data[index as usize].data)
    }

    fn __setitem__(&mut self, index: isize, value: u8) -> PyResult<()> {
        let mut guard = self.data.write().unwrap();
        let data = guard.as_mut().unwrap();

        if index < 0 || index as usize >= data.len() {
            return Err(PyErr::new::<PyIndexError, _>("Index out of bounds"));
        }

        let byte = &mut data[index as usize];
        if byte.borrowed {
            return Err(PyErr::new::<PyIndexError, _>(
                "Can't mutate data while borrowed",
            ));
        }

        byte.data = value;
        Ok(())
    }

    fn __repr__(&self) -> PyResult<String> {
        let Ok(guard) = self.data.try_read() else {
            return Ok("Buffer(<mutably borrowed>)".to_string());
        };

        let data = guard.as_ref().unwrap();
        let hex = data
            .iter()
            .map(|byte| format!("{:02x}", byte.data))
            .collect::<Vec<String>>()
            .join(" ");

        Ok(format!("Buffer({hex})"))
    }
}

#[derive(Debug, Clone, Copy)]
struct Byte {
    data: u8,
    borrowed: bool,
}

impl Byte {
    fn new(data: u8) -> Self {
        Self {
            data,
            borrowed: false,
        }
    }

    fn borrow(&mut self) {
        self.borrowed = true;
    }

    fn release(&mut self) {
        self.borrowed = false;
    }
}

impl AsRef<u8> for Byte {
    fn as_ref(&self) -> &u8 {
        &self.data
    }
}

impl AsMut<u8> for Byte {
    fn as_mut(&mut self) -> &mut u8 {
        if self.borrowed {
            panic!("Can't mutate data while borrowed");
        }

        &mut self.data
    }
}

pub trait Clone1 {
    fn clone1(&self, py: Python<'_>) -> Self;
}

impl<T> Clone1 for Py<T> {
    fn clone1(&self, py: Python<'_>) -> Self {
        self.clone_ref(py)
    }
}

impl Clone1 for Buffer {
    fn clone1(&self, py: Python<'_>) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl Clone1 for usize {
    fn clone1(&self, _: Python<'_>) -> Self {
        *self
    }
}
"#;
