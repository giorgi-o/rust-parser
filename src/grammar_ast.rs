#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<Variable>, // Function parameters
    pub body: Vec<Statement>,  // Function body consisting of statements
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub var_type: Type, // Variable type (Int, String, etc.)
}

#[derive(Debug)]
pub enum Statement {
    Let(String, Expr),                   // let buffer = allocateMemory(size);
    Return(Expr),                        // return buffer;
    Call(String, Vec<Expr>),             // Function call
    If(Expr, Vec<Statement>),            // if condition
    ForLoop(Expr, Expr, Vec<Statement>), // For loop (simplified for now)
    Empty,
    
}

#[derive(Debug)]
pub enum Expr {
    Number(i32),             // Integer literals
    StringLiteral(String),   // String literals
    Variable(String),        // Variable references
    Call(String, Vec<Expr>), // Function call as an expression
}

#[derive(Debug)]
pub enum Type {
    Int32,      // Integer type
    StringType, // String type
    Bool,       // Boolean type
}