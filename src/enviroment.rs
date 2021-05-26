use crate::ast::FunctionDecleration;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    Function(FunctionDecleration),
    String(String),
    NoValue,
}


pub type Env = Rc<RefCell<Enviroment>>;

#[derive(Debug)]
pub struct Enviroment {
    values: HashMap<String, Value>,
    pub enclosing_enviroment: Option<Env>,
}

impl Enviroment {

    pub fn new(enclosing_enviroment: Option<Env>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing_enviroment
        }
    }

    pub fn look_up(&self, name: &str, current_env_only: bool) -> Option<Value> {
        self.values
            .get(name)
            .map(|value| value.clone())
            .or_else(|| if current_env_only { None } else { 
                self.enclosing_enviroment.as_ref()
                    .and_then(|env| env.borrow().look_up(name, false))
                    .map(|value| value.clone())
            })
    }


    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some(val) = self.values.get_mut(name) {
            *val = value;
            Ok(())
        } else {
            if let Some(env) = &self.enclosing_enviroment {
                env.borrow_mut().assign(name, value)
            } else {
                Err(format!("Attempt to assign to undefined variable {}", name))
            }
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }
}