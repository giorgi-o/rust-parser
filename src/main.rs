use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar);
mod grammar_ast;

use std::env;
use std::fs;

use string_enum::StringEnum;

#[derive(StringEnum, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(StringEnum, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// `<=`
    LessThanOrEqual,
    /// `>=`
    GreaterThanOrEqual,
    /// `.`
    Dot,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let file_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("src/source_code.txt");

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

    println!("1. Code without comments:");
    for line in &code_lines_without_comments {
        println!("{line}");
    }

    let code_without_comments = code_lines_without_comments.join("\n");

    println!("\n2. tokeniser + parser:");

    // Parse the input using the generated parser
    match grammar::RegionParser::new().parse(&code_without_comments) {
        Ok(region) => {
            // Print the entire Region AST
            println!("Parsed AST: {:#?}", region);
        }
        Err(e) => {
            // If there's a parsing error, print the error
            println!("Error parsing: {:?}", e);
        }
    }
}
