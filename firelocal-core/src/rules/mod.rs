pub mod ast;
pub mod parser;

use crate::rules::ast::Ruleset;
use crate::rules::parser::RulesParser;
use std::collections::HashMap;

pub struct RulesEngine {
    ruleset: Option<Ruleset>,
}

impl RulesEngine {
    pub fn new() -> Self {
        Self { ruleset: None }
    }

    pub fn is_empty(&self) -> bool {
        self.ruleset.is_none()
    }

    pub fn load_rules(&mut self, rules_str: &str) -> Result<(), String> {
        let mut parser = RulesParser::new(rules_str);
        let ruleset = parser.parse()?;
        self.ruleset = Some(ruleset);
        Ok(())
    }

    pub fn evaluate(&self, path: &str, operation: &str, context: &HashMap<String, String>) -> bool {
        if let Some(ruleset) = &self.ruleset {
            // TODO: Traverse match blocks and evaluate allow conditions
            // For M4 MVP, we will implement a basic traversal
            return ruleset.is_allowed(path, operation, context);
        }
        // distinct from Firestore default? TDD says "Deny by default" usually.
        false
    }
}
