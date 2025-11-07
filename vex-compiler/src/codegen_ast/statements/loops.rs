// statements/loops.rs
// if / while / for / switch

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile if statement with elif support
    pub(crate) fn compile_if_statement(
        &mut self,
        condition: &Expression,
        then_block: &Block,
        elif_branches: &[(Expression, Block)],
        else_block: &Option<Block>,
    ) -> Result<(), String> {
        let cond_val = self.compile_expression(condition)?;

        // Convert to boolean (i1)
        let bool_val = if let BasicValueEnum::IntValue(iv) = cond_val {
            let zero = iv.get_type().const_int(0, false);
            self.builder
                .build_int_compare(IntPredicate::NE, iv, zero, "ifcond")
                .map_err(|e| format!("Failed to compare: {}", e))?
        } else {
            return Err("Condition must be integer value".to_string());
        };

        let fn_val = self.current_function.ok_or("No current function")?;

        let then_bb = self.context.append_basic_block(fn_val, "then");
        let merge_bb = self.context.append_basic_block(fn_val, "ifcont");

        // Create blocks for elif branches
        let mut elif_blocks = Vec::new();
        for (i, _) in elif_branches.iter().enumerate() {
            let elif_cond_bb = self
                .context
                .append_basic_block(fn_val, &format!("elif.cond.{}", i));
            let elif_then_bb = self
                .context
                .append_basic_block(fn_val, &format!("elif.then.{}", i));
            elif_blocks.push((elif_cond_bb, elif_then_bb));
        }

        // Else block or final fallthrough
        let else_bb = self.context.append_basic_block(fn_val, "else");

        // Build initial conditional branch
        self.builder
            .build_conditional_branch(
                bool_val,
                then_bb,
                if !elif_blocks.is_empty() {
                    elif_blocks[0].0
                } else {
                    else_bb
                },
            )
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Compile then block
        self.builder.position_at_end(then_bb);
        self.compile_block(then_block)?;
        let then_terminated = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_some();
        if !then_terminated {
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // Compile elif branches
        let mut any_unterminated = !then_terminated;
        for (i, (elif_cond, elif_body)) in elif_branches.iter().enumerate() {
            let (cond_bb, then_bb) = elif_blocks[i];

            // Position at condition block
            self.builder.position_at_end(cond_bb);

            // Evaluate elif condition
            let cond_val = self.compile_expression(elif_cond)?;
            let bool_val = if let BasicValueEnum::IntValue(iv) = cond_val {
                let zero = iv.get_type().const_int(0, false);
                self.builder
                    .build_int_compare(IntPredicate::NE, iv, zero, "elifcond")
                    .map_err(|e| format!("Failed to compare: {}", e))?
            } else {
                return Err("Elif condition must be integer value".to_string());
            };

            // Branch to elif body or next elif/else
            let next_bb = if i + 1 < elif_blocks.len() {
                elif_blocks[i + 1].0
            } else {
                else_bb
            };

            self.builder
                .build_conditional_branch(bool_val, then_bb, next_bb)
                .map_err(|e| format!("Failed to build elif branch: {}", e))?;

            // Compile elif body
            self.builder.position_at_end(then_bb);
            self.compile_block(elif_body)?;
            let elif_terminated = self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_some();
            if !elif_terminated {
                self.builder
                    .build_unconditional_branch(merge_bb)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;
                any_unterminated = true;
            }
        }

        // Compile else block
        self.builder.position_at_end(else_bb);
        if let Some(eb) = else_block {
            self.compile_block(eb)?;
        }
        let else_terminated = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_some();
        if !else_terminated {
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
            any_unterminated = true;
        }

        // Continue at merge block if at least one branch didn't terminate
        if any_unterminated {
            self.builder.position_at_end(merge_bb);
        } else {
            // All branches terminated - merge block is unreachable
            self.builder.position_at_end(merge_bb);
            self.builder
                .build_unreachable()
                .map_err(|e| format!("Failed to build unreachable: {}", e))?;
        }

        Ok(())
    }

    /// Compile while loop: while condition { body }
    pub(crate) fn compile_while_loop(
        &mut self,
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

    /// Compile for loop: for init; condition; post { body }
    pub(crate) fn compile_for_loop(
        &mut self,
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
            .unwrap()
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

    /// Compile switch statement: switch value { case x: ... default: ... }
    pub(crate) fn compile_switch_statement(
        &mut self,
        value: &Option<Expression>,
        cases: &[SwitchCase],
        default_case: &Option<Block>,
    ) -> Result<(), String> {
        // Evaluate the switch value
        let switch_val = if let Some(val_expr) = value {
            self.compile_expression(val_expr)?
        } else {
            return Err("Type switches not yet supported".to_string());
        };

        let switch_int = if let BasicValueEnum::IntValue(iv) = switch_val {
            iv
        } else {
            return Err("Switch value must be an integer".to_string());
        };

        let fn_val = self.current_function.ok_or("No current function")?;

        // Create basic blocks for each case and default
        let mut case_blocks = Vec::new();
        for _ in cases {
            case_blocks.push(self.context.append_basic_block(fn_val, "switch.case"));
        }

        let default_bb = self.context.append_basic_block(fn_val, "switch.default");
        let end_bb = self.context.append_basic_block(fn_val, "switch.end");

        // Build case values for switch instruction
        let mut switch_cases = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            let case_bb = case_blocks[i];

            // Add each pattern as a case
            for pattern in &case.patterns {
                let pattern_val = self.compile_expression(pattern)?;
                if let BasicValueEnum::IntValue(pv) = pattern_val {
                    switch_cases.push((pv, case_bb));
                } else {
                    return Err("Case pattern must be an integer constant".to_string());
                }
            }
        }

        // Build the switch instruction with all cases
        self.builder
            .build_switch(switch_int, default_bb, &switch_cases)
            .map_err(|e| format!("Failed to build switch: {}", e))?;

        // Compile each case body
        for (i, case) in cases.iter().enumerate() {
            self.builder.position_at_end(case_blocks[i]);
            self.compile_block(&case.body)?;

            // Add branch to end if not already terminated
            if self
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                self.builder
                    .build_unconditional_branch(end_bb)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;
            }
        }

        // Compile default case
        self.builder.position_at_end(default_bb);
        if let Some(def_block) = default_case {
            self.compile_block(def_block)?;
        }
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(end_bb)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // Always position at end block for subsequent code
        // Even if unreachable, LLVM will optimize it away
        self.builder.position_at_end(end_bb);

        Ok(())
    }

    /// Compile for-in loop: for i in 0..10 { body }
    /// Desugars to: let! range = 0..10; let! i: i64; while range.next(&i) { body }
    pub(crate) fn compile_for_in_loop(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &Block,
    ) -> Result<(), String> {
        // Compile iterable (Range expression)
        let range_val = self.compile_expression(iterable)?;

        // Determine if Range or RangeInclusive based on expression type
        let is_inclusive = matches!(iterable, Expression::RangeInclusive { .. });
        let range_type_name = if is_inclusive {
            "RangeInclusive"
        } else {
            "Range"
        };

        // Create temporary range variable
        let range_var_name = format!("__forin_range_{}", variable);
        let range_alloca = self.create_entry_block_alloca(
            &range_var_name,
            &Type::Named(range_type_name.to_string()),
            true, // mutable
        )?;
        self.build_store_aligned(range_alloca, range_val)?;

        // Track range variable for method calls
        self.variables.insert(range_var_name.clone(), range_alloca);
        self.variable_struct_names
            .insert(range_var_name.clone(), range_type_name.to_string());

        // Create loop variable (i64)
        let loop_var_alloca = self.create_entry_block_alloca(
            variable,
            &Type::I64,
            true, // mutable
        )?;
        self.variables.insert(variable.to_string(), loop_var_alloca);
        self.variable_types
            .insert(variable.to_string(), self.context.i64_type().into());

        // Create loop blocks
        let fn_val = self.current_function.ok_or("No current function")?;
        let loop_cond = self.context.append_basic_block(fn_val, "for.cond");
        let loop_body = self.context.append_basic_block(fn_val, "for.body");
        let loop_end = self.context.append_basic_block(fn_val, "for.end");

        // Push loop context for break/continue
        self.loop_context_stack.push((loop_cond, loop_end));

        // Branch to condition
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| format!("Failed to branch to loop: {}", e))?;

        // Condition: range.next(&loop_var)
        self.builder.position_at_end(loop_cond);

        // Call range.next(&loop_var) -> bool
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let fn_name = if is_inclusive {
            "vex_range_inclusive_next"
        } else {
            "vex_range_next"
        };

        let next_fn = self.declare_runtime_fn(
            fn_name,
            &[ptr_type.into(), ptr_type.into()],
            self.context.bool_type().into(),
        );

        let has_next = self
            .builder
            .build_call(
                next_fn,
                &[range_alloca.into(), loop_var_alloca.into()],
                "has_next",
            )
            .map_err(|e| format!("Failed to call {}: {}", fn_name, e))?
            .try_as_basic_value()
            .left()
            .ok_or_else(|| format!("{} returned void", fn_name))?;

        // Branch based on has_next
        self.builder
            .build_conditional_branch(has_next.into_int_value(), loop_body, loop_end)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

        // Body
        self.builder.position_at_end(loop_body);
        self.compile_block(body)?;

        // Branch back to condition (if not terminated)
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(loop_cond)
                .map_err(|e| format!("Failed to branch back: {}", e))?;
        }

        // End
        self.builder.position_at_end(loop_end);

        // Pop loop context
        self.loop_context_stack.pop();

        Ok(())
    }
}
