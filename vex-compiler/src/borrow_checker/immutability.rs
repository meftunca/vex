// Phase 1: Immutability Checker
// Enforces let vs let! semantics

use std::collections::HashSet;
use vex_ast::{Expression, Program, Statement};

use super::errors::{BorrowError, BorrowResult};

/// Checks that immutable variables (let) are not reassigned
pub struct ImmutabilityChecker {
    /// Variables declared with `let` (immutable)
    pub(crate) immutable_vars: HashSet<String>,

    /// Variables declared with `let!` (mutable)
    mutable_vars: HashSet<String>,

    /// Builtin function registry for identifying builtin functions
    builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry,

    /// Current function being checked (for error location tracking)
    current_function: Option<String>,
}

impl ImmutabilityChecker {
    pub fn new() -> Self {
        Self {
            immutable_vars: HashSet::new(),
            mutable_vars: HashSet::new(),
            builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry::new(),
            current_function: None,
        }
    }

    /// Check an entire program for immutability violations
    pub fn check_program(&mut self, program: &Program) -> BorrowResult<()> {
        for item in &program.items {
            self.check_item(item)?;
        }
        Ok(())
    }

    /// Check a top-level item (function, struct, etc.)
    fn check_item(&mut self, item: &vex_ast::Item) -> BorrowResult<()> {
        use vex_ast::Item;

        match item {
            Item::Function(func) => {
                // Track current function name for error messages
                self.current_function = Some(func.name.clone());

                // Create new scope for function
                let saved_immutable = self.immutable_vars.clone();
                let saved_mutable = self.mutable_vars.clone();

                // Function parameters are always mutable (local bindings)
                for param in &func.params {
                    self.mutable_vars.insert(param.name.clone());
                }

                // ⭐ NEW: Handle method receiver (self)
                if func.receiver.is_some() {
                    if func.is_mutable {
                        // Mutable method: self can be mutated
                        self.mutable_vars.insert("self".to_string());
                    } else {
                        // Immutable method: self is immutable
                        self.immutable_vars.insert("self".to_string());
                    }
                }

                // Check function body
                for stmt in &func.body.statements {
                    self.check_statement(stmt)?;
                }

                // Restore scope
                self.immutable_vars = saved_immutable;
                self.mutable_vars = saved_mutable;
                self.current_function = None;

                Ok(())
            }
            Item::Struct(strukt) => {
                // ⭐ NEW: Check struct methods
                for method in &strukt.methods {
                    // Create new scope for method
                    let saved_immutable = self.immutable_vars.clone();
                    let saved_mutable = self.mutable_vars.clone();

                    // Method parameters are always mutable (local bindings)
                    for param in &method.params {
                        self.mutable_vars.insert(param.name.clone());
                    }

                    // Handle method receiver (self)
                    // Methods always have implicit or explicit receiver
                    if method.is_mutable {
                        // Mutable method: self can be mutated
                        self.mutable_vars.insert("self".to_string());
                    } else {
                        // Immutable method: self is immutable
                        self.immutable_vars.insert("self".to_string());
                    }

                    // Check method body
                    for stmt in &method.body.statements {
                        self.check_statement(stmt)?;
                    }

                    // Restore scope
                    self.immutable_vars = saved_immutable;
                    self.mutable_vars = saved_mutable;
                }

                Ok(())
            }
            Item::Enum(_)
            | Item::Trait(_)
            | Item::TraitImpl(_)
            | Item::TypeAlias(_)
            | Item::Policy(_)
            | Item::ExternBlock(_)
            | Item::Export(_)
            | Item::Const(_) => {
                // No immutability checks needed for type definitions
                Ok(())
            }
        }
    }

    /// Check a statement for immutability violations
    fn check_statement(&mut self, stmt: &Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let {
                name,
                is_mutable,
                value,
                ..
            } => {
                // Register the variable
                if *is_mutable {
                    self.mutable_vars.insert(name.clone());
                } else {
                    self.immutable_vars.insert(name.clone());
                }

                // Check the initializer expression
                self.check_expression(value)?;

                Ok(())
            }

            Statement::Assign { target, value } => {
                // Check if assigning to immutable variable or its fields
                match target {
                    Expression::Ident(name) => {
                        // Direct assignment: x = ...
                        if self.immutable_vars.contains(name) {
                            let location = self
                                .current_function
                                .as_ref()
                                .map(|f| format!("in function `{}`", f));
                            return Err(BorrowError::AssignToImmutable {
                                variable: name.clone(),
                                location,
                            });
                        }
                    }
                    Expression::FieldAccess { object, field } => {
                        // Field assignment: x.field = ...
                        // ⭐ NEW: Check if base object is immutable
                        if let Expression::Ident(base_name) = &**object {
                            if self.immutable_vars.contains(base_name) {
                                let location = self
                                    .current_function
                                    .as_ref()
                                    .map(|f| format!("in function `{}`", f));
                                return Err(BorrowError::AssignToImmutableField {
                                    variable: base_name.clone(),
                                    field: field.clone(),
                                    location,
                                });
                            }
                        }
                    }
                    Expression::Index { object, .. } => {
                        // Index assignment: x[i] = ...
                        if let Expression::Ident(base_name) = &**object {
                            if self.immutable_vars.contains(base_name) {
                                let location = self
                                    .current_function
                                    .as_ref()
                                    .map(|f| format!("in function `{}`", f));
                                return Err(BorrowError::AssignToImmutable {
                                    variable: base_name.clone(),
                                    location,
                                });
                            }
                        }
                    }
                    _ => {}
                }

                // Check the value expression
                self.check_expression(value)?;

                Ok(())
            }

            Statement::Return(value) => {
                if let Some(expr) = value {
                    self.check_expression(expr)?;
                }
                Ok(())
            }

            Statement::Expression(expr) => {
                // Just check the expression
                self.check_expression(expr)?;
                Ok(())
            }

            Statement::If {
                span_id: _,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                // Check condition
                self.check_expression(condition)?;

                // Check branches
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

                if let Some(else_blk) = else_block {
                    for stmt in &else_blk.statements {
                        self.check_statement(stmt)?;
                    }
                }

                Ok(())
            }

            Statement::While {
                span_id: _,
                condition,
                body,
            } => {
                // Check condition
                self.check_expression(condition)?;

                // Check body
                for stmt in &body.statements {
                    self.check_statement(stmt)?;
                }

                Ok(())
            }

            Statement::For {
                span_id: _,
                init,
                condition,
                post,
                body,
            } => {
                // Check init if present
                if let Some(init_stmt) = init {
                    self.check_statement(init_stmt)?;
                }

                // Check condition if present
                if let Some(cond) = condition {
                    self.check_expression(cond)?;
                }

                // Check post if present
                if let Some(post_stmt) = post {
                    self.check_statement(post_stmt)?;
                }

                // Check body
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

            _ => Ok(()),
        }
    }

    /// Check an expression (may contain nested assignments)
    fn check_expression(&mut self, expr: &Expression) -> BorrowResult<()> {
        match expr {
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

            Expression::Call {
                span_id: _,
                func,
                args,
            } => {
                // Skip checking builtin function names as variables
                if let Expression::Ident(func_name) = func.as_ref() {
                    if !self.builtin_registry.is_builtin(func_name) {
                        self.check_expression(func)?;
                    }
                } else {
                    self.check_expression(func)?;
                }
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::MethodCall { receiver, args, .. } => {
                self.check_expression(receiver)?;
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::FieldAccess { object, .. } => {
                self.check_expression(object)?;
                Ok(())
            }

            Expression::StructLiteral { fields, .. } => {
                for (_field_name, field_value) in fields {
                    self.check_expression(field_value)?;
                }
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

            Expression::Index { object, index } => {
                self.check_expression(object)?;
                self.check_expression(index)?;
                Ok(())
            }

            // Literals and identifiers don't need checking
            Expression::IntLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BoolLiteral(_)
            | Expression::Ident(_) => Ok(()),

            Expression::Closure { params, body, .. } => {
                // Register closure parameters as immutable variables
                for param in params {
                    self.immutable_vars.insert(param.name.clone());
                }

                // Check closure body
                self.check_expression(body)?;

                // Note: We don't remove params from scope here because
                // ImmutabilityChecker doesn't track scopes, just tracks
                // which variables are mutable vs immutable across the whole program
                Ok(())
            }

            _ => Ok(()),
        }
    }
}

impl Default for ImmutabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vex_ast::Type;

    #[test]
    fn test_immutable_assignment_error() {
        let mut checker = ImmutabilityChecker::new();

        // Simulate: let x = 42;
        checker.immutable_vars.insert("x".to_string());

        // Simulate: x = 50;
        let assign_stmt = Statement::Assign {
            target: Expression::Ident("x".to_string()),
            value: Expression::IntLiteral(50),
        };

        let result = checker.check_statement(&assign_stmt);
        assert!(result.is_err());

        if let Err(BorrowError::AssignToImmutable { variable, .. }) = result {
            assert_eq!(variable, "x");
        } else {
            panic!("Expected AssignToImmutable error");
        }
    }

    #[test]
    fn test_mutable_assignment_ok() {
        let mut checker = ImmutabilityChecker::new();

        // Simulate: let! y = 10;
        checker.mutable_vars.insert("y".to_string());

        // Simulate: y = 20;
        let assign_stmt = Statement::Assign {
            target: Expression::Ident("y".to_string()),
            value: Expression::IntLiteral(20),
        };

        let result = checker.check_statement(&assign_stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_let_declaration() {
        let mut checker = ImmutabilityChecker::new();

        // Simulate: let x = 42;
        let let_stmt = Statement::Let {
            name: "x".to_string(),
            ty: Some(Type::I32),
            value: Expression::IntLiteral(42),
            is_mutable: false,
        };

        checker.check_statement(&let_stmt).unwrap();
        assert!(checker.immutable_vars.contains("x"));
        assert!(!checker.mutable_vars.contains("x"));
    }

    #[test]
    fn test_let_mutable_declaration() {
        let mut checker = ImmutabilityChecker::new();

        // Simulate: let! y = 10;
        let let_stmt = Statement::Let {
            name: "y".to_string(),
            ty: Some(Type::I32),
            value: Expression::IntLiteral(10),
            is_mutable: true,
        };

        checker.check_statement(&let_stmt).unwrap();
        assert!(checker.mutable_vars.contains("y"));
        assert!(!checker.immutable_vars.contains("y"));
    }
}
