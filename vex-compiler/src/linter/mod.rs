// Linter module - static analysis and warnings for Vex code
// Detects code quality issues and unused code

use std::collections::{HashMap, HashSet};
use vex_ast::{Expression, Item, Program, Statement};
use vex_diagnostics::{Diagnostic, DiagnosticEngine};

pub mod unused_variables;
// TODO: Fix AST structure compatibility
// pub mod dead_code;
// pub mod unreachable_code;
// pub mod naming_convention;

pub use unused_variables::UnusedVariableRule;
// pub use dead_code::DeadCodeRule;
// pub use unreachable_code::UnreachableCodeRule;
// pub use naming_convention::NamingConventionRule;

/// Trait for implementing lint rules
pub trait LintRule {
    /// Check the AST and return diagnostics
    fn check(&self, program: &Program) -> Vec<Diagnostic>;

    /// Name of the lint rule
    fn name(&self) -> &str;

    /// Whether this rule is enabled by default
    fn enabled_by_default(&self) -> bool {
        true
    }
}

/// Main linter that runs all lint rules
pub struct Linter {
    rules: Vec<Box<dyn LintRule>>,
    diagnostics: DiagnosticEngine,
}

impl Linter {
    /// Create a new linter with default rules
    pub fn new() -> Self {
        let mut linter = Self {
            rules: Vec::new(),
            diagnostics: DiagnosticEngine::new(),
        };

        // Add default rules
        linter.add_rule(Box::new(UnusedVariableRule::new()));
        // TODO: Add more rules when AST structure is fully compatible
        // linter.add_rule(Box::new(DeadCodeRule::new()));
        // linter.add_rule(Box::new(UnreachableCodeRule::new()));
        // linter.add_rule(Box::new(NamingConventionRule::new()));

        linter
    }

    /// Create a linter with no rules (for custom configuration)
    pub fn empty() -> Self {
        Self {
            rules: Vec::new(),
            diagnostics: DiagnosticEngine::new(),
        }
    }

    /// Add a lint rule
    pub fn add_rule(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }

    /// Run all lint rules on the program
    pub fn lint(&mut self, program: &Program) -> Vec<Diagnostic> {
        let mut all_diagnostics = Vec::new();

        for rule in &self.rules {
            let diagnostics = rule.check(program);
            all_diagnostics.extend(diagnostics);
        }

        all_diagnostics
    }

    /// Get the diagnostic engine
    pub fn diagnostics(&self) -> &DiagnosticEngine {
        &self.diagnostics
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .diagnostics()
            .iter()
            .any(|d| d.level == vex_diagnostics::ErrorLevel::Warning)
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linter_creation() {
        let linter = Linter::new();
        assert_eq!(linter.rules.len(), 1); // Only unused_variables for now
    }

    #[test]
    fn test_empty_linter() {
        let linter = Linter::empty();
        assert_eq!(linter.rules.len(), 0);
    }
}
