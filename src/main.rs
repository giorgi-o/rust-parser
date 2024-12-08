pub mod codegen;
mod grammar_ast;
mod clean_ast;
use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);

use grammar::RegionParser;

pub mod token_fsm;

use std::env;
use std::fs;

use std::process::Command;
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

impl Token {
    /// Print the token in the format required by programming assignment 1
    /// <Token Type, Token Value>
    pub fn fmt_type_and_value(&self) -> String {
        match self {
            Token::Identifier(value) => format!("<Identifier, {}>", value),
            Token::Number(value) => format!("<Number, {}>", value),
            Token::Keyword(value) => format!("<Keyword, {}>", value.as_ref()),
            Token::Operator(value) => format!("<Operator, {}>", value.as_ref()),
            Token::Lcur => format!("<Lcur, {{>"),
            Token::Rcur => format!("<Rcur, }}>"),
            Token::Lpar => format!("<Lpar, (>"),
            Token::Rpar => format!("<Rpar, )>"),
            Token::Semi => format!("<Semi, ;>"),
            Token::Comma => format!("<Comma, ,>"),
            Token::Dot => format!("<Dot, .>"),
            Token::Lbrack => format!("<Lbrack, [>"),
            Token::Rbrack => format!("<Rbrack, ]>"),
        }
    }
}

#[derive(AsRefStr, Display, EnumIter, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    #[strum(serialize = "region")]
    Region,

    #[strum(serialize = "let")]
    Let,

    #[strum(serialize = "function")]
    Function,

    #[strum(serialize = "return")]
    Return,

    #[strum(serialize = "if")]
    If,

    #[strum(serialize = "else")]
    Else,

    #[strum(serialize = "for")]
    For,
}

#[derive(AsRefStr, Display, EnumIter, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    #[strum(serialize = "+")]
    Plus,

    #[strum(serialize = "-")]
    Minus,

    #[strum(serialize = "*")]
    Mult,

    #[strum(serialize = "/")]
    Div,

    #[strum(serialize = "=")]
    Assign,

    #[strum(serialize = "<")]
    LessThan,

    #[strum(serialize = ">")]
    GreaterThan,

    #[strum(serialize = "<=")]
    LessThanOrEqual,

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
        .unwrap_or("example_input_source_code\\full.txt".to_string());

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

    let tokens_result = Tokeniser::tokenise(&file_path, &code_without_comments);

    let tokens = match tokens_result {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!("\n{error}");
            std::process::exit(1);
        }
    };

    println!("\n3. Tokens:");
    for token in &tokens {
        print!("{} ", token.fmt_type_and_value());
    }

    let mut serialized_tokens = String::new();
    println!("\n3.5. Serialized Tokens:"); // Updated label for clarity
    for token in &tokens {
        // Here we get the formatted string for each token
        let token_str = token.fmt_type_and_value();
        serialized_tokens.push_str(&token_str);
        serialized_tokens.push(' '); // Add a space between tokens
    }

    // Trim any trailing whitespace for clean output
    serialized_tokens = serialized_tokens.trim_end().to_string();

    // Print the serialized tokens to verify correctness
    println!("Serialized Tokens: {}", serialized_tokens);

    println!("\n4. SerializedTokens:");
    println!("Serialized Tokens: {:?}", serialized_tokens);

    let tokens_str = serialized_tokens.clone();
    let mut region = match RegionParser::new().parse(&tokens_str) {
        Ok(region) => region,
        Err(e) => {
            handle_ast_error(serialized_tokens, e);
            return;
        }
    };

    println!("Parsed AST: {:#?}", region);

    println!("\n5. Generated code:");

    clean_ast::clean_ast(&mut region);
    let code = codegen::gen_code(region);
    println!("\n{code}");

    // write code to output.rs and pybind_test/lib.rs
    for outfile in ["output.rs", "pybind_test/src/lib.rs"] {
        fs::write(outfile, code.clone()).unwrap();

        // run rustfmt on output.rs
        println!("\nFormatting output.rs with rustfmt...");
        let output = Command::new("rustfmt")
            .arg(outfile)
            .output()
            .expect("Failed to run rustfmt");

        if !output.status.success() {
            eprintln!("rustfmt failed:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        } else {
            println!("rustfmt completed successfully.");
        }
    }

    // run cargo fix on pybind_test (swallow warnings in autogenerated code)
    Command::new("cargo")
        .args([
            "fix",
            "--lib",
            "--allow-dirty",
            "--allow-staged",
        ])
        .current_dir("pybind_test")
        .output()
        .expect("Failed to run cargo fix");

    println!();
}

fn handle_ast_error(
    serialized_tokens: String,
    e: lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token, &str>,
) {
    println!("Error parsing: {:?}", e);
    println!("Full error: {:?}", e);

    if let lalrpop_util::ParseError::InvalidToken { location } = e {
        println!("Invalid token at character location: {}", location);

        // Split `serialized_tokens` into individual tokens
        let tokens: Vec<&str> = serialized_tokens.split_whitespace().collect();

        // Convert character location to token index
        let mut char_count = 0;
        let problematic_token_index = tokens
            .iter()
            .position(|&token| {
                char_count += token.len() + 1; // +1 for the space separator
                char_count > location
            })
            .unwrap_or_else(|| tokens.len().saturating_sub(1));

        // Identify the problematic token
        let problematic_token = tokens
            .get(problematic_token_index)
            .unwrap_or(&"Unknown token");
        println!("Problematic token: {}", problematic_token);

        // Define the context range to show surrounding tokens
        let context_range = 3;
        let start = problematic_token_index.saturating_sub(context_range);
        let end = (problematic_token_index + context_range + 1).min(tokens.len());

        // Print context with improved formatting
        println!("Context around the problematic token:");
        for (i, token) in tokens.iter().enumerate().take(end).skip(start) {
            if i == problematic_token_index {
                println!("--> Problematic token: {}", token);
            } else {
                println!("    Token {}: {}", i, token);
            }
        }
    }
}
