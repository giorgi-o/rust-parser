pub mod token;
pub mod ast;
pub mod ast_rules;

use std::env;
use std::fs;

use string_enum::StringEnum;
use token::Token;
use token::TokenContext;




#[derive(StringEnum, Clone, PartialEq, Eq, Hash)]
pub enum Keyword {
    /// `region`
    Region,
    /// `let`
    Let,
    /// `fn`
    Fn,
}

#[derive(StringEnum, Clone, PartialEq, Eq, Hash)]
pub enum Operator {
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Mult,
    /// `/`
    Div,
    /// `=`
    Assign,
    /// `<`
    LessThan,
    /// `>`
    GreaterThan,
    /// `.`
    Dot,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let file_path = &args[1];

    let source_code = fs::read_to_string(file_path);
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

    let mut tokens = vec![];
    for (line_number, line) in code_lines_without_comments.iter().enumerate() {
        let context = TokenContext {
            filename: file_path.clone(),
            line: line_number + 1,
            column: 1,
            line_content: line.to_string(),
        };

        let result = Token::tokenise_line(context);
        match result {
            Ok(mut t) => tokens.append(&mut t),
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }

    println!("\n3. Tokens:");
    let mut prev_line_number = 1;
    for token in &tokens {
        print!("{:?} ", token.token);
        if token.context.line > prev_line_number {
            println!();
            prev_line_number = token.context.line;
        }
    }
    
    
    println!("\n4. AST:");
    let ast_nodes = ast::parse(&tokens).expect("Error parsing AST");
    for node in &ast_nodes {
        println!("{node:?}");
    }
}
