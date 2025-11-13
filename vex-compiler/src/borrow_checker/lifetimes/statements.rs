// Statement checking logic for lifetime analysis
// Handles all statement types and their lifetime implications

use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use vex_ast::*;

impl super::LifetimeChecker {
    /// Check a single statement for lifetime violations
    pub(super) fn check_statement(&mut self, stmt: &Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let {
                name,
                value,
                is_mutable: _,
                ty: _,
            } => {
                // Check the value expression first
                self.check_expression(value)?;

                // Check if this is a reference assignment
                if let Expression::Reference { expr: ref_expr, .. } = value {
                    // Track what this reference points to
                    if let Expression::Ident(target) = ref_expr.as_ref() {
                        self.references.insert(name.clone(), target.clone());

                        // Verify the target is still in scope
                        if !self.in_scope.contains(target) {
                            return Err(BorrowError::DanglingReference {
                                reference: name.clone(),
                                referent: target.clone(),
                            });
                        }
                    }
                }

                // Declare the new variable
                self.declare_variable(name);
                Ok(())
            }

            Statement::LetPattern { pattern, value, .. } => {
                // Check value expression
                self.check_expression(value)?;
                // Declare pattern bindings
                self.declare_pattern_bindings(pattern);
                Ok(())
            }

            Statement::Assign { target, value } => {
                self.check_expression(target)?;
                self.check_expression(value)?;

                // Track reference assignments (e.g., `ref_var = &local;`)
                if let Expression::Ident(var_name) = target {
                    if let Expression::Reference { expr: ref_expr, .. } = value {
                        if let Expression::Ident(target_name) = ref_expr.as_ref() {
                            // Check if target is still in scope
                            if let Some(&target_scope) = self.variable_scopes.get(target_name) {
                                // If target is in a deeper scope than the reference variable,
                                // this will create a dangling reference when target goes out of scope
                                if let Some(&ref_scope) = self.variable_scopes.get(var_name) {
                                    if target_scope > ref_scope {
                                        return Err(BorrowError::DanglingReference {
                                            reference: var_name.clone(),
                                            referent: target_name.clone(),
                                        });
                                    }
                                }
                            }

                            // Update reference tracking
                            self.references
                                .insert(var_name.clone(), target_name.clone());
                        }
                    }
                }
                Ok(())
            }

            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expression(e)?;

                    // CRITICAL: Check if returning a reference to local variable
                    if let Expression::Reference { expr: ref_expr, .. } = e {
                        if let Expression::Ident(var_name) = ref_expr.as_ref() {
                            // Check if the variable is local (not a parameter)
                            if let Some(&scope) = self.variable_scopes.get(var_name) {
                                // scope 1 = function params (OK to return)
                                // scope 2+ = local variables (ERROR - will be dropped)
                                if scope >= 2 {
                                    return Err(BorrowError::ReturnDanglingReference {
                                        variable: var_name.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(())
            }

            Statement::If {
                span_id: _,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.check_expression(condition)?;

                // Each branch is a new scope
                self.enter_scope();
                self.check_block(then_block)?;
                self.exit_scope();

                for (elif_cond, elif_body) in elif_branches {
                    self.check_expression(elif_cond)?;
                    self.enter_scope();
                    self.check_block(elif_body)?;
                    self.exit_scope();
                }

                if let Some(else_body) = else_block {
                    self.enter_scope();
                    self.check_block(else_body)?;
                    self.exit_scope();
                }

                Ok(())
            }

            Statement::While {
                span_id: _,
                condition,
                body,
            } => {
                self.check_expression(condition)?;
                self.enter_scope();
                self.check_block(body)?;
                self.exit_scope();
                Ok(())
            }

            Statement::Loop { body } => {
                self.enter_scope();
                self.check_block(body)?;
                self.exit_scope();
                Ok(())
            }

            Statement::For {
                span_id: _,
                init,
                condition,
                post,
                body,
            } => {
                self.enter_scope();

                // Check init statement if present
                if let Some(init_stmt) = init {
                    self.check_statement(init_stmt)?;
                }

                // Check condition if present
                if let Some(cond) = condition {
                    self.check_expression(cond)?;
                }

                // Check body
                self.check_block(body)?;

                // Check post statement if present
                if let Some(post_stmt) = post {
                    self.check_statement(post_stmt)?;
                }

                self.exit_scope();
                Ok(())
            }

            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                self.check_expression(iterable)?;
                self.enter_scope();
                self.declare_variable(variable);
                self.check_block(body)?;
                self.exit_scope();
                Ok(())
            }

            Statement::Switch {
                value,
                cases,
                default_case,
            } => {
                if let Some(val) = value {
                    self.check_expression(val)?;
                }

                for case in cases {
                    // Check all pattern expressions
                    for pattern_expr in &case.patterns {
                        self.check_expression(pattern_expr)?;
                    }

                    self.enter_scope();
                    self.check_block(&case.body)?;
                    self.exit_scope();
                }

                if let Some(default) = default_case {
                    self.enter_scope();
                    self.check_block(default)?;
                    self.exit_scope();
                }

                Ok(())
            }

            Statement::Expression(expr) => self.check_expression(expr),

            Statement::CompoundAssign { target, value, .. } => {
                self.check_expression(target)?;
                self.check_expression(value)
            }

            Statement::Select { .. } => {
                // TODO: Implement select case checking when async is ready
                Ok(())
            }

            Statement::Unsafe(block) => {
                // Enter unsafe context
                let prev_unsafe = self.in_unsafe_block;
                self.in_unsafe_block = true;

                // Check unsafe block content
                self.check_block(block)?;

                // Restore previous unsafe context
                self.in_unsafe_block = prev_unsafe;
                Ok(())
            }

            Statement::Defer(_) | Statement::Go(_) | Statement::Break | Statement::Continue => {
                Ok(())
            }
        }
    }
}