// statements/loops.rs
// if / while / for / switch

mod for_in_loop;
mod for_loop;
mod if_statement;
mod infinite_loop;
mod switch_statement;
mod while_loop;

use super::ASTCodeGen;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile if statement with elif support
    pub(crate) fn compile_if_statement(
        &mut self,
        span_id: &Option<String>,
        condition: &Expression,
        then_block: &Block,
        elif_branches: &[(Expression, Block)],
        else_block: &Option<Block>,
    ) -> Result<(), String> {
        self.compile_if_statement_dispatch(
            span_id,
            condition,
            then_block,
            elif_branches,
            else_block,
        )
    }

    /// Compile while loop
    pub(crate) fn compile_while_loop(
        &mut self,
        span_id: &Option<String>,
        condition: &Expression,
        body: &Block,
    ) -> Result<(), String> {
        self.compile_while_loop_dispatch(span_id, condition, body)
    }

    /// Compile for loop: for init; condition; post { body }
    pub(crate) fn compile_for_loop(
        &mut self,
        _span_id: &Option<String>,
        init: &Option<Box<Statement>>,
        condition: &Option<Expression>,
        post: &Option<Box<Statement>>,
        body: &Block,
    ) -> Result<(), String> {
        self.compile_for_loop_dispatch(_span_id, init, condition, post, body)
    }

    /// Compile switch statement: switch value { case x: ... default: ... }
    pub(crate) fn compile_switch_statement(
        &mut self,
        value: &Option<Expression>,
        cases: &[SwitchCase],
        default_case: &Option<Block>,
    ) -> Result<(), String> {
        self.compile_switch_statement_dispatch(value, cases, default_case)
    }

    /// Compile for-in loop: for item in iterator { body }
    /// Works with:
    /// 1. Range/RangeInclusive (0..10, 0..=10)
    /// 2. Any type implementing Iterator trait
    pub(crate) fn compile_for_in_loop(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &Block,
    ) -> Result<(), String> {
        self.compile_for_in_loop_dispatch(variable, iterable, body)
    }

    /// Compile loop statement: loop { body } (infinite loop)
    pub(crate) fn compile_loop(&mut self, body: &Block) -> Result<(), String> {
        self.compile_loop_dispatch(body)
    }
}
