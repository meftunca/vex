//! Statement-level borrow checking

use super::checker::BorrowRulesChecker;
use super::tracking::BorrowKind;
use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use vex_ast::{Expression, Statement};

impl BorrowRulesChecker {
    /// Check a statement for borrow rule violations
    pub(super) fn check_statement(&mut self, stmt: &Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let { name, value, .. } => self.check_let_statement(name, value),

            Statement::LetPattern {
                pattern: _, value, ..
            } => {
                // Check value for borrows
                self.check_expression_for_borrows(value)?;
                Ok(())
            }

            Statement::Assign { target, value } => self.check_assign_statement(target, value),

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

            Statement::ForIn { iterable, body, .. } => self.check_for_in_statement(iterable, body),

            Statement::Switch {
                value,
                cases,
                default_case,
            } => self.check_switch_statement(value, cases, default_case),

            Statement::Unsafe(block) => self.check_unsafe_block(block),

            _ => Ok(()),
        }
    }

    fn check_let_statement(&mut self, name: &str, value: &Expression) -> BorrowResult<()> {
        // Check if the value creates any borrows
        self.check_expression_for_borrows(value)?;

        // If value is a reference expression, track the borrow
        if let Expression::Reference { is_mutable, expr } = value {
            if let Expression::Ident(var) = expr.as_ref() {
                self.create_borrow(
                    name.to_string(),
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

    fn check_assign_statement(
        &mut self,
        target: &Expression,
        value: &Expression,
    ) -> BorrowResult<()> {
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

    fn check_if_statement(
        &mut self,
        condition: &Expression,
        then_block: &vex_ast::Block,
        elif_branches: &[(Expression, vex_ast::Block)],
        else_block: &Option<vex_ast::Block>,
    ) -> BorrowResult<()> {
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

    fn check_while_statement(
        &mut self,
        condition: &Expression,
        body: &vex_ast::Block,
    ) -> BorrowResult<()> {
        self.check_expression_for_borrows(condition)?;

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

    fn check_for_in_statement(
        &mut self,
        iterable: &Expression,
        body: &vex_ast::Block,
    ) -> BorrowResult<()> {
        self.check_expression_for_borrows(iterable)?;

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

    fn check_unsafe_block(&mut self, block: &vex_ast::Block) -> BorrowResult<()> {
        // Enter unsafe context
        let prev_unsafe = self.in_unsafe_block;
        self.in_unsafe_block = true;

        // Check block content
        for stmt in &block.statements {
            self.check_statement(stmt)?;
        }

        // Restore previous unsafe context
        self.in_unsafe_block = prev_unsafe;
        Ok(())
    }
}
