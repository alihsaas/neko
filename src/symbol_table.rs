use crate::symbol::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>,
    pub scope_name: String,
    pub scope_level: u64,
    pub enclosing_scope: Option<Rc<RefCell<SymbolTable>>>,
}

impl SymbolTable {
    pub fn new(
        scope_name: &str,
        scope_level: u64,
        enclosing_scope: Option<Rc<RefCell<SymbolTable>>>,
    ) -> Self {
        Self {
            symbols: HashMap::new(),
            scope_name: scope_name.to_string(),
            scope_level,
            enclosing_scope,
        }
    }

    pub fn insert(&mut self, name: &str, symbol: Symbol) {
        self.symbols.insert(name.to_string(), symbol);
    }

    pub fn look_up(&self, name: &str, current_scope_only: bool) -> Option<Symbol> {
        self.symbols.get(name).cloned().or_else(|| {
            if current_scope_only {
                None
            } else {
                self.enclosing_scope
                    .as_ref()
                    .and_then(|scope| scope.borrow().look_up(name, false))
            }
        })
    }

    pub fn remove(&mut self, name: &str) {
        self.symbols.remove(name);
    }
}
