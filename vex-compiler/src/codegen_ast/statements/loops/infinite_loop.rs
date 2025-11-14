// statements/loops/loop.rs
// loop statement compilation

use super::super::ASTCodeGen;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile loop statement: loop { body } (infinite loop)
    pub(crate) fn compile_loop_dispatch(&mut self, body: &Block) -> Result<(), String> {
        self.compile_loop_impl(body)
    }

    /// Compile loop statement: loop { body } (infinite loop)
    fn compile_loop_impl(&mut self, body: &Block) -> Result<(), String> {
        let fn_val = self.current_function.ok_or("No current function")?;

        let loop_body = self.context.append_basic_block(fn_val, "loop.body");
        let loop_end = self.context.append_basic_block(fn_val, "loop.end");

        // Jump to body (infinite loop)
        self.builder
            .build_unconditional_branch(loop_body)
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Body block
        self.builder.position_at_end(loop_body);

        // Push loop context for break/continue
        // continue → jump to loop_body, break → jump to loop_end
        self.loop_context_stack.push((loop_body, loop_end));

        let compile_result = self.compile_block(body);

        // Pop loop context
        self.loop_context_stack.pop();

        compile_result?;

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_body)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // End block
        self.builder.position_at_end(loop_end);

        Ok(())
    }
}
