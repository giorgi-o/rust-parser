use crate::{Keyword, Operator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextToken {
    pub token: Token,
    pub context: TokenContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TokenContext {
    pub line: usize,
    pub column: usize,
    pub line_content: String,
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Ident(String),
    Num(String),
    Keyw(Keyword),
    Op(Operator),
    Lcur,   // Left curly brace {
    Rcur,   // Right curly brace }
    Lpar,   // Left parenthesis (
    Rpar,   // Right parenthesis )
    Semi,   // Semicolon ;
    Comma,  // Comma ,
    Lbrack, // Left square bracket [
    Rbrack, // Right square bracket ]
}

impl Token {
    pub fn tokenise_line(context: TokenContext) -> Result<Vec<TextToken>, String> {
        let line = context.line_content.clone();

        let mut tokens = vec![];
        let mut column = 0;

        let words = line.split_whitespace();
        for word in words {
            let context = TokenContext {
                column,
                ..context.clone()
            };

            let parsed = Token::parse(word, context)?;
            tokens.extend(parsed);
            column += word.len() + 1; // +1 for the space
            // todo: above doesn't work if multiple spaces between words
        }

        Ok(tokens)
    }

    pub fn parse(s: &str, context: TokenContext) -> Result<Vec<TextToken>, String> {
        if s.is_empty() {
            return Ok(vec![]);
        }

        // Handle separator characters
        for (char, token) in [
            // special chars
            ("{", Token::Lcur),
            ("}", Token::Rcur),
            ("(", Token::Lpar),
            (")", Token::Rpar),
            (";", Token::Semi),
            (",", Token::Comma),
            ("[", Token::Lbrack),
            ("]", Token::Rbrack),
            // operators
            ("+", Token::Op(Operator::Plus)),
            ("-", Token::Op(Operator::Minus)),
            ("*", Token::Op(Operator::Mult)),
            ("/", Token::Op(Operator::Div)),
            ("=", Token::Op(Operator::Assign)),
            ("<", Token::Op(Operator::LessThan)),
            (">", Token::Op(Operator::GreaterThan)),
            (".", Token::Op(Operator::Dot)),
            // keywords
            ("region", Token::Keyw(Keyword::Region)),
            ("let", Token::Keyw(Keyword::Let)),
            ("fn", Token::Keyw(Keyword::Fn)),
        ] {
            if let Some(tokens) = Token::parse_with_separator(s, context.clone(), token, char)? {
                return Ok(tokens);
            }
        }

        // Handle numbers
        if s.parse::<i128>().is_ok() || s.parse::<f64>().is_ok() {
            return Ok(vec![TextToken {
                token: Token::Num(s.to_string()),
                context,
            }]);
        }

        // Handle identifiers (like function names, variable names, etc.)
        // NOTE: this assumes variables can't be called e.g. `x2`
        if s.chars().all(char::is_alphabetic) {
            return Ok(vec![TextToken {
                token: Token::Ident(s.to_string()),
                context,
            }]);
        }

        // If the token cannot be parsed, return an error
        Err(format!(
            "Unparseable token: '{s}' at {}:{}:{}\nline: {}",
            context.filename, context.line, context.column, context.line_content
        ))
    }

    /// Given a piece of code, check if it has the given separator.
    /// If it does, parse the bits before and after the separator,
    /// and return the [...before, separator, ...after] tokens.
    ///
    /// Possible returns:
    /// Err(...) -> parse error e.g. invalid token
    /// Ok(None) -> separator not found
    /// Ok(Some(tokens)) -> separator found, and tokens recursively parsed
    fn parse_with_separator(
        code: &str,
        context: TokenContext,
        separator: Token,
        sep_str: &str,
    ) -> Result<Option<Vec<TextToken>>, String> {
        let sep_index = code.find(sep_str);
        let Some(sep_index) = sep_index else {
            // separator not found
            return Ok(None);
        };

        // separator found!
        // split the string, and recursively parse the before and after parts
        let mut tokens = vec![];

        // before separator
        tokens.extend(Token::parse(&code[..sep_index], context.clone())?);

        // separator token
        let context = TokenContext {
            column: context.column + sep_index,
            ..context
        };
        let sep_token = TextToken {
            token: separator,
            context: context.clone(),
        };
        tokens.push(sep_token);

        // after separator
        let context = TokenContext {
            column: context.column + sep_str.len(),
            ..context
        };
        tokens.extend(Token::parse(&code[sep_index + sep_str.len()..], context)?);

        Ok(Some(tokens))
    }

    /// If this token is an identifier, stringify it.
    /// Otherwise, panic.
    pub fn as_ident(&self) -> String {
        match self {
            Token::Ident(s) => s.to_string(),
            _ => panic!("Token::as_ident called on non-Identifier token"),
        }
    }

    /// If this token is a variable name (Identifier) or a number, stringify it.
    /// Otherwise, panic.
    pub fn as_value(&self) -> String {
        match self {
            Token::Ident(s) => s.to_string(),
            Token::Num(s) => s.to_string(),
            _ => panic!("Token::as_value called on non-Value (non-leaf) token"),
        }
    }
}
