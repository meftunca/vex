// statements/loops.rs
// if / while / for / switch

use super::ASTCodeGen;
use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
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
        let cond_val = self.compile_expression(condition)?;

        // Convert to boolean (i1)
        let bool_val = if let BasicValueEnum::IntValue(iv) = cond_val {
            let zero = iv.get_type().const_int(0, false);
            self.builder
                .build_int_compare(IntPredicate::NE, iv, zero, "ifcond")
                .map_err(|e| format!("Failed to compare: {}", e))?
        } else {
            // ⭐ Get span from span_id
            let span = span_id
                .as_ref()
                .and_then(|id| self.span_map.get(id))
                .cloned()
                .unwrap_or_else(Span::unknown);

            self.diagnostics.emit(Diagnostic {
                level: ErrorLevel::Error,
                code: error_codes::TYPE_MISMATCH.to_string(),
                message: "If condition must be an integer or boolean value".to_string(),
                span,
                notes: vec![format!("Got non-integer type in if condition")],
                help: Some(
                    "Ensure the condition evaluates to a boolean (i1) or integer type".to_string(),
                ),
                suggestion: None,
            });
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

    /// Compile for loop: for init; condition; post { body }
    pub(crate) fn compile_for_loop(
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
        // Check if iterable is a Range expression (special case)
        let is_range = matches!(
            iterable,
            Expression::Range { .. } | Expression::RangeInclusive { .. }
        );

        if is_range {
            // Use old Range-based implementation
            self.compile_for_in_range(variable, iterable, body)
        } else {
            // Use Iterator trait-based implementation
            self.compile_for_in_iterator(variable, iterable, body)
        }
    }

    /// Compile for-in loop with Range (legacy implementation)
    fn compile_for_in_range(
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

        // Get LLVM type for Range
        let range_llvm_type = self.ast_type_to_llvm(&Type::Named(range_type_name.to_string()));

        // Track range variable for method calls
        self.variables.insert(range_var_name.clone(), range_alloca);
        self.variable_struct_names
            .insert(range_var_name.clone(), range_type_name.to_string());
        self.variable_types
            .insert(range_var_name.clone(), range_llvm_type);

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

    /// Compile for-in loop with Iterator trait
    /// Desugars to: while let Some(item) = iterator.next() { body }
    fn compile_for_in_iterator(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &Block,
    ) -> Result<(), String> {
        // Get iterator type from expression
        let iter_type_name = match iterable {
            Expression::Ident(name) => {
                // Lookup variable type
                self.variable_struct_names
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Iterator variable '{}' not found", name))?
            }
            Expression::StructLiteral { name, .. } => name.clone(),
            _ => return Err("Iterator expression must be a variable or struct literal that implements Iterator trait".to_string()),
        };

        // For identifiers, use mutable reference to existing variable
        // For struct literals, create new temporary
        let (iter_var_name, needs_temp) = match iterable {
            Expression::Ident(name) => (name.clone(), false),
            _ => ("__forin_iter".to_string(), true),
        };

        if needs_temp {
            // 1. Compile iterator expression and store in temporary variable
            let iter_val = self.compile_expression(iterable)?;

            let iter_alloca = self.create_entry_block_alloca(
                &iter_var_name,
                &Type::Named(iter_type_name.clone()),
                true, // mutable
            )?;
            self.build_store_aligned(iter_alloca, iter_val)?;

            // Get LLVM type for iterator
            let iter_llvm_type = self.ast_type_to_llvm(&Type::Named(iter_type_name.clone()));

            // Track iterator variable
            self.variables.insert(iter_var_name.clone(), iter_alloca);
            self.variable_struct_names
                .insert(iter_var_name.clone(), iter_type_name.clone());
            self.variable_types
                .insert(iter_var_name.clone(), iter_llvm_type);
        }

        // 2. Create loop blocks
        let fn_val = self.current_function.ok_or("No current function")?;
        let loop_cond = self.context.append_basic_block(fn_val, "for_iter.cond");
        let loop_body = self.context.append_basic_block(fn_val, "for_iter.body");
        let loop_end = self.context.append_basic_block(fn_val, "for_iter.end");

        // Push loop context
        self.loop_context_stack.push((loop_cond, loop_end));

        // Branch to condition
        self.builder
            .build_unconditional_branch(loop_cond)
            .map_err(|e| format!("Failed to branch to loop: {}", e))?;

        // 3. Condition block: call iterator.next() -> Option<Item>
        self.builder.position_at_end(loop_cond);

        // Build method call expression: iterator.next()
        let next_call_expr = Expression::MethodCall {
            receiver: Box::new(Expression::Ident(iter_var_name.clone())),
            method: "next".to_string(),
            type_args: vec![],
            args: vec![],
            is_mutable_call: true, // next() is mutable
        };

        let option_val = self.compile_expression(&next_call_expr)?;

        // Option is returned as struct { tag: i32, value: T }
        // Extract tag field to check if Some(0) or None(1)
        let option_ptr = if let BasicValueEnum::StructValue(sv) = option_val {
            // Option is returned by value, need to allocate space for it
            let option_alloca = self
                .builder
                .build_alloca(sv.get_type(), "option_temp")
                .map_err(|e| format!("Failed to allocate option temp: {}", e))?;
            self.build_store_aligned(option_alloca, option_val)?;
            option_alloca
        } else {
            return Err("Iterator.next() must return Option<T>".to_string());
        };

        // Load tag field (first element of struct)
        let tag_ptr = self
            .builder
            .build_struct_gep(
                option_val.into_struct_value().get_type(),
                option_ptr,
                0,
                "tag_ptr",
            )
            .map_err(|e| format!("Failed to get tag ptr: {}", e))?;

        let tag_val = self
            .builder
            .build_load(self.context.i32_type(), tag_ptr, "tag")
            .map_err(|e| format!("Failed to load tag: {}", e))?;

        // Check if tag == 0 (Some variant)
        let zero = self.context.i32_type().const_int(0, false);
        let is_some = self
            .builder
            .build_int_compare(IntPredicate::EQ, tag_val.into_int_value(), zero, "is_some")
            .map_err(|e| format!("Failed to compare tag: {}", e))?;

        // Branch: if Some -> body, if None -> end
        self.builder
            .build_conditional_branch(is_some, loop_body, loop_end)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

        // 4. Body block: extract value from Option and bind to variable
        self.builder.position_at_end(loop_body);

        // Get Item type from Iterator trait's associated type binding
        let item_type = self
            .associated_type_bindings
            .get(&(iter_type_name.clone(), "Item".to_string()))
            .cloned()
            .unwrap_or(Type::I32); // Default to i32 if not found

        // Get LLVM type for Item
        let item_llvm_type = self.ast_type_to_llvm(&item_type);

        // Extract value field (second element of struct)
        let value_ptr = self
            .builder
            .build_struct_gep(
                option_val.into_struct_value().get_type(),
                option_ptr,
                1,
                "value_ptr",
            )
            .map_err(|e| format!("Failed to get value ptr: {}", e))?;

        // Create variable for loop item
        let item_val = self
            .builder
            .build_load(item_llvm_type, value_ptr, variable)
            .map_err(|e| format!("Failed to load value: {}", e))?;

        // Store in loop variable
        let item_alloca = self.create_entry_block_alloca(
            variable, &item_type, false, // not mutable by default
        )?;
        self.build_store_aligned(item_alloca, item_val)?;
        self.variables.insert(variable.to_string(), item_alloca);
        self.variable_types
            .insert(variable.to_string(), item_llvm_type);

        // Compile loop body
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

        // 5. End block
        self.builder.position_at_end(loop_end);

        // Pop loop context
        self.loop_context_stack.pop();

        Ok(())
    }

    /// Compile loop statement: loop { body } (infinite loop)
    pub(crate) fn compile_loop(&mut self, body: &Block) -> Result<(), String> {
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
