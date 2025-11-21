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
            .create_borrow("r1".to_string(), "x".to_string(), BorrowKind::Immutable, None)
            .unwrap();

        // let r2 = &x;  (should be OK)
        let result =
            checker.create_borrow("r2".to_string(), "x".to_string(), BorrowKind::Immutable, None);

        assert!(result.is_ok());
    }

    #[test]
    fn test_mutable_borrow_blocks_immutable() {
        let mut checker = BorrowRulesChecker::new();

        // let m = &x!;
        checker
            .create_borrow("m".to_string(), "x".to_string(), BorrowKind::Mutable, None)
            .unwrap();

        // let r = &x;  (should fail)
        let result = checker.create_borrow("r".to_string(), "x".to_string(), BorrowKind::Immutable, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_immutable_borrow_blocks_mutable() {
        let mut checker = BorrowRulesChecker::new();

        // let r = &x;
        checker
            .create_borrow("r".to_string(), "x".to_string(), BorrowKind::Immutable, None)
            .unwrap();

        // let m = &x!;  (should fail)
        let result = checker.create_borrow("m".to_string(), "x".to_string(), BorrowKind::Mutable, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_mutable_borrows_fail() {
        let mut checker = BorrowRulesChecker::new();

        // let m1 = &x!;
        checker
            .create_borrow("m1".to_string(), "x".to_string(), BorrowKind::Mutable, None)
            .unwrap();

        // let m2 = &x!;  (should fail)
        let result = checker.create_borrow("m2".to_string(), "x".to_string(), BorrowKind::Mutable, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_mutation_while_borrowed_fails() {
        let mut checker = BorrowRulesChecker::new();

        // let r = &x;
        checker
            .create_borrow("r".to_string(), "x".to_string(), BorrowKind::Immutable, None)
            .unwrap();

        // x = 10;  (should fail)
        let assign = Statement::Assign {
            span_id: None,
            target: Expression::Ident("x".to_string()),
            value: Expression::IntLiteral(10),
        };

        let result = checker.check_statement(&assign, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_borrow_related_location_in_error() {
        let mut checker = BorrowRulesChecker::new();

        // record a borrow with a span id
        let span_id = "span_99".to_string();
        checker
            .create_borrow("r".to_string(), "x".to_string(), BorrowKind::Immutable, Some(span_id.clone()))
            .unwrap();

        // try to create a conflicting mutable borrow
        let result = checker.create_borrow("m".to_string(), "x".to_string(), BorrowKind::Mutable, None);
        assert!(result.is_err());
        if let Err(err) = result {
            // Build span map that resolves span_99
            let mut span_map = vex_diagnostics::SpanMap::new();
            let path = std::env::temp_dir().join("file2.vx");
            span_map.record(span_id.clone(), vex_diagnostics::Span::new(path.display().to_string(), 12, 4, 1));

            let diag = err.to_diagnostic(&span_map);
            assert_eq!(diag.related.len(), 1);
            assert!(diag.related[0].0.file.contains("file2.vx"));
        }
    }

    #[test]
    fn test_mutation_while_borrowed_parent_span_fallback() {
        let mut checker = BorrowRulesChecker::new();

        // Create a borrow without an explicit recorded location
        checker
            .create_borrow("r".to_string(), "x".to_string(), BorrowKind::Immutable, None)
            .unwrap();

        // Prepare an assignment that will trigger MutationWhileBorrowed
        let assign = Statement::Assign {
            span_id: None,
            target: Expression::Ident("x".to_string()),
            value: Expression::IntLiteral(10),
        };

        // Provide a parent span id as the contextual location
        let parent_span = "span_parent".to_string();
        let result = checker.check_statement(&assign, Some(&parent_span));
        assert!(result.is_err());

        if let Err(err) = result {
            // Make sure the error includes the parent_span as borrowed_at
            use crate::borrow_checker::errors::BorrowError;
            match &err {
                BorrowError::MutationWhileBorrowed { borrowed_at, .. } => {
                    assert_eq!(borrowed_at, &Some(parent_span.clone()));
                }
                _ => panic!("Unexpected error type"),
            }
            // Build span map that resolves parent span
            let mut span_map = vex_diagnostics::SpanMap::new();
            let path = std::env::temp_dir().join("file_parent.vx");
            span_map.record(parent_span.clone(), vex_diagnostics::Span::new(path.display().to_string(), 3, 2, 1));

            let diag = err.to_diagnostic(&span_map);
            // The diagnostic should include a related span pointing to parent_span
            assert_eq!(diag.related.len(), 1);
            assert!(diag.related[0].0.file.contains("file_parent.vx"));
            assert!(diag.related[0].1.contains("borrow occurs") || diag.related[0].1.contains("borrow"));
        }
    }
}
