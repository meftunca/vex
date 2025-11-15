// statements/loops/for_in_loop.rs
// for-in loop compilation

use super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile for-in loop: for item in iterator { body }
    /// Works with:
    /// 1. Range/RangeInclusive (0..10, 0..=10)
    /// 2. Any type implementing Iterator trait
    pub(crate) fn compile_for_in_loop_dispatch(
        &mut self,
        variable: &str,
        iterable: &Expression,
        body: &Block,
    ) -> Result<(), String> {
        self.compile_for_in_loop_impl(variable, iterable, body)
    }

    /// Compile for-in loop: for item in iterator { body }
    /// Works with:
    /// 1. Range/RangeInclusive (0..10, 0..=10)
    /// 2. Any type implementing Iterator trait
    fn compile_for_in_loop_impl(
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
            self.compile_for_in_range_impl(variable, iterable, body)
        } else {
            // Use Iterator trait-based implementation
            self.compile_for_in_iterator_impl(variable, iterable, body)
        }
    }

    /// Compile for-in loop with Range (legacy implementation)
    fn compile_for_in_range_impl(
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
            .unwrap_basic();

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
            .ok_or("No active basic block")?
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
    fn compile_for_in_iterator_impl(
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
            .ok_or("No active basic block")?
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
}
