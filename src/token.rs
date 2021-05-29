use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),

    Operator(Operator),
    Keyword(Keyword),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    EndOfFile,
    Semicolon,
    Unknown,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Plus,
    Minus,
    Mul,
    Div,
    Modulus,
    Exponent,

    Equal,
    PlusEqual,
    MinusEqual,
    MulEqual,
    DivEqual,
    ModulusEqual,
    ExponentEqual,

    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    DoubleEqual,
    NotEqual,

    DoublePipe,
    Pipe,

    Not,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    Let,
    Function,
}

pub fn extract_op(token: Token) -> Result<Operator, String> {
    if let Token::Operator(op) = token {
        Ok(op)
    } else {
        Err(format!("Expected unary '+' or '-', got {}", token))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}
