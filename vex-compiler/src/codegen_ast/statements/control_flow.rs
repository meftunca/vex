// statements/control_flow.rs
// return / break / continue / defer

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_return_statement(
        &mut self,
        expr: Option<&Expression>,
    ) -> Result<(), String> {
        // Compile return value FIRST (may reference variables)
        let return_val = if let Some(e) = expr {
            Some(self.compile_expression(e)?)
        } else {
            None
        };

        // Pop scope and emit automatic cleanup AFTER computing value
        self.pop_scope()?;

        // Execute deferred statements in reverse order before returning
        self.execute_deferred_statements()?;

        // Build return instruction
        if let Some(val) = return_val {
            self.builder
                .build_return(Some(&val))
                .map_err(|e| format!("Failed to build return: {}", e))?;
        } else {
            let zero = self.context.i32_type().const_int(0, false);
            self.builder
                .build_return(Some(&zero))
                .map_err(|e| format!("Failed to build return: {}", e))?;
        }
        Ok(())
    }

    pub(crate) fn compile_break_statement(&mut self) -> Result<(), String> {
        // Execute deferred statements before break
        self.execute_deferred_statements()?;

        // Get current loop context
        if let Some((_, break_block)) = self.loop_context_stack.last() {
            let break_block = *break_block;
            self.builder
                .build_unconditional_branch(break_block)
                .map_err(|e| format!("Failed to build break branch: {}", e))?;
            Ok(())
        } else {
            Err("Break statement outside of loop".to_string())
        }
    }

    pub(crate) fn compile_continue_statement(&mut self) -> Result<(), String> {
        // Execute deferred statements before continue
        self.execute_deferred_statements()?;

        // Get current loop context
        if let Some((continue_block, _)) = self.loop_context_stack.last() {
            let continue_block = *continue_block;
            self.builder
                .build_unconditional_branch(continue_block)
                .map_err(|e| format!("Failed to build continue branch: {}", e))?;
            Ok(())
        } else {
            Err("Continue statement outside of loop".to_string())
        }
    }

    pub(crate) fn compile_defer_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        // Add statement to defer stack (LIFO). Do not execute now.
        self.deferred_statements.push(stmt.clone());
        Ok(())
    }

    /// Compile go statement: go { ... } or go func()
    /// For now, just execute the expression/block directly (no actual async spawning yet)
    pub(crate) fn compile_go_statement(&mut self, expr: &Expression) -> Result<(), String> {
        // TODO: Implement actual async task spawning
        // For now, just compile the expression directly
        self.compile_expression(expr)?;
        Ok(())
    }
}
