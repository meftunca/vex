// statements/control_flow.rs
// return / break / continue / defer

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_return_statement(
        &mut self,
        expr: Option<&Expression>,
    ) -> Result<(), String> {
        // Compile return value FIRST (may reference variables)
        let return_val = if let Some(e) = expr {
            let val = self.compile_expression(e)?;

            eprintln!(
                "🔄 Return statement compiled expression: is_pointer={}, is_struct={}",
                val.is_pointer_value(),
                val.is_struct_value()
            );

            // ⭐ CRITICAL FIX: If returning a pointer to a struct, LOAD the value
            // The function signature expects a struct BY VALUE, not a pointer.
            // compile_struct_literal returns a POINTER, so we must load it.
            if val.is_pointer_value() {
                let ptr_val = val.into_pointer_value();
                if let Some(func) = self.current_function {
                    if let Some(ret_ty) = func.get_type().get_return_type() {
                        let basic_type: BasicTypeEnum = ret_ty.try_into().map_err(|_| {
                            "Failed to convert return type to BasicType".to_string()
                        })?;

                        eprintln!("🔄 Function return type: {:?}", basic_type);

                        // Only load if the return type is a struct (not a pointer)
                        if matches!(basic_type, BasicTypeEnum::StructType(_)) {
                            eprintln!("🔄 Loading struct value from pointer before return");
                            Some(
                                self.builder
                                    .build_load(basic_type, ptr_val, "ret_load")
                                    .map_err(|e| format!("Failed to load return value: {}", e))?,
                            )
                        } else {
                            Some(val)
                        }
                    } else {
                        Some(val)
                    }
                } else {
                    Some(val)
                }
            } else {
                Some(val)
            }
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
