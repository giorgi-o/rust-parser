use std::collections::HashMap;

use crate::ast::{Parser, ParserState, SyntaxNode};

impl Parser {
    /*
        START => <Keyword:region> <Identifier> { <Functions> } START
        START => <Empty>

        Functions => <Function>
        Functions => <Function> <Functions>

        Function => <Keyword:function> <Identifier> ( <Identifier> ) { <Lines> }

        Lines => <LineAndSemi>
        Lines => <LineAndSemi> ; <Lines>

        LineAndSemi => Line ;

        Line => <Keyword:let> <Identifier> = <Rvalue>
        Line => <Lvalue> = <Rvalue>
        Line => return <Rvalue>
        Line => FunctionCall
        Line => IfStatement

        Lvalue => <Identifier>
        Lvalue => Lvalue . <Identifier>

        Rvalue => Lvalue
        Rvalue => <Number>
        Rvalue => FunctionCall
        Rvalue => Rvalue + Rvalue
        Rvalue => Rvalue - Rvalue
        Rvalue => Rvalue * Rvalue
        Rvalue => Rvalue / Rvalue

        FunctionCall => <Identifier> ( <FunctionArgs> )

        FunctionArgs => Rvalue
        FunctionArgs => Rvalue , FunctionArgs

        IfStatement => if Rvalue { Lines }

        ForStatement => for ( Line ; Lvalue ; Line ) { Lines }
    */

    pub fn new() -> Self {
        use crate::token::Token::*;
        use crate::Keyword;
        use ParserState::*;

        let mut p = Self {
            rules: HashMap::new(),
        };

        /**********************************
         *             START              *
         **********************************/

        // START => <Keyword:region> <Identifier> { <Functions> } START
        p.add_rule(
            Start,
            &[
                Keyword(Keyword::Region).into(),
                Ident,
                Lcur.into(),
                Functions,
                Rcur.into(),
            ],
            |s| {
                vec![SyntaxNode::Region {
                    name: s[1].state.as_token().as_ident().to_string(),
                    body: vec![],
                }]
            },
        );

        // START => <Empty>
        p.add_rule(Start, &[], |_| vec![]);

        /**********************************
         *           FUNCTIONS            *
         **********************************/

        // Functions => <Function>
        p.add_rule(Functions, &[Function.into()], |_| vec![]);

        // Functions => <Function> <Functions>
        p.add_rule(Functions, &[Function.into(), Functions.into()], |_| None);

        /**********************************
         *           FUNCTION             *
         **********************************/

        // Function => <Keyword:function> <Identifier> ( <Identifier> ) { <Lines> }
        p.add_rule(
            Function,
            &[
                Keyword(Keyword::Fn).into(),
                Ident,
                Lpar.into(),
                Ident,
                Rpar.into(),
                Lcur.into(),
                Lines,
                Rcur.into(),
            ],
            |s| {
                vec![SyntaxNode::FunctionDefinition {
                    name: s[1].state.as_token().as_ident(),
                    parameters: vec![s[3].state.as_token().as_ident()],
                    body: todo!(),
                }]
            },
        );

        /**********************************
         *             LINES              *
         **********************************/

        // Lines => <LineAndSemi>
        p.add_rule(Lines, &[LineAndSemi.into()], |_| None);

        // Lines => <LineAndSemi> ; <Lines>
        p.add_rule(Lines, &[LineAndSemi.into(), Semi.into(), Lines.into()], |_| None);

        p
    }
}
