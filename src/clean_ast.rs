use crate::grammar_ast::*;

pub fn clean_ast(region: &mut Region) {
    for item in &mut region.body {
        match item {
            RegionItem::Function(function) => {
                // do 3 passes
                for _ in 0..3 {
                    clean_function(function);
                }
            }
            RegionItem::Statement(_) => {}
        }
    }
}

fn clean_function(function: &mut Function) {
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
}

fn exprs_in_statment(statement: &Statement) -> Vec<&Box<Expr>> {
    match statement {
        Statement::Noop => vec![],
        Statement::Let(_, expr) => vec![expr],
        Statement::Return(expr) => vec![expr],
        Statement::Expression(expr) => vec![expr],
        Statement::Call(_, args) => args.iter().collect(),
        Statement::If(expr, statements) => {
            let mut exprs = vec![expr];
            for statement in statements {
                exprs.extend(exprs_in_statment(statement));
            }
            exprs
        }
        Statement::IfElse(expr, if_statements, else_statements) => {
            let mut exprs = vec![expr];
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
        Statement::Assignment(_, expr) => vec![expr],
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
                (Expr::Number(lhs), Expr::Number(rhs)) => {
                    *expr = Expr::Number(match op {
                        BinaryOp::Add => *lhs + *rhs,
                        BinaryOp::LessThan => (lhs < rhs) as i32,
                    });
                }
                _ => {}
            }
        }

        _ => {}
    }
}
