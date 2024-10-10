use strum::IntoEnumIterator;

use crate::{Keyword, Operator, Token};

/// The three states in our tokeniser FSM.
///
/// Our language has been designed in such a way that at no point will the FSM
/// be in a non-accepting state, but then with more characters will transition
/// into an accepting state.
/// So if the tokeniser enters the Error state, we already know that no
/// combination of characters afterwards will result in a valid token.
///
/// The only exception to the above is the empty state, before the first
/// character. Although this technically just results in no tokens being added
/// to the list, not a parsing error.
#[derive(Debug, Clone, PartialEq)]
pub enum TokeniserState {
    /// The starting state, no characters have been read.
    Start,

    /// The characters do not and will not lead to any valid token.
    Error(String),

    /// This FSM is in an accepting state, and if all the token's characters
    /// have been parsed, it would result in a valid token.
    ///
    /// Note: Although this is only one enum variant, it technically represents
    /// an infinite number of states, since there could be an infinite number
    /// of different tokens as the variant data.
    /// Handling all types of different tokens happens in the delta() function.
    Accepting(Token),
}

/// Convenience: define a conversion function from a Token to a TokeniserState.
impl From<Token> for TokeniserState {
    fn from(value: Token) -> Self {
        TokeniserState::Accepting(value)
    }
}

pub struct Tokeniser;

impl Tokeniser {
    /// Tokenise some code into a list of tokens.
    ///
    /// # Arguments
    /// filename: the name of the file being tokenised. Only passed to make
    ///           error messages more informative.
    /// s: the code to tokenise.
    ///
    /// # Returns
    /// A list of tokens if the code was successfully tokenised, or an error
    /// message containing the filename, line number, column number, and the
    /// characters that caused the error.
    pub fn tokenise(filename: &str, s: &str) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];

        // these variables will be incremented as we iterate through the code
        let mut curr_line = 1;
        let mut curr_col = 1;

        // the current state of the tokeniser FSM, will change over the
        // course of the iteration
        let mut state = TokeniserState::Start;

        for c in s.chars() {
            // increment the line/column counters
            if c == '\n' {
                curr_line += 1;
                curr_col = 1;
            } else {
                curr_col += 1;
            }

            // check if this char is the start of a new token
            if Self::is_token_separator(c) {
                // the current token is a "token separator", which indicates the
                // end of the current token and the start of a new one.

                // check if we are in the accepting state
                // i.e. we just parsed a token
                if let TokeniserState::Accepting(token) = state {
                    tokens.push(token);

                    state = TokeniserState::Start;
                }

                // if the separator is not a whitespace, then it is a special
                // token (a dot, a bracket, etc.). Don't forget to add it to the
                // list of tokens.
                if let Some(token) = Self::is_special_token(c) {
                    tokens.push(token);
                }

                continue;
            }

            // call the FSM's transition function to get the new state
            state = Self::delta(state, c);

            // check if error after transition
            if let TokeniserState::Error(e) = &state {
                // abort parsing
                return Err(format!(
                    "Error parsing file {filename}:{curr_line}:{curr_col} while parsing token: {e}",
                ));
            }
        }

        // we've reached the end of the code!

        // add the last token if it exists
        if let TokeniserState::Accepting(token) = state {
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Check if the character is a character that separates tokens,
    /// i.e. signifies the end of a token and the start of a new one.
    fn is_token_separator(c: char) -> bool {
        c.is_whitespace() || Self::is_special_token(c).is_some()
    }

    /// The FSM's transition function.
    /// Given the current state and the next character, return the new state.
    fn delta(state: TokeniserState, c: char) -> TokeniserState {
        // c = char, cs = char string
        // (because some functions only take strings, not chars)
        let cs = c.to_string();

        // the entire string so far, since the start of the FSM.
        // this variable is used if we can't parse the current character, it is
        // added to the error message.
        // This variable is the concatenation of the characters until now + the
        // current character.
        // However the "characters until now" part is embedded in the state's
        // token info, so there isn't a simple function call to get it as a
        // string.
        // So we define it here, and initialise it in the individual match
        // statements below.
        let full_token_str;

        match state {
            TokeniserState::Start => {
                // this is the first character so far

                if let Some(op) = Self::is_operator(&cs) {
                    return Token::Operator(op).into();
                }

                if let Some(special_token) = Self::is_special_token(c) {
                    return special_token.into();
                }

                if c.is_alphabetic() {
                    return Token::Identifier(cs).into();
                }

                if c.is_ascii_digit() {
                    return Token::Number(cs).into();
                }

                if c.is_whitespace() {
                    // whitespace is ignored
                    return TokeniserState::Start;
                }

                // error - first letter won't lead to any valid token!
                // set "full_token_str" to the current character, and let the
                // end of the function handle the error.
                full_token_str = cs;
            }

            // adding more characters to an error state won't make it better.
            TokeniserState::Error(e) => return TokeniserState::Error(e + &cs),

            TokeniserState::Accepting(token) => match token {
                // the characters so far (before this one) form a valid token.
                // but what token?

                Token::Identifier(s) => {
                    // the characters so far form a valid identifier
                    full_token_str = s + &cs;

                    if let Some(kw) = Self::is_keyword(&full_token_str) {
                        return Token::Keyword(kw).into();
                    }

                    if c.is_alphanumeric() {
                        return Token::Identifier(full_token_str).into();
                    }
                }

                Token::Operator(op) => {
                    // the characters so far form a valid operator
                    full_token_str = op.to_string() + &cs;

                    if let Some(new_op) = Self::is_operator(&full_token_str) {
                        return Token::Operator(new_op).into();
                    }

                    // edge case: for the string "-1", the parser will first
                    // think it's a minus operator, but then realise it's a
                    // number.
                    if op == Operator::Minus && c.is_ascii_digit() {
                        return Token::Number(full_token_str).into();
                    }
                }

                Token::Keyword(s) => {
                    full_token_str = s.to_string() + &cs;

                    if let Some(kw) = Self::is_keyword(&full_token_str) {
                        return Token::Keyword(kw).into();
                    }
                    
                    if c.is_alphanumeric() {
                        return Token::Identifier(full_token_str).into();
                    }
                }

                Token::Number(s) => {
                    full_token_str = s.clone() + &cs;

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
                    // none of these are more than 1 char.
                    // if we were in these states and we added a character,
                    // no valid token would be formed.
                    return TokeniserState::Error(cs);
                }
            },
        };

        // if we reach this point, we didn't match any state transitions to any
        // valid tokens
        // => error tokenising this character
        TokeniserState::Error(full_token_str)
    }

    /// Is this string a valid keyword, and if so, which one?
    fn is_keyword(s: &str) -> Option<Keyword> {
        Keyword::iter().find(|kw| kw.as_ref() == s)
    }

    /// Is this string a valid operator, and if so, which one?
    fn is_operator(s: &str) -> Option<Operator> {
        Operator::iter().find(|op| op.as_ref() == s)
    }

    /// Is this character a special token, and if so, which one?
    fn is_special_token(c: char) -> Option<Token> {
        match c {
            '{' => Some(Token::Lcur),
            '}' => Some(Token::Rcur),
            '(' => Some(Token::Lpar),
            ')' => Some(Token::Rpar),
            ';' => Some(Token::Semi),
            ',' => Some(Token::Comma),
            '.' => Some(Token::Dot),
            '[' => Some(Token::Lbrack),
            ']' => Some(Token::Rbrack),
            _ => None,
        }
    }
}
