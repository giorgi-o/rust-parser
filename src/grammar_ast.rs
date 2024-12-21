#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    pub body: Vec<RegionItem>,
}

#[derive(Debug, Clone)]
pub enum RegionItem {
    Function(Function),
    Statement(Statement),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Variable>, // Function parameters
    pub body: Vec<Statement>,  // Function body consisting of statements
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub var_type: Type, // Variable type (Int, String, etc.)
}

#[derive(Debug, Clone)]
pub enum Statement {
    Noop,
    Let(String, Box<Expr>),
    Return(Box<Expr>),
    Expression(Box<Expr>),
    Call(String, Vec<Box<Expr>>),
    If(Box<Expr>, Vec<Statement>),
    IfElse(Box<Expr>, Vec<Statement>, Vec<Statement>), // Added
    ForLoop(Box<Statement>, Box<Expr>, Box<Statement>, Vec<Statement>),
    Assignment(String, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Uninitialized, // Added
    Number(i32),
    StringLiteral(String),
    Variable(String),
    Call(String, Vec<Box<Expr>>),
    Array(Vec<Box<Expr>>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    MethodCall(Box<Expr>, String, Vec<Box<Expr>>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Mult,
    LessThan,
    // Add other operators as needed
}

#[derive(Debug, Clone)]
pub enum Type {
    Int32,      // Integer type
    StringType, // String type
    Bool,       // Boolean type
}
