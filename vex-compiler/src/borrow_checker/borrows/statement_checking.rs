//! Statement-level borrow checking

use super::checker::BorrowRulesChecker;
use super::tracking::BorrowKind;
use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use vex_ast::{Expression, Statement};

impl BorrowRulesChecker {
    /// Check a statement for borrow rule violations
    pub(super) fn check_statement(
        &mut self,
        stmt: &Statement,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        match stmt {
            Statement::Let { name, value, .. } => {
                self.check_let_statement(name, value, parent_span)
            }

            Statement::LetPattern {
                pattern: _, value, ..
            } => {
                // Check value for borrows
                self.check_expression_for_borrows(value, parent_span)?;
                Ok(())
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
                    self.check_expression_for_borrows(expr, span_id.as_ref().or(parent_span))?;
                }
                Ok(())
            }

            Statement::Expression(expr) => {
                self.check_expression_for_borrows(expr, parent_span)?;
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
                iterable,
                body,
                ..
            } => self.check_for_in_statement(iterable, body, span_id.as_ref().or(parent_span)),

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

            Statement::Unsafe { span_id, block } => {
                self.check_unsafe_block(block, span_id.as_ref().or(parent_span))
            }

            _ => Ok(()),
        }
    }

    fn check_let_statement(
        &mut self,
        name: &str,
        value: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Check if the value creates any borrows
        self.check_expression_for_borrows(value, parent_span)?;

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
                    parent_span.cloned(),
                )?;
            }
        }

        Ok(())
    }

    fn check_assign_statement(
        &mut self,
        target: &Expression,
        value: &Expression,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Check if we're trying to mutate a borrowed variable
        if let Expression::Ident(var) = target {
            if let Some(borrows) = self.borrowed_vars.get(var) {
                if !borrows.is_empty() {
                    // Try to find a borrow location for diagnostics, or fallback to parent_span
                    let mut borrowed_loc: Option<String> = None;
                    for (_ref_name, borrows_vec) in &self.active_borrows {
                        for b in borrows_vec {
                            if b.variable == *var {
                                if let Some(loc) = &b.location {
                                    borrowed_loc = Some(loc.clone());
                                    break;
                                }
                            }
                        }
                        if borrowed_loc.is_some() {
                            break;
                        }
                    }
                    return Err(BorrowError::MutationWhileBorrowed {
                        variable: var.clone(),
                        borrowed_at: borrowed_loc.or_else(|| parent_span.cloned()),
                    });
                }
            }
        }

        self.check_expression_for_borrows(target, parent_span)?;
        self.check_expression_for_borrows(value, parent_span)?;

        // If assigning a reference, track the created borrow with location
        if let Expression::Reference { is_mutable, expr } = value {
            if let Expression::Ident(var) = expr.as_ref() {
                self.create_borrow(
                    match target {
                        Expression::Ident(n) => n.clone(),
                        _ => "<unknown>".to_string(),
                    },
                    var.clone(),
                    if *is_mutable {
                        BorrowKind::Mutable
                    } else {
                        BorrowKind::Immutable
                    },
                    parent_span.cloned(),
                )?;
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
        self.check_expression_for_borrows(condition, parent_span)?;

        for stmt in &then_block.statements {
            self.check_statement(stmt, parent_span)?;
        }

        // Check elif branches
        for (elif_cond, elif_block) in elif_branches {
            self.check_expression_for_borrows(elif_cond, parent_span)?;
            for stmt in &elif_block.statements {
                self.check_statement(stmt, parent_span)?;
            }
        }

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
        self.check_expression_for_borrows(condition, parent_span)?;

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
            self.check_expression_for_borrows(cond, parent_span)?;
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
        iterable: &Expression,
        body: &vex_ast::Block,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        self.check_expression_for_borrows(iterable, parent_span)?;

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
            self.check_expression_for_borrows(expr, parent_span)?;
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

    fn check_unsafe_block(
        &mut self,
        block: &vex_ast::Block,
        parent_span: Option<&String>,
    ) -> BorrowResult<()> {
        // Enter unsafe context
        let prev_unsafe = self.in_unsafe_block;
        self.in_unsafe_block = true;

        // Check block content
        for stmt in &block.statements {
            self.check_statement(stmt, parent_span)?;
        }

        // Restore previous unsafe context
        self.in_unsafe_block = prev_unsafe;
        Ok(())
    }
}
