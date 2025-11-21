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
            move_locations: std::collections::HashMap::new(),
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
        assert!(checker.check_expression(&expr, None).is_ok());

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

        assert!(checker.check_expression(&call, None).is_ok());

        // s should now be moved
        assert!(checker.moved_vars.contains("s"));

        // Using s again should fail
        let use_s = Expression::Ident("s".to_string());
        let result = checker.check_expression(&use_s, None);
        assert!(result.is_err());
        assert!(matches!(result, Err(BorrowError::UseAfterMove { .. })));
    }

    #[test]
    fn test_use_after_move_diagnostic_related_span() {
        let mut checker = MoveChecker::new();
        checker.valid_vars.insert("t".to_string());
        checker.var_types.insert("t".to_string(), Type::String);

        // make a span map and record spans
        let mut span_map = vex_diagnostics::SpanMap::new();
        let span_id = span_map.generate_id();
        let path = std::env::temp_dir().join("file.vx");
        span_map.record(
            span_id.clone(),
            vex_diagnostics::Span::new(path.display().to_string(), 5, 1, 1),
        );

        // foo(t) moves t and should record move_locations
        let call = Expression::Call {
            span_id: Some(span_id.clone()),
            func: Box::new(Expression::Ident("foo".to_string())),
            type_args: vec![],
            args: vec![Expression::Ident("t".to_string())],
        };
        assert!(checker.check_expression(&call, None).is_ok());
        // Now using t should cause UseAfterMove
        let result = checker.check_expression(&Expression::Ident("t".to_string()), None);
        assert!(result.is_err());
        let err = result.err().unwrap();
        match &err {
            crate::borrow_checker::errors::BorrowError::UseAfterMove { moved_at, .. } => {
                assert_eq!(moved_at, &Some(span_id.clone()));
            }
            _ => panic!("Expected UseAfterMove"),
        }
        let diag = err.to_diagnostic(&span_map);
        assert_eq!(diag.related.len(), 1);
        assert!(diag.related[0].0.file.contains("file.vx"));
    }

    #[test]
    fn test_use_after_move_error() {
        let mut checker = MoveChecker::new();

        // Simulate: s was moved
        checker.moved_vars.insert("s".to_string());

        // Try to use s
        let expr = Expression::Ident("s".to_string());
        let result = checker.check_expression(&expr, None);

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
            span_id: None,
            target: Expression::Ident("s".to_string()),
            value: Expression::StringLiteral("new".to_string()),
        };

        assert!(checker.check_statement(&assign, None).is_ok());

        // s should be valid now
        assert!(!checker.moved_vars.contains("s"));
        assert!(checker.valid_vars.contains("s"));
    }

    #[test]
    fn test_move_locations_recorded_for_assign() {
        let mut checker = MoveChecker::new();
        checker.valid_vars.insert("t".to_string());
        checker.var_types.insert("t".to_string(), Type::String);

        let assign = Statement::Assign {
            span_id: Some("span_42".to_string()),
            target: Expression::Ident("s".to_string()),
            value: Expression::Ident("t".to_string()),
        };

        assert!(checker.check_statement(&assign, None).is_ok());
        assert_eq!(
            checker.move_locations.get("t"),
            Some(&Some("span_42".to_string()))
        );
    }

    #[test]
    fn test_move_locations_recorded_for_call_parent_span() {
        let mut checker = MoveChecker::new();
        checker.valid_vars.insert("t".to_string());
        checker.var_types.insert("t".to_string(), Type::String);

        // call with no span id but parent_span is provided
        let call = Expression::Call {
            span_id: None,
            func: Box::new(Expression::Ident("foo".to_string())),
            type_args: vec![],
            args: vec![Expression::Ident("t".to_string())],
        };

        let parent_span = "span_call".to_string();
        assert!(checker.check_expression(&call, Some(&parent_span)).is_ok());
        assert_eq!(checker.move_locations.get("t"), Some(&Some("span_call".to_string())));
    }

    #[test]
    fn test_move_locations_recorded_for_assign_parent_span() {
        let mut checker = MoveChecker::new();
        checker.valid_vars.insert("t".to_string());
        checker.var_types.insert("t".to_string(), Type::String);

        let assign = Statement::Assign {
            span_id: None,
            target: Expression::Ident("s".to_string()),
            value: Expression::Ident("t".to_string()),
        };

        let parent_span = "span_assign".to_string();
        assert!(checker.check_statement(&assign, Some(&parent_span)).is_ok());
        assert_eq!(checker.move_locations.get("t"), Some(&Some("span_assign".to_string())));
    }

    #[test]
    fn test_move_locations_recorded_for_let_initializer_parent_span() {
        let mut checker = MoveChecker::new();
        checker.valid_vars.insert("t".to_string());
        checker.var_types.insert("t".to_string(), Type::String);

        let let_stmt = Statement::LetPattern {
            is_mutable: true,
            pattern: vex_ast::Pattern::Ident("s".to_string()),
            ty: None,
            value: Expression::Ident("t".to_string()),
        };

        let parent_span = "span_let".to_string();
        assert!(checker.check_statement(&let_stmt, Some(&parent_span)).is_ok());
        assert_eq!(checker.move_locations.get("t"), Some(&Some("span_let".to_string())));
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

        assert!(checker.check_expression(&ref_expr, None).is_ok());

        // s should still be valid
        assert!(!checker.moved_vars.contains("s"));
    }
}
