//! Statement-level move checking

use super::checker::MoveChecker;
use crate::borrow_checker::errors::BorrowResult;
use vex_ast::{Expression, Statement, Type};

impl MoveChecker {
    /// Check a statement for move violations
    pub(super) fn check_statement(
        &mut self,
        stmt: &Statement,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        match stmt {
            Statement::Let {
                name, ty, value, ..
            } => self.check_let_statement(name, ty, value, parent_span),

            Statement::LetPattern { pattern, value, .. } => {
                self.check_let_pattern_statement(pattern, value, parent_span)
            }

            Statement::Assign {
                span_id,
                target,
                value,
            } => self.check_assign_statement(target, value, span_id.as_ref().or(parent_span)),

            Statement::Return {
                span_id,
                value: expr_opt,
            } => {
                if let Some(expr) = expr_opt {
                    self.check_expression(expr, span_id.as_ref().or(parent_span))?;
                }
                Ok(())
            }

            Statement::Expression(expr) => {
                self.check_expression(expr, parent_span)?;
                Ok(())
            }

            Statement::If {
                span_id,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => self.check_if_statement(
                condition,
                then_block,
                elif_branches,
                else_block,
                span_id.as_ref().or(parent_span),
            ),

            Statement::While {
                span_id,
                condition,
                body,
            } => self.check_while_statement(condition, body, span_id.as_ref().or(parent_span)),

            Statement::For {
                span_id,
                init,
                condition,
                post,
                body,
            } => self.check_for_statement(
                init,
                condition,
                post,
                body,
                span_id.as_ref().or(parent_span),
            ),

            Statement::ForIn {
                span_id,
                variable,
                iterable,
                body,
            } => self.check_for_in_statement(
                variable,
                iterable,
                body,
                span_id.as_ref().or(parent_span),
            ),

            Statement::Switch {
                span_id,
                value,
                cases,
                default_case,
            } => self.check_switch_statement(
                value,
                cases,
                default_case,
                span_id.as_ref().or(parent_span),
            ),

            _ => Ok(()), // Other statement types don't affect moves
        }
    }

    fn check_let_statement(
        &mut self,
        name: &str,
        ty: &Option<Type>,
        value: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Check if the initializer moves any variables
        self.check_expression(value, parent_span)?;

        // Helper to extract a span id from common Expression variants
        fn expr_span_id(expr: &Expression) -> Option<String> {
            match expr {
                Expression::Binary { span_id, .. }
                | Expression::Call { span_id, .. }
                | Expression::Unary { span_id, .. } => span_id.clone(),
                _ => None,
            }
        }

        // If value is a move type identifier, mark it as moved
        if let Expression::Ident(var) = value {
            if let Some(var_ty) = self.var_types.get(var) {
                if self.is_move_type(var_ty) {
                    self.moved_vars.insert(var.clone());
                    self.valid_vars.remove(var);
                    // Record the move location (value expression's span id) when available
                    self.move_locations.insert(
                        var.clone(),
                        expr_span_id(value).or_else(|| parent_span.map(|s| s.clone())),
                    );
                }
            }
        }

        // Register the new variable (or re-declare existing one)
        // If this is a re-declaration (shadowing), it makes the name valid again
        self.moved_vars.remove(name);
        self.valid_vars.insert(name.to_string());

        if let Some(t) = ty {
            self.var_types.insert(name.to_string(), t.clone());
        } else {
            // Infer type from the initializer expression
            let inferred_ty = match value {
                Expression::StringLiteral(_) | Expression::FStringLiteral(_) => Some(Type::String),
                Expression::IntLiteral(_) => Some(Type::I32),
                Expression::TypedIntLiteral { type_suffix, .. } => {
                    Some(match type_suffix.as_str() {
                        "i8" => Type::I8,
                        "i16" => Type::I16,
                        "i32" => Type::I32,
                        "i64" => Type::I64,
                        "u8" => Type::U8,
                        "u16" => Type::U16,
                        "u32" => Type::U32,
                        "u64" => Type::U64,
                        _ => Type::I32,
                    })
                }
                Expression::FloatLiteral(_) => Some(Type::F64),
                Expression::BoolLiteral(_) => Some(Type::Bool),
                Expression::Ident(var) => self.var_types.get(var).cloned(),
                _ => None,
            };

            if let Some(ty_val) = inferred_ty {
                self.var_types.insert(name.to_string(), ty_val);
            }
        }

        Ok(())
    }

    fn check_let_pattern_statement(
        &mut self,
        pattern: &vex_ast::Pattern,
        value: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Check value for moves
        self.check_expression(value, parent_span)?;
        // If value is a simple identifier and is a move type, record move location like Let
        if let Expression::Ident(var) = value {
            if let Some(var_ty) = self.var_types.get(var) {
                if self.is_move_type(var_ty) {
                    self.moved_vars.insert(var.clone());
                    self.valid_vars.remove(var);
                    // Record move location using expression span id or fallback to parent_span
                    fn expr_span_id(expr: &Expression) -> Option<String> {
                        match expr {
                            Expression::Binary { span_id, .. }
                            | Expression::Call { span_id, .. }
                            | Expression::Unary { span_id, .. } => span_id.clone(),
                            _ => None,
                        }
                    }
                    self.move_locations.insert(
                        var.clone(),
                        expr_span_id(value).or_else(|| parent_span.map(|s| s.clone())),
                    );
                }
            }
        }
        // Declare pattern variables
        self.declare_pattern_variables(pattern);
        Ok(())
    }

    fn check_assign_statement(
        &mut self,
        target: &Expression,
        value: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Check the value expression (right side)
        self.check_expression(value, parent_span)?;

        // If assigning to a simple variable, it becomes valid again
        // (don't check target for moves - assignment reinitializes it)
        if let Expression::Ident(var) = target {
            self.moved_vars.remove(var);
            self.valid_vars.insert(var.clone());
        } else {
            // For complex targets (field access, index), check them
            self.check_expression(target, parent_span)?;
        }

        // If value is a simple identifier and is a move type, record move location
        if let Expression::Ident(src_var) = value {
            if let Some(ty) = self.var_types.get(src_var) {
                if self.is_move_type(ty) {
                    let val_span = match value {
                        Expression::Binary { span_id, .. }
                        | Expression::Call { span_id, .. }
                        | Expression::Unary { span_id, .. } => span_id.clone(),
                        _ => None,
                    };
                    let move_span = val_span.or_else(|| parent_span.map(|s| s.clone()));
                    self.move_locations.insert(src_var.clone(), move_span);
                }
            }
        }

        Ok(())
    }

    fn check_if_statement(
        &mut self,
        condition: &Expression,
        then_block: &vex_ast::Block,
        elif_branches: &[(Expression, vex_ast::Block)],
        else_block: &Option<vex_ast::Block>,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        self.check_expression(condition, parent_span)?;

        // Check then branch
        for stmt in &then_block.statements {
            self.check_statement(stmt, parent_span)?;
        }

        // Check elif branches
        for (elif_cond, elif_block) in elif_branches {
            self.check_expression(elif_cond, parent_span)?;
            for stmt in &elif_block.statements {
                self.check_statement(stmt, parent_span)?;
            }
        }

        // Check else branch
        if let Some(else_blk) = else_block {
            for stmt in &else_blk.statements {
                self.check_statement(stmt, parent_span)?;
            }
        }

        Ok(())
    }

    fn check_while_statement(
        &mut self,
        condition: &Expression,
        body: &vex_ast::Block,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        self.check_expression(condition, parent_span)?;

        for stmt in &body.statements {
            self.check_statement(stmt, parent_span)?;
        }

        Ok(())
    }

    fn check_for_statement(
        &mut self,
        init: &Option<Box<Statement>>,
        condition: &Option<Expression>,
        post: &Option<Box<Statement>>,
        body: &vex_ast::Block,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        if let Some(init_stmt) = init {
            self.check_statement(init_stmt, parent_span)?;
        }

        if let Some(cond) = condition {
            self.check_expression(cond, parent_span)?;
        }

        if let Some(post_stmt) = post {
            self.check_statement(post_stmt, parent_span)?;
        }

        for stmt in &body.statements {
            self.check_statement(stmt, parent_span)?;
        }

        Ok(())
    }

    fn check_for_in_statement(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &vex_ast::Block,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        self.check_expression(iterable, parent_span)?;

        // Loop variable is valid in the loop body
        self.valid_vars.insert(variable.to_string());

        for stmt in &body.statements {
            self.check_statement(stmt, parent_span)?;
        }

        Ok(())
    }

    fn check_switch_statement(
        &mut self,
        value: &Option<Expression>,
        cases: &[vex_ast::SwitchCase],
        default_case: &Option<vex_ast::Block>,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        if let Some(expr) = value {
            self.check_expression(expr, parent_span)?;
        }

        for case in cases {
            for stmt in &case.body.statements {
                self.check_statement(stmt, parent_span)?;
            }
        }

        if let Some(default) = default_case {
            for stmt in &default.statements {
                self.check_statement(stmt, parent_span)?;
            }
        }

        Ok(())
    }
}
