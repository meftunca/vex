// Unreachable code detection linter rule
// Detects code after:
// - return statements
// - break/continue in loops
// - panic/abort calls

use vex_ast::{Expression, Function, Program, Statement};
use vex_diagnostics::{Diagnostic, ErrorLevel, Span};

use super::LintRule;

pub struct UnreachableCodeRule;

impl UnreachableCodeRule {
    pub fn new() -> Self {
        Self
    }

    fn check_function(&self, func: &Function) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        self.check_statements(&func.body, &mut diagnostics, false);
        diagnostics
    }

    /// Check statements for unreachable code
    /// Returns true if control flow definitely exits (return/panic/etc)
    fn check_statements(
        &self,
        stmts: &[Statement],
        diagnostics: &mut Vec<Diagnostic>,
        in_loop: bool,
    ) -> bool {
        let mut terminated = false;

        for (i, stmt) in stmts.iter().enumerate() {
            if terminated {
                // Found code after terminating statement
                diagnostics.push(Diagnostic {
                    level: ErrorLevel::Warning,
                    code: "W0006".to_string(),
                    message: "unreachable code".to_string(),
                    span: Span::unknown(), // TODO: Get actual span
                    help: Some("remove this code or fix control flow".to_string()),
                    notes: vec![],
                });
                break; // Only warn once per block
            }

            terminated = self.statement_terminates(stmt, diagnostics, in_loop);
        }

        terminated
    }

    /// Check if a statement definitely terminates control flow
    fn statement_terminates(
        &self,
        stmt: &Statement,
        diagnostics: &mut Vec<Diagnostic>,
        in_loop: bool,
    ) -> bool {
        match stmt {
            Statement::Return { .. } => true,
            
            Statement::Break | Statement::Continue => {
                // break/continue terminate current block but not the function
                in_loop
            }

            Statement::Expression(expr) => {
                self.expression_terminates(expr)
            }

            Statement::If { then_block, else_block, .. } => {
                let then_terminates = self.check_statements(then_block, diagnostics, in_loop);
                
                if let Some(else_stmts) = else_block {
                    let else_terminates = self.check_statements(else_stmts, diagnostics, in_loop);
                    // Only terminates if BOTH branches terminate
                    then_terminates && else_terminates
                } else {
                    false // No else branch = might not execute
                }
            }

            Statement::While { body, .. } => {
                self.check_statements(body, diagnostics, true);
                false // Loops might not execute
            }

            Statement::For { body, .. } => {
                self.check_statements(body, diagnostics, true);
                false // Loops might not execute
            }

            Statement::Block(stmts) => {
                self.check_statements(stmts, diagnostics, in_loop)
            }

            Statement::Match { arms, .. } => {
                // Check if all arms terminate
                if arms.is_empty() {
                    return false;
                }

                let mut all_terminate = true;
                for arm in arms {
                    if !self.check_statements(&arm.body, diagnostics, in_loop) {
                        all_terminate = false;
                    }
                }
                all_terminate
            }

            _ => false,
        }
    }

    /// Check if expression is a terminating call (panic, abort, etc)
    fn expression_terminates(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Call { func, .. } => {
                if let Expression::Ident(name) = &**func {
                    // Calls that never return
                    matches!(name.as_str(), "panic" | "abort" | "exit" | "unreachable")
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl LintRule for UnreachableCodeRule {
    fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for item in &program.items {
            if let vex_ast::Item::Function(func) = item {
                diagnostics.extend(self.check_function(func));
            }
        }

        diagnostics
    }

    fn name(&self) -> &str {
        "unreachable-code"
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}
