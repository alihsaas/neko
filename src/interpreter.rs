use crate::{ast::*, parser::Parser, semantic_analyzer::SemanticAnalyzer, token::*};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Interpreter {
    globals: HashMap<String, Value>,
    semantic_analyzer: SemanticAnalyzer,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    NoValue,
}

type IResult = Result<Value, String>;

fn convert_f64_usize(x: f64) -> Result<usize, String> {
    let result = x as usize;
    if result as f64 != x {
        Err(String::from("Cannot convert"))
    } else {
        Ok(result)
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            semantic_analyzer: SemanticAnalyzer::new(),
        }
    }

    fn is_number_operation(&mut self, node: &BinOperator) -> Result<bool, String> {
        if let (Value::Number(_), Value::Number(_)) =
            (self.visit(&node.left)?, self.visit(&node.right)?)
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn string_operation(
        &mut self,
        node: &BinOperator,
        callback: fn(&str, f64) -> Result<String, String>,
    ) -> IResult {
        match (self.visit(&node.left)?, self.visit(&node.right)?) {
            (Value::String(a), Value::Number(b)) => Ok(Value::String(callback(a.as_str(), b)?)),
            (Value::Number(a), Value::String(b)) => Ok(Value::String(callback(b.as_str(), a)?)),
            _ => Err(format!(
                "Expected Number for binary {:?}, got {:?}, {:?}",
                node.operator, node.left, node.right
            )),
        }
    }

    fn number_operation(&mut self, node: &BinOperator, callback: fn(f64, f64) -> f64) -> IResult {
        match (self.visit(&node.left)?, self.visit(&node.right)?) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(callback(a, b))),
            _ => Err(format!(
                "Expected Number for binary {:?}, got {:?}, {:?}",
                node.operator, node.left, node.right
            )),
        }
    }

    fn visit_bin_operator(&mut self, node: &BinOperator) -> IResult {
        match node.operator {
            Operator::Plus => self.number_operation(node, |a, b| a + b),
            Operator::Minus => self.number_operation(node, |a, b| a - b),
            Operator::Mul => {
                if self.is_number_operation(node)? {
                    self.number_operation(node, |a, b| a * b)
                } else {
                    self.string_operation(node, |a, b| {
                        Ok(a.repeat(convert_f64_usize(b).or(Err(String::from(
                            "Can't multiply sequence by non-positive int of type float or negative int",
                        )))?))
                    })
                }
            }
            Operator::Div => self.number_operation(node, |a, b| a / b),
            Operator::Modulus => self.number_operation(node, |a, b| a % b),
            Operator::Exponent => self.number_operation(node, |a, b| a.powf(b)),
            _ => Err(format!("Expected Operator, got {}.", node)),
        }
    }

    fn visit_unary_operator(&mut self, node: &UnaryOperator) -> IResult {
        match node.operator {
            Operator::Plus => self.visit(&node.expression),
            Operator::Minus => match self.visit(&node.expression)? {
                Value::Number(num) => Ok(Value::Number(-num)),
                other => Err(format!(
                    "Expected Number for Unary {:?}, got {:?}",
                    node.operator, other
                )),
            },
            _ => Err(format!(
                "Expected Unary Operator '+' or '-', got {}",
                node.operator
            )),
        }
    }

    fn visit_compound(&mut self, nodes: &Vec<Node>) -> IResult {
        let mut result = Value::NoValue;

        for node in nodes {
            match self.visit(&node)? {
                Value::NoValue => (),
                val => result = val,
            }
        }

        Ok(result)
    }

    fn visit_variable_decleration(&mut self, node: &VariabeDecleration) -> IResult {
        let value = match &node.value {
            Some(value_node) => match self.visit(value_node) {
                Ok(val) => val,
                Err(err) => {
                    self.semantic_analyzer.symbol_table.remove(&node.identifier);
                    return Err(err);
                }
            },
            None => Value::NoValue,
        };
        self.globals.insert(node.identifier.clone(), value);
        Ok(Value::NoValue)
    }

    fn visit_expression(&mut self, node: &Node) -> IResult {
        match node {
            Node::BinOperator(node) => self.visit_bin_operator(node),
            Node::Number(num) => Ok(Value::Number(*num)),
            Node::Boolean(boolean) => Ok(Value::Boolean(*boolean)),
            Node::String(string) => Ok(Value::String(string.clone())),
            Node::Identifier(iden) => self
                .globals
                .get(iden)
                .map(|val| val.clone())
                .ok_or(format!("{} is not defined", iden)),
            Node::UnaryOperator(node) => self.visit_unary_operator(node),
            Node::AssignmentExpr(node) => self.visit_assignment(node),
            _ => Err(String::from("Invalid Syntax")),
        }
    }

    fn visit_assignment(&mut self, node: &AssignmentExpr) -> IResult {
        let value = self.visit(&node.value)?;
        self.globals.insert(node.identifier.clone(), value.clone());
        Ok(value.clone())
    }

    fn visit(&mut self, node: &Node) -> IResult {
        match node {
            Node::Compound(nodes) => self.visit_compound(nodes),
            Node::VariabeDecleration(node) => self.visit_variable_decleration(node),
            Node::Expression(node) => self.visit_expression(node),
            node => self.visit_expression(node),
        }
    }

    pub fn interpret(&mut self, text: &str) -> IResult {
        let mut parser = Parser::new(text);
        let ast = parser.parse()?;
        self.semantic_analyzer.visit(&ast)?;
        self.visit(&ast)
    }
}
