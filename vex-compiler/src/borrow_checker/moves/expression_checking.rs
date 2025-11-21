//! Expression-level move checking

use super::checker::MoveChecker;
use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use vex_ast::{Expression, Param};

impl MoveChecker {
    /// Check an expression for use of moved variables
    pub(super) fn check_expression(
        &mut self,
        expr: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Derive the effective span for this expression (fall back to parent span)
        let this_span: Option<&String> = match expr {
            Expression::Binary { span_id, .. }
            | Expression::Call { span_id, .. }
            | Expression::Unary { span_id, .. } => span_id.as_ref().or(parent_span),
            _ => parent_span,
        };
        match expr {
            Expression::Ident(name) => self.check_identifier(name, this_span),

            Expression::Binary {
                span_id: _,
                left,
                right,
                ..
            } => {
                self.check_expression(left, this_span)?;
                self.check_expression(right, this_span)?;
                Ok(())
            }

            Expression::Unary {
                span_id: _, expr, ..
            } => {
                self.check_expression(expr, this_span)?;
                Ok(())
            }

            Expression::Call {
                func,
                args,
                span_id,
                ..
            } => self.check_call_expression(func, args, span_id.as_ref().or(this_span)),

            Expression::MethodCall {
                receiver,
                args,
                is_mutable_call,
                ..
            } => self.check_method_call_expression(receiver, args, *is_mutable_call, this_span),

            Expression::FieldAccess { object, .. } => {
                self.check_expression(object, this_span)?;
                Ok(())
            }

            Expression::Index { object, index } => {
                self.check_expression(object, this_span)?;
                self.check_expression(index, this_span)?;
                Ok(())
            }

            Expression::Array(elements) => {
                for elem in elements {
                    self.check_expression(elem, this_span)?;
                }
                Ok(())
            }

            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.check_expression(elem, this_span)?;
                }
                Ok(())
            }

            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.check_expression(expr, this_span)?;
                }
                Ok(())
            }

            Expression::Range { start, end } => {
                if let Some(s) = start {
                    self.check_expression(s, this_span)?;
                }
                if let Some(e) = end {
                    self.check_expression(e, this_span)?;
                }
                Ok(())
            }

            Expression::Reference { expr, .. } => {
                // Taking a reference doesn't move
                self.check_expression(expr, this_span)?;
                Ok(())
            }

            Expression::Deref(expr) => {
                self.check_expression(expr, this_span)?;
                Ok(())
            }

            Expression::Await(expr)
            | Expression::TryOp { expr }
            | Expression::ChannelReceive(expr) => {
                self.check_expression(expr, this_span)?;
                Ok(())
            }

            Expression::Match { value, arms } => {
                self.check_match_expression(value, arms, this_span)
            }

            Expression::New(expr) => {
                self.check_expression(expr, this_span)?;
                Ok(())
            }

            Expression::Closure { params, body, .. } => {
                self.check_closure_expression(params, body, this_span)
            }

            // Literals don't reference variables
            Expression::IntLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::FStringLiteral(_)
            | Expression::BoolLiteral(_)
            | Expression::Nil => Ok(()),

            _ => Ok(()), // Other expressions don't affect moves
        }
    }

    fn check_identifier(&self, name: &str, used_at: Option<&String>) -> BorrowResult<()> {
        // Skip builtin functions - they're not variables
        if self.builtin_registry.is_builtin(name) {
            return Ok(());
        }

        // Global variables (extern functions) are always valid
        if self.global_vars.contains(name) {
            return Ok(());
        }

        // Skip builtin type names (Vec, Box, Map, etc.) - O(1) hash lookup
        // These are used in static method calls like Vec.new()
        if crate::type_registry::is_builtin_type(name) {
            return Ok(());
        }

        // Check if this variable has been moved
        if self.moved_vars.contains(name) {
            // Try to obtain where the move occurred from move_locations (if recorded)
            let moved_at = self.move_locations.get(name).cloned().unwrap_or(None);
            let used_at = used_at.cloned();
            return Err(BorrowError::UseAfterMove {
                variable: name.to_string(),
                moved_at,
                used_at,
            });
        }

        // Check if it's a valid variable
        if !self.valid_vars.contains(name) {
            // It's either undefined or moved
            // (undefined will be caught by semantic analyzer)
            return Ok(());
        }

        Ok(())
    }

    fn check_call_expression(
        &mut self,
        func: &Expression,
        args: &[Expression],
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Skip builtin function check if func is an identifier
        if let Expression::Ident(func_name) = func {
            if !self.builtin_registry.is_builtin(func_name) {
                self.check_expression(func, parent_span)?;
            }
            // Builtin functions are always valid, skip checking
        } else {
            self.check_expression(func, parent_span)?;
        }

        for arg in args {
            // Arguments might be moved into the function
            // For now, we'll check if they're valid
            self.check_expression(arg, parent_span)?;

            // If arg is a move type identifier, mark it as moved
            if let Expression::Ident(var) = arg {
                if let Some(ty) = self.var_types.get(var) {
                    if self.is_move_type(ty) {
                        self.moved_vars.insert(var.clone());
                        self.valid_vars.remove(var);
                        // Record where the move occurred for diagnostics
                        self.move_locations
                            .insert(var.clone(), parent_span.map(|s| s.clone()));
                    }
                }
            }
        }

        Ok(())
    }

    fn check_method_call_expression(
        &mut self,
        receiver: &Expression,
        args: &[Expression],
        is_mutable_call: bool,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        self.check_expression(receiver, parent_span)?;

        for arg in args {
            self.check_expression(arg, parent_span)?;

            // For mutable method calls (method()!), arguments are mutable borrows, not moves
            // Only mark as moved for non-mutable calls
            if !is_mutable_call {
                if let Expression::Ident(var) = arg {
                    if let Some(ty) = self.var_types.get(var) {
                        if self.is_move_type(ty) {
                            self.moved_vars.insert(var.clone());
                            self.valid_vars.remove(var);
                            self.move_locations
                                .insert(var.clone(), parent_span.map(|s| s.clone()));
                        }
                    }
                }
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
        self.check_expression(value, parent_span)?;

        for arm in arms {
            if let Some(guard) = &arm.guard {
                self.check_expression(guard, parent_span)?;
            }
            self.check_expression(&arm.body, parent_span)?;
        }

        Ok(())
    }

    fn check_closure_expression(
        &mut self,
        params: &[Param],
        body: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Register closure parameters as valid variables
        for param in params {
            self.valid_vars.insert(param.name.clone());
            self.var_types.insert(param.name.clone(), param.ty.clone());
        }

        // Check closure body
        self.check_expression(body, parent_span)?;

        // Note: We don't remove params from scope here because
        // MoveChecker doesn't track scopes, just tracks which variables
        // are valid vs moved across the whole expression
        Ok(())
    }
}
