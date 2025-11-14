//! Statement-level move checking

use super::checker::MoveChecker;
use crate::borrow_checker::errors::BorrowResult;
use vex_ast::{Expression, Statement, Type};

impl MoveChecker {
    /// Check a statement for move violations
    pub(super) fn check_statement(&mut self, stmt: &Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let {
                name, ty, value, ..
            } => self.check_let_statement(name, ty, value),

            Statement::LetPattern { pattern, value, .. } => {
                self.check_let_pattern_statement(pattern, value)
            }

            Statement::Assign { target, value } => self.check_assign_statement(target, value),

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
                span_id: _,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => self.check_if_statement(condition, then_block, elif_branches, else_block),

            Statement::While {
                span_id: _,
                condition,
                body,
            } => self.check_while_statement(condition, body),

            Statement::For {
                span_id: _,
                init,
                condition,
                post,
                body,
            } => self.check_for_statement(init, condition, post, body),

            Statement::ForIn {
                variable,
                iterable,
                body,
            } => self.check_for_in_statement(variable, iterable, body),

            Statement::Switch {
                value,
                cases,
                default_case,
            } => self.check_switch_statement(value, cases, default_case),

            _ => Ok(()), // Other statement types don't affect moves
        }
    }

    fn check_let_statement(
        &mut self,
        name: &str,
        ty: &Option<Type>,
        value: &Expression,
    ) -> BorrowResult<()> {
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
        self.valid_vars.insert(name.to_string());

        if let Some(t) = ty {
            self.var_types.insert(name.to_string(), t.clone());
        } else {
            // Infer type from the initializer expression
            let inferred_ty = match value {
                Expression::StringLiteral(_) | Expression::FStringLiteral(_) => Some(Type::String),
                Expression::IntLiteral(_) => Some(Type::I32),
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
    ) -> BorrowResult<()> {
        // Check value for moves
        self.check_expression(value)?;
        // Declare pattern variables
        self.declare_pattern_variables(pattern);
        Ok(())
    }

    fn check_assign_statement(
        &mut self,
        target: &Expression,
        value: &Expression,
    ) -> BorrowResult<()> {
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

    fn check_if_statement(
        &mut self,
        condition: &Expression,
        then_block: &vex_ast::Block,
        elif_branches: &[(Expression, vex_ast::Block)],
        else_block: &Option<vex_ast::Block>,
    ) -> BorrowResult<()> {
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

    fn check_while_statement(
        &mut self,
        condition: &Expression,
        body: &vex_ast::Block,
    ) -> BorrowResult<()> {
        self.check_expression(condition)?;

        for stmt in &body.statements {
            self.check_statement(stmt)?;
        }

        Ok(())
    }

    fn check_for_statement(
        &mut self,
        init: &Option<Box<Statement>>,
        condition: &Option<Expression>,
        post: &Option<Box<Statement>>,
        body: &vex_ast::Block,
    ) -> BorrowResult<()> {
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

    fn check_for_in_statement(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &vex_ast::Block,
    ) -> BorrowResult<()> {
        self.check_expression(iterable)?;

        // Loop variable is valid in the loop body
        self.valid_vars.insert(variable.to_string());

        for stmt in &body.statements {
            self.check_statement(stmt)?;
        }

        Ok(())
    }

    fn check_switch_statement(
        &mut self,
        value: &Option<Expression>,
        cases: &[vex_ast::SwitchCase],
        default_case: &Option<vex_ast::Block>,
    ) -> BorrowResult<()> {
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
}
