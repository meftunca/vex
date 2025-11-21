//! Expression-level borrow checking

use super::checker::BorrowRulesChecker;
use super::tracking::BorrowKind;
use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use vex_ast::Expression;

impl BorrowRulesChecker {
    /// Check an expression for borrow rule violations
    pub(super) fn check_expression_for_borrows(
        &mut self,
        expr: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        match expr {
            Expression::Reference { is_mutable, expr } => {
                self.check_reference_expression(*is_mutable, expr, parent_span)
            }

            Expression::Binary {
                span_id,
                left,
                right,
                ..
            } => {
                let this_span = span_id.as_ref().or(parent_span);
                self.check_expression_for_borrows(left, this_span)?;
                self.check_expression_for_borrows(right, this_span)?;
                Ok(())
            }

            Expression::Unary { span_id, expr, .. } => {
                let this_span = span_id.as_ref().or(parent_span);
                self.check_expression_for_borrows(expr, this_span)?;
                Ok(())
            }

            Expression::Call {
                func,
                args,
                span_id,
                ..
            } => {
                let this_span = span_id.as_ref().or(parent_span);
                self.check_call_expression(func, args, this_span)
            }

            Expression::MethodCall { receiver, args, .. } => {
                self.check_method_call_expression(receiver, args, parent_span)
            }

            Expression::FieldAccess { object, .. } => {
                self.check_expression_for_borrows(object, parent_span)?;
                Ok(())
            }

            Expression::Index { object, index } => {
                self.check_expression_for_borrows(object, parent_span)?;
                self.check_expression_for_borrows(index, parent_span)?;
                Ok(())
            }

            Expression::Array(elements) => {
                for elem in elements {
                    self.check_expression_for_borrows(elem, parent_span)?;
                }
                Ok(())
            }

            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.check_expression_for_borrows(elem, parent_span)?;
                }
                Ok(())
            }

            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.check_expression_for_borrows(expr, parent_span)?;
                }
                Ok(())
            }

            Expression::Range { start, end } => {
                if let Some(s) = start {
                    self.check_expression_for_borrows(s, parent_span)?;
                }
                if let Some(e) = end {
                    self.check_expression_for_borrows(e, parent_span)?;
                }
                Ok(())
            }

            Expression::Deref(expr) => self.check_deref_expression(expr, parent_span),

            Expression::Await(expr)
            | Expression::TryOp { expr }
            | Expression::ChannelReceive(expr) => {
                self.check_expression_for_borrows(expr, parent_span)?;
                Ok(())
            }

            Expression::Match { value, arms } => {
                self.check_match_expression(value, arms, parent_span)
            }

            Expression::New(expr) => {
                self.check_expression_for_borrows(expr, parent_span)?;
                Ok(())
            }

            Expression::Closure { body, .. } => {
                // Check closure body for borrow violations
                // Note: Closure parameters are validated by LifetimeChecker
                self.check_expression_for_borrows(body, parent_span)?;
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
        parent_span: Option<&String>,
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
                parent_span.cloned(),
            )?;
        }

        self.check_expression_for_borrows(expr, parent_span)?;
        Ok(())
    }

    fn check_call_expression(
        &mut self,
        func: &Expression,
        args: &[Expression],
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Skip checking builtin function names as variables
        if let Expression::Ident(func_name) = func {
            if !self.builtin_registry.is_builtin(func_name) {
                self.check_expression_for_borrows(func, parent_span)?;
            }
        } else {
            self.check_expression_for_borrows(func, parent_span)?;
        }

        // Check if this is a builtin function call
        if let Expression::Ident(func_name) = func {
            if let Some(metadata) = self.builtin_registry.get(func_name).cloned() {
                return self.check_builtin_call(func_name, args, &metadata, parent_span);
            }
        }

        // Not a builtin, check args normally
        for arg in args {
            self.check_expression_for_borrows(arg, parent_span)?;
        }
        Ok(())
    }

    fn check_builtin_call(
        &mut self,
        _func_name: &str,
        args: &[Expression],
        metadata: &super::super::builtin_metadata::BuiltinMetadata,
        parent_span: Option<&String>,
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
                                    // Try to find the location of an existing borrow for better diagnostics
                                    let mut borrowed_loc: Option<String> = None;
                                    for (_ref_name, borrows_vec) in &self.active_borrows {
                                        for b in borrows_vec {
                                            if b.variable == *var_name {
                                                if let Some(loc) = &b.location {
                                                    borrowed_loc = Some(loc.clone());
                                                    break;
                                                }
                                            }
                                        }
                                        if borrowed_loc.is_some() {
                                            break;
                                        }
                                    }
                                    return Err(BorrowError::MutationWhileBorrowed {
                                        variable: var_name.clone(),
                                        borrowed_at: borrowed_loc.or_else(|| parent_span.cloned()),
                                    });
                                }
                            }
                        }
                        ParamEffect::Moves => {
                            // Check if variable is currently borrowed (cannot move)
                            if let Some(borrows) = self.borrowed_vars.get(var_name) {
                                if !borrows.is_empty() {
                                    // Find an active borrow location, or use parent span as fallback
                                    let mut borrow_loc: Option<String> = None;
                                    for (_ref_name, borrows_vec) in &self.active_borrows {
                                        for b in borrows_vec {
                                            if b.variable == *var_name {
                                                if let Some(loc) = &b.location {
                                                    borrow_loc = Some(loc.clone());
                                                    break;
                                                }
                                            }
                                        }
                                        if borrow_loc.is_some() {
                                            break;
                                        }
                                    }
                                    return Err(BorrowError::MoveWhileBorrowed {
                                        variable: var_name.clone(),
                                        borrow_location: borrow_loc
                                            .or_else(|| parent_span.cloned()),
                                    });
                                }
                            }
                        }
                        _ => {} // ReadOnly, BorrowsImmut are fine
                    }
                }
            }

            self.check_expression_for_borrows(arg, parent_span)?;
        }
        Ok(())
    }

    fn check_method_call_expression(
        &mut self,
        receiver: &Expression,
        args: &[Expression],
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        self.check_expression_for_borrows(receiver, parent_span)?;
        for arg in args {
            self.check_expression_for_borrows(arg, parent_span)?;
        }
        Ok(())
    }

    fn check_deref_expression(
        &mut self,
        expr: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        self.check_expression_for_borrows(expr, parent_span)?;

        // Raw pointer dereference requires unsafe
        if !self.in_unsafe_block {
            // Only enforce unsafe for explicit raw pointer derefs
            // Smart pointer derefs (like Box) don't need unsafe
            if Self::is_likely_raw_pointer(expr) {
                return Err(BorrowError::UnsafeOperationOutsideUnsafeBlock {
                    operation: "raw pointer dereference".to_string(),
                    location: parent_span.cloned(),
                });
            }
        }

        Ok(())
    }

    fn check_match_expression(
        &mut self,
        value: &Expression,
        arms: &[vex_ast::MatchArm],
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        self.check_expression_for_borrows(value, parent_span)?;
        for arm in arms {
            if let Some(guard) = &arm.guard {
                self.check_expression_for_borrows(guard, parent_span)?;
            }
            self.check_expression_for_borrows(&arm.body, parent_span)?;
        }
        Ok(())
    }
}
