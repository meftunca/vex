// statements/loops/for_loop.rs
// for loop compilation

use super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile for loop: for init; condition; post { body }
    pub(crate) fn compile_for_loop_dispatch(
        &mut self,
        _span_id: &Option<String>,
        init: &Option<Box<Statement>>,
        condition: &Option<Expression>,
        post: &Option<Box<Statement>>,
        body: &Block,
    ) -> Result<(), String> {
        self.compile_for_loop_impl(_span_id, init, condition, post, body)
    }

    /// Compile for loop: for init; condition; post { body }
    fn compile_for_loop_impl(
        &mut self,
        _span_id: &Option<String>,
        init: &Option<Box<Statement>>,
        condition: &Option<Expression>,
        post: &Option<Box<Statement>>,
        body: &Block,
    ) -> Result<(), String> {
        let fn_val = self.current_function.ok_or("No current function")?;

        // Compile init statement
        if let Some(i) = init {
            self.compile_statement(i)?;
        }

        let loop_cond = self.context.append_basic_block(fn_val, "loop.cond");
        let loop_body = self.context.append_basic_block(fn_val, "loop.body");
        let loop_post = self.context.append_basic_block(fn_val, "loop.post");
        let loop_end = self.context.append_basic_block(fn_val, "loop.end");

        // Jump to condition check
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Condition block
        self.builder.position_at_end(loop_cond);
        if let Some(cond) = condition {
            let cond_val = self.compile_expression(cond)?;
            let bool_val = if let BasicValueEnum::IntValue(iv) = cond_val {
                let zero = iv.get_type().const_int(0, false);
                self.builder
                    .build_int_compare(IntPredicate::NE, iv, zero, "loopcond")
                    .map_err(|e| format!("Failed to compare: {}", e))?
            } else {
                return Err("Loop condition must be integer".to_string());
            };
            self.builder
                .build_conditional_branch(bool_val, loop_body, loop_end)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        } else {
            // Infinite loop
            self.builder
                .build_unconditional_branch(loop_body)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // Body block
        self.builder.position_at_end(loop_body);

        // Push loop context for break/continue
        // continue → jump to loop_post, break → jump to loop_end
        self.loop_context_stack.push((loop_post, loop_end));

        let compile_result = self.compile_block(body);

        // Pop loop context
        self.loop_context_stack.pop();

        compile_result?;

        if self
            .builder
            .get_insert_block()
            .ok_or("No active basic block")?
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_post)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // Post block
        self.builder.position_at_end(loop_post);
        if let Some(p) = post {
            self.compile_statement(p)?;
        }
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // End block
        self.builder.position_at_end(loop_end);

        Ok(())
    }
}
