use crate::token::*;
use std::{collections::VecDeque, iter::Peekable, str::Chars};

#[derive(Debug)]
pub struct Lexer<'a> {
    tokens: VecDeque<Token>,
    char_iter: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            tokens: VecDeque::new(),
            char_iter: text.trim().chars().peekable(),
        }
    }

    pub fn next(&mut self) -> Token {
        self.tokens.pop_front().unwrap_or(Token::Unknown)
    }

    pub fn peek(&self) -> Token {
        self.tokens.front().unwrap_or(&Token::Unknown).clone()
    }

    pub fn get_index(&self, index: usize) -> Token {
        self.tokens.get(index).unwrap_or(&Token::Unknown).clone()
    }

    pub fn lex(&mut self) -> &VecDeque<Token> {
        while let Some(c) = self.char_iter.next() {
            let peek = *self.char_iter.peek().unwrap_or(&'\0');
            match c {
                '0'..='9' => {
                    let float = self.parse_float(&c.to_string());
                    self.tokens.push_back(Token::Number(float))
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let word = self.parse_word(&c.to_string());
                    match word.as_str() {
                        "let" => self.tokens.push_back(Token::Keyword(Keyword::Let)),
                        "true" => self.tokens.push_back(Token::Boolean(true)),
                        "false" => self.tokens.push_back(Token::Boolean(false)),
                        "not" => self.tokens.push_back(Token::Operator(Operator::Not)),
                        "function" => self.tokens.push_back(Token::Keyword(Keyword::Function)),
                        _ => self.tokens.push_back(Token::Identifier(word)),
                    }
                }

                '+' => {
                    let token = self.match_char(
                        peek,
                        '=',
                        Token::Operator(Operator::PlusEqual),
                        Token::Operator(Operator::Plus),
                    );
                    self.tokens.push_back(token)
                }
                '-' => {
                    let token = self.match_char(
                        peek,
                        '=',
                        Token::Operator(Operator::MinusEqual),
                        Token::Operator(Operator::Minus),
                    );
                    self.tokens.push_back(token)
                }
                '*' => {
                    let token = self.match_char(
                        peek,
                        '*',
                        Token::Operator(Operator::Exponent),
                        Token::Operator(Operator::Mul),
                    );
                    let peek = *self.char_iter.peek().unwrap_or(&'\0');
                    let token = self.match_char(
                        peek,
                        '=',
                        if token == Token::Operator(Operator::Exponent) {
                            Token::Operator(Operator::ExponentEqual)
                        } else {
                            Token::Operator(Operator::MulEqual)
                        },
                        token,
                    );
                    self.tokens.push_back(token)
                }
                '/' => {
                    let token = self.match_char(
                        peek,
                        '=',
                        Token::Operator(Operator::DivEqual),
                        Token::Operator(Operator::Div),
                    );
                    self.tokens.push_back(token)
                }
                '%' => {
                    let token = self.match_char(
                        peek,
                        '=',
                        Token::Operator(Operator::ModulusEqual),
                        Token::Operator(Operator::Modulus),
                    );
                    self.tokens.push_back(token)
                }
                '"' | '\'' => {
                    let string = self.parse_string(&c.to_string());
                    self.tokens.push_back(Token::String(string));
                }
                '>' => {
                    let token = self.match_char(
                        peek,
                        '=',
                        Token::Operator(Operator::GreaterThanOrEqual),
                        Token::Operator(Operator::GreaterThan),
                    );
                    self.tokens.push_back(token)
                }
                '<' => {
                    let token = self.match_char(
                        peek,
                        '=',
                        Token::Operator(Operator::LessThanOrEqual),
                        Token::Operator(Operator::LessThan),
                    );
                    self.tokens.push_back(token)
                }
                '=' => {
                    let token = self.match_char(
                        peek,
                        '=',
                        Token::Operator(Operator::DoubleEqual),
                        Token::Operator(Operator::Equal),
                    );
                    self.tokens.push_back(token)
                }
                '!' => {
                    let token = self.match_char(
                        peek,
                        '=',
                        Token::Operator(Operator::NotEqual),
                        Token::Unknown,
                    );
                    self.tokens.push_back(token)
                }
                '|' => {
                    let token = self.match_char(
                        peek,
                        '|',
                        Token::Operator(Operator::DoublePipe),
                        Token::Operator(Operator::Pipe),
                    );
                    self.tokens.push_back(token);
                }
                '(' => self.tokens.push_back(Token::LParen),
                ')' => self.tokens.push_back(Token::RParen),
                '{' => self.tokens.push_back(Token::LBrace),
                '}' => self.tokens.push_back(Token::RBrace),
                ',' => self.tokens.push_back(Token::Comma),
                ';' => self.tokens.push_back(Token::Semicolon),
                _ => (),
            }
        }

        self.tokens.push_back(Token::EndOfFile);

        &self.tokens
    }

    fn match_char(
        &mut self,
        peek: char,
        match_char: char,
        matched: Token,
        unmatched: Token,
    ) -> Token {
        if peek == match_char {
            self.char_iter.next();
            matched
        } else {
            unmatched
        }
    }

    fn parse_string(&mut self, start: &str) -> String {
        let mut buffer = String::new();

        while let Some(c) = self.char_iter.next() {
            if c.to_string() == start {
                break;
            } else {
                buffer.push(c)
            }
        }

        buffer
    }

    fn parse_word(&mut self, text: &str) -> String {
        let mut buffer = text.to_string();

        while let Some(c) = self.char_iter.peek() {
            match c {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => {
                    buffer.push(self.char_iter.next().unwrap())
                }
                _ => break,
            }
        }

        buffer
    }

    fn parse_float(&mut self, text: &str) -> f64 {
        let mut buffer = text.to_string();

        while let Some(c) = self.char_iter.peek() {
            match c {
                '0'..='9' | '.' => buffer.push(self.char_iter.next().unwrap()),
                _ => break,
            }
        }

        buffer.parse().expect("Failed to parse float")
    }
}

#[test]
fn should_lex_addsub() {
    let mut lexer = Lexer::new("9.10 + 2 - 10");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::Number(9.1),
            Token::Operator(Operator::Plus),
            Token::Number(2.0),
            Token::Operator(Operator::Minus),
            Token::Number(10.0),
            Token::EndOfFile,
        ]
    );
}

#[test]
fn should_lex_muldivmod() {
    let mut lexer = Lexer::new("5 * 40 % 10 / 10");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::Number(5.0),
            Token::Operator(Operator::Mul),
            Token::Number(40.0),
            Token::Operator(Operator::Modulus),
            Token::Number(10.0),
            Token::Operator(Operator::Div),
            Token::Number(10.0),
            Token::EndOfFile,
        ]
    );
}

#[test]
fn should_lex_paren() {
    let mut lexer = Lexer::new("5 * (2 + 5)");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::Number(5.0),
            Token::Operator(Operator::Mul),
            Token::LParen,
            Token::Number(2.0),
            Token::Operator(Operator::Plus),
            Token::Number(5.0),
            Token::RParen,
            Token::EndOfFile,
        ]
    );
}

#[test]
fn should_lex_exponent() {
    let mut lexer = Lexer::new("5 ** (2 + 5)");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::Number(5.0),
            Token::Operator(Operator::Exponent),
            Token::LParen,
            Token::Number(2.0),
            Token::Operator(Operator::Plus),
            Token::Number(5.0),
            Token::RParen,
            Token::EndOfFile,
        ]
    );
}

#[test]
fn should_lex_words() {
    let mut lexer = Lexer::new("let some_word some24_4");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::Keyword(Keyword::Let),
            Token::Identifier(String::from("some_word")),
            Token::Identifier(String::from("some24_4")),
            Token::EndOfFile,
        ]
    );
}

#[test]
fn should_lex_compound_assignments() {
    let mut lexer = Lexer::new("+= -= *= /= %= **=");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::Operator(Operator::PlusEqual),
            Token::Operator(Operator::MinusEqual),
            Token::Operator(Operator::MulEqual),
            Token::Operator(Operator::DivEqual),
            Token::Operator(Operator::ModulusEqual),
            Token::Operator(Operator::ExponentEqual),
            Token::EndOfFile,
        ]
    );
}

#[test]
fn should_lex_booleans() {
    let mut lexer = Lexer::new("true false");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::Boolean(true),
            Token::Boolean(false),
            Token::EndOfFile,
        ]
    );
}

#[test]
fn should_lex_strings() {
    let mut lexer = Lexer::new("'hello world' \"hello world2\"");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::String(String::from("hello world")),
            Token::String(String::from("hello world2")),
            Token::EndOfFile,
        ]
    );
}

#[test]
fn should_lex_bool_operations() {
    let mut lexer = Lexer::new("== != >= <= < > not");
    lexer.lex();
    assert_eq!(
        lexer.tokens,
        [
            Token::Operator(Operator::DoubleEqual),
            Token::Operator(Operator::NotEqual),
            Token::Operator(Operator::GreaterThanOrEqual),
            Token::Operator(Operator::LessThanOrEqual),
            Token::Operator(Operator::LessThan),
            Token::Operator(Operator::GreaterThan),
            Token::Operator(Operator::Not),
            Token::EndOfFile,
        ]
    );
}
