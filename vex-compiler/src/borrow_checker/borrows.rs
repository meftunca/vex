//! Borrow Rules Checker
//!
//! Phase 3 of borrow checker: Enforces Rust-style borrowing rules.
//!
//! Key rules:
//! 1. You can have either ONE mutable reference OR any number of immutable references
//! 2. References must not outlive the data they refer to
//! 3. Cannot mutate data while it's borrowed immutably
//!
//! Examples:
//! ```vex
//! let! x = 10;
//! let r1 = &x;     // immutable borrow
//! let r2 = &x;     // OK: multiple immutable borrows
//! let r3 = &x!;    // ❌ ERROR: cannot borrow as mutable while immutably borrowed
//!
//! let! y = 20;
//! let m1 = &y!;    // mutable borrow
//! let m2 = &y!;    // ❌ ERROR: cannot borrow as mutable more than once
//! let r = &y;      // ❌ ERROR: cannot borrow as immutable while mutably borrowed
//!
//! let! z = 30;
//! let rz = &z;
//! z = 40;          // ❌ ERROR: cannot mutate while borrowed
//! ```

use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use std::collections::HashMap;
use vex_ast::{Expression, Item, Program, Statement};

/// Type of borrow (for tracking)
#[derive(Debug, Clone, PartialEq)]
enum BorrowKind {
    Immutable,
    Mutable,
}

/// Information about an active borrow
#[derive(Debug, Clone)]
struct Borrow {
    kind: BorrowKind,
    variable: String, // Which variable is being borrowed from
}

/// Tracks active borrows to enforce borrow rules
#[derive(Debug)]
pub struct BorrowRulesChecker {
    /// Active borrows: reference -> borrow info
    active_borrows: HashMap<String, Vec<Borrow>>,

    /// Variables that are currently borrowed (cannot be moved or mutated)
    borrowed_vars: HashMap<String, Vec<BorrowKind>>,

    /// Registry of builtin function metadata
    builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry,
}

impl Default for BorrowRulesChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl BorrowRulesChecker {
    /// Create a new borrow rules checker
    pub fn new() -> Self {
        Self {
            active_borrows: HashMap::new(),
            borrowed_vars: HashMap::new(),
            builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry::new(),
        }
    }

    /// Check an entire program for borrow rule violations
    pub fn check_program(&mut self, program: &Program) -> BorrowResult<()> {
        for item in &program.items {
            self.check_item(item)?;
        }
        Ok(())
    }

    /// Check a top-level item
    fn check_item(&mut self, item: &Item) -> BorrowResult<()> {
        match item {
            Item::Function(func) => {
                // Create new scope for function
                let saved_borrows = self.active_borrows.clone();
                let saved_borrowed = self.borrowed_vars.clone();

                // Check function body
                for stmt in &func.body.statements {
                    self.check_statement(stmt)?;
                }

                // Restore scope
                self.active_borrows = saved_borrows;
                self.borrowed_vars = saved_borrowed;

                Ok(())
            }
            _ => Ok(()), // No borrow rules in type definitions
        }
    }

    /// Check a statement for borrow rule violations
    fn check_statement(&mut self, stmt: &Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let { name, value, .. } => {
                // Check if the value creates any borrows
                self.check_expression_for_borrows(value)?;

                // If value is a reference expression, track the borrow
                if let Expression::Reference { is_mutable, expr } = value {
                    if let Expression::Ident(var) = expr.as_ref() {
                        self.create_borrow(
                            name.clone(),
                            var.clone(),
                            if *is_mutable {
                                BorrowKind::Mutable
                            } else {
                                BorrowKind::Immutable
                            },
                        )?;
                    }
                }

                Ok(())
            }

            Statement::Assign { target, value } => {
                // Check if we're trying to mutate a borrowed variable
                if let Expression::Ident(var) = target {
                    if let Some(borrows) = self.borrowed_vars.get(var) {
                        if !borrows.is_empty() {
                            return Err(BorrowError::MutationWhileBorrowed {
                                variable: var.clone(),
                                borrowed_at: None,
                            });
                        }
                    }
                }

                self.check_expression_for_borrows(target)?;
                self.check_expression_for_borrows(value)?;

                Ok(())
            }

            Statement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.check_expression_for_borrows(expr)?;
                }
                Ok(())
            }

            Statement::Expression(expr) => {
                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            Statement::If {
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.check_expression_for_borrows(condition)?;

                for stmt in &then_block.statements {
                    self.check_statement(stmt)?;
                }

                // Check elif branches
                for (elif_cond, elif_block) in elif_branches {
                    self.check_expression_for_borrows(elif_cond)?;
                    for stmt in &elif_block.statements {
                        self.check_statement(stmt)?;
                    }
                }

                if let Some(else_blk) = else_block {
                    for stmt in &else_blk.statements {
                        self.check_statement(stmt)?;
                    }
                }

                Ok(())
            }

            Statement::While { condition, body } => {
                self.check_expression_for_borrows(condition)?;

                for stmt in &body.statements {
                    self.check_statement(stmt)?;
                }

                Ok(())
            }

            Statement::For {
                init,
                condition,
                post,
                body,
            } => {
                if let Some(init_stmt) = init {
                    self.check_statement(init_stmt)?;
                }

                if let Some(cond) = condition {
                    self.check_expression_for_borrows(cond)?;
                }

                if let Some(post_stmt) = post {
                    self.check_statement(post_stmt)?;
                }

                for stmt in &body.statements {
                    self.check_statement(stmt)?;
                }

                Ok(())
            }

            Statement::ForIn { iterable, body, .. } => {
                self.check_expression_for_borrows(iterable)?;

                for stmt in &body.statements {
                    self.check_statement(stmt)?;
                }

                Ok(())
            }

            Statement::Switch {
                value,
                cases,
                default_case,
            } => {
                if let Some(expr) = value {
                    self.check_expression_for_borrows(expr)?;
                }

                for case in cases {
                    for stmt in &case.body.statements {
                        self.check_statement(stmt)?;
                    }
                }

                if let Some(default) = default_case {
                    for stmt in &default.statements {
                        self.check_statement(stmt)?;
                    }
                }

                Ok(())
            }

            _ => Ok(()),
        }
    }

    /// Check an expression for borrow rule violations
    fn check_expression_for_borrows(&mut self, expr: &Expression) -> BorrowResult<()> {
        match expr {
            Expression::Reference { is_mutable, expr } => {
                // Check if we can create this borrow
                if let Expression::Ident(var) = expr.as_ref() {
                    self.check_can_borrow(
                        var,
                        if *is_mutable {
                            BorrowKind::Mutable
                        } else {
                            BorrowKind::Immutable
                        },
                    )?;
                }

                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            Expression::Binary { left, right, .. } => {
                self.check_expression_for_borrows(left)?;
                self.check_expression_for_borrows(right)?;
                Ok(())
            }

            Expression::Unary { expr, .. } => {
                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            Expression::Call { func, args } => {
                self.check_expression_for_borrows(func)?;

                // Check if this is a builtin function call
                if let Expression::Ident(func_name) = func.as_ref() {
                    if let Some(metadata) = self.builtin_registry.get(func_name).cloned() {
                        // Check each argument against builtin metadata
                        for (i, arg) in args.iter().enumerate() {
                            if i < metadata.param_effects.len() {
                                let effect = &metadata.param_effects[i];

                                // Check if we're passing a borrowed variable to a mutating builtin
                                if let Expression::Ident(var_name) = arg {
                                    use super::builtin_metadata::ParamEffect;

                                    match effect {
                                        ParamEffect::BorrowsMut | ParamEffect::Mutates => {
                                            // Check if variable is currently borrowed
                                            if let Some(borrows) = self.borrowed_vars.get(var_name)
                                            {
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
                                            if let Some(borrows) = self.borrowed_vars.get(var_name)
                                            {
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
                        return Ok(());
                    }
                }

                // Not a builtin, check args normally
                for arg in args {
                    self.check_expression_for_borrows(arg)?;
                }
                Ok(())
            }

            Expression::MethodCall { receiver, args, .. } => {
                self.check_expression_for_borrows(receiver)?;
                for arg in args {
                    self.check_expression_for_borrows(arg)?;
                }
                Ok(())
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
                self.check_expression_for_borrows(start)?;
                self.check_expression_for_borrows(end)?;
                Ok(())
            }

            Expression::Deref(expr) => {
                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            Expression::Await(expr) | Expression::Go(expr) | Expression::Try(expr) => {
                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            Expression::Match { value, arms } => {
                self.check_expression_for_borrows(value)?;
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.check_expression_for_borrows(guard)?;
                    }
                    self.check_expression_for_borrows(&arm.body)?;
                }
                Ok(())
            }

            Expression::New(expr) => {
                self.check_expression_for_borrows(expr)?;
                Ok(())
            }

            // Literals and identifiers don't create borrows
            _ => Ok(()),
        }
    }

    /// Check if a variable can be borrowed with the given kind
    fn check_can_borrow(&self, var: &str, kind: BorrowKind) -> BorrowResult<()> {
        if let Some(existing_borrows) = self.borrowed_vars.get(var) {
            match kind {
                BorrowKind::Mutable => {
                    // Cannot borrow as mutable if ANY borrows exist
                    if !existing_borrows.is_empty() {
                        if existing_borrows.contains(&BorrowKind::Mutable) {
                            return Err(BorrowError::MutableBorrowWhileBorrowed {
                                variable: var.to_string(),
                                existing_borrow: None,
                                new_borrow: None,
                            });
                        } else {
                            return Err(BorrowError::MutableBorrowWhileBorrowed {
                                variable: var.to_string(),
                                existing_borrow: Some("immutably borrowed".to_string()),
                                new_borrow: None,
                            });
                        }
                    }
                }
                BorrowKind::Immutable => {
                    // Cannot borrow as immutable if mutable borrow exists
                    if existing_borrows.contains(&BorrowKind::Mutable) {
                        return Err(BorrowError::ImmutableBorrowWhileMutableBorrowed {
                            variable: var.to_string(),
                            mutable_borrow: None,
                            new_borrow: None,
                        });
                    }
                    // Multiple immutable borrows are OK
                }
            }
        }

        Ok(())
    }

    /// Create a new borrow
    fn create_borrow(
        &mut self,
        ref_name: String,
        var: String,
        kind: BorrowKind,
    ) -> BorrowResult<()> {
        // First check if this borrow is allowed
        self.check_can_borrow(&var, kind.clone())?;

        // Track the borrow
        self.active_borrows
            .entry(ref_name)
            .or_insert_with(Vec::new)
            .push(Borrow {
                kind: kind.clone(),
                variable: var.clone(),
            });

        // Mark the variable as borrowed
        self.borrowed_vars
            .entry(var)
            .or_insert_with(Vec::new)
            .push(kind);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vex_ast::{Expression, Statement, Type};

    #[test]
    fn test_multiple_immutable_borrows_ok() {
        let mut checker = BorrowRulesChecker::new();

        // let r1 = &x;
        checker
            .create_borrow("r1".to_string(), "x".to_string(), BorrowKind::Immutable)
            .unwrap();

        // let r2 = &x;  (should be OK)
        let result =
            checker.create_borrow("r2".to_string(), "x".to_string(), BorrowKind::Immutable);

        assert!(result.is_ok());
    }

    #[test]
    fn test_mutable_borrow_blocks_immutable() {
        let mut checker = BorrowRulesChecker::new();

        // let m = &x!;
        checker
            .create_borrow("m".to_string(), "x".to_string(), BorrowKind::Mutable)
            .unwrap();

        // let r = &x;  (should fail)
        let result = checker.create_borrow("r".to_string(), "x".to_string(), BorrowKind::Immutable);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BorrowError::ImmutableBorrowWhileMutableBorrowed { .. })
        ));
    }

    #[test]
    fn test_immutable_borrow_blocks_mutable() {
        let mut checker = BorrowRulesChecker::new();

        // let r = &x;
        checker
            .create_borrow("r".to_string(), "x".to_string(), BorrowKind::Immutable)
            .unwrap();

        // let m = &x!;  (should fail)
        let result = checker.create_borrow("m".to_string(), "x".to_string(), BorrowKind::Mutable);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BorrowError::MutableBorrowWhileBorrowed { .. })
        ));
    }

    #[test]
    fn test_multiple_mutable_borrows_fail() {
        let mut checker = BorrowRulesChecker::new();

        // let m1 = &x!;
        checker
            .create_borrow("m1".to_string(), "x".to_string(), BorrowKind::Mutable)
            .unwrap();

        // let m2 = &x!;  (should fail)
        let result = checker.create_borrow("m2".to_string(), "x".to_string(), BorrowKind::Mutable);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BorrowError::MutableBorrowWhileBorrowed { .. })
        ));
    }

    #[test]
    fn test_mutation_while_borrowed_fails() {
        let mut checker = BorrowRulesChecker::new();

        // let r = &x;
        checker
            .create_borrow("r".to_string(), "x".to_string(), BorrowKind::Immutable)
            .unwrap();

        // x = 10;  (should fail)
        let assign = Statement::Assign {
            target: Expression::Ident("x".to_string()),
            value: Expression::IntLiteral(10),
        };

        let result = checker.check_statement(&assign);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(BorrowError::MutationWhileBorrowed { .. })
        ));
    }
}
