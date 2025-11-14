//! Move Semantics Checker
//!
//! Phase 2 of borrow checker: Prevents use-after-move errors.
//!
//! Key concepts:
//! - Copy types: i32, f64, bool, etc. - values are copied on assignment
//! - Move types: String, Vec, custom structs - ownership is transferred
//! - After a move, the original variable is invalidated
//!
//! Examples:
//! ```vex
//! let s = "hello";
//! let s2 = s;      // Move (String is move type)
//! log(s);          // ❌ ERROR: use of moved value `s`
//!
//! let x = 42;
//! let y = x;       // Copy (i32 is copy type)
//! log(x);          // ✅ OK: x is still valid
//! ```

mod checker;
mod expression_checking;
mod pattern_helpers;
mod statement_checking;
mod type_classification;

use crate::borrow_checker::errors::BorrowResult;
use vex_ast::Program;

pub use checker::MoveChecker;

impl MoveChecker {
    /// Create a new move checker
    pub fn new() -> Self {
        Self {
            moved_vars: std::collections::HashSet::new(),
            valid_vars: std::collections::HashSet::new(),
            global_vars: std::collections::HashSet::new(),
            var_types: std::collections::HashMap::new(),
            builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry::new(),
            current_function: None,
        }
    }

    /// Check an entire program for move violations
    pub fn check_program(&mut self, program: &Program) -> BorrowResult<()> {
        for item in &program.items {
            self.check_item(item)?;
        }
        Ok(())
    }
}

impl Default for MoveChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borrow_checker::errors::BorrowError;
    use vex_ast::{Expression, Statement, Type};

    #[test]
    fn test_move_primitive_is_copy() {
        let mut checker = MoveChecker::new();

        // let x = 42;
        checker.valid_vars.insert("x".to_string());
        checker.var_types.insert("x".to_string(), Type::I32);

        // let y = x;  (copies x)
        let expr = Expression::Ident("x".to_string());
        assert!(checker.check_expression(&expr).is_ok());

        // x should still be valid (i32 is Copy)
        assert!(!checker.moved_vars.contains("x"));
    }

    #[test]
    fn test_move_string() {
        let mut checker = MoveChecker::new();

        // let s = "hello";
        checker.valid_vars.insert("s".to_string());
        checker.var_types.insert("s".to_string(), Type::String);

        // foo(s);  (moves s into function)
        let call = Expression::Call {
            span_id: None,
            func: Box::new(Expression::Ident("foo".to_string())),
            type_args: vec![],
            args: vec![Expression::Ident("s".to_string())],
        };

        assert!(checker.check_expression(&call).is_ok());

        // s should now be moved
        assert!(checker.moved_vars.contains("s"));

        // Using s again should fail
        let use_s = Expression::Ident("s".to_string());
        let result = checker.check_expression(&use_s);
        assert!(result.is_err());
        assert!(matches!(result, Err(BorrowError::UseAfterMove { .. })));
    }

    #[test]
    fn test_use_after_move_error() {
        let mut checker = MoveChecker::new();

        // Simulate: s was moved
        checker.moved_vars.insert("s".to_string());

        // Try to use s
        let expr = Expression::Ident("s".to_string());
        let result = checker.check_expression(&expr);

        assert!(result.is_err());
        if let Err(BorrowError::UseAfterMove { variable, .. }) = result {
            assert_eq!(variable, "s");
        } else {
            panic!("Expected UseAfterMove error");
        }
    }

    #[test]
    fn test_reassignment_makes_valid() {
        let mut checker = MoveChecker::new();

        // s was moved
        checker.moved_vars.insert("s".to_string());
        checker.var_types.insert("s".to_string(), Type::String);

        // s = "new value";  (reassignment makes it valid again)
        let assign = Statement::Assign {
            target: Expression::Ident("s".to_string()),
            value: Expression::StringLiteral("new".to_string()),
        };

        assert!(checker.check_statement(&assign).is_ok());

        // s should be valid now
        assert!(!checker.moved_vars.contains("s"));
        assert!(checker.valid_vars.contains("s"));
    }

    #[test]
    fn test_reference_doesnt_move() {
        let mut checker = MoveChecker::new();

        checker.valid_vars.insert("s".to_string());
        checker.var_types.insert("s".to_string(), Type::String);

        // &s  (borrowing doesn't move)
        let ref_expr = Expression::Reference {
            is_mutable: false,
            expr: Box::new(Expression::Ident("s".to_string())),
        };

        assert!(checker.check_expression(&ref_expr).is_ok());

        // s should still be valid
        assert!(!checker.moved_vars.contains("s"));
    }
}
