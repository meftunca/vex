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

use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use std::collections::{HashMap, HashSet};
use vex_ast::{Expression, Item, Program, Statement, Type};

/// Tracks which variables have been moved and are no longer accessible
#[derive(Debug)]
pub struct MoveChecker {
    /// Variables that have been moved (and are now invalid)
    moved_vars: HashSet<String>,

    /// Variables that are currently valid
    valid_vars: HashSet<String>,

    /// Type information for variables (to determine Copy vs Move)
    var_types: HashMap<String, Type>,

    /// Builtin function registry for identifying builtin functions
    builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry,
}

impl MoveChecker {
    /// Create a new move checker
    pub fn new() -> Self {
        Self {
            moved_vars: HashSet::new(),
            valid_vars: HashSet::new(),
            var_types: HashMap::new(),
            builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry::new(),
        }
    }

    /// Check an entire program for move violations
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
                let saved_moved = self.moved_vars.clone();
                let saved_valid = self.valid_vars.clone();
                let saved_types = self.var_types.clone();

                // Function parameters are valid at start
                for param in &func.params {
                    self.valid_vars.insert(param.name.clone());
                    self.var_types.insert(param.name.clone(), param.ty.clone());
                }

                // Check function body
                for stmt in &func.body.statements {
                    self.check_statement(stmt)?;
                }

                // Restore scope
                self.moved_vars = saved_moved;
                self.valid_vars = saved_valid;
                self.var_types = saved_types;

                Ok(())
            }
            _ => Ok(()), // No move semantics in type definitions
        }
    }

    /// Check a statement for move violations
    fn check_statement(&mut self, stmt: &Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let {
                name, ty, value, ..
            } => {
                // Check if the initializer moves any variables
                self.check_expression(value)?;

                // If value is a move type identifier, mark it as moved
                if let Expression::Ident(var) = value {
                    if let Some(var_ty) = self.var_types.get(var) {
                        if self.is_move_type(var_ty) {
                            self.moved_vars.insert(var.clone());
                            self.valid_vars.remove(var);
                        }
                    }
                }

                // Register the new variable (or re-declare existing one)
                // If this is a re-declaration (shadowing), it makes the name valid again
                self.moved_vars.remove(name);
                self.valid_vars.insert(name.clone());

                if let Some(t) = ty {
                    self.var_types.insert(name.clone(), t.clone());
                } else {
                    // Infer type from the initializer expression
                    let inferred_ty = match value {
                        Expression::StringLiteral(_) | Expression::FStringLiteral(_) => {
                            Some(Type::String)
                        }
                        Expression::IntLiteral(_) => Some(Type::I32),
                        Expression::FloatLiteral(_) => Some(Type::F64),
                        Expression::BoolLiteral(_) => Some(Type::Bool),
                        Expression::Ident(var) => self.var_types.get(var).cloned(),
                        _ => None,
                    };

                    if let Some(ty_val) = inferred_ty {
                        self.var_types.insert(name.clone(), ty_val);
                    }
                }

                Ok(())
            }

            Statement::Assign { target, value } => {
                // Check the value expression (right side)
                self.check_expression(value)?;

                // If assigning to a simple variable, it becomes valid again
                // (don't check target for moves - assignment reinitializes it)
                if let Expression::Ident(var) = target {
                    self.moved_vars.remove(var);
                    self.valid_vars.insert(var.clone());
                } else {
                    // For complex targets (field access, index), check them
                    self.check_expression(target)?;
                }

                Ok(())
            }

            Statement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.check_expression(expr)?;
                }
                Ok(())
            }

            Statement::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }

            Statement::If {
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.check_expression(condition)?;

                // Check then branch
                for stmt in &then_block.statements {
                    self.check_statement(stmt)?;
                }

                // Check elif branches
                for (elif_cond, elif_block) in elif_branches {
                    self.check_expression(elif_cond)?;
                    for stmt in &elif_block.statements {
                        self.check_statement(stmt)?;
                    }
                }

                // Check else branch
                if let Some(else_blk) = else_block {
                    for stmt in &else_blk.statements {
                        self.check_statement(stmt)?;
                    }
                }

                Ok(())
            }

            Statement::While { condition, body } => {
                self.check_expression(condition)?;

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
                    self.check_expression(cond)?;
                }

                if let Some(post_stmt) = post {
                    self.check_statement(post_stmt)?;
                }

                for stmt in &body.statements {
                    self.check_statement(stmt)?;
                }

                Ok(())
            }

            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                self.check_expression(iterable)?;

                // Loop variable is valid in the loop body
                self.valid_vars.insert(variable.clone());

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
                    self.check_expression(expr)?;
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

            _ => Ok(()), // Other statement types don't affect moves
        }
    }

    /// Check an expression for use of moved variables
    fn check_expression(&mut self, expr: &Expression) -> BorrowResult<()> {
        match expr {
            Expression::Ident(name) => {
                // Skip builtin functions - they're not variables
                if self.builtin_registry.is_builtin(name) {
                    return Ok(());
                }

                // Check if this variable has been moved
                if self.moved_vars.contains(name) {
                    return Err(BorrowError::UseAfterMove {
                        variable: name.clone(),
                        moved_at: None,
                        used_at: None,
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

            Expression::Binary { left, right, .. } => {
                self.check_expression(left)?;
                self.check_expression(right)?;
                Ok(())
            }

            Expression::Unary { expr, .. } => {
                self.check_expression(expr)?;
                Ok(())
            }

            Expression::Call { func, args } => {
                self.check_expression(func)?;

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

            Expression::MethodCall { receiver, args, .. } => {
                self.check_expression(receiver)?;

                for arg in args {
                    self.check_expression(arg)?;

                    // Mark move type args as moved
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
                self.check_expression(start)?;
                self.check_expression(end)?;
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

            Expression::Await(expr) | Expression::Go(expr) | Expression::Try(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }

            Expression::Match { value, arms } => {
                self.check_expression(value)?;

                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.check_expression(guard)?;
                    }
                    self.check_expression(&arm.body)?;
                }

                Ok(())
            }

            Expression::New(expr) => {
                self.check_expression(expr)?;
                Ok(())
            }

            // Literals don't reference variables
            Expression::IntLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::FStringLiteral(_)
            | Expression::BoolLiteral(_)
            | Expression::Nil => Ok(()),

            Expression::Closure { params, body, .. } => {
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

            _ => Ok(()), // Other expressions don't affect moves
        }
    }

    /// Determine if a type is Copy or Move
    ///
    /// Copy types: primitive integers, floats, bools, references
    /// Move types: String, structs, enums, arrays (for now)
    fn is_move_type(&self, ty: &Type) -> bool {
        match ty {
            // Primitive types are Copy
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::I128
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::U128
            | Type::F32
            | Type::F64
            | Type::F128
            | Type::Bool
            | Type::Byte => false,

            // References are Copy (copying a pointer)
            Type::Reference(_, _) => false,

            // String is Move
            Type::String => true,

            // Builtin types are Move (Phase 0)
            Type::Option(_) => true,    // Option<T> is Move (contains T)
            Type::Result(_, _) => true, // Result<T,E> is Move
            Type::Vec(_) => true,       // Vec<T> is Move (owns heap data)
            Type::Box(_) => true,       // Box<T> is Move (owns heap allocation)

            // Named types are Move by default (structs, enums)
            Type::Named(_) => true,

            // Generic types are Move by default
            Type::Generic { .. } => true,

            // Arrays and slices are Move
            Type::Array(_, _) | Type::Slice(_, _) => true,

            // Tuples are Move if any element is Move
            Type::Tuple(types) => types.iter().any(|t| self.is_move_type(t)),

            // Function types are Copy (function pointers)
            Type::Function { .. } => false,

            // Complex types are Move
            Type::Union(_) | Type::Intersection(_) | Type::Conditional { .. } => true,

            // Unit type is Copy
            Type::Unit | Type::Nil | Type::Error => false,

            Type::Infer(_) => false, // Infer is only for type checking
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            func: Box::new(Expression::Ident("foo".to_string())),
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
