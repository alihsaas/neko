use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::*, interpreter_option::InterpreterOptions, misc::NekoError, symbol::*,
    symbol_table::SymbolTable,
};

type SResult = Result<(), NekoError>;

#[derive(Debug)]
pub struct SemanticAnalyzer {
    pub scope: Rc<RefCell<SymbolTable>>,
    interpreter_options: InterpreterOptions,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let scope = Rc::new(RefCell::new(SymbolTable::new("global", 1, None)));
        let built_in = vec![Symbol::BuiltInSymbol(String::from("print")), Symbol::BuiltInSymbol(String::from("error"))];

        for built in built_in {
            match built {
                Symbol::BuiltInSymbol(ref name) => scope.borrow_mut().insert(&name, built.clone()),
                _ => unreachable!(),
            }
        }

        Self {
            scope,
            interpreter_options: InterpreterOptions::new(),
        }
    }

    fn visit_compound(&mut self, nodes: &[Node]) -> SResult {
        for node in nodes {
            self.visit(&node)?
        }

        Ok(())
    }

    fn visit_block(&mut self, nodes: &[Node]) -> SResult {
        for node in nodes {
            self.visit(&node)?
        }

        Ok(())
    }

    fn visit_assignment(&mut self, node: &AssignmentExpr) -> SResult {
        if self
            .scope
            .borrow()
            .look_up(&node.identifier, false)
            .is_some()
        {
            self.visit(&node.value)?;
            Ok(())
        } else {
            Err(NekoError::ReferenceError(format!(
                "Cannot find value '{}' in this scope",
                &node.identifier
            )))
        }
    }

    fn visit_variable_decleration(&mut self, node: &VariabeDecleration) -> SResult {
        if !self.interpreter_options.disable_decleration {
            if self
                .scope
                .borrow()
                .look_up(&node.identifier, true)
                .is_some()
            {
                Err(NekoError::SyntaxError(format!(
                    "Duplicate variable {}",
                    &node.identifier
                )))
            } else {
                self.scope.borrow_mut().insert(
                    &node.identifier,
                    Symbol::VarSymbol(VarSymbol {
                        name: node.identifier.clone(),
                        symbol_type: TypeSymbol::Unknown,
                    }),
                );
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn visit_bin_operator(&mut self, node: &BinOperator) -> SResult {
        self.visit(&node.right)?;
        self.visit(&node.left)?;
        Ok(())
    }

    fn visit_unary_operation(&mut self, node: &UnaryOperator) -> SResult {
        self.visit(&node.expression)?;
        Ok(())
    }

    fn visit_function_call(&mut self, _node: &FunctionCall) -> SResult {
        /*
        if let Node::Identifier(identifier) = &node.function {
            if let Some(symbol) = self.scope.borrow().look_up(identifier, false) {
                if let Symbol::FunctionSymbol(symbol) = symbol {
                    if symbol.param.len() != node.arguments.len() {
                        Err(format!(
                            "Expected {} number of arguments got {}",
                            symbol.param.len(),
                            node.arguments.len()
                        ))
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("Attempt to call non-function {:?}", symbol))
                }
            } else {
                Err(format!("Attempt to call undefined function {}", identifier))
            }
        } else {
            Ok(())
        }
        */
        Ok(())
    }

    fn visit_expression(&mut self, node: &Node) -> SResult {
        match node {
            Node::BinOperator(node) => self.visit_bin_operator(node),
            Node::Number(_) => Ok(()),
            Node::Boolean(_) => Ok(()),
            Node::String(_) => Ok(()),
            Node::Object(_) => Ok(()),
            Node::None => Ok(()),
            Node::Identifier(iden) => self
                .scope
                .borrow()
                .look_up(iden, false)
                .and(Some(()))
                .ok_or_else(|| NekoError::ReferenceError(format!("{} is not defined", iden))),
            Node::UnaryOperator(node) => self.visit_unary_operation(node),
            Node::AssignmentExpr(node) => self.visit_assignment(node),
            Node::SetPropertyExpr(_) => Ok(()),
            Node::FunctionCall(node) => self.visit_function_call(node),
            Node::Lambda(lambda) => self.visit_lambda(lambda),
            Node::Index(_) => Ok(()),
            _ => Err(NekoError::SyntaxError(String::from("Invalid Syntax"))),
        }
    }

    fn visit_function_decleration(&mut self, node: &FunctionDecleration) -> SResult {
        if !self.interpreter_options.disable_decleration {
            let function_name = &node.name;
            if self.scope.borrow().look_up(function_name, true).is_none() {
                self.scope.borrow_mut().insert(
                    &function_name,
                    Symbol::FunctionSymbol(FunctionSymbol {
                        name: function_name.clone(),
                        param: node.params.clone(),
                    }),
                );
                let level = self.scope.borrow().scope_level + 1;
                self.scope = Rc::new(RefCell::new(SymbolTable::new(
                    &function_name,
                    level,
                    Some(Rc::clone(&self.scope)),
                )));

                for param in &node.params {
                    self.scope.borrow_mut().insert(
                        &param,
                        Symbol::VarSymbol(VarSymbol {
                            name: param.to_string(),
                            symbol_type: TypeSymbol::Unknown,
                        }),
                    );
                }

                self.visit(&node.block)?;

                self.scope = Rc::clone(
                    Rc::clone(&self.scope)
                        .borrow()
                        .enclosing_scope
                        .as_ref()
                        .unwrap(),
                );

                Ok(())
            } else {
                Err(NekoError::SyntaxError(format!(
                    "Duplicate variable {}",
                    function_name
                )))
            }
        } else {
            Ok(())
        }
    }

    fn visit_lambda(&mut self, node: &Lambda) -> SResult {
        let id = &node.id;
        self.scope.borrow_mut().insert(
            &id,
            Symbol::FunctionSymbol(FunctionSymbol {
                name: id.clone(),
                param: node.params.clone(),
            }),
        );
        let level = self.scope.borrow().scope_level + 1;
        self.scope = Rc::new(RefCell::new(SymbolTable::new(
            &id,
            level,
            Some(Rc::clone(&self.scope)),
        )));

        for param in &node.params {
            self.scope.borrow_mut().insert(
                &param,
                Symbol::VarSymbol(VarSymbol {
                    name: param.to_string(),
                    symbol_type: TypeSymbol::Unknown,
                }),
            );
        }

        self.visit(&node.block)?;

        self.scope = Rc::clone(
            Rc::clone(&self.scope)
                .borrow()
                .enclosing_scope
                .as_ref()
                .unwrap(),
        );

        Ok(())
    }

    fn visit(&mut self, node: &Node) -> SResult {
        match node {
            Node::Compound(nodes) => self.visit_compound(nodes),
            Node::VariabeDecleration(node) => self.visit_variable_decleration(node),
            Node::FunctionDecleration(node) => self.visit_function_decleration(node),
            Node::Expression(node) => self.visit_expression(node),
            Node::Block(nodes) => self.visit_block(nodes),
            node => self.visit_expression(node),
        }
    }

    pub fn analyze(&mut self, node: &Node) -> SResult {
        self.interpreter_options = InterpreterOptions::new();
        self.visit(node)
    }

    pub fn analyze_with_options(&mut self, node: &Node, option: &InterpreterOptions) -> SResult {
        self.interpreter_options = option.clone();
        self.visit(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    #[should_panic]
    fn should_catch_duplicate_decleration() {
        let mut parser = Parser::new("let w = 20; let w = 20;");
        let mut semantic_analyzer = SemanticAnalyzer::new();
        let ast = parser.parse().unwrap();
        semantic_analyzer.analyze(&ast).unwrap();
    }

    #[test]
    #[should_panic]
    fn should_catch_undefined_variable_assignment() {
        let mut parser = Parser::new("w = 20;");
        let mut semantic_analyzer = SemanticAnalyzer::new();
        let ast = parser.parse().unwrap();
        semantic_analyzer.analyze(&ast).unwrap();
    }

    #[test]
    #[should_panic]
    fn should_catch_usage_of_undefined_variable() {
        let mut parser = Parser::new("20 + -w;");
        let mut semantic_analyzer = SemanticAnalyzer::new();
        let ast = parser.parse().unwrap();
        semantic_analyzer.analyze(&ast).unwrap();
    }
}
