// statements/loops/while_loop.rs
// while loop compilation

use super::super::ASTCodeGen;
use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile while loop
    pub(crate) fn compile_while_loop_dispatch(
        &mut self,
        span_id: &Option<String>,
        condition: &Expression,
        body: &Block,
    ) -> Result<(), String> {
        self.compile_while_loop_impl(span_id, condition, body)
    }

    /// Compile while loop
    fn compile_while_loop_impl(
        &mut self,
        span_id: &Option<String>,
        condition: &Expression,
        body: &Block,
    ) -> Result<(), String> {
        let fn_val = self.current_function.ok_or("No current function")?;

        let loop_cond = self.context.append_basic_block(fn_val, "while.cond");
        let loop_body = self.context.append_basic_block(fn_val, "while.body");
        let loop_end = self.context.append_basic_block(fn_val, "while.end");

        // Jump to condition check
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Condition block
        self.builder.position_at_end(loop_cond);
        let cond_val = self.compile_expression(condition)?;
        let bool_val = if let BasicValueEnum::IntValue(iv) = cond_val {
            let zero = iv.get_type().const_int(0, false);
            self.builder
                .build_int_compare(IntPredicate::NE, iv, zero, "whilecond")
                .map_err(|e| format!("Failed to compare: {}", e))?
        } else {
            // ⭐ Use span_id for better error location
            let span = span_id
                .as_ref()
                .and_then(|id| self.span_map.get(id))
                .cloned()
                .unwrap_or_else(Span::unknown);

            self.diagnostics.emit(Diagnostic {
                level: ErrorLevel::Error,
                code: error_codes::TYPE_MISMATCH.to_string(),
                message: "While condition must be an integer or boolean value".to_string(),
                span,
                notes: vec![format!("Got non-integer type in while condition")],
                help: Some(
                    "Ensure the condition evaluates to a boolean (i1) or integer type".to_string(),
                ),
                suggestion: None,
            });
            return Err("While condition must be integer".to_string());
        };
        self.builder
            .build_conditional_branch(bool_val, loop_body, loop_end)
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Body block
        self.builder.position_at_end(loop_body);

        // Push loop context for break/continue
        // continue → jump to loop_cond, break → jump to loop_end
        self.loop_context_stack.push((loop_cond, loop_end));

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
                .build_unconditional_branch(loop_cond)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // End block
        self.builder.position_at_end(loop_end);

        Ok(())
    }
}