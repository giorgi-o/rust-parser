use std::collections::{HashMap, HashSet};

use crate::grammar_ast::*;

pub fn clean_ast(region: &mut Region) {
    move_toplevel_statements_to_function(region);

    for item in &mut region.body {
        match item {
            RegionItem::Function(function) => {
                // do 3 passes
                for _ in 0..3 {
                    clean_function(function);
                }
            }

            // all toplevel statements were put in a function by
            // move_toplevel_statements_to_function()
            RegionItem::Statement(s) => unreachable!(),
        }
    }
}

fn move_toplevel_statements_to_function(region: &mut Region) {
    let mut toplevel_statements = vec![];

    // take toplevel statements out of region body and put them in vec above
    region.body = region
        .body
        .clone()
        .into_iter()
        .filter_map(|item| match item {
            RegionItem::Function(function) => Some(RegionItem::Function(function)),
            RegionItem::Statement(s) => {
                toplevel_statements.push(s);
                None
            }
        })
        .collect();

    if toplevel_statements.is_empty() {
        return; // no top-level statements, nothing to do
    }

    // create a main function for those toplevel statements
    let toplevel = Function {
        name: "<Identifier, main>".to_string(),
        params: vec![],
        body: toplevel_statements.clone(),
    };
    region.body.push(RegionItem::Function(toplevel));
}

fn clean_function(function: &mut Function) {
    // add "return none" to end of function (will be removed later if not needed)
    let return_none = Statement::Return(Box::new(Expr::Uninitialized));
    function.body.push(return_none);

    // eliminate unreachable code
    unreachable_code_elimination(&mut function.body);

    // simplify all statements in function
    for statement in &mut function.body {
        simplify_statement(statement);
    }

    let mut declared = function
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>();
    let mut used = vec![];

    // get all variables declared and used
    for statement in &function.body {
        declared.extend(variables_declared(statement));

        for expr in exprs_in_statment(statement) {
            used.extend(variables_used(expr));
        }
    }

    // get used but undeclared variables
    let undeclared = used
        .iter()
        .filter(|var| !declared.contains(var))
        .map(|var| var.to_string())
        .collect::<Vec<_>>();
    if !undeclared.is_empty() {
        for var in undeclared {
            eprintln!("Error: undeclared variable `{}` used", var);
        }
        std::process::exit(1);
    }

    // get unused variables
    let unused = declared
        .iter()
        .filter(|var| !used.contains(var))
        .map(|var| var.to_string())
        .collect::<Vec<_>>();

    // replace unused variables declarations and assignments with just the rhs
    for statement in &mut function.body {
        match statement {
            Statement::Let(name, expr) | Statement::Assignment(name, expr) => {
                if !unused.contains(name) {
                    continue;
                }

                // remove assignment and keep rhs
                *statement = Statement::Expression(expr.clone());
            }

            _ => {}
        }
    }

    // do cse
    eliminate_common_subexpressions(&mut function.body, SubexprGraph::default());

    // move loop invariant expressions outside of loop
    loop_invariant_motion(&mut function.body);

}

fn expr_and_nested_exprs(expr: &Box<Expr>) -> Vec<&Box<Expr>> {
    let mut exprs = vec![expr];
    match expr.as_ref() {
        Expr::Uninitialized => {}
        Expr::Number(_) => {}
        Expr::StringLiteral(_) => {}
        Expr::Variable(_) => {}
        Expr::Call(_, args) => {
            for arg in args {
                exprs.extend(expr_and_nested_exprs(arg));
            }
        }
        Expr::Array(items) => {
            for item in items {
                exprs.extend(expr_and_nested_exprs(item));
            }
        }
        Expr::Binary(lhs, _, rhs) => {
            exprs.extend(expr_and_nested_exprs(lhs));
            exprs.extend(expr_and_nested_exprs(rhs));
        }
        Expr::MethodCall(expr, _, args) => {
            exprs.extend(expr_and_nested_exprs(expr));
            for arg in args {
                exprs.extend(expr_and_nested_exprs(arg));
            }
        }
    }
    exprs
}

fn exprs_in_statment(statement: &Statement) -> Vec<&Box<Expr>> {
    match statement {
        Statement::Noop => vec![],
        Statement::Let(_, expr) => expr_and_nested_exprs(expr),
        Statement::Return(expr) => expr_and_nested_exprs(expr),
        Statement::Expression(expr) => expr_and_nested_exprs(expr),
        Statement::Call(_, args) => args.iter().flat_map(expr_and_nested_exprs).collect(),
        Statement::If(expr, statements) => {
            let mut exprs = expr_and_nested_exprs(expr);
            for statement in statements {
                exprs.extend(exprs_in_statment(statement));
            }
            exprs
        }
        Statement::IfElse(expr, if_statements, else_statements) => {
            let mut exprs = expr_and_nested_exprs(expr);
            for statement in if_statements {
                exprs.extend(exprs_in_statment(statement));
            }
            for statement in else_statements {
                exprs.extend(exprs_in_statment(statement));
            }
            exprs
        }
        Statement::ForLoop(init, cond, update, statements) => {
            let mut exprs = vec![cond];
            for statement in [init, update] {
                exprs.extend(exprs_in_statment(statement));
            }
            for statement in statements {
                exprs.extend(exprs_in_statment(statement));
            }
            exprs
        }
        Statement::Assignment(_, expr) => expr_and_nested_exprs(expr),
    }
}

fn variables_declared(statement: &Statement) -> Vec<&str> {
    match statement {
        Statement::Noop => vec![],
        Statement::Let(name, _) => vec![name.as_str()],
        Statement::Return(_) => vec![],
        Statement::Expression(_) => vec![],
        Statement::Call(_, _) => vec![],
        Statement::If(_, statements) => statements.iter().flat_map(variables_declared).collect(),
        Statement::IfElse(_, if_statements, else_statements) => {
            let mut vars = vec![];
            for statement in if_statements {
                vars.extend(variables_declared(statement));
            }
            for statement in else_statements {
                vars.extend(variables_declared(statement));
            }
            vars
        }
        Statement::ForLoop(init, _, update, statements) => {
            let mut vars = variables_declared(init);
            for statement in statements {
                vars.extend(variables_declared(statement));
            }
            vars.extend(variables_declared(update));
            vars
        }
        Statement::Assignment(name, _) => vec![name.as_str()],
    }
}

fn variables_modified(statement: &Statement) -> Vec<&str> {
    match statement {
        Statement::Noop => vec![],
        Statement::Return(_) => vec![],
        Statement::Expression(_) => vec![],
        Statement::Call(_, _) => vec![],
        Statement::If(_, statements) => statements.iter().flat_map(variables_modified).collect(),
        Statement::IfElse(_, if_statements, else_statements) => {
            let mut vars = vec![];
            for statement in if_statements {
                vars.extend(variables_modified(statement));
            }
            for statement in else_statements {
                vars.extend(variables_modified(statement));
            }
            vars
        }
        Statement::ForLoop(init, _, update, statements) => {
            let mut vars = variables_modified(init);
            for statement in statements {
                vars.extend(variables_modified(statement));
            }
            vars.extend(variables_modified(update));
            vars
        }
        Statement::Assignment(name, _) | Statement::Let(name, _) => vec![name.as_str()],
    }
}

/// traverse a statement and return a list of variables used
fn variables_used(expr: &Expr) -> Vec<&str> {
    match expr {
        Expr::Uninitialized => vec![],
        Expr::Number(_) => vec![],
        Expr::StringLiteral(_) => vec![],
        Expr::Variable(name) => vec![name.as_str()],
        Expr::Call(_, args) => args.iter().flat_map(|arg| variables_used(arg)).collect(),
        Expr::Array(items) => items.iter().flat_map(|item| variables_used(item)).collect(),
        Expr::Binary(lhs, _, rhs) => {
            let mut vars = variables_used(lhs);
            vars.extend(variables_used(rhs));
            vars
        }
        Expr::MethodCall(expr, _, args) => {
            let mut vars = variables_used(expr);
            vars.extend(args.iter().flat_map(|arg| variables_used(arg)));
            vars
        }
    }
}

fn simplify_statement(statement: &mut Statement) {
    match statement {
        Statement::Noop => {}
        Statement::Let(_, expr) | Statement::Return(expr) => {
            simplify_expression(expr);
        }
        Statement::Expression(expr) => {
            simplify_expression(expr);

            // if expression doesn't do anything, remove it
            match expr.as_ref() {
                Expr::Number(_) | Expr::StringLiteral(_) | Expr::Variable(_) => {
                    *statement = Statement::Noop;
                }
                _ => {}
            }
        }
        Statement::Call(_, args) => {
            for arg in args {
                simplify_expression(arg);
            }
        }
        Statement::If(expr, statements) => {
            simplify_expression(expr);
            for statement in statements {
                simplify_statement(statement);
            }
        }
        Statement::IfElse(expr, if_statements, else_statements) => {
            simplify_expression(expr);
            for statement in if_statements {
                simplify_statement(statement);
            }
            for statement in else_statements {
                simplify_statement(statement);
            }
        }
        Statement::ForLoop(init, cond, update, statements) => {
            simplify_statement(init);
            simplify_expression(cond);
            simplify_statement(update);
            for statement in statements {
                simplify_statement(statement);
            }
        }
        Statement::Assignment(_, expr) => {
            simplify_expression(expr);
        }
    }
}

fn simplify_expression(expr: &mut Expr) {
    match expr {
        Expr::Binary(rhs, op, lhs) => {
            simplify_expression(lhs);
            simplify_expression(rhs);

            match (lhs.as_ref(), rhs.as_ref()) {
                // constant folding
                (Expr::Number(lhs), Expr::Number(rhs)) => {
                    *expr = Expr::Number(match op {
                        BinaryOp::Add => *lhs + *rhs,
                        BinaryOp::Mult => *lhs * *rhs,
                        BinaryOp::LessThan => (lhs < rhs) as i32,
                    });
                }

                // algebraic simplification: x + 0
                (Expr::Number(0), _) if *op == BinaryOp::Add => {
                    *expr = (**rhs).clone();
                }
                (_, Expr::Number(0)) if *op == BinaryOp::Add => {
                    *expr = (**lhs).clone();
                }

                // algebraic simplification: x * 0
                (Expr::Number(0), _) | (_, Expr::Number(0)) if *op == BinaryOp::Mult => {
                    *expr = Expr::Number(0);
                }

                // algebraic simplification: x * 1
                (Expr::Number(1), _) if *op == BinaryOp::Mult => {
                    *expr = (**rhs).clone();
                }
                (_, Expr::Number(1)) if *op == BinaryOp::Mult => {
                    *expr = (**lhs).clone();
                }

                _ => {}
            }
        }

        Expr::Call(_, args) => {
            for arg in args {
                simplify_expression(arg);
            }
        }
        Expr::Array(items) => {
            for item in items {
                simplify_expression(item);
            }
        }
        Expr::MethodCall(expr, _, args) => {
            simplify_expression(expr);
            for arg in args {
                simplify_expression(arg);
            }
        }

        _ => {}
    }
}

#[derive(Debug, Clone, Default)]
struct SubexprGraph {
    subexprs: Vec<(String, Expr)>,
}

impl SubexprGraph {
    fn expr_is_repeated(&self, expr: &Expr) -> Option<String> {
        for (name, e) in &self.subexprs {
            if e == expr {
                return Some(name.clone());
            }
        }
        None
    }

    fn variable_modified(&mut self, var: impl Into<String>, new_expr: Expr) {
        let var: String = var.into();

        self.subexprs.retain(|(name, expr)| {
            // remove the variable itself
            if name == &var {
                return false;
            }

            // remove all expressions that depend on the variable
            !variables_used(expr).contains(&var.as_ref())
        });

        self.subexprs.push((var, new_expr));
    }
}

fn eliminate_common_subexpressions(body: &mut [Statement], mut subexprs: SubexprGraph) {
    let Some(head) = body.first_mut() else {
        return;
    };

    let replace_if_repeated = |expr: &mut Expr| {
        if let Some(var) = subexprs.expr_is_repeated(expr) {
            *expr = Expr::Variable(var);
        }
    };

    match head {
        Statement::Let(name, expr) | Statement::Assignment(name, expr) => {
            let name = name.clone();

            replace_if_repeated(expr);
            subexprs.variable_modified(name, expr.as_ref().clone());
        }
        Statement::Return(expr) | Statement::Expression(expr) => {
            replace_if_repeated(expr);
        }
        Statement::Call(_, args) => {
            for arg in args {
                replace_if_repeated(arg);
            }
        }
        Statement::If(expr, statements) => {
            replace_if_repeated(expr);
            eliminate_common_subexpressions(statements, subexprs.clone());
        }
        Statement::IfElse(expr, if_statements, else_statements) => {
            replace_if_repeated(expr);
            eliminate_common_subexpressions(if_statements, subexprs.clone());
            eliminate_common_subexpressions(else_statements, subexprs.clone());
        }
        Statement::ForLoop(init, cond, update, statements) => {
            replace_if_repeated(cond);

            let mut loop_body = vec![init.as_ref().clone(), update.as_ref().clone()];
            loop_body.extend(statements.iter().cloned());

            eliminate_common_subexpressions(&mut loop_body, subexprs.clone());

            *statements = loop_body[2..].to_vec();
        }
        Statement::Noop => {}
    }

    eliminate_common_subexpressions(&mut body[1..], subexprs);
}

fn loop_invariant_motion(body: &mut Vec<Statement>) {
    let mut new_body = vec![];

    for mut fn_statement in body.clone() {
        match &mut fn_statement {
            Statement::ForLoop(init, _cond, update, loop_statements) => {
                let exprs_in_loop = loop_statements
                    .iter()
                    .flat_map(exprs_in_statment)
                    .collect::<Vec<_>>();

                let modified_variables = variables_modified(&init)
                    .into_iter()
                    .chain(loop_statements.iter().flat_map(variables_modified))
                    .chain(variables_modified(update))
                    .collect::<Vec<_>>();

                let mut invariant_exprs = HashSet::new();
                for expr in exprs_in_loop {
                    if let Expr::Variable(_) | Expr::Number(_) | Expr::StringLiteral(_) =
                        expr.as_ref()
                    {
                        continue;
                    }

                    let vars_used_in_expr = variables_used(expr);
                    if vars_used_in_expr
                        .iter()
                        .all(|var| !modified_variables.contains(var))
                    {
                        invariant_exprs.insert(expr);
                    }
                }

                // invariant expressions are detected, now create a temp
                // variable for them and replace their uses
                let temp_vars = invariant_exprs
                    .into_iter()
                    .enumerate()
                    .map(|(i, expr)| {
                        let temp_var = format!("<Identifier, __temp_{}>", i);
                        (expr.as_ref().clone(), temp_var)
                    })
                    .collect::<HashMap<_, _>>();

                for loop_statement in loop_statements {
                    run_on_all_exprs(loop_statement, |expr| {
                        if let Some(temp_var) = temp_vars.get(expr) {
                            *expr = Expr::Variable(temp_var.clone());
                        }
                    });
                }

                // add temp variables before the loop
                for (expr, temp_var) in temp_vars {
                    new_body.push(Statement::Let(temp_var, Box::new(expr)));
                }
                new_body.push(fn_statement);
            }

            Statement::If(_, statements) | Statement::IfElse(_, statements, _) => {
                loop_invariant_motion(statements);
                new_body.push(fn_statement);
            }

            _ => {
                new_body.push(fn_statement);
            }
        }
    }

    *body = new_body;
}

fn run_on_all_exprs<F>(statement: &mut Statement, f: F)
where
    F: FnMut(&mut Expr) + Copy,
{
    match statement {
        Statement::Noop => {}
        Statement::Let(_, expr)
        | Statement::Return(expr)
        | Statement::Expression(expr)
        | Statement::Assignment(_, expr) => {
            run_on_expr_and_nested(expr, f);
        }
        Statement::Call(_, args) => {
            for arg in args {
                run_on_expr_and_nested(arg, f);
            }
        }
        Statement::If(expr, statements) => {
            run_on_expr_and_nested(expr, f);
            for statement in statements {
                run_on_all_exprs(statement, f);
            }
        }
        Statement::IfElse(expr, if_statements, else_statements) => {
            run_on_expr_and_nested(expr, f);
            for statement in if_statements {
                run_on_all_exprs(statement, f);
            }
            for statement in else_statements {
                run_on_all_exprs(statement, f);
            }
        }
        Statement::ForLoop(init, cond, update, statements) => {
            run_on_all_exprs(init, f);
            run_on_expr_and_nested(cond, f);
            for statement in statements {
                run_on_all_exprs(statement, f);
            }
            run_on_all_exprs(update, f);
        }
    }
}

fn run_on_expr_and_nested<F>(expr: &mut Expr, mut f: F)
where
    F: FnMut(&mut Expr) + Copy,
{
    f(expr);

    match expr {
        Expr::Uninitialized => {}
        Expr::Number(_) => {}
        Expr::StringLiteral(_) => {}
        Expr::Variable(_) => {}
        Expr::Call(_, args) => {
            for arg in args {
                run_on_expr_and_nested(arg, f);
            }
        }
        Expr::Array(items) => {
            for item in items {
                run_on_expr_and_nested(item, f);
            }
        }
        Expr::Binary(lhs, _, rhs) => {
            run_on_expr_and_nested(lhs, f);
            run_on_expr_and_nested(rhs, f);
        }
        Expr::MethodCall(expr, _, args) => {
            run_on_expr_and_nested(expr, f);
            for arg in args {
                run_on_expr_and_nested(arg, f);
            }
        }
    }
}

fn unreachable_code_elimination(body: &mut Vec<Statement>) {
    let mut new_body = vec![];

    for statement in std::mem::take(body) {
        let returns = statement_returns(&statement);
        
        new_body.push(statement);

        if returns {
            break;
        }
    }

    *body = new_body;
}

fn statement_returns(statement: &Statement) -> bool {
    match statement {
        Statement::Noop => false,
        Statement::Let(_, _) => false,
        Statement::Return(_) => true,
        Statement::Expression(_) => false,
        Statement::Call(_, _) => false,
        Statement::If(_, _) => false,
        Statement::IfElse(_, if_statements, else_statements) => {
            if_statements.iter().any(statement_returns)
                && else_statements.iter().any(statement_returns)
        }
        Statement::ForLoop(_, _, _, statements) => false,
        Statement::Assignment(_, _) => false,
    }
}