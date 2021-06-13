use crate::token::*;
use std::{collections::HashMap, fmt};

#[derive(Debug, PartialEq, Clone)]
pub struct BinOperator {
    pub left: Node,
    pub operator: Token,
    pub right: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryOperator {
    pub operator: Token,
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
pub struct FunctionDecleration {
    pub name: String,
    pub params: Vec<String>,
    pub block: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Lambda {
    pub id: String,
    pub params: Vec<String>,
    pub block: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub function: Node,
    pub arguments: Vec<Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub values: HashMap<String, Node>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Index {
    pub target: Node,
    pub key: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetPropertyExpr {
    pub target: Node,
    pub key: String,
    pub value: Node,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Compound(Vec<Node>),
    Block(Vec<Node>),
    Lambda(Box<Lambda>),
    Object(Box<Object>),
    None,
    Index(Box<Index>),
    FunctionDecleration(Box<FunctionDecleration>),
    FunctionCall(Box<FunctionCall>),
    VariabeDecleration(Box<VariabeDecleration>),
    AssignmentExpr(Box<AssignmentExpr>),
    SetPropertyExpr(Box<SetPropertyExpr>),
    BinOperator(Box<BinOperator>),
    UnaryOperator(Box<UnaryOperator>),
    Expression(Box<Node>),
}

impl fmt::Display for BinOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

fn join_nodes(node: &[Node]) -> String {
    node.iter()
        .map(|node| format!("{}", node))
        .collect::<Vec<_>>()
        .join(", ")
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&match self {
            Node::Number(num) => num.to_string(),
            Node::String(string) => format!("'{}'", string.to_string()),
            Node::Boolean(boolean) => boolean.to_string(),
            Node::Identifier(iden) => iden.to_string(),
            Node::Compound(statments) => format!("[{}]", join_nodes(&statments)),
            Node::Block(block) => format!("[{}]", join_nodes(&block)),
            Node::Lambda(_) => String::from("Lambda"),
            Node::Object(_) => String::from("Object"),
            Node::Index(index) => format!("{}", index),
            Node::FunctionDecleration(_) => String::from("FunctionDecleration"),
            Node::FunctionCall(function_call) => format!(
                "{}({})",
                function_call.function,
                join_nodes(&function_call.arguments)
            ),
            Node::VariabeDecleration(variable_decleration) => {
                if let Some(val) = &variable_decleration.value {
                    format!("let {} = {};", variable_decleration.identifier, val)
                } else {
                    format!("let {};", variable_decleration.identifier)
                }
            }
            Node::AssignmentExpr(assignment) => {
                format!("{} = {};", assignment.identifier, assignment.value)
            }
            Node::SetPropertyExpr(set_property) => format!(
                "{}.{} = {};",
                set_property.target, set_property.key, set_property.value
            ),
            Node::BinOperator(bin_operation) => format!(
                "{} {} {}",
                bin_operation.left, bin_operation.operator, bin_operation.right
            ),
            Node::UnaryOperator(unary_operation) => {
                format!("{}{}", unary_operation.operator, unary_operation.expression)
            }
            Node::Expression(expression) => format!("{}", expression),
            Node::None => String::from("none"),
        })
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}.{}", &self.target, &self.key))
    }
}
