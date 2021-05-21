use crate::token::*;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct BinOperator {
    pub left: Node,
    pub operator: Operator,
    pub right: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryOperator {
    pub operator: Operator,
    pub expression: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariabeDecleration {
    pub identifier: String,
    pub value: Option<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AssignmentExpr {
    pub identifier: String,
    pub value: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Compound(Vec<Node>),
    VariabeDecleration(Box<VariabeDecleration>),
    AssignmentExpr(Box<AssignmentExpr>),
    BinOperator(Box<BinOperator>),
    UnaryOperator(Box<UnaryOperator>),
    Expression(Box<Node>),
}

impl fmt::Display for BinOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}
