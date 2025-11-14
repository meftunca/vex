// statements/control_flow.rs
// return / break / continue / defer

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_return_statement(
        &mut self,
        expr: Option<&Expression>,
    ) -> Result<(), String> {
        // ⚠️ ASYNC: If we're in an async resume function, return CORO_STATUS_DONE instead
        if let Some(func) = self.current_function {
            let func_name = func.get_name().to_str().unwrap_or("");
            if func_name.ends_with("_resume") {
                eprintln!("🔄 Async resume function return - returning CORO_STATUS_DONE");

                // TODO: Store the actual return value in the state struct for Future<T>
                // For now, just ignore it and return DONE status

                // Execute deferred statements before returning
                self.execute_deferred_statements()?;

                let done_status = self.context.i32_type().const_int(2, false);
                self.builder
                    .build_return(Some(&done_status))
                    .map_err(|e| format!("Failed to build async return: {}", e))?;
                return Ok(());
            }
        }

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

        // ⭐ ASYNC: If returning from main() with runtime, call runtime_run() and runtime_destroy()
        // BUT only if we're in the actual main() function, not in an async resume function
        if let Some(func) = self.current_function {
            let func_name = func.get_name().to_str().unwrap_or("");
            if func_name == "main" && self.global_runtime.is_some() {
                eprintln!(
                    "🔄 Intercepting main() return - adding runtime_run() and runtime_destroy()"
                );

                // Load runtime from global: Runtime* rt = __vex_global_runtime;
                let global_runtime_var = self.global_runtime.unwrap();
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let runtime_ptr = self
                    .builder
                    .build_load(ptr_type, global_runtime_var, "runtime_load")
                    .map_err(|e| format!("Failed to load runtime: {}", e))?
                    .into_pointer_value();

                // void runtime_run(Runtime* runtime);
                let runtime_run = self.get_or_declare_runtime_run();
                self.builder
                    .build_call(runtime_run, &[runtime_ptr.into()], "run_runtime")
                    .map_err(|e| format!("Failed to call runtime_run: {}", e))?;

                // void runtime_destroy(Runtime* runtime);
                let runtime_destroy = self.get_or_declare_runtime_destroy();
                self.builder
                    .build_call(runtime_destroy, &[runtime_ptr.into()], "destroy_runtime")
                    .map_err(|e| format!("Failed to call runtime_destroy: {}", e))?;
            }
        }

        // Build return instruction
        if let Some(mut val) = return_val {
            // Check if we need to extend/truncate the value to match function return type
            if let Some(func) = self.current_function {
                if let Some(ret_ty) = func.get_type().get_return_type() {
                    let basic_type: BasicTypeEnum = ret_ty
                        .try_into()
                        .map_err(|_| "Failed to convert return type to BasicType".to_string())?;

                    // If function returns i64 but we have i32, extend it
                    if let (BasicTypeEnum::IntType(expected), true) =
                        (basic_type, val.is_int_value())
                    {
                        let val_int = val.into_int_value();
                        let val_type = val_int.get_type();

                        // Only convert if types differ
                        if val_type.get_bit_width() != expected.get_bit_width() {
                            if val_type.get_bit_width() < expected.get_bit_width() {
                                // Sign-extend or zero-extend
                                let extended = self
                                    .builder
                                    .build_int_s_extend(val_int, expected, "ret_extend")
                                    .map_err(|e| format!("Failed to extend return value: {}", e))?;
                                val = extended.into();
                            } else {
                                // Truncate
                                let truncated = self
                                    .builder
                                    .build_int_truncate(val_int, expected, "ret_trunc")
                                    .map_err(|e| {
                                        format!("Failed to truncate return value: {}", e)
                                    })?;
                                val = truncated.into();
                            }
                        }
                    }
                }
            }

            self.builder
                .build_return(Some(&val))
                .map_err(|e| format!("Failed to build return: {}", e))?;
        } else {
            // Get the function's actual return type
            if let Some(func) = self.current_function {
                if let Some(ret_ty) = func.get_type().get_return_type() {
                    let basic_type: BasicTypeEnum = ret_ty
                        .try_into()
                        .map_err(|_| "Failed to convert return type to BasicType".to_string())?;

                    // Return appropriate zero value based on type
                    match basic_type {
                        BasicTypeEnum::IntType(int_ty) => {
                            let zero = int_ty.const_int(0, false);
                            self.builder
                                .build_return(Some(&zero))
                                .map_err(|e| format!("Failed to build return: {}", e))?;
                        }
                        _ => {
                            let zero = self.context.i32_type().const_int(0, false);
                            self.builder
                                .build_return(Some(&zero))
                                .map_err(|e| format!("Failed to build return: {}", e))?;
                        }
                    }
                } else {
                    let zero = self.context.i32_type().const_int(0, false);
                    self.builder
                        .build_return(Some(&zero))
                        .map_err(|e| format!("Failed to build return: {}", e))?;
                }
            } else {
                let zero = self.context.i32_type().const_int(0, false);
                self.builder
                    .build_return(Some(&zero))
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            }
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
