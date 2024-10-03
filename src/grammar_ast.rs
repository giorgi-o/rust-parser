#[derive(Debug)]
pub struct Region {
    pub name: String,
    pub functions: Vec<Function>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
}


// #[derive(Debug)]
// pub struct Region {
//     pub name: String,
//     pub functions: Vec<Function>,
// }

// #[derive(Debug)]
// pub struct Function {
//     pub name: String,
//     pub params: Vec<Variable>,   // Parameters as variables
//     pub body: Vec<Expr>,         // Function body as a list of expressions
// }

// #[derive(Debug)]
// pub enum Type {
//     Int32,
//     Bool,
//     StringType,  // To avoid conflict with the Rust "String" type
// }

// #[derive(Debug)]
// pub enum Expr {
//     Number(i32),                          // For integer literals
//     Bool(bool),                           // For boolean literals
//     StringLiteral(String),                // For string literals
//     Variable(String),                     // Reference to a variable
//     BinaryOp(Box<Expr>, Opcode, Box<Expr>), // Binary operations like a + b
//     Call(String, Vec<Expr>),              // Function call (name and arguments)
// }

// #[derive(Debug)]
// pub struct Variable {
//     pub name: String,
//     pub var_type: Type,
// }


// #[derive(Debug)]
// pub enum Opcode {
//     Add,
//     Sub,
//     Mul,
//     Div,
// }




