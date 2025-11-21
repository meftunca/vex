// Expression compilation - control flow (match, block, ?, await)
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile match expressions
    pub(crate) fn compile_match_dispatch(
        &mut self,
        value: &vex_ast::Expression,
        arms: &[vex_ast::MatchArm],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_match_expression(value, arms)
    }

    /// Compile block expressions
    pub(crate) fn compile_block_dispatch(
        &mut self,
        statements: &[vex_ast::Statement],
        return_expr: &Option<Box<vex_ast::Expression>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_block_expression(statements, return_expr)
    }

    /// Compile async block expressions: async { stmts; expr }
    /// Creates an anonymous async function and returns Future<T>
    pub(crate) fn compile_async_block_dispatch(
        &mut self,
        statements: &[vex_ast::Statement],
        return_expr: &Option<Box<vex_ast::Expression>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // For now, just compile as a regular block and return the value
        // TODO: Full implementation needs:
        // 1. Generate unique anonymous async function
        // 2. Capture free variables
        // 3. Compile as async function
        // 4. Return Future<T> handle

        self.compile_block_expression(statements, return_expr)
    }

    /// Compile question mark operator (?)
    pub(crate) fn compile_try_dispatch(
        &mut self,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // ? operator: Unwrap Result, propagate Err
        // Desugar: let x = expr? => match expr { Ok(v) => v, Err(e) => return Err(e) }

        // Compile the Result expression
        let result_val = self.compile_expression(expr)?;

        // Check if this is a Result/Option enum (has tag + data struct)
        if !result_val.is_struct_value() {
            return Err("? operator can only be used on Result/Option enums".to_string());
        }

        // Result is a struct value, but we need to work with it on stack
        // Allocate temporary space and store it
        let result_ptr = self
            .builder
            .build_alloca(result_val.get_type(), "result_tmp")
            .map_err(|e| format!("Failed to allocate result temp: {}", e))?;

        self.builder
            .build_store(result_ptr, result_val)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Extract tag (field 0)
        let tag_ptr = self
            .builder
            .build_struct_gep(result_val.get_type(), result_ptr, 0, "tag_ptr")
            .map_err(|e| format!("Failed to get tag pointer: {}", e))?;

        let tag = self
            .builder
            .build_load(self.context.i32_type(), tag_ptr, "tag")
            .map_err(|e| format!("Failed to load tag: {}", e))?
            .into_int_value();

        // Extract data (field 1)
        let data_ptr = self
            .builder
            .build_struct_gep(result_val.get_type(), result_ptr, 1, "data_ptr")
            .map_err(|e| format!("Failed to get data pointer: {}", e))?;

        // Create blocks for Ok and Err paths
        let ok_block = self.context.append_basic_block(
            self.current_function.ok_or("? operator outside function")?,
            "try_ok",
        );
        let err_block = self.context.append_basic_block(
            self.current_function.ok_or("? operator outside function")?,
            "try_err",
        );
        let merge_block = self.context.append_basic_block(
            self.current_function.ok_or("? operator outside function")?,
            "try_merge",
        );

        // Check if tag == 0 (Ok variant)
        let is_ok = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                tag,
                self.context.i32_type().const_int(0, false),
                "is_ok",
            )
            .map_err(|e| format!("Failed to compare tag: {}", e))?;

        // Branch: if Ok goto ok_block, else goto err_block
        self.builder
            .build_conditional_branch(is_ok, ok_block, err_block)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

        // Ok block: unwrap data and continue
        self.builder.position_at_end(ok_block);
        let data_type = self.context.i32_type(); // TODO: Infer from Result<T, E>
        let ok_value = self
            .builder
            .build_load(data_type, data_ptr, "ok_value")
            .map_err(|e| format!("Failed to load ok value: {}", e))?;
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| format!("Failed to branch to merge: {}", e))?;

        // Err block: early return with Err
        self.builder.position_at_end(err_block);

        // Execute deferred statements before early return
        self.execute_deferred_statements()?;

        // Return the error Result value
        self.builder
            .build_return(Some(&result_val))
            .map_err(|e| format!("Failed to build error return: {}", e))?;

        // Merge block: continue with unwrapped value
        self.builder.position_at_end(merge_block);

        Ok(ok_value)
    }

    /// Compile await expressions with full state machine support
    pub(crate) fn compile_await_dispatch(
        &mut self,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        eprintln!("⏸️ compile_await_dispatch: expr={:?}", expr);

        // Check if we're inside an async function
        if self.async_state_stack.is_empty() {
            // Not in async context - just compile expression and return it
            return self.compile_expression(expr);
        }

        // ⭐ PHASE 2: Full state machine implementation
        // Get current state machine context
        let (state_ptr, state_field_ptr, current_state_id) = self
            .async_state_stack
            .last()
            .copied()
            .ok_or("Await outside async context")?;

        eprintln!("  Current state ID: {}", current_state_id);
        eprintln!(
            "  Resume blocks available: {}",
            self.async_resume_blocks.len()
        );

        // Compile the future expression
        let _future_val = self.compile_expression(expr)?;

        // Generate next state ID
        let next_state_id = current_state_id + 1;

        eprintln!("  Next state ID: {}", next_state_id);

        // Get the resume block for this await point (pre-allocated in async function compilation)
        let resume_block = self
            .async_resume_blocks
            .get((next_state_id - 1) as usize)
            .copied()
            .ok_or_else(|| format!("Resume block {} not pre-allocated", next_state_id))?;

        eprintln!("  Got resume block: {:?}", resume_block);

        // Save state: store next_state_id to state field
        self.builder
            .build_store(
                state_field_ptr,
                self.context
                    .i32_type()
                    .const_int(next_state_id as u64, false),
            )
            .map_err(|e| format!("Failed to save state: {}", e))?;

        // Return CORO_STATUS_YIELDED (1) to runtime
        let yielded_status = self.context.i32_type().const_int(1, false);
        self.builder
            .build_return(Some(&yielded_status))
            .map_err(|e| format!("Failed to build yield return: {}", e))?;

        eprintln!("  Positioned at resume block");

        // ⭐ Position builder at resume block - this is where execution continues after yield
        // No need for separate continuation block - resume block IS the continuation
        self.builder.position_at_end(resume_block);

        // Verify builder position
        let current_block_after = self
            .builder
            .get_insert_block()
            .and_then(|bb| bb.get_name().to_str().ok().map(|s| s.to_string()));
        eprintln!("  🔍 Builder positioned at: {:?}", current_block_after);

        // Update state machine context with new state ID
        self.async_state_stack.pop();
        self.async_state_stack
            .push((state_ptr, state_field_ptr, next_state_id));

        // Ensure resume block gets terminated - if we're here, it means there are more statements
        // coming after this await that will compile into this block and add the terminator

        eprintln!("  Returning placeholder value");

        // TODO: Load future result from runtime when available
        // For now, return placeholder (0)
        Ok(self.context.i32_type().const_int(0, false).into())
    }
}
