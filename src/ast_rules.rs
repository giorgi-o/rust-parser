use std::collections::HashMap;

use crate::{
    ast::{Parser, ParserState, SyntaxNode},
    Operator,
};

impl Parser {
    /*
    START => <Keyword:region> <Identifier> { <Functions> } START
    START => <Empty>

    Functions => <Function>
    Functions => <Function> <Functions>

    Function => <Keyword:function> <Identifier> ( <Identifier> ) { <Lines> }

    Lines => <LineAndSemi>
    Lines => <LineAndSemi> <Lines>

    LineAndSemi => Line ;

    Line => <Keyword:let> <Identifier> = <Rvalue>
    Line => <Lvalue> = <Rvalue>
    Line => return <Rvalue>
    Line => FunctionCall
    Line => IfStatement
    Line => ForStatement

    Lvalue => <Identifier>
    Lvalue => Lvalue . <Identifier>

    Rvalue => Lvalue
    Rvalue => <Number>
    Rvalue => FunctionCall
    Rvalue => Rvalue + Rvalue
    Rvalue => Rvalue - Rvalue
    Rvalue => Rvalue * Rvalue
    Rvalue => Rvalue / Rvalue
    Rvalue => Rvalue > Rvalue
    Rvalue => Rvalue < Rvalue
    Rvalue => Rvalue >= Rvalue
    Rvalue => Rvalue <= Rvalue

    FunctionCall => <Identifier> ( <FunctionArgs> )

    FunctionArgs => Rvalue
    FunctionArgs => Rvalue , FunctionArgs

    IfStatement => if Rvalue { Lines }

    ForStatement => for ( Line ; Rvalue ; Line ) { Lines }

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
                Keyw(Keyword::Region).into(),
                AnyIdent,
                Lcur.into(),
                Functions,
                Rcur.into(),
            ],
            |s| {
                vec![SyntaxNode::Region {
                    name: s[1].state.as_ident_token().to_string(),
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
        p.add_rule(Functions, &[Function.into(), Functions.into()], |_| vec![]);

        /**********************************
         *           FUNCTION             *
         **********************************/

        // Function => <Keyword:function> <Identifier> ( <Identifier> ) { <Lines> }
        p.add_rule(
            Function,
            &[
                Keyw(Keyword::Fn).into(),
                AnyIdent,
                Lpar.into(),
                AnyIdent,
                Rpar.into(),
                Lcur.into(),
                Lines,
                Rcur.into(),
            ],
            |s| {
                vec![SyntaxNode::FunctionDefinition {
                    name: s[1].state.as_ident_token(),
                    parameters: vec![s[3].state.as_ident_token()],
                    body: s[6].result.clone(),
                }]
            },
        );

        /**********************************
         *             LINES              *
         **********************************/

        // Lines => <LineAndSemi>
        p.add_rule(Lines, &[LineAndSemi], |s| s[0].result.clone());

        // Lines => <LineAndSemi> <Lines>
        p.add_rule(Lines, &[LineAndSemi, Lines], |s| {
            s[0].result.iter().chain(&s[2].result).cloned().collect()
        });

        /**********************************
         *          LINE AND SEMI         *
         **********************************/

        // LineAndSemi => Line ;
        p.add_rule(LineAndSemi, &[Line, Token(Semi)], |s| s[0].result.clone());

        /**********************************
         *             LINE               *
         **********************************/

        // Line => <Keyword:let> <Identifier> = <Rvalue>
        p.add_rule(
            Line,
            &[
                Keyw(Keyword::Let).into(),
                AnyIdent,
                Token(Op(Operator::Assign)),
                Rvalue,
            ],
            |s| {
                let variable_name = s[1].state.as_ident_token();
                vec![
                    SyntaxNode::VariableDefinition {
                        name: variable_name.clone(),
                    },
                    SyntaxNode::Assignment {
                        assignee: variable_name,
                        assigner: Box::new(s[3].result[0].clone()),
                    },
                ]
            },
        );

        // Line => <Lvalue> = <Rvalue>
        p.add_rule(
            Line,
            &[Lvalue.into(), Token(Op(Operator::Assign)), Rvalue],
            |s| {
                vec![SyntaxNode::Assignment {
                    assignee: s[0].state.as_ident_token(),
                    assigner: Box::new(s[2].result[0].clone()),
                }]
            },
        );

        // Line => return <Rvalue>
        p.add_rule(Line, &[Keyw(Keyword::Region).into(), Rvalue], |s| {
            vec![SyntaxNode::FunctionReturn {
                value: Box::new(s[1].result[0].clone()),
            }]
        });

        // Line => FunctionCall
        p.add_rule(Line, &[FunctionCall.into()], |s| s[0].result.clone());

        // Line => IfStatement
        p.add_rule(Line, &[IfStatement.into()], |s| s[0].result.clone());

        // Line => ForStatement
        p.add_rule(Line, &[ForStatement.into()], |s| s[0].result.clone());

        /**********************************
         *            LVALUE               *
         **********************************/

        // Lvalue => <Identifier>
        p.add_rule(Lvalue, &[AnyIdent], |s| {
            vec![SyntaxNode::Value(s[0].state.as_ident_token())]
        });

        // Lvalue => Lvalue . <Identifier>
        p.add_rule(
            Lvalue,
            &[Lvalue.into(), Token(Op(Operator::Dot)), AnyIdent],
            |s| {
                vec![SyntaxNode::ObjAccessor {
                    obj: Box::new(s[0].result.first().unwrap().clone()),
                    field: s[2].state.as_ident_token(),
                }]
            },
        );

        /**********************************
         *            RVALUE               *
         **********************************/

        // Rvalue => Lvalue
        p.add_rule(Rvalue, &[Lvalue.into()], |s| s[0].result.clone());

        // Rvalue => <Number>
        p.add_rule(Rvalue, &[AnyNumber], |s| {
            vec![SyntaxNode::Value(s[0].state.as_token().as_value())]
        });

        // Rvalue => FunctionCall
        p.add_rule(Rvalue, &[FunctionCall.into()], |s| s[0].result.clone());

        // Rvalue => Rvalue +-*/<>>=<= Rvalue
        for op in [
            Operator::Plus,
            Operator::Minus,
            Operator::Mult,
            Operator::Div,
            Operator::LessThan,
            Operator::GreaterThan,
            Operator::LessThanOrEqual,
            Operator::GreaterThanOrEqual,
        ] {
            p.add_rule(Rvalue, &[Rvalue.into(), Token(Op(op)), Rvalue], move |s| {
                vec![SyntaxNode::Operation {
                    left: Box::new(s[0].result.first().unwrap().clone()),
                    operator: op.clone(),
                    right: Box::new(s[2].result.first().unwrap().clone()),
                }]
            });
        }

        /**********************************
         *         FUNCTION CALL          *
         **********************************/

        // FunctionCall => <Identifier> ( <FunctionArgs> )
        p.add_rule(
            FunctionCall,
            &[AnyIdent, Token(Lpar), FunctionArgs, Token(Rpar)],
            |s| {
                vec![SyntaxNode::FunctionCall {
                    name: s[0].state.as_ident_token(),
                    arguments: s[2].result.clone(),
                }]
            },
        );

        /**********************************
         *         FUNCTION ARGS          *
         **********************************/

        // FunctionArgs => Rvalue
        p.add_rule(FunctionArgs, &[Rvalue.into()], |s| s[0].result.clone());

        // FunctionArgs => Rvalue , FunctionArgs
        p.add_rule(
            FunctionArgs,
            &[Rvalue.into(), Token(Comma), FunctionArgs.into()],
            |s| s[0].result.iter().chain(&s[2].result).cloned().collect(),
        );

        /**********************************
         *         IF STATEMENT           *
         **********************************/

        // IfStatement => if Rvalue { Lines }
        p.add_rule(
            IfStatement,
            &[
                Keyw(Keyword::If).into(),
                Rvalue,
                Lcur.into(),
                Lines,
                Rcur.into(),
            ],
            |s| {
                vec![SyntaxNode::IfStatement {
                    condition: Box::new(s[1].result.first().unwrap().clone()),
                    body: s[3].result.clone(),
                }]
            },
        );

        /**********************************
         *         FOR STATEMENT          *
         **********************************/

        // ForStatement => for ( Line ; Rvalue ; Line ) { Lines }
        p.add_rule(
            ForStatement,
            &[
                Keyw(Keyword::For).into(),
                Token(Lpar),
                Line.into(),
                Token(Semi),
                Rvalue,
                Token(Semi),
                Line.into(),
                Token(Rpar),
                Lcur.into(),
                Lines,
                Rcur.into(),
            ],
            |s| {
                vec![SyntaxNode::ForStatement {
                    initializer: Box::new(s[2].result.first().unwrap().clone()),
                    condition: Box::new(s[4].result.first().unwrap().clone()),
                    increment: Box::new(s[6].result.first().unwrap().clone()),
                    body: s[9].result.clone(),
                }]
            },
        );

        p
    }
}
