//! Expression-level move checking

use super::checker::MoveChecker;
use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use vex_ast::{Expression, Param};

impl MoveChecker {
    /// Check an expression for use of moved variables
    pub(super) fn check_expression(&mut self, expr: &Expression) -> BorrowResult<()> {
        match expr {
            Expression::Ident(name) => self.check_identifier(name),

            Expression::Binary {
                span_id: _,
                left,
                right,
                ..
            } => {
                self.check_expression(left)?;
                self.check_expression(right)?;
                Ok(())
            }

            Expression::Unary {
                span_id: _, expr, ..
            } => {
                self.check_expression(expr)?;
                Ok(())
            }

            Expression::Call { func, args, .. } => self.check_call_expression(func, args),

            Expression::MethodCall {
                receiver,
                args,
                is_mutable_call,
                ..
            } => self.check_method_call_expression(receiver, args, *is_mutable_call),

            Expression::FieldAccess { object, .. } => {
                self.check_expression(object)?;
                Ok(())
            }

            Expression::Index { object, index } => {
                self.check_expression(object)?;
                self.check_expression(index)?;
                Ok(())
            }

            Expression::Array(elements) => {
                for elem in elements {
                    self.check_expression(elem)?;
                }
                Ok(())
            }

            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.check_expression(elem)?;
                }
                Ok(())
            }

            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.check_expression(expr)?;
                }
                Ok(())
            }

            Expression::Range { start, end } => {
                if let Some(s) = start {
                    self.check_expression(s)?;
                }
                if let Some(e) = end {
                    self.check_expression(e)?;
                }
                Ok(())
            }

            Expression::Reference { expr, .. } => {
                // Taking a reference doesn't move
                self.check_expression(expr)?;
                Ok(())
            }

            Expression::Deref(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }

            Expression::Await(expr)
            | Expression::QuestionMark(expr)
            | Expression::ChannelReceive(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }

            Expression::Match { value, arms } => self.check_match_expression(value, arms),

            Expression::New(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }

            Expression::Closure { params, body, .. } => self.check_closure_expression(params, body),

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

    fn check_identifier(&self, name: &str) -> BorrowResult<()> {
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
            let used_at = self
                .current_function
                .as_ref()
                .map(|f| format!("in function `{}`", f));
            return Err(BorrowError::UseAfterMove {
                variable: name.to_string(),
                moved_at: None, // TODO: Track where the move happened
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
    ) -> BorrowResult<()> {
        // Skip builtin function check if func is an identifier
        if let Expression::Ident(func_name) = func {
            if !self.builtin_registry.is_builtin(func_name) {
                self.check_expression(func)?;
            }
            // Builtin functions are always valid, skip checking
        } else {
            self.check_expression(func)?;
        }

        for arg in args {
            // Arguments might be moved into the function
            // For now, we'll check if they're valid
            self.check_expression(arg)?;

            // If arg is a move type identifier, mark it as moved
            if let Expression::Ident(var) = arg {
                if let Some(ty) = self.var_types.get(var) {
                    if self.is_move_type(ty) {
                        self.moved_vars.insert(var.clone());
                        self.valid_vars.remove(var);
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
    ) -> BorrowResult<()> {
        self.check_expression(receiver)?;

        for arg in args {
            self.check_expression(arg)?;

            // For mutable method calls (method()!), arguments are mutable borrows, not moves
            // Only mark as moved for non-mutable calls
            if !is_mutable_call {
                if let Expression::Ident(var) = arg {
                    if let Some(ty) = self.var_types.get(var) {
                        if self.is_move_type(ty) {
                            self.moved_vars.insert(var.clone());
                            self.valid_vars.remove(var);
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
    ) -> BorrowResult<()> {
        self.check_expression(value)?;

        for arm in arms {
            if let Some(guard) = &arm.guard {
                self.check_expression(guard)?;
            }
            self.check_expression(&arm.body)?;
        }

        Ok(())
    }

    fn check_closure_expression(
        &mut self,
        params: &[Param],
        body: &Expression,
    ) -> BorrowResult<()> {
        // Register closure parameters as valid variables
        for param in params {
            self.valid_vars.insert(param.name.clone());
            self.var_types.insert(param.name.clone(), param.ty.clone());
        }

        // Check closure body
        self.check_expression(body)?;

        // Note: We don't remove params from scope here because
        // MoveChecker doesn't track scopes, just tracks which variables
        // are valid vs moved across the whole expression
        Ok(())
    }
}
