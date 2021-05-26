use crate::{ast::*, parser::Parser, semantic_analyzer::SemanticAnalyzer, token::*, enviroment::*};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct Interpreter {
    env: Env,
    semantic_analyzer: SemanticAnalyzer,
}

pub type IResult = Result<Value, String>;

fn convert_f64_usize(x: f64) -> Result<usize, String> {
    let result = x as usize;
    if (result as f64 - x).abs() > 0.0 {
        Err(String::from("Cannot convert"))
    } else {
        Ok(result)
    }
}

fn to_bool(val: &Value) -> bool {
    match val {
        Value::Number(num) => num.ne(&0.0),
        Value::String(string) => !string.is_empty(),
        Value::Boolean(boolean) => *boolean,
        Value::Function(_) => true,
        Value::NoValue => false,
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Enviroment::new(None))),
            semantic_analyzer: SemanticAnalyzer::new(),
        }
    }

    fn number_operation(
        &mut self,
        operator: Operator,
        left: Value,
        right: Value,
        callback: fn(f64, f64) -> f64,
    ) -> IResult {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(callback(a, b))),
            (a, b) => Err(format!(
                "Expected Number for binary {:?}, got {:?}, {:?}",
                operator, a, b
            )),
        }
    }

    fn bool_operation(
        &mut self,
        operator: Operator,
        left: Value,
        right: Value,
        callback: fn(f64, f64) -> bool,
    ) -> IResult {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(callback(a, b))),
            (a, b) => Err(format!(
                "Expected Number for binary {:?}, got {:?}, {:?}",
                operator, a, b
            )),
        }
    }

    fn visit_bin_operator(&mut self, node: &BinOperator) -> IResult {
        let (left, right) = (
            self.visit_expression(&node.left)?,
            self.visit_expression(&node.right)?,
        );
        match node.operator {
            Operator::Plus => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (a, b) => Err(format!(
                    "Mismatched types for binary Add, got {:?} and {:?}",
                    a, b
                )),
            },
            Operator::Minus => self.number_operation(node.operator, left, right, |a, b| a - b),
            Operator::Mul => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                (Value::String(a), Value::Number(b)) | (Value::Number(b), Value::String(a)) => Ok(
                    Value::String(a.repeat(convert_f64_usize(b).map_err(|_| {
                        String::from(
                        "Can't multiply sequence by non-positive int of type float or negative int",
                    )
                    })?)),
                ),
                (a, b) => Err(format!(
                    "Mismatched types for binary Mul, got {:?} and {:?}",
                    a, b
                )),
            },
            Operator::Div => self.number_operation(node.operator, left, right, |a, b| a / b),
            Operator::Modulus => self.number_operation(node.operator, left, right, |a, b| a % b),
            Operator::Exponent => {
                self.number_operation(node.operator, left, right, |a, b| a.powf(b))
            }
            Operator::DoubleEqual => Ok(Value::Boolean(left == right)),
            Operator::NotEqual => Ok(Value::Boolean(left != right)),
            Operator::GreaterThan => self.bool_operation(node.operator, left, right, |a, b| a > b),
            Operator::GreaterThanOrEqual => {
                self.bool_operation(node.operator, left, right, |a, b| a >= b)
            }
            Operator::LessThan => self.bool_operation(node.operator, left, right, |a, b| a < b),
            Operator::LessThanOrEqual => {
                self.bool_operation(node.operator, left, right, |a, b| a <= b)
            }
            _ => Err(format!("Expected Operator, got {}.", node)),
        }
    }

    fn visit_unary_operator(&mut self, node: &UnaryOperator) -> IResult {
        match node.operator {
            Operator::Plus => self.visit_expression(&node.expression),
            Operator::Minus => match self.visit_expression(&node.expression)? {
                Value::Number(num) => Ok(Value::Number(-num)),
                other => Err(format!(
                    "Expected Number for Unary {:?}, got {:?}",
                    node.operator, other
                )),
            },
            Operator::Not => {
                let value = self.visit_expression(&node.expression)?;
                match value {
                    Value::Boolean(boolean) => Ok(Value::Boolean(!boolean)),
                    Value::String(_) => Ok(Value::Boolean(!to_bool(&value))),
                    Value::Number(_) => Ok(Value::Boolean(!to_bool(&value))),
                    other => Err(format!(
                        "Expected Number for Unary {:?}, got {:?}",
                        node.operator, other
                    )),
                }
            }
            _ => Err(format!(
                "Expected Unary Operator '+' or '-', got {}",
                node.operator
            )),
        }
    }

    fn visit_compound(&mut self, nodes: &[Node]) -> IResult {
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
                    self.semantic_analyzer
                        .scope
                        .borrow_mut()
                        .remove(&node.identifier);
                    return Err(err);
                }
            },
            None => Value::NoValue,
        };
        self.env.borrow_mut().define(&node.identifier, value);
        Ok(Value::NoValue)
    }

    fn visit_function_decleration(&mut self, node: &FunctionDecleration) -> IResult {
        let function = Value::Function(node.clone());
        self.env.borrow_mut().define(&node.name, function.clone());
        Ok(function)
    }

    fn visit_block(&mut self, nodes: &[Node]) -> IResult {
        let mut result = Value::NoValue;

        for node in nodes {
            match self.visit(&node)? {
                Value::NoValue => (),
                val => result = val,
            }
        }

        Ok(result)
    }

    fn visit_function_call(&mut self, node: &FunctionCall) -> IResult {
        match &node.function {
            Node::Identifier(identifier) => {
                let current_env = self.env.borrow().look_up(identifier, false);
                match current_env {
                    Some(value) => match value {
                        Value::Function(function) => {
                            self.env = Rc::new(RefCell::new(Enviroment::new(Some(Rc::clone(&self.env)))));

                            for (index, param) in function.params.iter().enumerate() {
                                let value = match node.arguments.get(index) {
                                    Some(node) => self.visit(node)?,
                                    None => Value::NoValue
                                };
                                self.env.borrow_mut().define(&param, value)
                            }

                            let result = self.visit(&function.block)?;

                            self.env = Rc::clone(
                                Rc::clone(&self.env)
                                    .borrow()
                                    .enclosing_enviroment
                                    .as_ref()
                                    .unwrap(),
                            );

                            Ok(result)
                        },
                        value => Err(format!("{:?} is not a function", value))
                    },
                    None => Err(format!("{} is not defined", identifier))
                }
            },
            node => Err(format!("{} is not a function", node)),
        }
    }

    fn visit_expression(&mut self, node: &Node) -> IResult {
        match node {
            Node::BinOperator(node) => self.visit_bin_operator(node),
            Node::Number(num) => Ok(Value::Number(*num)),
            Node::Boolean(boolean) => Ok(Value::Boolean(*boolean)),
            Node::String(string) => Ok(Value::String(string.clone())),
            Node::Identifier(iden) => self
                .env
                .borrow()
                .look_up(iden, false)
                .ok_or(format!("{} is not defined", iden)),
            Node::UnaryOperator(node) => self.visit_unary_operator(node),
            Node::AssignmentExpr(node) => self.visit_assignment(node),
            Node::FunctionCall(node) => self.visit_function_call(node),
            _ => Err(String::from("Invalid Syntax")),
        }
    }

    fn visit_assignment(&mut self, node: &AssignmentExpr) -> IResult {
        let value = self.visit_expression(&node.value)?;
        self.env.borrow_mut().assign(&node.identifier, value.clone())?;
        Ok(value)
    }

    fn visit(&mut self, node: &Node) -> IResult {
        match node {
            Node::Compound(nodes) => self.visit_compound(nodes),
            Node::VariabeDecleration(node) => self.visit_variable_decleration(node),
            Node::FunctionDecleration(function) => self.visit_function_decleration(function),
            Node::Block(nodes) => self.visit_block(nodes),
            Node::Expression(node) => self.visit_expression(node),
            node => self.visit_expression(node),
        }
    }

    pub fn interpret(&mut self, text: &str) -> IResult {
        let mut parser = Parser::new(text);
        let ast = parser.parse()?;
        self.semantic_analyzer.analyze(&ast)?;
        self.visit(&ast)
    }
}

#[test]
fn should_eval_math_expression() {
    let mut interpreter = Interpreter::new();
    let result = interpreter
        .interpret("((20 + 40) ** 20 * 2 - 10) / 10 % 100;")
        .unwrap();
    assert_eq!(result, Value::Number(56.0))
}

#[test]
fn should_handle_var_deceration() {
    let mut interpreter = Interpreter::new();
    let result = interpreter
        .interpret("let foo = 'Hello World!'; foo;")
        .unwrap();
    assert_eq!(result, Value::String(String::from("Hello World!")))
}

#[test]
fn should_handle_var_assignment() {
    let mut interpreter = Interpreter::new();
    let result = interpreter
        .interpret("let foo = 'Hello World'; foo += '!' * 10;")
        .unwrap();
    assert_eq!(result, Value::String(String::from("Hello World!!!!!!!!!!")))
}

#[test]
fn should_handle_comparison() {
    let mut interpreter = Interpreter::new();
    assert_eq!(
        interpreter.interpret("let foo = 20; foo == 20;").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        interpreter.interpret("let foo2 = 20; foo2 != 20;").unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        interpreter.interpret("let foo3 = 20; foo3 >= 10;").unwrap(),
        Value::Boolean(true)
    );
    assert_eq!(
        interpreter.interpret("let foo4 = 20; foo4 < 20;").unwrap(),
        Value::Boolean(false)
    )
}
