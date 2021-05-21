use std::vec;

use crate::{ast::*, lexer::Lexer, token::*};

type PResult = Result<Node, String>;

#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut lexer = Lexer::new(&text);
        lexer.lex();
        Self { lexer }
    }

    /*
        expr : addition-expression
        addition-expression : multiplication-expression ((PLUS|MINUS) multiplication-expression)*
        multiplication-expression : exponent-expression ((MUL|DIV|MODULUS) exponent-expression)*
        exponent-expression: term (EXPONENT term)*
        term : (PLUS | MINUS) term | NUMBER | LPAREN expr RPAREN
    */

    fn term(&mut self) -> PResult {
        let token = self.lexer.next();

        match token {
            Token::Number(num) => Ok(Node::Number(num)),
            Token::Identifier(iden) => Ok(Node::Identifier(iden)),
            Token::String(string) => Ok(Node::String(string)),
            Token::Boolean(boolean) => Ok(Node::Boolean(boolean)),
            Token::Operator(Operator::Plus)
            | Token::Operator(Operator::Minus)
            | Token::Operator(Operator::Not) => Ok(Node::UnaryOperator(Box::new(UnaryOperator {
                operator: extract_op(token)?,
                expression: self.term()?,
            }))),
            Token::LParen => {
                let result = self.expression();
                let current_token = self.lexer.next();

                match current_token {
                    Token::RParen => result,
                    _ => Err(format!("Expected closing ')', got {}", current_token)),
                }
            }
            _ => Err(format!("Expected number got {:?}", token)),
        }
    }

    fn exponent_expr(&mut self) -> PResult {
        let mut node = self.term()?;

        loop {
            let token = self.lexer.peek();
            match token {
                Token::Operator(Operator::Exponent) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: extract_op(token)?,
                        right: self.term()?,
                    }))
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn addition_expr(&mut self) -> PResult {
        let mut node = self.multiplication_expr()?;

        loop {
            let token = self.lexer.peek();
            match token {
                Token::Operator(Operator::Plus) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: extract_op(token)?,
                        right: self.multiplication_expr()?,
                    }))
                }
                Token::Operator(Operator::Minus) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: extract_op(token)?,
                        right: self.multiplication_expr()?,
                    }))
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn multiplication_expr(&mut self) -> PResult {
        let mut node = self.exponent_expr()?;

        loop {
            let token = self.lexer.peek();
            match token {
                Token::Operator(Operator::Mul) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: extract_op(token)?,
                        right: self.exponent_expr()?,
                    }))
                }
                Token::Operator(Operator::Div) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: extract_op(token)?,
                        right: self.exponent_expr()?,
                    }))
                }
                Token::Operator(Operator::Modulus) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: extract_op(token)?,
                        right: self.exponent_expr()?,
                    }))
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn eat(&mut self, token: Token) -> Result<Token, String> {
        if self.lexer.peek() == token {
            Ok(self.lexer.next())
        } else {
            Err(format!("Expected {}, got {}", token, self.lexer.peek()))
        }
    }

    fn comparison(&mut self) -> PResult {
        let mut node = self.addition_expr()?;

        loop {
            match self.lexer.peek() {
                Token::Operator(Operator::GreaterThan) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: Operator::GreaterThan,
                        right: self.addition_expr()?,
                    }))
                }
                Token::Operator(Operator::GreaterThanOrEqual) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: Operator::GreaterThanOrEqual,
                        right: self.addition_expr()?,
                    }))
                }
                Token::Operator(Operator::LessThan) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: Operator::LessThan,
                        right: self.addition_expr()?,
                    }))
                }
                Token::Operator(Operator::LessThanOrEqual) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: Operator::LessThanOrEqual,
                        right: self.addition_expr()?,
                    }))
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn equality(&mut self) -> PResult {
        let mut node = self.comparison()?;

        loop {
            match self.lexer.peek() {
                Token::Operator(Operator::DoubleEqual) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: Operator::DoubleEqual,
                        right: self.addition_expr()?,
                    }))
                }
                Token::Operator(Operator::NotEqual) => {
                    self.lexer.next();
                    node = Node::BinOperator(Box::new(BinOperator {
                        left: node,
                        operator: Operator::NotEqual,
                        right: self.addition_expr()?,
                    }))
                }
                _ => break,
            }
        }

        Ok(node)
    }

    pub fn expression(&mut self) -> PResult {
        let expression = self.equality()?;

        match self.lexer.peek() {
            Token::Operator(Operator::Equal)
            | Token::Operator(Operator::PlusEqual)
            | Token::Operator(Operator::MinusEqual)
            | Token::Operator(Operator::MulEqual)
            | Token::Operator(Operator::DivEqual)
            | Token::Operator(Operator::ExponentEqual)
            | Token::Operator(Operator::ModulusEqual) => {
                if let Node::Identifier(identifier) = &expression {
                    let operator = self.lexer.next();
                    let mut value = self.expression()?;
                    value = match operator {
                        Token::Operator(Operator::PlusEqual) => {
                            Node::BinOperator(Box::new(BinOperator {
                                left: expression.clone(),
                                operator: Operator::Plus,
                                right: value,
                            }))
                        }
                        Token::Operator(Operator::MinusEqual) => {
                            Node::BinOperator(Box::new(BinOperator {
                                left: expression.clone(),
                                operator: Operator::Minus,
                                right: value,
                            }))
                        }
                        Token::Operator(Operator::MulEqual) => {
                            Node::BinOperator(Box::new(BinOperator {
                                left: expression.clone(),
                                operator: Operator::Mul,
                                right: value,
                            }))
                        }
                        Token::Operator(Operator::DivEqual) => {
                            Node::BinOperator(Box::new(BinOperator {
                                left: expression.clone(),
                                operator: Operator::Div,
                                right: value,
                            }))
                        }
                        Token::Operator(Operator::ExponentEqual) => {
                            Node::BinOperator(Box::new(BinOperator {
                                left: expression.clone(),
                                operator: Operator::Exponent,
                                right: value,
                            }))
                        }
                        Token::Operator(Operator::ModulusEqual) => {
                            Node::BinOperator(Box::new(BinOperator {
                                left: expression.clone(),
                                operator: Operator::Modulus,
                                right: value,
                            }))
                        }
                        _ => value,
                    };
                    Ok(Node::AssignmentExpr(Box::new(AssignmentExpr {
                        identifier: identifier.clone(),
                        value,
                    })))
                } else {
                    Err(format!("Invalid assignment operator, got {:?}", expression))
                }
            }
            _ => Ok(expression),
        }
    }

    fn expression_statment(&mut self) -> PResult {
        let expr = self.expression()?;
        self.eat(Token::Semicolon)?;
        Ok(Node::Expression(Box::new(expr)))
    }

    fn variable_decleration(&mut self) -> PResult {
        self.eat(Token::Keyword(Keyword::Let))?;

        match self.lexer.peek() {
            Token::Identifier(identifier) => {
                self.eat(Token::Identifier(identifier.clone()))?;
                match self.lexer.next() {
                    Token::Operator(Operator::Equal) => {
                        let node = Node::VariabeDecleration(Box::new(VariabeDecleration {
                            identifier,
                            value: Some(self.expression()?),
                        }));
                        self.eat(Token::Semicolon)?;
                        Ok(node)
                    }
                    Token::Semicolon => {
                        Ok(Node::VariabeDecleration(Box::new(VariabeDecleration {
                            identifier,
                            value: None,
                        })))
                    }
                    _ => Err(format!("Invalid syntax")),
                }
            }
            _ => Err(format!("Expected identifier, got {}", self.lexer.peek())),
        }
    }

    fn statment(&mut self) -> PResult {
        let token = self.lexer.peek();

        match token {
            Token::Keyword(Keyword::Let) => self.variable_decleration(),
            _ => self.expression_statment(),
        }
    }

    fn program(&mut self) -> PResult {
        let mut declarations = vec![];

        loop {
            match self.lexer.peek() {
                Token::EndOfFile => break,
                _ => declarations.push(self.statment()?),
            }
        }

        Ok(Node::Compound(declarations))
    }

    pub fn parse(&mut self) -> PResult {
        self.program()
    }
}

#[test]
fn should_parse_variable_decleration() {
    let mut parser = Parser::new("let foo = 10;");
    let result = parser.parse().unwrap();
    assert_eq!(
        result,
        Node::Compound(vec![Node::VariabeDecleration(Box::new(
            VariabeDecleration {
                identifier: String::from("foo"),
                value: Some(Node::Number(10.0)),
            },
        ))])
    );
}

#[test]
fn shouldnt_parse_variable_assignment() {
    let mut parser = Parser::new("foo = 10; foo = true;");
    let result = parser.parse().unwrap();
    assert_eq!(
        result,
        Node::Compound(vec![
            Node::Expression(Box::new(Node::AssignmentExpr(Box::new(AssignmentExpr {
                identifier: String::from("foo"),
                value: Node::Number(10.0),
            })))),
            Node::Expression(Box::new(Node::AssignmentExpr(Box::new(AssignmentExpr {
                identifier: String::from("foo"),
                value: Node::Boolean(true),
            })))),
        ])
    );
}

#[test]
fn should_parse_multiple_statements() {
    let mut parser = Parser::new("let foo = 10; foo = 20;");
    let result = parser.parse().unwrap();
    assert_eq!(
        result,
        Node::Compound(vec![
            Node::VariabeDecleration(Box::new(VariabeDecleration {
                identifier: String::from("foo"),
                value: Some(Node::Number(10.0)),
            })),
            Node::Expression(Box::new(Node::AssignmentExpr(Box::new(AssignmentExpr {
                identifier: String::from("foo"),
                value: Node::Number(20.0),
            })))),
        ])
    );
}

#[test]
fn should_parse_compound_assignments() {
    let mut parser = Parser::new("let foo = 10; foo += 20; foo /= 2; foo **= 2;");
    let result = parser.parse().unwrap();
    assert_eq!(
        result,
        Node::Compound(vec![
            Node::VariabeDecleration(Box::new(VariabeDecleration {
                identifier: String::from("foo"),
                value: Some(Node::Number(10.0)),
            })),
            Node::Expression(Box::new(Node::AssignmentExpr(Box::new(AssignmentExpr {
                identifier: String::from("foo"),
                value: Node::BinOperator(Box::new(BinOperator {
                    left: Node::Identifier(String::from("foo")),
                    operator: Operator::Plus,
                    right: Node::Number(20.0),
                })),
            })))),
            Node::Expression(Box::new(Node::AssignmentExpr(Box::new(AssignmentExpr {
                identifier: String::from("foo"),
                value: Node::BinOperator(Box::new(BinOperator {
                    left: Node::Identifier(String::from("foo")),
                    operator: Operator::Div,
                    right: Node::Number(2.0),
                })),
            })))),
            Node::Expression(Box::new(Node::AssignmentExpr(Box::new(AssignmentExpr {
                identifier: String::from("foo"),
                value: Node::BinOperator(Box::new(BinOperator {
                    left: Node::Identifier(String::from("foo")),
                    operator: Operator::Exponent,
                    right: Node::Number(2.0),
                })),
            })))),
        ])
    );
}

#[test]
fn should_parse_comparision() {
    let mut parser = Parser::new("let foo = 10; foo <= 20; foo >= 2; foo == 10;");
    let result = parser.parse().unwrap();
    assert_eq!(
        result,
        Node::Compound(vec![
            Node::VariabeDecleration(Box::new(VariabeDecleration {
                identifier: String::from("foo"),
                value: Some(Node::Number(10.0)),
            })),
            Node::Expression(Box::new(Node::BinOperator(Box::new(BinOperator {
                left: Node::Identifier(String::from("foo")),
                operator: Operator::LessThanOrEqual,
                right: Node::Number(20.0),
            })))),
            Node::Expression(Box::new(Node::BinOperator(Box::new(BinOperator {
                left: Node::Identifier(String::from("foo")),
                operator: Operator::GreaterThanOrEqual,
                right: Node::Number(2.0),
            })))),
            Node::Expression(Box::new(Node::BinOperator(Box::new(BinOperator {
                left: Node::Identifier(String::from("foo")),
                operator: Operator::DoubleEqual,
                right: Node::Number(10.0),
            })))),
        ])
    );
}
