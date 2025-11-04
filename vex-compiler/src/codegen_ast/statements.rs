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
                is_mutable,
                name,
                ty,
                value,
            } => {
                // v0.9: is_mutable determines if variable is mutable (let vs let!)
                // FIRST: Determine struct name from expression BEFORE compiling
                // (because after compilation, we lose the expression structure)
                eprintln!(
                    "🔷 Let statement: var={}, has_type_annotation={}",
                    name,
                    ty.is_some()
                );
                let struct_name_from_expr = if ty.is_none() {
                    eprintln!("  → Type inference needed, analyzing expression...");
                    match value {
                        Expression::StructLiteral {
                            name: s_name,
                            type_args,
                            ..
                        } => {
                            eprintln!("  → StructLiteral: {}", s_name);

                            // Handle generic struct literals: Box<i32> -> Box_i32
                            if !type_args.is_empty() {
                                // Instantiate the generic struct to get the mangled name
                                match self.instantiate_generic_struct(s_name, type_args) {
                                    Ok(mangled_name) => Some(mangled_name),
                                    Err(_) => None,
                                }
                            } else if self.struct_defs.contains_key(s_name) {
                                Some(s_name.clone())
                            } else {
                                None
                            }
                        }
                        Expression::MethodCall {
                            receiver, method, ..
                        } => {
                            eprintln!("  → MethodCall expression");
                            eprintln!("    → Method: {}", method);
                            // Get struct type from receiver
                            let struct_name = if let Expression::Ident(var_name) = receiver.as_ref()
                            {
                                if var_name == "self" {
                                    // Look up self's struct type
                                    self.variable_struct_names.get(var_name).cloned()
                                } else {
                                    self.variable_struct_names.get(var_name).cloned()
                                }
                            } else {
                                None
                            };

                            if let Some(struct_name) = struct_name {
                                let method_func_name = format!("{}_{}", struct_name, method);
                                if let Some(func_def) = self.function_defs.get(&method_func_name) {
                                    if let Some(Type::Named(s_name)) = &func_def.return_type {
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
                        }
                        Expression::Call { func, .. } => {
                            eprintln!("  → Call expression");
                            if let Expression::Ident(func_name) = func.as_ref() {
                                eprintln!("    → Function: {}", func_name);
                                if let Some(func_def) = self.function_defs.get(func_name) {
                                    eprintln!(
                                        "    → Found func_def, return_type: {:?}",
                                        func_def.return_type
                                    );
                                    if let Some(Type::Named(s_name)) = &func_def.return_type {
                                        eprintln!("    → Named return type: {}", s_name);
                                        if self.struct_defs.contains_key(s_name) {
                                            eprintln!("    ✅ Struct: {}", s_name);
                                            Some(s_name.clone())
                                        } else if self.enum_ast_defs.contains_key(s_name) {
                                            eprintln!("    ✅ Enum: {}", s_name);
                                            // Function returns enum - track it!
                                            Some(s_name.clone())
                                        } else {
                                            eprintln!("    ❌ Type not found: {}", s_name);
                                            None
                                        }
                                    } else {
                                        eprintln!("    → Return type is not Named");
                                        None
                                    }
                                } else {
                                    eprintln!("    ❌ func_def not found");
                                    None
                                }
                            } else {
                                eprintln!("    → Not an Ident");
                                None
                            }
                        }
                        _ => {
                            eprintln!("  → Other expression type");
                            None
                        }
                    }
                } else {
                    eprintln!("  → Type annotation present, skipping inference");
                    None
                };

                eprintln!("  → struct_name_from_expr: {:?}", struct_name_from_expr);

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
                let (var_type, llvm_type) = if let Some(t) = ty {
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
                    // Check if this is actually an enum (inferred as AnonymousStruct)
                    if struct_name == "AnonymousStruct" {
                        // Check if we have type info from expression analysis
                        if let Some(type_name) = struct_name_from_expr.as_ref() {
                            eprintln!("  → AnonymousStruct resolved to: {}", type_name);
                            if self.struct_defs.contains_key(type_name) {
                                eprintln!("  ✅ Tracking as struct (from expr): {}", type_name);
                                self.variable_struct_names
                                    .insert(name.clone(), type_name.clone());
                                Type::Named(type_name.clone())
                            } else if self.enum_ast_defs.contains_key(type_name) {
                                eprintln!("  ✅ Tracking as enum (from expr): {}", type_name);
                                self.variable_enum_names
                                    .insert(name.clone(), type_name.clone());
                                Type::Named(type_name.clone())
                            } else {
                                var_type.clone()
                            }
                        } else if let Expression::EnumLiteral { enum_name, .. } = value {
                            // Fallback: Check if value is an enum literal
                            self.variable_enum_names
                                .insert(name.clone(), enum_name.clone());
                            Type::Named(enum_name.clone())
                        } else {
                            var_type.clone()
                        }
                    } else if self.struct_defs.contains_key(struct_name) {
                        self.variable_struct_names
                            .insert(name.clone(), struct_name.clone());
                        var_type.clone()
                    } else if self.enum_ast_defs.contains_key(struct_name) {
                        // This is an enum type annotation
                        self.variable_enum_names
                            .insert(name.clone(), struct_name.clone());
                        var_type.clone()
                    } else {
                        var_type.clone()
                    }
                } else if let Type::Generic {
                    name: struct_name,
                    type_args,
                } = &var_type
                {
                    // Generic type annotation: Box<i32>, Pair<T, U>
                    // Instantiate to get mangled name and track it
                    match self.instantiate_generic_struct(struct_name, type_args) {
                        Ok(mangled_name) => {
                            self.variable_struct_names
                                .insert(name.clone(), mangled_name.clone());
                            Type::Generic {
                                name: struct_name.clone(),
                                type_args: type_args.clone(),
                            }
                        }
                        Err(_) => var_type.clone(),
                    }
                } else if let Some(type_name) = struct_name_from_expr {
                    // We found type name from expression analysis (could be struct or enum)
                    eprintln!("  → Checking type_name: {}", type_name);
                    eprintln!(
                        "    → Is struct: {}",
                        self.struct_defs.contains_key(&type_name)
                    );
                    eprintln!(
                        "    → Is enum: {}",
                        self.enum_ast_defs.contains_key(&type_name)
                    );
                    if self.struct_defs.contains_key(&type_name) {
                        eprintln!("  ✅ Tracking as struct: {}", type_name);
                        self.variable_struct_names
                            .insert(name.clone(), type_name.clone());
                        Type::Named(type_name)
                    } else if self.enum_ast_defs.contains_key(&type_name) {
                        eprintln!("  ✅ Tracking as enum: {} = {}", name, type_name);
                        self.variable_enum_names
                            .insert(name.clone(), type_name.clone());
                        Type::Named(type_name)
                    } else {
                        eprintln!("  ❌ Type {} not found!", type_name);
                        Type::Named(type_name)
                    }
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
                        // Enum literal: EnumName.Variant(...)
                        Expression::EnumLiteral { enum_name, .. } => {
                            if self.enum_ast_defs.contains_key(enum_name) {
                                self.variable_enum_names
                                    .insert(name.clone(), enum_name.clone());
                                Type::Named(enum_name.clone())
                            } else {
                                var_type.clone()
                            }
                        }
                        // Function call that returns a struct or enum
                        Expression::Call { func, .. } => {
                            // Check if this is an enum constructor call: EnumName.Variant(...)
                            if let Expression::FieldAccess { object, field: _ } = func.as_ref() {
                                if let Expression::Ident(enum_name) = object.as_ref() {
                                    if self.enum_ast_defs.contains_key(enum_name) {
                                        // This is an enum constructor call
                                        self.variable_enum_names
                                            .insert(name.clone(), enum_name.clone());
                                        Type::Named(enum_name.clone())
                                    } else {
                                        var_type.clone()
                                    }
                                } else {
                                    var_type.clone()
                                }
                            } else if let Expression::Ident(func_name) = func.as_ref() {
                                // Look up function's return type in function_defs
                                eprintln!(
                                    "  → Function call: {}, checking return type...",
                                    func_name
                                );
                                if let Some(func_def) = self.function_defs.get(func_name) {
                                    eprintln!(
                                        "  → Found func_def, return_type: {:?}",
                                        func_def.return_type
                                    );
                                    if let Some(Type::Named(type_name)) = &func_def.return_type {
                                        eprintln!("  → Named type: {}", type_name);
                                        if self.struct_defs.contains_key(type_name) {
                                            eprintln!("  ✅ Tracking as struct: {}", type_name);
                                            self.variable_struct_names
                                                .insert(name.clone(), type_name.clone());
                                            // Override the inferred type with the correct struct type
                                            Type::Named(type_name.clone())
                                        } else if self.enum_ast_defs.contains_key(type_name) {
                                            eprintln!("  ✅ Tracking as enum: {}", type_name);
                                            self.variable_enum_names
                                                .insert(name.clone(), type_name.clone());
                                            // Override the inferred type with the correct enum type
                                            Type::Named(type_name.clone())
                                        } else {
                                            eprintln!("  ❌ Type {} not found in struct_defs or enum_ast_defs", type_name);
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
                        // Field access: obj.field (returns struct type)
                        Expression::FieldAccess { object, field } => {
                            // Need to determine the type of the field being accessed
                            // First, get the struct type of the object
                            let object_struct_name = self.get_expression_struct_name(object)?;

                            if let Some(struct_name) = object_struct_name {
                                // Look up the struct definition to get field type
                                // Clone field_type to avoid borrow issues with instantiate_generic_struct
                                let field_type_opt =
                                    self.struct_defs.get(&struct_name).and_then(|struct_def| {
                                        struct_def
                                            .fields
                                            .iter()
                                            .find(|(f, _)| f == field)
                                            .map(|(_, t)| t.clone())
                                    });

                                if let Some(field_type) = field_type_opt {
                                    // Check if field type is a struct
                                    match field_type {
                                        Type::Named(field_struct_name) => {
                                            if self.struct_defs.contains_key(&field_struct_name) {
                                                self.variable_struct_names.insert(
                                                    name.clone(),
                                                    field_struct_name.clone(),
                                                );
                                                Type::Named(field_struct_name)
                                            } else {
                                                var_type.clone()
                                            }
                                        }
                                        Type::Generic {
                                            name: field_struct_name,
                                            type_args,
                                        } => {
                                            // Field is generic type like Box<i32>
                                            match self.instantiate_generic_struct(
                                                &field_struct_name,
                                                &type_args,
                                            ) {
                                                Ok(mangled_name) => {
                                                    self.variable_struct_names
                                                        .insert(name.clone(), mangled_name.clone());
                                                    Type::Generic {
                                                        name: field_struct_name,
                                                        type_args,
                                                    }
                                                }
                                                Err(_) => var_type.clone(),
                                            }
                                        }
                                        _ => var_type.clone(),
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
                // BUT: Skip for tuple and enum variables (they already have correct llvm_type)
                // IMPORTANT: For struct variables, use the VALUE type from expression, not pointer type
                let final_llvm_type = if let Type::Named(type_name) = &final_var_type {
                    if type_name == "Tuple" {
                        // Tuple variables already have correct struct type in llvm_type
                        llvm_type
                    } else if self.enum_ast_defs.contains_key(type_name) {
                        // Enum variables: use the inferred LLVM type from value
                        llvm_type
                    } else if self.struct_defs.contains_key(type_name) {
                        // CRITICAL FIX: For struct variables, the value from compile_expression
                        // is already a pointer (from struct literal). We need to store that pointer,
                        // so we allocate a pointer-sized slot, NOT another pointer to pointer!
                        // Solution: Use the actual pointer type we got from the value
                        llvm_type
                    } else {
                        self.ast_type_to_llvm(&final_var_type)
                    }
                } else if let Type::Generic { .. } = &final_var_type {
                    // Generic struct: use value type from expression
                    llvm_type
                } else {
                    llvm_type
                };

                // Create alloca using final_llvm_type directly for tuples
                // v0.9: Use is_mutable flag to distinguish let vs let!
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

                    let alloca = builder
                        .build_alloca(final_llvm_type, name)
                        .map_err(|e| format!("Failed to create tuple alloca: {}", e))?;

                    // Mark as readonly if immutable (let without !)
                    if !is_mutable {
                        // TODO: Add LLVM metadata for immutability optimization
                    }

                    alloca
                } else {
                    self.create_entry_block_alloca(name, &final_var_type, *is_mutable)?
                };

                // CRITICAL FIX: For struct variables, the value is already a pointer to stack-allocated struct
                // We should NOT allocate another slot and store the pointer - just use the struct pointer directly!
                if let Type::Named(type_name) = &final_var_type {
                    if self.struct_defs.contains_key(type_name) {
                        // Struct variable: val is already the pointer to the struct on stack
                        // Just register this pointer directly as the variable
                        if let BasicValueEnum::PointerValue(struct_ptr) = val {
                            self.variables.insert(name.clone(), struct_ptr);
                            self.variable_types.insert(name.clone(), final_llvm_type);
                        } else {
                            return Err(format!(
                                "Struct literal should return pointer, got {:?}",
                                val
                            ));
                        }
                    } else {
                        // Non-struct variable: normal store
                        self.builder
                            .build_store(alloca, val)
                            .map_err(|e| format!("Failed to store variable: {}", e))?;
                        self.variables.insert(name.clone(), alloca);
                        self.variable_types.insert(name.clone(), final_llvm_type);
                    }
                } else {
                    // Non-named type: normal store
                    self.builder
                        .build_store(alloca, val)
                        .map_err(|e| format!("Failed to store variable: {}", e))?;
                    self.variables.insert(name.clone(), alloca);
                    self.variable_types.insert(name.clone(), final_llvm_type);
                }

                // Check if this variable holds a closure with environment
                if let BasicValueEnum::PointerValue(fn_ptr) = val {
                    if let Some(env_ptr) = self.closure_envs.get(&fn_ptr) {
                        eprintln!(
                            "📝 Tracking closure variable: {} -> fn={:?}, env={:?}",
                            name, fn_ptr, env_ptr
                        );
                        self.closure_variables
                            .insert(name.clone(), (fn_ptr, *env_ptr));
                    }
                }
            }

            Statement::Assign { target, value } => {
                let val = self.compile_expression(value)?;

                match target {
                    // Simple variable assignment: x = value
                    Expression::Ident(name) => {
                        let ptr = self
                            .variables
                            .get(name)
                            .ok_or_else(|| format!("Variable {} not found", name))?;
                        self.builder
                            .build_store(*ptr, val)
                            .map_err(|e| format!("Failed to assign: {}", e))?;
                    }

                    // Field assignment: obj.field = value
                    Expression::FieldAccess { object, field } => {
                        // Get field pointer using similar logic to compile_field_access
                        if let Expression::Ident(var_name) = &**object {
                            // Get struct name from tracking
                            let struct_name = self
                                .variable_struct_names
                                .get(var_name)
                                .ok_or_else(|| format!("Variable {} is not a struct", var_name))?
                                .clone();

                            // Get variable pointer
                            let var_ptr = *self
                                .variables
                                .get(var_name)
                                .ok_or_else(|| format!("Variable {} not found", var_name))?;

                            // CRITICAL FIX: After struct variable storage fix, self.variables[name] now holds
                            // the DIRECT pointer to the struct (not a pointer to a pointer variable).
                            // So we should use var_ptr directly, NOT load it!
                            let struct_ptr_val = var_ptr;

                            // Get struct definition
                            let struct_def = self
                                .struct_defs
                                .get(&struct_name)
                                .cloned()
                                .ok_or_else(|| format!("Struct '{}' not found", struct_name))?;

                            // Find field index
                            let field_index = struct_def
                                .fields
                                .iter()
                                .position(|(name, _)| name == field)
                                .ok_or_else(|| {
                                    format!(
                                        "Field '{}' not found in struct '{}'",
                                        field, struct_name
                                    )
                                })? as u32;

                            // Build struct type
                            let field_types: Vec<inkwell::types::BasicTypeEnum> = struct_def
                                .fields
                                .iter()
                                .map(|(_, ty)| self.ast_type_to_llvm(ty))
                                .collect();
                            let struct_type = self.context.struct_type(&field_types, false);

                            // Get field pointer
                            let field_ptr = self
                                .builder
                                .build_struct_gep(
                                    struct_type,
                                    struct_ptr_val,
                                    field_index,
                                    &format!("{}.{}", var_name, field),
                                )
                                .map_err(|e| format!("Failed to get field pointer: {}", e))?;

                            // Store value
                            self.builder
                                .build_store(field_ptr, val)
                                .map_err(|e| format!("Failed to store field: {}", e))?;
                        } else {
                            return Err("Complex field assignment not yet supported".to_string());
                        }
                    }

                    _ => {
                        return Err(
                            "Complex assignment targets not yet supported (array indexing, etc.)"
                                .to_string(),
                        );
                    }
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
                // Execute deferred statements in reverse order before returning
                self.execute_deferred_statements()?;

                if let Some(e) = expr {
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

            Statement::Break => {
                // Execute deferred statements before break
                self.execute_deferred_statements()?;

                // Get current loop context
                if let Some((_, break_block)) = self.loop_context_stack.last() {
                    let break_block = *break_block;
                    self.builder
                        .build_unconditional_branch(break_block)
                        .map_err(|e| format!("Failed to build break branch: {}", e))?;
                } else {
                    return Err("Break statement outside of loop".to_string());
                }
            }

            Statement::Continue => {
                // Execute deferred statements before continue
                self.execute_deferred_statements()?;

                // Get current loop context
                if let Some((continue_block, _)) = self.loop_context_stack.last() {
                    let continue_block = *continue_block;
                    self.builder
                        .build_unconditional_branch(continue_block)
                        .map_err(|e| format!("Failed to build continue branch: {}", e))?;
                } else {
                    return Err("Continue statement outside of loop".to_string());
                }
            }

            Statement::Defer(stmt) => {
                // Add statement to defer stack (LIFO)
                // Don't execute now, execute on function exit
                self.deferred_statements.push(stmt.as_ref().clone());
            }

            Statement::If {
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.compile_if_statement(condition, then_block, elif_branches, else_block)?;
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

    /// Compile if statement with elif support
    fn compile_if_statement(
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
    fn compile_for_loop(
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
    fn compile_switch_statement(
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
}
