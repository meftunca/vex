// Statement code generation

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a block of statements
    pub(crate) fn compile_block(&mut self, block: &Block) -> Result<(), String> {
        for stmt in &block.statements {
            self.compile_statement(stmt)?;

            // Check if block is terminated (return/break/continue)
            if let Some(current_block) = self.builder.get_insert_block() {
                if current_block.get_terminator().is_some() {
                    break; // Stop compiling statements after terminator
                }
            }
        }
        Ok(())
    }

    /// Compile a statement
    pub(crate) fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let {
                is_mutable: _,
                name,
                ty,
                value,
            }
            | Statement::VarDecl {
                is_const: _,
                name,
                ty,
                value,
            } => {
                // FIRST: Determine struct name from expression BEFORE compiling
                // (because after compilation, we lose the expression structure)
                let struct_name_from_expr = if ty.is_none() {
                    match value {
                        Expression::StructLiteral { name: s_name, .. } => {
                            if self.struct_defs.contains_key(s_name) {
                                Some(s_name.clone())
                            } else {
                                None
                            }
                        }
                        Expression::Call { func, .. } => {
                            if let Expression::Ident(func_name) = func.as_ref() {
                                if let Some(func_def) = self.function_defs.get(func_name) {
                                    if let Some(Type::Named(s_name)) = &func_def.return_type {
                                        if self.struct_defs.contains_key(s_name) {
                                            Some(s_name.clone())
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                // Special handling for tuple literals: pre-compute struct type before compilation
                let tuple_struct_type = if let Expression::TupleLiteral(elements) = value {
                    // Pre-compute element types to build the struct type
                    let mut element_types = Vec::new();
                    for elem_expr in elements.iter() {
                        let elem_val = self.compile_expression(elem_expr)?;
                        element_types.push(elem_val.get_type());
                    }
                    let struct_ty = self.context.struct_type(&element_types, false);
                    // Save it for pattern matching
                    self.tuple_variable_types.insert(name.clone(), struct_ty);
                    Some(struct_ty)
                } else {
                    None
                };

                let mut val = self.compile_expression(value)?;

                // For tuple literals, load the struct value (tuple literal returns pointer)
                if let Some(struct_ty) = tuple_struct_type {
                    if val.is_pointer_value() {
                        val = self
                            .builder
                            .build_load(struct_ty, val.into_pointer_value(), "tuple_val_loaded")
                            .map_err(|e| format!("Failed to load tuple value: {}", e))?;
                    }
                }

                // Determine type from value or explicit type
                let (var_type, llvm_type) = if let Some(ref t) = ty {
                    (t.clone(), self.ast_type_to_llvm(t))
                } else if let Some(struct_ty) = tuple_struct_type {
                    // Use the pre-computed tuple struct type
                    // For tuples, use a placeholder AST type and directly use the struct LLVM type
                    eprintln!("[DEBUG LET] Using tuple struct_ty: {:?}", struct_ty);
                    (Type::Named("Tuple".to_string()), struct_ty.into())
                } else {
                    // Infer type from LLVM value
                    let inferred_llvm_type = val.get_type();
                    let inferred_ast_type = self.infer_ast_type_from_llvm(inferred_llvm_type)?;
                    (inferred_ast_type, inferred_llvm_type)
                };

                // Track struct type name if this is a named struct type or inferred from expression
                let final_var_type = if let Type::Named(struct_name) = &var_type {
                    if self.struct_defs.contains_key(struct_name) {
                        self.variable_struct_names
                            .insert(name.clone(), struct_name.clone());
                    }
                    var_type.clone()
                } else if let Some(struct_name) = struct_name_from_expr {
                    // We found struct name from expression analysis
                    self.variable_struct_names
                        .insert(name.clone(), struct_name.clone());
                    Type::Named(struct_name)
                } else if ty.is_none() {
                    // Type inference: check value expression type
                    match value {
                        // Struct literal: TypeName { ... }
                        Expression::StructLiteral {
                            name: struct_name, ..
                        } => {
                            if self.struct_defs.contains_key(struct_name) {
                                self.variable_struct_names
                                    .insert(name.clone(), struct_name.clone());
                                Type::Named(struct_name.clone())
                            } else {
                                var_type.clone()
                            }
                        }
                        // Function call that returns a struct
                        Expression::Call { func, .. } => {
                            if let Expression::Ident(func_name) = func.as_ref() {
                                // Look up function's return type in function_defs
                                if let Some(func_def) = self.function_defs.get(func_name) {
                                    if let Some(Type::Named(struct_name)) = &func_def.return_type {
                                        if self.struct_defs.contains_key(struct_name) {
                                            self.variable_struct_names
                                                .insert(name.clone(), struct_name.clone());
                                            // Override the inferred type with the correct struct type
                                            Type::Named(struct_name.clone())
                                        } else {
                                            var_type.clone()
                                        }
                                    } else {
                                        var_type.clone()
                                    }
                                } else {
                                    var_type.clone()
                                }
                            } else {
                                var_type.clone()
                            }
                        }
                        _ => var_type.clone(),
                    }
                } else {
                    var_type.clone()
                };

                // Recalculate LLVM type if we found a struct
                // BUT: Skip for tuple variables (they already have correct llvm_type)
                let final_llvm_type = if let Type::Named(type_name) = &final_var_type {
                    if type_name == "Tuple" {
                        // Tuple variables already have correct struct type in llvm_type
                        llvm_type
                    } else {
                        self.ast_type_to_llvm(&final_var_type)
                    }
                } else {
                    llvm_type
                };

                // Create alloca using final_llvm_type directly for tuples
                let alloca = if tuple_struct_type.is_some() {
                    // For tuples, use the LLVM struct type directly
                    let builder = self.context.create_builder();
                    let entry = self
                        .current_function
                        .ok_or("No current function")?
                        .get_first_basic_block()
                        .ok_or("Function has no entry block")?;

                    match entry.get_first_instruction() {
                        Some(first_instr) => builder.position_before(&first_instr),
                        None => builder.position_at_end(entry),
                    }

                    builder
                        .build_alloca(final_llvm_type, name)
                        .map_err(|e| format!("Failed to create tuple alloca: {}", e))?
                } else {
                    self.create_entry_block_alloca(name, &final_var_type)?
                };

                self.builder
                    .build_store(alloca, val)
                    .map_err(|e| format!("Failed to store variable: {}", e))?;
                self.variables.insert(name.clone(), alloca);
                self.variable_types.insert(name.clone(), final_llvm_type);
            }

            Statement::Assign { target, value } => {
                let val = self.compile_expression(value)?;

                // Get target pointer
                if let Expression::Ident(name) = target {
                    let ptr = self
                        .variables
                        .get(name)
                        .ok_or_else(|| format!("Variable {} not found", name))?;
                    self.builder
                        .build_store(*ptr, val)
                        .map_err(|e| format!("Failed to assign: {}", e))?;
                } else {
                    return Err(
                        "Complex assignment targets not yet supported (field access, etc.)"
                            .to_string(),
                    );
                }
            }

            Statement::CompoundAssign { target, op, value } => {
                // Compound assignment: x += expr is equivalent to x = x + expr
                // First load the current value
                let current_val = self.compile_expression(target)?;
                let rhs_val = self.compile_expression(value)?;

                // Perform the operation based on type
                let result: BasicValueEnum = if current_val.is_int_value() {
                    let lhs = current_val.into_int_value();
                    let rhs = rhs_val.into_int_value();
                    match op {
                        CompoundOp::Add => self
                            .builder
                            .build_int_add(lhs, rhs, "add_tmp")
                            .map_err(|e| format!("Failed to build add: {}", e))?
                            .into(),
                        CompoundOp::Sub => self
                            .builder
                            .build_int_sub(lhs, rhs, "sub_tmp")
                            .map_err(|e| format!("Failed to build sub: {}", e))?
                            .into(),
                        CompoundOp::Mul => self
                            .builder
                            .build_int_mul(lhs, rhs, "mul_tmp")
                            .map_err(|e| format!("Failed to build mul: {}", e))?
                            .into(),
                        CompoundOp::Div => self
                            .builder
                            .build_int_signed_div(lhs, rhs, "div_tmp")
                            .map_err(|e| format!("Failed to build div: {}", e))?
                            .into(),
                    }
                } else if current_val.is_float_value() {
                    let lhs = current_val.into_float_value();
                    let rhs = rhs_val.into_float_value();
                    match op {
                        CompoundOp::Add => self
                            .builder
                            .build_float_add(lhs, rhs, "add_tmp")
                            .map_err(|e| format!("Failed to build add: {}", e))?
                            .into(),
                        CompoundOp::Sub => self
                            .builder
                            .build_float_sub(lhs, rhs, "sub_tmp")
                            .map_err(|e| format!("Failed to build sub: {}", e))?
                            .into(),
                        CompoundOp::Mul => self
                            .builder
                            .build_float_mul(lhs, rhs, "mul_tmp")
                            .map_err(|e| format!("Failed to build mul: {}", e))?
                            .into(),
                        CompoundOp::Div => self
                            .builder
                            .build_float_div(lhs, rhs, "div_tmp")
                            .map_err(|e| format!("Failed to build div: {}", e))?
                            .into(),
                    }
                } else {
                    return Err("Invalid type for compound assignment".to_string());
                };

                // Store the result back
                match target {
                    Expression::Ident(name) => {
                        // Simple variable assignment
                        let ptr = self
                            .variables
                            .get(name)
                            .ok_or_else(|| format!("Variable {} not found", name))?;
                        self.builder
                            .build_store(*ptr, result)
                            .map_err(|e| format!("Failed to store compound assignment: {}", e))?;
                    }
                    Expression::FieldAccess { object, field } => {
                        // Field access assignment: obj.field += value
                        let field_ptr = self.get_field_pointer(object, field)?;
                        self.builder.build_store(field_ptr, result).map_err(|e| {
                            format!("Failed to store field compound assignment: {}", e)
                        })?;
                    }
                    Expression::Index { object, index } => {
                        // Array index assignment: arr[i] += value
                        let elem_ptr = self.get_index_pointer(object, index)?;
                        self.builder.build_store(elem_ptr, result).map_err(|e| {
                            format!("Failed to store array compound assignment: {}", e)
                        })?;
                    }
                    _ => {
                        return Err(
                            "This compound assignment target is not yet supported".to_string()
                        );
                    }
                }
            }

            Statement::Return(expr) => {
                if let Some(ref e) = expr {
                    let val = self.compile_expression(e)?;
                    self.builder
                        .build_return(Some(&val))
                        .map_err(|e| format!("Failed to build return: {}", e))?;
                } else {
                    let zero = self.context.i32_type().const_int(0, false);
                    self.builder
                        .build_return(Some(&zero))
                        .map_err(|e| format!("Failed to build return: {}", e))?;
                }
            }

            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.compile_if_statement(condition, then_block, else_block)?;
            }

            Statement::For {
                init,
                condition,
                post,
                body,
            } => {
                self.compile_for_loop(init, condition, post, body)?;
            }

            Statement::While { condition, body } => {
                self.compile_while_loop(condition, body)?;
            }

            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
            }

            Statement::Switch {
                value,
                cases,
                default_case,
            } => {
                self.compile_switch_statement(value, cases, default_case)?;
            }

            _ => return Err(format!("Statement not yet implemented: {:?}", stmt)),
        }

        Ok(())
    }

    /// Compile if statement
    fn compile_if_statement(
        &mut self,
        condition: &Expression,
        then_block: &Block,
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
        let else_bb = self.context.append_basic_block(fn_val, "else");
        let merge_bb = self.context.append_basic_block(fn_val, "ifcont");

        // Build conditional branch
        self.builder
            .build_conditional_branch(bool_val, then_bb, else_bb)
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

        // Compile else block
        self.builder.position_at_end(else_bb);
        if let Some(ref eb) = else_block {
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
        }

        // Continue at merge block ONLY if at least one branch didn't terminate
        // If both branches terminated (return/break/continue), merge block is unreachable
        if !then_terminated || !else_terminated {
            self.builder.position_at_end(merge_bb);
        } else {
            // Both branches terminated - merge block is unreachable
            // We need to add a terminator to it to satisfy LLVM
            self.builder.position_at_end(merge_bb);
            self.builder
                .build_unreachable()
                .map_err(|e| format!("Failed to build unreachable: {}", e))?;
        }

        Ok(())
    }

    /// Compile while loop: while condition { body }
    fn compile_while_loop(&mut self, condition: &Expression, body: &Block) -> Result<(), String> {
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
        self.compile_block(body)?;
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
    fn compile_for_loop(
        &mut self,
        init: &Option<Box<Statement>>,
        condition: &Option<Expression>,
        post: &Option<Box<Statement>>,
        body: &Block,
    ) -> Result<(), String> {
        let fn_val = self.current_function.ok_or("No current function")?;

        // Compile init statement
        if let Some(ref i) = init {
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
        if let Some(ref cond) = condition {
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
        self.compile_block(body)?;
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
        if let Some(ref p) = post {
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
    fn compile_switch_statement(
        &mut self,
        value: &Option<Expression>,
        cases: &[SwitchCase],
        default_case: &Option<Block>,
    ) -> Result<(), String> {
        // Evaluate the switch value
        let switch_val = if let Some(ref val_expr) = value {
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
        if let Some(ref def_block) = default_case {
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
}
