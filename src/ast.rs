use std::collections::HashMap;

use crate::token::{TextToken, Token};

pub fn parse(tokens: &[TextToken]) -> Result<Vec<ParsedState>, String> {
    let parser = Parser::new();
    let mut parsed_states = vec![];

    let mut tokens = tokens;
    while !tokens.is_empty() {
        let parse_result = parser.try_parse_tokens_as(tokens, &ParserState::Start);
        if parse_result.is_none() {
            return Err("Failed to parse tokens".to_string());
        }

        let (parsed, unconsumed) = parse_result.unwrap();
        parsed_states.push(parsed);
        tokens = unconsumed;
    }

    Ok(parsed_states)
}

#[derive(Debug)]
pub enum SyntaxNode {
    Region {
        name: String,
        body: Vec<SyntaxNode>,
    },
    Assignment {
        assignee: Token,
        assigner: Box<SyntaxNode>,
    },
    FunctionDefinition {
        name: String,
        parameters: Vec<String>,
        body: Vec<SyntaxNode>,
    },
    FunctionCall {
        name: Token,
        arguments: Vec<SyntaxNode>,
    },
    Value(Token),
}

/// state as in finite state machine state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParserState {
    Start,
    Functions,
    Function,
    Lines,
    LineAndSemi,
    Line,
    Lvalue,
    Rvalue,
    FunctionCall,
    FunctionArgs,
    IfStatement,
    ForStatement,
    Token(Token),
    Ident,
    Number,
}

impl ParserState {
    pub fn as_token(&self) -> Token {
        match self {
            Self::Token(t) => t.clone(),
            _ => panic!("ParserState is not a token"),
        }
    }

    fn is_leaf(&self) -> bool {
        matches!(self, Self::Token(_) | Self::Ident | Self::Number)
    }

    fn try_parse_leaf(&self, token: &TextToken) -> Option<Token> {
        if !self.is_leaf() {
            panic!("ParserState is not a leaf");
        }

        let token = &token.token;

        match self {
            Self::Token(t) => {
                if t == token {
                    Some(token.clone())
                } else {
                    None
                }
            }

            Self::Ident => {
                if let Token::Identifier(_) = token {
                    Some(token.clone())
                } else {
                    None
                }
            }

            Self::Number => {
                if let Token::Number(_) = token {
                    Some(token.clone())
                } else {
                    None
                }
            }

            _ => unreachable!(),
        }
    }
}

impl From<Token> for ParserState {
    fn from(token: Token) -> Self {
        Self::Token(token)
    }
}

pub struct ParserRule {
    from: ParserState,
    to: Vec<ParserState>,
    transformer: fn(&[ParsedState]) -> Vec<SyntaxNode>,
}

impl ParserRule {
    pub fn new(
        from: ParserState,
        rules: Vec<ParserState>,
        transformer: fn(&[ParsedState]) -> Vec<SyntaxNode>,
    ) -> Self {
        Self {
            from,
            to: rules,
            transformer,
        }
    }
}

#[derive(Debug)]
pub struct ParsedState {
    pub state: ParserState,
    pub result: Vec<SyntaxNode>,
    pub intermediate_states: Vec<ParsedState>,
}

pub struct Parser {
    pub rules: HashMap<ParserState, Vec<ParserRule>>,
}

impl Parser {

    fn try_parse_tokens_as<'a>(
        &self,
        tokens: &'a [TextToken],
        state: &ParserState,
    ) -> Option<(ParsedState, &'a [TextToken] /* unconsumed */)> {
        let rules = self.rules.get(state)?;

        for rule in rules {
            // try to match the remaining tokens with this rule

            let mut tokens = tokens.iter();
            let mut intermediate_states = vec![];
            let mut matches = true;

            for intermediate_target in rule.to.iter() {
                if tokens.as_slice().is_empty() {
                    // no more tokens, but stil more states that need matching
                    // => this rule doesn't match
                    matches = false;
                    break;
                }

                if intermediate_target.is_leaf() {
                    let next_token = tokens.next().unwrap();
                    let parsed = intermediate_target.try_parse_leaf(next_token);
                    if parsed.is_none() {
                        // this rule doesn't match
                        matches = false;
                        break;
                    }

                    let parsed = parsed.unwrap();
                    let intermediate_state = ParsedState {
                        state: intermediate_target.clone(),
                        result: vec![SyntaxNode::Value(parsed)],
                        intermediate_states: vec![],
                    };

                    intermediate_states.push(intermediate_state);
                } else {
                    // recursively try to parse the next tokens
                    let result = self.try_parse_tokens_as(tokens.as_slice(), intermediate_target);
                    if result.is_none() {
                        // this rule doesn't match
                        matches = false;
                        break;
                    }

                    let (parsed, unconsumed) = result.unwrap();

                    let intermediate_state = ParsedState {
                        state: intermediate_target.clone(),
                        result: parsed.result,
                        intermediate_states: parsed.intermediate_states,
                    };
                    intermediate_states.push(intermediate_state);

                    tokens = unconsumed.iter();
                }
            }

            if matches {
                // this rule matches
                let parsed = ParsedState {
                    state: rule.from.clone(),
                    result: (rule.transformer)(&intermediate_states),
                    intermediate_states,
                };

                let unconsumed = tokens.as_slice();
                return Some((parsed, unconsumed));
            }
        }

        // no rule matched
        None
    }

    pub fn add_rule(
        &mut self,
        from: ParserState,
        rules: &[ParserState],
        transformer: fn(&[ParsedState]) -> Vec<SyntaxNode>,
    ) {
        let rule = ParserRule::new(from.clone(), rules.to_vec(), transformer);

        self.rules.entry(from).or_insert_with(Vec::new).push(rule);
    }
}

// example syntax:
// Lvalue, [Lvalue, Operator::Dot, ]
