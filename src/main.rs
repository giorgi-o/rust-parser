pub mod token_fsm;

use std::env;
use std::fs;

use strum::AsRefStr;
use strum::Display;
use strum::EnumIter;
use token_fsm::Tokeniser;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String), // [a-zA-Z][a-zA-Z0-9]*
    Number(String),     // -?[0-9]+(.[0-9]+)?
    Keyword(Keyword),   // region, let, function, return, if, for
    Operator(Operator), // +, -, *, /, =, <, >, <=, >=
    Lcur,               // Left curly brace {
    Rcur,               // Right curly brace }
    Lpar,               // Left parenthesis (
    Rpar,               // Right parenthesis )
    Semi,               // Semicolon ;
    Comma,              // Comma ,
    Dot,                // Dot .
    Lbrack,             // Left square bracket [
    Rbrack,             // Right square bracket ]
}

#[derive(AsRefStr, Display, EnumIter, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    /// `region`
    Region,
    /// `let`
    Let,
    /// `function`
    Function,
    /// `return`
    Return,
    /// `if`
    If,
    /// `for`
    For,
}

#[derive(AsRefStr, Display, EnumIter, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    /// `+`
    #[strum(serialize = "+")]
    Plus,
    /// `-`
    #[strum(serialize = "-")]
    Minus,
    /// `*`
    #[strum(serialize = "*")]
    Mult,
    /// `/`
    #[strum(serialize = "/")]
    Div,
    /// `=`
    #[strum(serialize = "=")]
    Assign,
    /// `<`
    #[strum(serialize = "<")]
    LessThan,
    /// `>`
    #[strum(serialize = ">")]
    GreaterThan,
    /// `<=`
    #[strum(serialize = "<=")]
    LessThanOrEqual,
    /// `>=`
    #[strum(serialize = ">=")]
    GreaterThanOrEqual,
}

#[derive(Debug)]
enum SyntaxNode {
    Region {
        name: String,
        body: Vec<SyntaxNode>,
    },
    Assignment {
        assignee: Token,
        assigner: Box<SyntaxNode>,
    },
    FunctionDefinition {
        name: Token,
        parameters: Vec<Token>,
        body: Vec<SyntaxNode>,
    }, // Changed
    FunctionCall {
        name: Token,
        arguments: Vec<Box<SyntaxNode>>,
    }, // Changed
    Value(Token),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let file_path = args
        .get(1)
        .cloned()
        .unwrap_or("src/source_code.txt".to_string());

    let source_code = fs::read_to_string(&file_path);
    let mut source_code_string: Option<String> = None;

    match source_code {
        Ok(source_code_str) => {
            println!("Source code: \n{source_code_str}");
            source_code_string = Some(source_code_str);
        }
        Err(error) => {
            eprintln!("Error reading from file:\n{error}");
            std::process::exit(1);
        }
    }

    let mut code_lines_without_comments = vec![];
    if let Some(ref code_with_comments) = source_code_string {
        for line in code_with_comments.lines() {
            let line_without_comments = line.split("//").next().unwrap();
            code_lines_without_comments.push(line_without_comments);
        }
    } else {
        eprintln!("No source code in file :)");
        std::process::exit(1);
    }

    let code_without_comments = code_lines_without_comments.join("\n");

    println!("1. Code without comments:");
    for line in &code_lines_without_comments {
        println!("{line}");
    }

    let tokens =
        Tokeniser::tokenise(&file_path, &code_without_comments).expect("Error parsing tokens");

    println!("\n3. Tokens:");
    for token in &tokens {
        print!("{:?} ", token);
        if matches!(token, Token::Semi | Token::Lcur | Token::Rcur) {
            println!();
        }
    }
    println!();
}
