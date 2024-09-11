#[derive(Debug)]
enum Token {
    Identifier(String),
    Number(String),
    Keyword(String),
    Operator(String),
    Lcur, // Left curly brace {
    Rcur, // Right curly brace }
    Lpar, // Left parenthesis (
    Rpar, // Right parenthesis )
    Semi, // Semicolon ;
}

const KEYWORDS: &'static [&'static str] = &["region", "let"];
const OPERATORS: &'static [&'static str] = &["+", "-", "*", "/", "="];

const CODE: &str = "region SafeProcessing {
    let buffer = allocate(1024);  // Allocate memory
    free(buffer);  // Free memory explicitly
    let error = borrow(buffer);  // Compile-time error: Cannot borrow after free
}";

fn main() {
    // 1. remove comments
    let mut code_lines_without_comments = vec![];
    for line in CODE.lines() {
        let line_without_comments = line.split("//").next().unwrap();
        code_lines_without_comments.push(line_without_comments);
    }
    let code_without_comments = code_lines_without_comments.join("\n");

    println!("1. Code without comments:");
    for line in code_lines_without_comments {
        println!("{line}");
    }

    // 2. split by whitespace
    let str_tokens = code_without_comments.split_whitespace();

    // 3. tokenize
    let tokens: Vec<Token> = str_tokens.map(|s| Token::parse(s)).flatten().collect();

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
    fn parse(s: &str) -> Vec<Token> {
        if s.is_empty() {
            return vec![];
        }

        for (special_char, token) in [
            ("{", Token::Lcur),
            ("}", Token::Rcur),
            ("(", Token::Lpar),
            (")", Token::Rpar),
            (";", Token::Semi),
        ] {
            if s == special_char {
                return vec![token];
            }

            // process tokens not seperated by whitespace e.v. free(buffer)
            let index_of_special_char = s.find(special_char);
            if let Some(index) = index_of_special_char {
                let mut tokens = vec![];

                tokens.extend(Token::parse(&s[..index]));
                tokens.push(token);
                tokens.extend(Token::parse(&s[index + 1..]));

                return tokens;
            }
        }

        if KEYWORDS.contains(&s) {
            return vec![Token::Keyword(s.to_string())];
        }

        if OPERATORS.contains(&s) {
            return vec![Token::Operator(s.to_string())];
        }

        if s.parse::<i128>().is_ok() || s.parse::<f64>().is_ok() {
            return vec![Token::Number(s.to_string())];
        }

        if s.chars().all(char::is_alphabetic) {
            return vec![Token::Identifier(s.to_string())];
        }

        panic!("Unparseable token: {}", s);
    }
}
