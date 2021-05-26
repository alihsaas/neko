use std::{cell::RefCell, rc::Rc};

use crate::{ast::*, symbol::*, symbol_table::SymbolTable};

type SResult = Result<(), String>;

#[derive(Debug)]
pub struct SemanticAnalyzer {
    pub scope: Rc<RefCell<SymbolTable>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scope: Rc::new(RefCell::new(SymbolTable::new("global", 1, None))),
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
        if self.scope.borrow().look_up(&node.identifier, false).is_some() {
            self.visit(&node.value)?;
            Ok(())
        } else {
            Err(format!(
                "Cannot find value '{}' in this scope",
                &node.identifier
            ))
        }
    }

    fn visit_variable_decleration(&mut self, node: &VariabeDecleration) -> SResult {
        if self.scope.borrow().look_up(&node.identifier, true).is_some() {
            Err(format!("Duplicate variable {}", &node.identifier))
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

    fn visit_function_call(&mut self, node: &FunctionCall) -> SResult {
        if let Node::Identifier(identifier) = &node.function {
            if let Some(symbol) = self.scope.borrow().look_up(identifier, false) {
                if let Symbol::FunctionSymbol(symbol) = symbol {
                    if symbol.param.len() != node.arguments.len() {
                        Err(format!("Expected {} number of arguments got {}", symbol.param.len(), node.arguments.len()))
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
    }

    fn visit_expression(&mut self, node: &Node) -> SResult {
        match node {
            Node::BinOperator(node) => self.visit_bin_operator(node),
            Node::Number(_) => Ok(()),
            Node::Boolean(_) => Ok(()),
            Node::String(_) => Ok(()),
            Node::Identifier(iden) => self
                .scope
                .borrow()
                .look_up(iden, false)
                .and(Some(()))
                .ok_or(format!("{} is not defined", iden)),
            Node::UnaryOperator(node) => self.visit_unary_operation(node),
            Node::AssignmentExpr(node) => self.visit_assignment(node),
            Node::FunctionCall(node) => self.visit_function_call(node),
            _ => Err(String::from("Invalid Syntax")),
        }
    }

    fn visit_function_decleration(&mut self, node: &FunctionDecleration) -> SResult {
        let function_name = &node.name;
        if self.scope.borrow().look_up(function_name, true).is_none() {
            self.scope.borrow_mut().insert(&function_name, Symbol::FunctionSymbol(FunctionSymbol {
                name: function_name.clone(),
                param: node.params.clone(),
            }));
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
            Err(format!("Duplicate variable {}", function_name))
        }
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
        let result = self.visit(node);
        result
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
