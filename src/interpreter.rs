use crate::{
    ast::*, enviroment::*, interpreter_option::InterpreterOptions, misc::NekoError, parser::Parser,
    semantic_analyzer::SemanticAnalyzer, token::*,
};
use ansi_term::Colour;
use std::{cell::RefCell, rc::Rc};

pub type IResult = Result<Value, NekoError>;

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
        Value::Function(..) => true,
        Value::None => false,
    }
}

pub fn loggable_value(val: &Value) -> String {
    match val {
        Value::Number(num) => num.to_string(),
        Value::Boolean(boolean) => boolean.to_string(),
        Value::String(string) => string.to_string(),
        Value::Function(function_type, _) => match function_type {
            FunctionType::Function(function) => format!("[Function: {}]", function.name),
            FunctionType::Lambda(_) => String::from("[Function: (lambda)]"),
            FunctionType::BuiltIn { name, .. } => format!("[Built-In Function: {}]", name),
        },
        Value::None => String::from("none"),
    }
}

pub fn colored_output(val: &Value) -> String {
    match val {
        Value::Number(num) => format!("{}", Colour::Yellow.paint(num.to_string())),
        Value::Boolean(boolean) => format!("{}", Colour::Yellow.paint(boolean.to_string())),
        Value::String(string) => format!("{}", Colour::Green.paint(format!("{:?}", string))),
        Value::Function(function_type, _) => format!(
            "{}",
            match function_type {
                FunctionType::Function(function) =>
                    Colour::Green.paint(format!("[Function: {}]", function.name)),
                FunctionType::Lambda(_) => Colour::Green.paint("[Function: (lambda)]"),
                FunctionType::BuiltIn { name, .. } =>
                    Colour::Green.paint(format!("[Built-In Function: {}]", name)),
            }
        ),
        Value::None => Colour::RGB(128, 127, 113).paint("none").to_string(),
    }
}

#[derive(Debug)]
pub struct Interpreter {
    env: Env,
    semantic_analyzer: SemanticAnalyzer,
    interpreter_options: InterpreterOptions,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            env: Rc::new(RefCell::new(Enviroment::new(None))),
            semantic_analyzer: SemanticAnalyzer::new(),
            interpreter_options: InterpreterOptions::new(),
        };
        interpreter.set_up_env();
        interpreter
    }

    fn set_up_env(&mut self) {
        let built_in = vec![
            Value::Function(
                FunctionType::BuiltIn {
                    name: String::from("print"),
                    function: |args| {
                        println!(
                            "{}",
                            args.iter()
                                .map(colored_output)
                                .collect::<Vec<String>>()
                                .join(" ")
                        );
                        Ok(Value::None)
                    },
                },
                Rc::clone(&self.env),
            ),
            Value::Function(
                FunctionType::BuiltIn {
                    name: String::from("error"),
                    function: |args| {
                        if let Some(val) = args.first() {
                            Err(NekoError::UnknownError(loggable_value(val)))
                        } else {
                            Err(NekoError::TypeError(String::from("Expect value got none.")))
                        }
                    },
                },
                Rc::clone(&self.env),
            ),
        ];

        for built in built_in {
            match built {
                Value::Function(FunctionType::BuiltIn { ref name, .. }, _) => {
                    self.env.borrow_mut().define(&name, built.clone())
                }
                _ => unreachable!(),
            }
        }
    }

    fn number_operation(
        &mut self,
        operator: &Token,
        left: Value,
        right: Value,
        callback: fn(f64, f64) -> f64,
    ) -> IResult {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(callback(a, b))),
            (a, b) => Err(NekoError::TypeError(format!(
                "Expected Number for binary {:?}, got {:?}, {:?}",
                operator, a, b
            ))),
        }
    }

    fn bool_operation(
        &mut self,
        operator: &Token,
        left: Value,
        right: Value,
        callback: fn(f64, f64) -> bool,
    ) -> IResult {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(callback(a, b))),
            (a, b) => Err(NekoError::TypeError(format!(
                "Expected Number for binary {:?}, got {:?}, {:?}",
                operator, a, b
            ))),
        }
    }

    fn visit_bin_operator(&mut self, node: &BinOperator) -> IResult {
        let (left, right) = (
            self.visit_expression(&node.left)?,
            self.visit_expression(&node.right)?,
        );
        match node.operator {
            Token::Operator(Operator::Plus) => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                (a, b) => Err(NekoError::TypeError(format!(
                    "Mismatched types for binary Add, got {:?} and {:?}",
                    a, b
                ))),
            },
            Token::Operator(Operator::Minus) => {
                self.number_operation(&node.operator, left, right, |a, b| a - b)
            }
            Token::Operator(Operator::Mul) => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                (Value::String(a), Value::Number(b)) | (Value::Number(b), Value::String(a)) => Ok(
                    Value::String(a.repeat(convert_f64_usize(b).map_err(|_| {
                        NekoError::TypeError(
                        String::from("Can't multiply sequence by non-positive int of type float or negative int"),
                    )
                    })?)),
                ),
                (a, b) => Err(NekoError::TypeError(format!(
                    "Mismatched types for binary Mul, got {:?} and {:?}",
                    a, b
                ))),
            },
            Token::Operator(Operator::Div) => {
                self.number_operation(&node.operator, left, right, |a, b| a / b)
            }
            Token::Operator(Operator::Modulus) => {
                self.number_operation(&node.operator, left, right, |a, b| a % b)
            }
            Token::Operator(Operator::Exponent) => {
                self.number_operation(&node.operator, left, right, |a, b| a.powf(b))
            }
            Token::Operator(Operator::DoubleEqual) => Ok(Value::Boolean(left == right)),
            Token::Operator(Operator::NotEqual) => Ok(Value::Boolean(left != right)),
            Token::Operator(Operator::GreaterThan) => {
                self.bool_operation(&node.operator, left, right, |a, b| a > b)
            }
            Token::Operator(Operator::GreaterThanOrEqual) => {
                self.bool_operation(&node.operator, left, right, |a, b| a >= b)
            }
            Token::Operator(Operator::LessThan) => {
                self.bool_operation(&node.operator, left, right, |a, b| a < b)
            }
            Token::Operator(Operator::LessThanOrEqual) => {
                self.bool_operation(&node.operator, left, right, |a, b| a <= b)
            }
            Token::Keyword(Keyword::Or) => {
                if to_bool(&left) {
                    Ok(left)
                } else {
                    Ok(right)
                }
            }
            Token::Keyword(Keyword::And) => {
                if !to_bool(&left) {
                    Ok(left)
                } else {
                    Ok(right)
                }
            }
            _ => Err(NekoError::SyntaxError(format!("Expected Operator, got {}.", node))),
        }
    }

    fn visit_unary_operator(&mut self, node: &UnaryOperator) -> IResult {
        match node.operator {
            Token::Operator(Operator::Plus) => self.visit_expression(&node.expression),
            Token::Operator(Operator::Minus) => match self.visit_expression(&node.expression)? {
                Value::Number(num) => Ok(Value::Number(-num)),
                other => Err(NekoError::TypeError(format!(
                    "Expected Number for Unary {:?}, got {:?}",
                    node.operator, other
                ))),
            },
            Token::Operator(Operator::Not) => {
                let value = self.visit_expression(&node.expression)?;
                match value {
                    Value::Boolean(boolean) => Ok(Value::Boolean(!boolean)),
                    Value::String(_) => Ok(Value::Boolean(!to_bool(&value))),
                    Value::Number(_) => Ok(Value::Boolean(!to_bool(&value))),
                    other => Err(NekoError::TypeError(format!(
                        "Expected Number for Unary {:?}, got {:?}",
                        node.operator, other
                    ))),
                }
            }
            _ => Err(NekoError::SyntaxError(format!(
                "Expected Unary Operator '+' or '-', got {}",
                node.operator
            ))),
        }
    }

    fn visit_compound(&mut self, nodes: &[Node]) -> IResult {
        let mut result = Value::None;

        for node in nodes {
            match self.visit(&node)? {
                Value::None => (),
                val => result = val,
            }
        }

        Ok(result)
    }

    fn visit_variable_decleration(&mut self, node: &VariabeDecleration) -> IResult {
        if !self.interpreter_options.disable_decleration {
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
                None => Value::None,
            };
            self.env.borrow_mut().define(&node.identifier, value);
        }
        Ok(Value::None)
    }

    fn visit_function_decleration(&mut self, node: &FunctionDecleration) -> IResult {
        if !self.interpreter_options.disable_decleration {
            let function =
                Value::Function(FunctionType::Function(node.clone()), Rc::clone(&self.env));
            self.env.borrow_mut().define(&node.name, function);
        }
        Ok(Value::None)
    }

    fn visit_lambda_decleration(&mut self, node: &Lambda) -> IResult {
        let function = Value::Function(FunctionType::Lambda(node.clone()), Rc::clone(&self.env));
        self.env.borrow_mut().define(&node.id, function.clone());
        Ok(function)
    }

    fn visit_block(&mut self, nodes: &[Node]) -> IResult {
        let mut result = Value::None;

        for node in nodes {
            match self.visit(&node)? {
                Value::None => (),
                val => result = val,
            }
        }

        Ok(result)
    }

    fn function_call(
        &mut self,
        node: &FunctionCall,
        params: &[String],
        block: &Node,
        closure: Env,
    ) -> IResult {
        self.env = Rc::new(RefCell::new(Enviroment::new(Some(closure))));

        for (index, param) in params.iter().enumerate() {
            let value = match node.arguments.get(index) {
                Some(node) => self.visit(node)?,
                None => Value::None,
            };
            self.env.borrow_mut().define(&param, value)
        }

        let result = self.visit(&block)?;

        self.env = Rc::clone(
            Rc::clone(&self.env)
                .borrow()
                .enclosing_enviroment
                .as_ref()
                .unwrap(),
        );

        Ok(result)
    }

    fn handle_function(&mut self, node: &FunctionCall, value: Value) -> IResult {
        match value {
            Value::Function(FunctionType::Function(function), closure) => {
                self.function_call(node, &function.params, &function.block, closure)
            }
            Value::Function(FunctionType::Lambda(lambda), closure) => {
                self.function_call(node, &lambda.params, &lambda.block, closure)
            }
            Value::Function(FunctionType::BuiltIn { name: _, function }, _) => {
                let mut args = vec![];
                for arg in &node.arguments {
                    args.push(self.visit(&arg)?)
                }
                Ok(function(args)?)
            }
            value => Err(NekoError::TypeError(format!(
                "{:?} is not a function",
                value
            ))),
        }
    }

    fn visit_function_call(&mut self, node: &FunctionCall) -> IResult {
        if !self.interpreter_options.disable_calls {
            match &node.function {
                Node::Identifier(identifier) => {
                    let current_env = self.env.borrow().look_up(identifier, false);
                    match current_env {
                        Some(value) => self.handle_function(node, value),
                        None => Err(NekoError::ReferenceError(format!(
                            "{} is not defined",
                            identifier
                        ))),
                    }
                }
                Node::FunctionCall(call) => {
                    let result = self.visit_function_call(call)?;
                    self.handle_function(node, result)
                }
                Node::Lambda(lambda) => {
                    self.function_call(node, &lambda.params, &lambda.block, Rc::clone(&self.env))
                }
                node => Err(NekoError::TypeError(format!("{} is not a function", node))),
            }
        } else {
            Err(NekoError::UnknownError(String::from("Calls Disabled")))
        }
    }

    fn visit_expression(&mut self, node: &Node) -> IResult {
        match node {
            Node::BinOperator(node) => self.visit_bin_operator(node),
            Node::Number(num) => Ok(Value::Number(*num)),
            Node::Boolean(boolean) => Ok(Value::Boolean(*boolean)),
            Node::String(string) => Ok(Value::String(string.clone())),
            Node::None => Ok(Value::None),
            Node::Identifier(iden) => self
                .env
                .borrow()
                .look_up(iden, false)
                .ok_or_else(|| NekoError::ReferenceError(format!("{} is not defined", iden))),
            Node::UnaryOperator(node) => self.visit_unary_operator(node),
            Node::AssignmentExpr(node) => self.visit_assignment(node),
            Node::FunctionCall(node) => self.visit_function_call(node),
            Node::Lambda(lambda) => self.visit_lambda_decleration(lambda),
            _ => Err(NekoError::SyntaxError(String::from("Invalid Syntax"))),
        }
    }

    fn visit_assignment(&mut self, node: &AssignmentExpr) -> IResult {
        let value = self.visit_expression(&node.value)?;
        self.env
            .borrow_mut()
            .assign(&node.identifier, value.clone())
            .map_err(NekoError::ReferenceError)?;
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
        self.interpreter_options = InterpreterOptions::new();
        let mut parser = Parser::new(text);
        let ast = parser.parse()?;
        self.semantic_analyzer
            .analyze_with_options(&ast, &self.interpreter_options)?;
        self.visit(&ast)
    }

    pub fn interpret_with_option(&mut self, text: &str, option: &InterpreterOptions) -> IResult {
        self.interpreter_options = option.clone();
        let mut parser = Parser::new(text);
        let ast = parser.parse()?;
        self.semantic_analyzer
            .analyze_with_options(&ast, &self.interpreter_options)?;
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
