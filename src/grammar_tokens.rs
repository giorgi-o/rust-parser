// src/token.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Region,
    Function,
    // Add other keywords if needed
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String),
    Lcur,      // `{`
    Rcur,      // `}`
    Lpar,      // `(`
    Rpar,      // `)`
    Semi,      // `;`
    Comma,     // `,`
    Operator(String),
    Number(String),
}
