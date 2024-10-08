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

const KEYWORDS: &'static [&'static str] = &["region", "let"];
const OPERATORS: &'static [&'static str] = &["+", "-", "*", "/", "="];

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
        Tokeniser::parse(&file_path, &code_without_comments).expect("Error parsing tokens");

    println!("\n3. Tokens:");
    for token in &tokens {
        print!("{:?} ", token);
        if matches!(token, Token::Semi | Token::Lcur | Token::Rcur) {
            println!();
        }
    }
    println!();
}
impl Token {
    fn parse(s: &str, line_number: usize, line_content: &str) -> Result<Vec<Token>, String> {
        if s.is_empty() {
            return Ok(vec![]);
        }

        // Handle special characters like parentheses, commas, braces, and square brackets
        for (special_char, token) in [
            ("{", Token::Lcur),
            ("}", Token::Rcur),
            ("(", Token::Lpar),
            (")", Token::Rpar),
            (";", Token::Semi),
            (",", Token::Comma),
            ("[", Token::Lbrack), // Left square bracket [
            ("]", Token::Rbrack), // Right square bracket ]
                                  // ("<", Token::Operator("<".to_string())), // Comparison operator <
                                  // (">", Token::Operator(">".to_string())), // Comparison operator >
                                  // (".", Token::Operator(".".to_string())), // Handle the dot (.)
        ] {
            if s == special_char {
                return Ok(vec![token]);
            }

            let index_of_special_char = s.find(special_char);
            if let Some(index) = index_of_special_char {
                let mut tokens = vec![];
                tokens.extend(Token::parse(&s[..index], line_number, line_content)?);
                tokens.push(token);
                tokens.extend(Token::parse(&s[index + 1..], line_number, line_content)?);
                return Ok(tokens);
            }
        }

        // Handle keywords
        if KEYWORDS.contains(&s) {
            // return Ok(vec![Token::Keyword(s.to_string())]);
        }

        // Handle operators
        if OPERATORS.contains(&s) {
            // return Ok(vec![Token::Operator(s.to_string())]);
        }

        // Handle numbers
        if s.parse::<i128>().is_ok() || s.parse::<f64>().is_ok() {
            return Ok(vec![Token::Number(s.to_string())]);
        }

        // Handle identifiers (like function names, variable names, etc.)
        if s.chars().all(char::is_alphabetic) {
            return Ok(vec![Token::Identifier(s.to_string())]);
        }

        // If the token cannot be parsed, return an error
        Err(format!(
            "Unparseable token: '{}' on line {}: '{}'",
            s, line_number, line_content
        ))
    }
}
