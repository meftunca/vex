// Expression checking logic for lifetime analysis
// Handles all expression types and their lifetime implications

use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use vex_ast::*;

impl super::LifetimeChecker {
    /// Check a single expression for lifetime violations
    pub(super) fn check_expression(&mut self, expr: &Expression) -> BorrowResult<()> {
        match expr {
            Expression::Ident(name) => {
                // Skip checking builtin functions/types as variables
                if self.builtin_registry.is_builtin(name) {
                    return Ok(());
                }

                // Global variables (extern functions) are always in scope
                if self.global_vars.contains(name) {
                    return Ok(());
                }

                // Skip builtin type names (Vec, Box, Map, etc.) - O(1) hash lookup
                // These are used in static method calls like Vec.new()
                if crate::type_registry::is_builtin_type(name) {
                    return Ok(());
                }

                // Verify variable is in scope
                if !self.in_scope.contains(name) {
                    // Collect available names for fuzzy matching
                    let available_names: Vec<String> = self.in_scope.iter().cloned().collect();

                    return Err(BorrowError::UseAfterScopeEnd {
                        variable: name.clone(),
                        available_names,
                    });
                }
                Ok(())
            }

            Expression::Reference { expr: ref_expr, .. } => self.check_expression(ref_expr),

            Expression::Deref(expr) => self.check_expression(expr),

            Expression::Binary {
                span_id: _,
                left,
                right,
                ..
            } => {
                self.check_expression(left)?;
                self.check_expression(right)
            }

            Expression::Unary {
                span_id: _, expr, ..
            } => self.check_expression(expr),

            Expression::Call { func, args, .. } => {
                // Skip checking builtin function names as variables
                if let Expression::Ident(func_name) = func.as_ref() {
                    if !self.builtin_registry.is_builtin(func_name) {
                        self.check_expression(func)?;
                    }
                } else {
                    self.check_expression(func)?;
                }

                // Validate reference arguments
                for arg in args {
                    // If passing a reference to a local variable, ensure it's valid
                    if let Expression::Reference { expr: ref_expr, .. } = arg {
                        if let Expression::Ident(var_name) = ref_expr.as_ref() {
                            // Check if the variable is still in scope
                            if !self.in_scope.contains(var_name) {
                                let available_names: Vec<String> =
                                    self.in_scope.iter().cloned().collect();
                                return Err(BorrowError::UseAfterScopeEnd {
                                    variable: var_name.clone(),
                                    available_names,
                                });
                            }
                        }
                    }
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::MethodCall { receiver, args, .. } => {
                self.check_expression(receiver)?;

                // Validate reference arguments in method calls
                for arg in args {
                    if let Expression::Reference { expr: ref_expr, .. } = arg {
                        if let Expression::Ident(var_name) = ref_expr.as_ref() {
                            if !self.in_scope.contains(var_name) {
                                let available_names: Vec<String> =
                                    self.in_scope.iter().cloned().collect();
                                return Err(BorrowError::UseAfterScopeEnd {
                                    variable: var_name.clone(),
                                    available_names,
                                });
                            }
                        }
                    }
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::FieldAccess { object, .. } => self.check_expression(object),

            Expression::Index { object, index } => {
                self.check_expression(object)?;
                self.check_expression(index)
            }

            Expression::Array(elements) => {
                for elem in elements {
                    self.check_expression(elem)?;
                }
                Ok(())
            }

            Expression::ArrayRepeat(value, count) => {
                self.check_expression(value)?;
                self.check_expression(count)?;
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

            Expression::MapLiteral(entries) => {
                for (key, value) in entries {
                    self.check_expression(key)?;
                    self.check_expression(value)?;
                }
                Ok(())
            }

            Expression::Match { value, arms } => {
                self.check_expression(value)?;

                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.check_expression(guard)?;
                    }

                    // Each match arm is a new scope (for pattern bindings)
                    self.enter_scope();

                    // Extract and declare pattern bindings in this scope
                    self.declare_pattern_bindings(&arm.pattern);

                    self.check_expression(&arm.body)?;
                    self.exit_scope();
                }

                Ok(())
            }

            Expression::Block {
                statements,
                return_expr,
            } => {
                self.enter_scope();

                for stmt in statements {
                    self.check_statement(stmt)?;
                }

                if let Some(expr) = return_expr {
                    self.check_expression(expr)?;
                }

                self.exit_scope();
                Ok(())
            }

            Expression::AsyncBlock {
                statements,
                return_expr,
            } => {
                self.enter_scope();

                for stmt in statements {
                    self.check_statement(stmt)?;
                }

                if let Some(expr) = return_expr {
                    self.check_expression(expr)?;
                }

                self.exit_scope();
                Ok(())
            }

            // Literals have no lifetime concerns
            Expression::IntLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::FStringLiteral(_)
            | Expression::BoolLiteral(_)
            | Expression::Nil => Ok(()),

            Expression::EnumLiteral { data, .. } => {
                for expr in data {
                    self.check_expression(expr)?;
                }
                Ok(())
            }

            Expression::Range { start, end } | Expression::RangeInclusive { start, end } => {
                if let Some(s) = start {
                    self.check_expression(s)?;
                }
                if let Some(e) = end {
                    self.check_expression(e)?;
                }
                Ok(())
            }

            Expression::PostfixOp { expr, .. } => self.check_expression(expr),

            Expression::Await(expr)
            | Expression::QuestionMark(expr)
            | Expression::ChannelReceive(expr) => self.check_expression(expr),

            Expression::Launch { args, .. } => {
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::New(expr) => self.check_expression(expr),

            Expression::Make { size, .. } => self.check_expression(size),

            Expression::Cast { expr, .. } => self.check_expression(expr),

            Expression::ErrorNew(expr) => self.check_expression(expr),

            Expression::Typeof(expr) => {
                // typeof is compile-time, just check the inner expression
                self.check_expression(expr)
            }

            Expression::Closure { params, body, .. } => {
                // Enter a new scope for closure parameters
                self.enter_scope();

                // Register closure parameters in scope
                for param in params {
                    self.variable_scopes
                        .insert(param.name.clone(), self.current_scope);
                    self.in_scope.insert(param.name.clone());
                }

                // Check closure body with parameters in scope
                self.check_expression(body)?;

                // Exit closure scope
                self.exit_scope();

                Ok(())
            }

            Expression::TypeConstructor { args, .. } => {
                // Check all constructor arguments
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::BigIntLiteral(_) => Ok(()), // Literals are always valid
        }
    }
}