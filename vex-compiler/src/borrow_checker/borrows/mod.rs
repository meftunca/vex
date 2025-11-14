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

mod checker;
mod expression_checking;
mod statement_checking;
mod tracking;

use crate::borrow_checker::errors::BorrowResult;
use std::collections::{HashMap, HashSet};
use vex_ast::Program;

pub use checker::BorrowRulesChecker;

impl BorrowRulesChecker {
    /// Create a new borrow rules checker
    pub fn new() -> Self {
        Self {
            active_borrows: HashMap::new(),
            borrowed_vars: HashMap::new(),
            valid_vars: HashSet::new(),
            builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry::new(),
            current_function: None,
            in_unsafe_block: false,
        }
    }

    /// Check an entire program for borrow rule violations
    pub fn check_program(&mut self, program: &Program) -> BorrowResult<()> {
        for item in &program.items {
            self.check_item(item)?;
        }
        Ok(())
    }
}

impl Default for BorrowRulesChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracking::BorrowKind;
    use vex_ast::{Expression, Statement};

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
    }
}
