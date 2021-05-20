use crate::{
    ast::{AssignmentExpr, Node, VariabeDecleration},
    symbol::*,
};
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

    fn visit_expression(&mut self, node: &Node) -> SResult {
        match node {
            Node::BinOperator(_) => Ok(()),
            Node::Number(_) => Ok(()),
            Node::String(iden) => self
                .symbol_table
                .get(iden)
                .and(Some(()))
                .ok_or(format!("{} is not defined", iden)),
            Node::UnaryOperator(_) => Ok(()),
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
