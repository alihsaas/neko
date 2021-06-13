use crate::{
    ast::{FunctionDecleration, Lambda},
    misc::NekoError,
};
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

pub type Env = Rc<RefCell<Enviroment>>;

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Function(FunctionDecleration),
    Lambda(Lambda),
    BuiltIn {
        name: String,
        function: fn(args: Vec<Value>) -> Result<Value, NekoError>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Object(Rc<RefCell<HashMap<String, Box<Value>>>>),
    Function(FunctionType, Env),
    String(String),
    None,
}

#[derive(Debug, PartialEq)]
pub struct Enviroment {
    values: HashMap<String, Value>,
    pub enclosing_enviroment: Option<Env>,
}

impl Enviroment {
    pub fn new(enclosing_enviroment: Option<Env>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing_enviroment,
        }
    }

    pub fn look_up(&self, name: &str, current_env_only: bool) -> Option<Value> {
        self.values.get(name).cloned().or_else(|| {
            if current_env_only {
                None
            } else {
                self.enclosing_enviroment
                    .as_ref()
                    .and_then(|env| env.borrow().look_up(name, false))
            }
        })
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some(val) = self.values.get_mut(name) {
            *val = value;
            Ok(())
        } else if let Some(env) = &self.enclosing_enviroment {
            env.borrow_mut().assign(name, value)
        } else {
            Err(format!("Attempt to assign to undefined variable {}", name))
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }
}

impl Value {
    pub fn stringify(&self) -> String {
        match self {
            Value::Number(num) => num.to_string(),
            Value::Boolean(boolean) => boolean.to_string(),
            Value::String(string) => string.to_string(),
            Value::Function(function_type, _) => match function_type {
                FunctionType::Function(function) => format!("[Function: {}]", function.name),
                FunctionType::Lambda(_) => String::from("[Function: (lambda)]"),
                FunctionType::BuiltIn { name, .. } => format!("[Built-In Function: {}]", name),
            },
            Value::Object(obj) => {
                let mut result = String::from("{");
                result.push_str(
                    &obj.borrow()
                        .iter()
                        .map(|(key, value)| format!(" {}: {}", key, &value.to_string()))
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                result.push('}');
                result
            }
            Value::None => String::from("none"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.stringify())
    }
}
