//! Expression-level borrow checking

use super::checker::BorrowRulesChecker;
use super::tracking::BorrowKind;
use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use vex_ast::Expression;

impl BorrowRulesChecker {
    /// Check an expression for borrow rule violations
    pub(super) fn check_expression_for_borrows(&mut self, expr: &Expression) -> BorrowResult<()> {
        match expr {
            Expression::Reference { is_mutable, expr } => {
                self.check_reference_expression(*is_mutable, expr)
            }

            Expression::Binary {
                span_id: _,
                left,
                right,
                ..
            } => {
                self.check_expression_for_borrows(left)?;
                self.check_expression_for_borrows(right)?;
                Ok(())
            }

            Expression::Unary {
                span_id: _, expr, ..
            } => {
                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            Expression::Call { func, args, .. } => self.check_call_expression(func, args),

            Expression::MethodCall { receiver, args, .. } => {
                self.check_method_call_expression(receiver, args)
            }

            Expression::FieldAccess { object, .. } => {
                self.check_expression_for_borrows(object)?;
                Ok(())
            }

            Expression::Index { object, index } => {
                self.check_expression_for_borrows(object)?;
                self.check_expression_for_borrows(index)?;
                Ok(())
            }

            Expression::Array(elements) => {
                for elem in elements {
                    self.check_expression_for_borrows(elem)?;
                }
                Ok(())
            }

            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.check_expression_for_borrows(elem)?;
                }
                Ok(())
            }

            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.check_expression_for_borrows(expr)?;
                }
                Ok(())
            }

            Expression::Range { start, end } => {
                if let Some(s) = start {
                    self.check_expression_for_borrows(s)?;
                }
                if let Some(e) = end {
                    self.check_expression_for_borrows(e)?;
                }
                Ok(())
            }

            Expression::Deref(expr) => self.check_deref_expression(expr),

            Expression::Await(expr)
            | Expression::QuestionMark(expr)
            | Expression::ChannelReceive(expr) => {
                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            Expression::Match { value, arms } => self.check_match_expression(value, arms),

            Expression::New(expr) => {
                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            Expression::Closure { body, .. } => {
                // Check closure body for borrow violations
                // Note: Closure parameters are validated by LifetimeChecker
                self.check_expression_for_borrows(body)?;
                Ok(())
            }

            // Literals and identifiers don't create borrows
            _ => Ok(()),
        }
    }

    fn check_reference_expression(
        &mut self,
        is_mutable: bool,
        expr: &Expression,
    ) -> BorrowResult<()> {
        // Check if we can create this borrow
        if let Expression::Ident(var) = expr {
            self.check_can_borrow(
                var,
                if is_mutable {
                    BorrowKind::Mutable
                } else {
                    BorrowKind::Immutable
                },
            )?;
        }

        self.check_expression_for_borrows(expr)?;
        Ok(())
    }

    fn check_call_expression(
        &mut self,
        func: &Expression,
        args: &[Expression],
    ) -> BorrowResult<()> {
        // Skip checking builtin function names as variables
        if let Expression::Ident(func_name) = func {
            if !self.builtin_registry.is_builtin(func_name) {
                self.check_expression_for_borrows(func)?;
            }
        } else {
            self.check_expression_for_borrows(func)?;
        }

        // Check if this is a builtin function call
        if let Expression::Ident(func_name) = func {
            if let Some(metadata) = self.builtin_registry.get(func_name).cloned() {
                return self.check_builtin_call(func_name, args, &metadata);
            }
        }

        // Not a builtin, check args normally
        for arg in args {
            self.check_expression_for_borrows(arg)?;
        }
        Ok(())
    }

    fn check_builtin_call(
        &mut self,
        func_name: &str,
        args: &[Expression],
        metadata: &super::super::builtin_metadata::BuiltinMetadata,
    ) -> BorrowResult<()> {
        // Check each argument against builtin metadata
        for (i, arg) in args.iter().enumerate() {
            if i < metadata.param_effects.len() {
                let effect = &metadata.param_effects[i];

                // Check if we're passing a borrowed variable to a mutating builtin
                if let Expression::Ident(var_name) = arg {
                    use super::super::builtin_metadata::ParamEffect;

                    match effect {
                        ParamEffect::BorrowsMut | ParamEffect::Mutates => {
                            // Check if variable is currently borrowed
                            if let Some(borrows) = self.borrowed_vars.get(var_name) {
                                if !borrows.is_empty() {
                                    return Err(BorrowError::MutationWhileBorrowed {
                                        variable: var_name.clone(),
                                        borrowed_at: Some(format!(
                                            "builtin function '{}' requires mutable access",
                                            func_name
                                        )),
                                    });
                                }
                            }
                        }
                        ParamEffect::Moves => {
                            // Check if variable is currently borrowed (cannot move)
                            if let Some(borrows) = self.borrowed_vars.get(var_name) {
                                if !borrows.is_empty() {
                                    return Err(BorrowError::MoveWhileBorrowed {
                                        variable: var_name.clone(),
                                        borrow_location: Some(format!(
                                            "builtin function '{}' takes ownership",
                                            func_name
                                        )),
                                    });
                                }
                            }
                        }
                        _ => {} // ReadOnly, BorrowsImmut are fine
                    }
                }
            }

            self.check_expression_for_borrows(arg)?;
        }
        Ok(())
    }

    fn check_method_call_expression(
        &mut self,
        receiver: &Expression,
        args: &[Expression],
    ) -> BorrowResult<()> {
        self.check_expression_for_borrows(receiver)?;
        for arg in args {
            self.check_expression_for_borrows(arg)?;
        }
        Ok(())
    }

    fn check_deref_expression(&mut self, expr: &Expression) -> BorrowResult<()> {
        self.check_expression_for_borrows(expr)?;

        // Raw pointer dereference requires unsafe
        if !self.in_unsafe_block {
            // Only enforce unsafe for explicit raw pointer derefs
            // Smart pointer derefs (like Box) don't need unsafe
            if Self::is_likely_raw_pointer(expr) {
                return Err(BorrowError::UnsafeOperationOutsideUnsafeBlock {
                    operation: "raw pointer dereference".to_string(),
                    location: None,
                });
            }
        }

        Ok(())
    }

    fn check_match_expression(
        &mut self,
        value: &Expression,
        arms: &[vex_ast::MatchArm],
    ) -> BorrowResult<()> {
        self.check_expression_for_borrows(value)?;
        for arm in arms {
            if let Some(guard) = &arm.guard {
                self.check_expression_for_borrows(guard)?;
            }
            self.check_expression_for_borrows(&arm.body)?;
        }
        Ok(())
    }
}
