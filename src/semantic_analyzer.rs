use crate::{ast::*, symbol::*};
use std::collections::HashMap;

type SResult = Result<(), String>;

#[derive(Debug)]
pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, Symbol>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
        }
    }

    fn visit_compound(&mut self, nodes: &Vec<Node>) -> SResult {
        for node in nodes {
            self.visit(&node)?
        }

        Ok(())
    }

    fn visit_assignment(&mut self, node: &AssignmentExpr) -> SResult {
        if let Some(_) = self.symbol_table.get(&node.identifier) {
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
        if let Some(_) = self.symbol_table.get(&node.identifier) {
            Err(format!("Duplicate variable {}", &node.identifier))
        } else {
            self.symbol_table.insert(
                node.identifier.clone(),
                Symbol::VarSymbol(Box::new(VarSymbol {
                    name: node.identifier.clone(),
                    symbol_type: TypeSymbol::Unknown,
                })),
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

    fn visit_expression(&mut self, node: &Node) -> SResult {
        match node {
            Node::BinOperator(node) => self.visit_bin_operator(node),
            Node::Number(_) => Ok(()),
            Node::String(iden) => self
                .symbol_table
                .get(iden)
                .and(Some(()))
                .ok_or(format!("{} is not defined", iden)),
            Node::UnaryOperator(node) => self.visit_unary_operation(node),
            Node::AssignmentExpr(node) => self.visit_assignment(node),
            _ => Err(String::from("Invalid Syntax")),
        }
    }

    pub fn visit(&mut self, node: &Node) -> SResult {
        match node {
            Node::Compound(nodes) => self.visit_compound(nodes),
            Node::VariabeDecleration(node) => self.visit_variable_decleration(node),
            Node::Expression(node) => self.visit_expression(node),
            node => self.visit_expression(node),
        }
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
        semantic_analyzer.visit(&ast).unwrap();
    }

    #[test]
    #[should_panic]
    fn should_catch_undefined_variable_assignment() {
        let mut parser = Parser::new("w = 20;");
        let mut semantic_analyzer = SemanticAnalyzer::new();
        let ast = parser.parse().unwrap();
        semantic_analyzer.visit(&ast).unwrap();
    }

    #[test]
    #[should_panic]
    fn should_catch_usage_of_undefined_variable() {
        let mut parser = Parser::new("20 + -w;");
        let mut semantic_analyzer = SemanticAnalyzer::new();
        let ast = parser.parse().unwrap();
        semantic_analyzer.visit(&ast).unwrap();
    }
}