use strum::IntoEnumIterator;

use crate::{Keyword, Operator, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum TokeniserState {
    Start,
    Error(String),

    /// Note: this is not the only accepting state, this just means the token
    /// type has been narrowed down to only one possible one.
    Accepting(Token),
}

impl From<Token> for TokeniserState {
    fn from(value: Token) -> Self {
        TokeniserState::Accepting(value)
    }
}

pub struct Tokeniser;

impl Tokeniser {
    pub fn parse(filename: &str, s: &str) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];

        let mut curr_line = 1;
        let mut curr_col = 1;

        let mut state = TokeniserState::Start;
        for c in s.chars() {
            if c == '\n' {
                curr_line += 1;
                curr_col = 1;
            } else {
                curr_col += 1;
            }

            // check if this char is the start of a new token
            if Self::is_token_separator(c) {
                if let TokeniserState::Accepting(token) = state {
                    tokens.push(token);
                    state = TokeniserState::Start;
                }

                // if the separator is a token e.g. '.' (i.e. not a space)
                // also add it to the list of tokens
                if let Some(token) = Self::is_special_token(&c.to_string()) {
                    tokens.push(token);
                }

                continue;
            }

            state = Self::delta(state, c);

            // check if error
            if let TokeniserState::Error(e) = &state {
                // abort parsing
                return Err(format!(
                    "Error parsing file {filename}:{curr_line}:{curr_col} while parsing token: {e}",
                ));
            }
        }

        Ok(tokens)
    }

    fn is_token_separator(c: char) -> bool {
        c.is_whitespace() || Self::is_special_token(&c.to_string()).is_some()
    }

    fn delta(state: TokeniserState, c: char) -> TokeniserState {
        // c = char, cs = char string
        let cs = c.to_string();

        // tmp
        if c == '1' {
            println!("{:?} {:?}", state, c);
        }

        // the entire string so far, since the start of the fsm
        let full_token_str;

        match state {
            TokeniserState::Start => {
                if c.is_alphabetic() {
                    return Token::Identifier(cs).into();
                }

                if c.is_ascii_digit() {
                    return Token::Number(cs).into();
                }

                if let Some(op) = Self::is_operator(&cs) {
                    return Token::Operator(op).into();
                }

                if let Some(token) = Self::is_special_token(&cs) {
                    return token.into();
                }

                if c.is_whitespace() {
                    return TokeniserState::Start;
                }

                full_token_str = cs;
            }

            TokeniserState::Error(e) => return TokeniserState::Error(e + &cs),

            TokeniserState::Accepting(token) => match token {
                Token::Identifier(s) => {
                    full_token_str = s + &cs;
                    if let Some(kw) = Self::is_keyword(&full_token_str) {
                        return Token::Keyword(kw).into();
                    }

                    if c.is_alphanumeric() {
                        return Token::Identifier(full_token_str).into();
                    }
                }

                Token::Operator(op) => {
                    full_token_str = op.to_string() + &cs;
                    if let Some(new_op) = Self::is_operator(&full_token_str) {
                        return Token::Operator(new_op).into();
                    }

                    // edge case: for the string "-1", the parser will first
                    // think it's a minus operator, but then realise it's a
                    // number.
                    if op == Operator::Minus && c.is_ascii_digit() {
                        return Token::Number(cs).into();
                    }
                }

                Token::Keyword(s) => {
                    full_token_str = s.to_string() + &cs;

                    if let Some(kw) = Self::is_keyword(&full_token_str) {
                        return Token::Keyword(kw).into();
                    } else if c.is_alphanumeric() {
                        return Token::Identifier(full_token_str).into();
                    }
                }

                Token::Number(s) => {
                    full_token_str = cs + &s;

                    if c.is_ascii_digit() {
                        return Token::Number(full_token_str).into();
                    }

                    if c == '.' && !s.contains('.') {
                        return Token::Number(full_token_str).into();
                    }
                }

                Token::Lcur
                | Token::Rcur
                | Token::Lpar
                | Token::Rpar
                | Token::Semi
                | Token::Comma
                | Token::Dot
                | Token::Lbrack
                | Token::Rbrack => {
                    // none of these are more than 1 char
                    return TokeniserState::Error(cs);
                }
            },
        };

        TokeniserState::Error(full_token_str)
    }

    fn is_keyword(s: &str) -> Option<Keyword> {
        Keyword::iter().find(|kw| kw.as_ref() == s)
    }

    fn is_operator(s: &str) -> Option<Operator> {
        Operator::iter().find(|op| op.as_ref() == s)
    }

    fn is_special_token(s: &str) -> Option<Token> {
        match s.as_ref() {
            "{" => Some(Token::Lcur),
            "}" => Some(Token::Rcur),
            "(" => Some(Token::Lpar),
            ")" => Some(Token::Rpar),
            ";" => Some(Token::Semi),
            "," => Some(Token::Comma),
            "." => Some(Token::Dot),
            "[" => Some(Token::Lbrack),
            "]" => Some(Token::Rbrack),
            _ => None,
        }
    }
}
