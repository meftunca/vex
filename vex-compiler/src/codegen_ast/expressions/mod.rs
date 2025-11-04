// Expression code generation
// This module dispatches expression compilation and coordinates submodules

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

// Submodules
mod access;
mod binary_ops;
mod calls;
mod literals;
mod special;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Main expression compiler - dispatches to specialized methods
    pub(crate) fn compile_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expression::IntLiteral(n) => {
                Ok(self.context.i32_type().const_int(*n as u64, false).into())
            }

            Expression::FloatLiteral(f) => Ok(self.context.f64_type().const_float(*f).into()),

            Expression::BoolLiteral(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }

            Expression::StringLiteral(s) => {
                // Create global string constant
                let global_str = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| format!("Failed to create string: {}", e))?;
                Ok(global_str.as_pointer_value().into())
            }

            Expression::FStringLiteral(s) => {
                // For now, handle F-strings as formatted strings with interpolation
                self.compile_fstring(s)
            }

            Expression::Ident(name) => {
                // Check if this is a function pointer parameter first
                if let Some(fn_ptr) = self.function_params.get(name) {
                    // Return function pointer directly
                    return Ok((*fn_ptr).into());
                }

                // Check if this is a variable (includes regular parameters)
                if let Some(ptr) = self.variables.get(name) {
                    let ty = self
                        .variable_types
                        .get(name)
                        .ok_or_else(|| format!("Type for variable {} not found", name))?;

                    if name == "t" {
                        eprintln!("[DEBUG VAR LOAD] Variable 't' type: {:?}", ty);
                    }

                    // IMPORTANT: For struct variables, return the pointer directly (zero-copy)
                    // Struct types in LLVM are already pointers (ast_type_to_llvm returns pointer for structs)
                    if self.variable_struct_names.contains_key(name) {
                        // This is a struct variable - return pointer without loading
                        return Ok((*ptr).into());
                    }

                    let loaded = self
                        .builder
                        .build_load(*ty, *ptr, name)
                        .map_err(|e| format!("Failed to load variable: {}", e))?;

                    if name == "t" {
                        eprintln!(
                            "[DEBUG VAR LOAD] Loaded value is_struct: {}",
                            loaded.is_struct_value()
                        );
                        eprintln!(
                            "[DEBUG VAR LOAD] Loaded value is_pointer: {}",
                            loaded.is_pointer_value()
                        );
                    }

                    return Ok(loaded);
                }

                // Check if this is a global function name (for function pointers)
                if let Some(func_val) = self.functions.get(name) {
                    // Return function as a pointer value
                    return Ok(func_val.as_global_value().as_pointer_value().into());
                }

                Err(format!("Variable or function {} not found", name))
            }

            Expression::Binary { left, op, right } => self.compile_binary_op(left, op, right),

            Expression::Unary { op, expr } => self.compile_unary_op(op, expr),

            Expression::Call { func, args } => self.compile_call(func, args),

            Expression::MethodCall {
                receiver,
                method,
                args,
            } => self.compile_method_call(receiver, method, args),

            Expression::Index { object, index } => self.compile_index(object, index),

            Expression::Array(elements) => self.compile_array_literal(elements),

            Expression::TupleLiteral(elements) => self.compile_tuple_literal(elements),

            Expression::StructLiteral {
                name,
                type_args,
                fields,
            } => self.compile_struct_literal(name, type_args, fields),

            Expression::FieldAccess { object, field } => self.compile_field_access(object, field),

            Expression::PostfixOp { expr, op } => self.compile_postfix_op(expr, op),

            Expression::Await(expr) => {
                // For now, await is just a pass-through - compile the inner expression
                // TODO: Proper async runtime with futures/promises and state machines
                self.compile_expression(expr)
            }

            Expression::Nil => {
                // Return zero/null for nil
                Ok(self.context.i8_type().const_int(0, false).into())
            }

            Expression::Match { value, arms } => self.compile_match_expression(value, arms),

            Expression::Block {
                statements,
                return_expr,
            } => self.compile_block_expression(statements, return_expr),

            Expression::Try(expr) => {
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

            Expression::Go(expr) => {
                // Spawn a new task with runtime
                // For now, just execute synchronously and return a task ID placeholder
                // TODO: Proper goroutine/task spawning with:
                // 1. Creating a new thread/task in runtime
                // 2. Copying captured variables
                // 3. Returning JoinHandle

                // Compile the expression (this executes it synchronously for now)
                let _result = self.compile_expression(expr)?;

                // Return a task ID (0 for now - placeholder)
                Ok(self.context.i32_type().const_int(0, false).into())
            }

            Expression::Reference { is_mutable, expr } => {
                // Take a reference to an expression: &expr or &expr!
                // This creates a pointer to the value
                match expr.as_ref() {
                    Expression::Ident(name) => {
                        // For identifiers, return the pointer directly (don't load)
                        let ptr = self
                            .variables
                            .get(name)
                            .ok_or_else(|| format!("Variable {} not found", name))?;
                        Ok((*ptr).into())
                    }
                    _ => {
                        // For other expressions, compile them, store in temporary, return pointer
                        let value = self.compile_expression(expr)?;
                        let value_type = value.get_type();
                        let temp_name = if *is_mutable {
                            "ref_temp_mut"
                        } else {
                            "ref_temp"
                        };
                        let temp_ptr =
                            self.builder
                                .build_alloca(value_type, temp_name)
                                .map_err(|e| {
                                    format!("Failed to allocate reference temporary: {}", e)
                                })?;
                        self.builder
                            .build_store(temp_ptr, value)
                            .map_err(|e| format!("Failed to store reference temporary: {}", e))?;
                        Ok(temp_ptr.into())
                    }
                }
            }

            Expression::Deref(expr) => {
                // Dereference a pointer: *ptr
                // Try to infer the inner type from the expression
                match expr.as_ref() {
                    Expression::Ident(name) => {
                        // For identifiers, we can load using the stored LLVM type
                        let ptr = self
                            .variables
                            .get(name)
                            .ok_or_else(|| format!("Variable {} not found", name))?;
                        let var_type = self
                            .variable_types
                            .get(name)
                            .ok_or_else(|| format!("Type for variable {} not found", name))?;

                        // Load the pointer value first (variables store the reference)
                        let ptr_loaded = self
                            .builder
                            .build_load(*var_type, *ptr, &format!("{}_ptr", name))
                            .map_err(|e| format!("Failed to load pointer variable: {}", e))?;

                        if !ptr_loaded.is_pointer_value() {
                            return Err(format!(
                                "Cannot dereference non-pointer variable {}",
                                name
                            ));
                        }

                        // Now dereference the pointer
                        // For references, the inner type is what we need to load
                        // Since we don't track AST types, we'll use a heuristic:
                        // Try common types (i32, i64, f64, bool)
                        // TODO: Add proper AST type tracking for variables
                        let deref_ptr = ptr_loaded.into_pointer_value();

                        // Try to determine the pointee type
                        // For now, default to i32 (most common case)
                        let inner_type = self.context.i32_type();
                        let loaded = self
                            .builder
                            .build_load(inner_type, deref_ptr, "deref")
                            .map_err(|e| format!("Failed to dereference pointer: {}", e))?;
                        Ok(loaded)
                    }
                    _ => {
                        // For other expressions, compile and dereference
                        let ptr_value = self.compile_expression(expr)?;
                        if !ptr_value.is_pointer_value() {
                            return Err("Cannot dereference non-pointer value".to_string());
                        }
                        let ptr = ptr_value.into_pointer_value();

                        // Default to i32 for now
                        let inner_type = self.context.i32_type();
                        let loaded = self
                            .builder
                            .build_load(inner_type, ptr, "deref")
                            .map_err(|e| format!("Failed to dereference pointer: {}", e))?;
                        Ok(loaded)
                    }
                }
            }

            Expression::EnumLiteral {
                enum_name,
                variant,
                data,
            } => {
                // For now, treat enums as tagged unions (C-style for unit variants)
                // Unit variants (no data): Just return the tag as i32
                // Data-carrying variants: Need struct with tag + data (TODO: full implementation)

                // Look up enum definition
                if let Some(enum_def) = self.enum_ast_defs.get(enum_name) {
                    // Find variant index
                    let variant_index = enum_def
                        .variants
                        .iter()
                        .position(|v| &v.name == variant)
                        .ok_or_else(|| {
                        format!("Variant {} not found in enum {}", variant, enum_name)
                    })?;

                    // Check if enum has ANY data-carrying variants
                    let enum_has_data = enum_def.variants.iter().any(|v| v.data.is_some());

                    if data.is_none() && !enum_has_data {
                        // Pure unit enum (all variants are unit): just return the tag
                        let tag = self
                            .context
                            .i32_type()
                            .const_int(variant_index as u64, false);
                        Ok(tag.into())
                    } else {
                        // Mixed enum (has data variants): create struct { tag: i32, data: T }
                        // For unit variants in mixed enums, use enum's data type from definition

                        let (data_value, actual_data_type) = if let Some(data_expr) = data {
                            // Variant has data: compile expression
                            let val = self.compile_expression(data_expr)?;
                            let ty = val.get_type();
                            (val, ty)
                        } else {
                            // Unit variant in mixed enum: use enum's largest data type with zero value
                            let enum_llvm_type =
                                self.ast_type_to_llvm(&Type::Named(enum_name.clone()));
                            if let BasicTypeEnum::StructType(struct_ty) = enum_llvm_type {
                                // Extract data field type (index 1)
                                let data_field_type = struct_ty
                                    .get_field_type_at_index(1)
                                    .ok_or_else(|| "Enum struct missing data field".to_string())?;
                                // Create zero/undef value for data field
                                let zero_value = match data_field_type {
                                    BasicTypeEnum::IntType(int_ty) => int_ty.const_zero().into(),
                                    BasicTypeEnum::FloatType(float_ty) => {
                                        float_ty.const_zero().into()
                                    }
                                    BasicTypeEnum::PointerType(ptr_ty) => {
                                        ptr_ty.const_null().into()
                                    }
                                    _ => {
                                        return Err(format!(
                                            "Unsupported enum data type: {:?}",
                                            data_field_type
                                        ))
                                    }
                                };
                                (zero_value, data_field_type)
                            } else {
                                return Err(format!(
                                    "Expected struct type for mixed enum {}",
                                    enum_name
                                ));
                            }
                        };

                        // Create struct type: { i32, T }
                        let tag_type = self.context.i32_type();
                        let struct_type = self
                            .context
                            .struct_type(&[tag_type.into(), actual_data_type], false);

                        // Allocate struct on stack
                        let struct_ptr = self
                            .builder
                            .build_alloca(struct_type, "enum_data_carrier")
                            .map_err(|e| format!("Failed to allocate enum struct: {}", e))?;

                        // Store tag at index 0
                        let tag = self
                            .context
                            .i32_type()
                            .const_int(variant_index as u64, false);
                        let tag_ptr = self
                            .builder
                            .build_struct_gep(struct_type, struct_ptr, 0, "enum_tag_ptr")
                            .map_err(|e| format!("Failed to get tag pointer: {}", e))?;
                        self.builder
                            .build_store(tag_ptr, tag)
                            .map_err(|e| format!("Failed to store tag: {}", e))?;

                        // Store data at index 1
                        let data_ptr = self
                            .builder
                            .build_struct_gep(struct_type, struct_ptr, 1, "enum_data_ptr")
                            .map_err(|e| format!("Failed to get data pointer: {}", e))?;
                        self.builder
                            .build_store(data_ptr, data_value)
                            .map_err(|e| format!("Failed to store data: {}", e))?;

                        // Load and return the struct value
                        let struct_value = self
                            .builder
                            .build_load(struct_type, struct_ptr, "enum_with_data")
                            .map_err(|e| format!("Failed to load enum struct: {}", e))?;

                        Ok(struct_value)
                    }
                } else {
                    Err(format!("Enum {} not found", enum_name))
                }
            }

            Expression::Closure {
                params,
                return_type,
                body,
                capture_mode,
            } => self.compile_closure(params, return_type, body, capture_mode),

            _ => Err(format!("Expression not yet implemented: {:?}", expr)),
        }
    }

    /// Compile match expression as a series of if-else comparisons
    fn compile_match_expression(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if arms.is_empty() {
            return Err("Match expression must have at least one arm".to_string());
        }

        // Compile the value to match against
        let mut match_value = self.compile_expression(value)?;
        eprintln!(
            "🟣 Match value compiled: type={:?}, is_struct={}",
            match_value.get_type(),
            match_value.is_struct_value()
        );

        // Special handling for tuple literals/variables: load struct value for pattern matching
        if matches!(value, Expression::TupleLiteral(_)) && match_value.is_pointer_value() {
            // Direct tuple literals return pointers, load the struct value
            if let Expression::TupleLiteral(elements) = value {
                // Compute struct type from elements
                let mut element_types = Vec::new();
                for elem_expr in elements.iter() {
                    let elem_val = self.compile_expression(elem_expr)?;
                    element_types.push(elem_val.get_type());
                }
                let struct_ty = self.context.struct_type(&element_types, false);
                match_value = self
                    .builder
                    .build_load(
                        struct_ty,
                        match_value.into_pointer_value(),
                        "match_tuple_literal_loaded",
                    )
                    .map_err(|e| format!("Failed to load tuple literal for match: {}", e))?;
            }
        } else if matches!(value, Expression::StructLiteral { .. })
            && match_value.is_pointer_value()
        {
            // Struct literals also return pointers, load the struct value
            if let Expression::StructLiteral { name, .. } = value {
                // Build struct type from definition
                let struct_def = self
                    .struct_defs
                    .get(name)
                    .ok_or_else(|| format!("Struct '{}' not found", name))?
                    .clone();

                let field_types: Vec<_> = struct_def
                    .fields
                    .iter()
                    .map(|(_, ty)| self.ast_type_to_llvm(ty))
                    .collect();
                let struct_type = self.context.struct_type(&field_types, false);

                match_value = self
                    .builder
                    .build_load(
                        struct_type,
                        match_value.into_pointer_value(),
                        "match_struct_literal_loaded",
                    )
                    .map_err(|e| format!("Failed to load struct literal for match: {}", e))?;
            }
        } else if let Expression::Ident(var_name) = value {
            // Check if this variable is a tuple (tracked separately)
            if let Some(tuple_struct_type) = self.tuple_variable_types.get(var_name) {
                // Variable lookup already loads, no need to load again
                if match_value.is_pointer_value() {
                    match_value = self
                        .builder
                        .build_load(
                            *tuple_struct_type,
                            match_value.into_pointer_value(),
                            "match_tuple_var_loaded",
                        )
                        .map_err(|e| format!("Failed to load tuple variable for match: {}", e))?;
                    eprintln!(
                        "[DEBUG MATCH] After load - is_struct: {}",
                        match_value.is_struct_value()
                    );
                }
            } else if match_value.is_pointer_value() {
                // Load the value for pattern matching
                // Could be struct, enum, or other composite type
                let ptr = match_value.into_pointer_value();

                // Get the actual type from variable_types map
                if let Some(var_type) = self.variable_types.get(var_name) {
                    match_value = self
                        .builder
                        .build_load(*var_type, ptr, "match_var_loaded")
                        .map_err(|e| format!("Failed to load variable for match: {}", e))?;
                }
            }

            // Also handle struct names for old code path
            if let Some(struct_name) = self.variable_struct_names.get(var_name) {
                // This variable holds a struct value, load it for pattern matching
                if match_value.is_pointer_value() {
                    // Build struct type from definition
                    let struct_def = self
                        .struct_defs
                        .get(struct_name)
                        .ok_or_else(|| format!("Struct '{}' not found", struct_name))?
                        .clone();

                    let field_types: Vec<_> = struct_def
                        .fields
                        .iter()
                        .map(|(_, ty)| self.ast_type_to_llvm(ty))
                        .collect();
                    let struct_type = self.context.struct_type(&field_types, false);

                    match_value = self
                        .builder
                        .build_load(
                            struct_type,
                            match_value.into_pointer_value(),
                            "match_struct_var_loaded",
                        )
                        .map_err(|e| format!("Failed to load struct variable for match: {}", e))?;
                }
            }

            // Also handle enum variables
            eprintln!("  → Checking enum tracking for var: {}", var_name);
            eprintln!(
                "  → variable_enum_names contains: {}",
                self.variable_enum_names.contains_key(var_name)
            );
            if let Some(enum_name) = self.variable_enum_names.get(var_name) {
                eprintln!(
                    "  → Found enum: {}, is_pointer: {}",
                    enum_name,
                    match_value.is_pointer_value()
                );
                // This variable holds an enum value, load it for pattern matching
                if match_value.is_pointer_value() {
                    eprintln!("  → Loading enum from pointer...");
                    // Use the stored type from variable_types
                    if let Some(enum_type) = self.variable_types.get(var_name) {
                        eprintln!("  → Found enum_type: {:?}", enum_type);
                        match_value = self
                            .builder
                            .build_load(
                                *enum_type,
                                match_value.into_pointer_value(),
                                "match_enum_var_loaded",
                            )
                            .map_err(|e| {
                                format!("Failed to load enum variable for match: {}", e)
                            })?;
                        eprintln!("  ✅ Loaded enum value!");
                    } else {
                        eprintln!("  ❌ enum_type not found in variable_types");
                    }
                } else {
                    eprintln!("  → Value is not pointer, skipping load");
                }
            } else {
                eprintln!("  ❌ Enum name not found in variable_enum_names");
            }
        }

        // Create the merge block where all arms converge
        let merge_block = self
            .context
            .append_basic_block(self.current_function.unwrap(), "match_merge");

        // We'll create result_ptr in the function entry block (before any branching)
        // To infer the type, we need to peek at the first arm's body type
        let mut result_ptr: Option<inkwell::values::PointerValue> = None;
        let mut result_type: Option<inkwell::types::BasicTypeEnum> = None;

        // Build chain of if-else blocks for each arm
        let mut current_block = self.builder.get_insert_block().unwrap();

        for (i, arm) in arms.iter().enumerate() {
            self.builder.position_at_end(current_block);

            let is_last_arm = i == arms.len() - 1;

            // Create then block for this arm (where we'll do binding)
            let then_block = self
                .context
                .append_basic_block(self.current_function.unwrap(), &format!("match_arm_{}", i));

            // Create else block (next arm check or merge if last)
            let else_block = if is_last_arm {
                merge_block
            } else {
                self.context.append_basic_block(
                    self.current_function.unwrap(),
                    &format!("match_check_{}", i + 1),
                )
            };

            // For identifier patterns: they always match, but we need to check this in the if-else chain
            // Strategy: For last arm with identifier, use unconditional; for earlier arms, still check
            let is_identifier = matches!(&arm.pattern, Pattern::Ident(_));
            let is_wildcard = matches!(&arm.pattern, Pattern::Wildcard);

            // If this is the last arm and it's identifier/wildcard, no need to check
            if (is_identifier || is_wildcard) && is_last_arm && arm.guard.is_none() {
                // Last arm is catch-all without guard, jump directly
                self.builder
                    .build_unconditional_branch(then_block)
                    .map_err(|e| format!("Failed to branch to match arm: {}", e))?;

                self.builder.position_at_end(then_block);

                // Do pattern binding ONLY (no check needed for last catch-all)
                self.compile_pattern_binding(&arm.pattern, match_value)?;
            } else {
                // For all other patterns, do conditional check WITHOUT binding
                let matches = self.compile_pattern_check(&arm.pattern, match_value)?;

                // Check guard if present
                let final_condition = if let Some(guard) = &arm.guard {
                    // For guards with identifier patterns, we need to bind BEFORE evaluating guard
                    // So we need to branch to then_block, bind, check guard, then conditionally branch
                    if is_identifier {
                        // Jump to then_block unconditionally
                        self.builder
                            .build_unconditional_branch(then_block)
                            .map_err(|e| format!("Failed to branch for guard check: {}", e))?;

                        self.builder.position_at_end(then_block);

                        // Bind the pattern
                        self.compile_pattern_binding(&arm.pattern, match_value)?;

                        // Evaluate guard
                        let guard_val = self.compile_expression(guard)?;
                        let guard_bool = guard_val.into_int_value();

                        // Create body block
                        let arm_body_block = self.context.append_basic_block(
                            self.current_function.unwrap(),
                            &format!("match_arm_{}_body", i),
                        );

                        // Branch based on guard
                        self.builder
                            .build_conditional_branch(guard_bool, arm_body_block, else_block)
                            .map_err(|e| format!("Failed to build guard branch: {}", e))?;

                        self.builder.position_at_end(arm_body_block);

                        // Skip the normal binding below since we already did it
                        let arm_result = self.compile_expression(&arm.body)?;

                        if result_ptr.is_none() {
                            let inferred_type = arm_result.get_type();
                            result_type = Some(inferred_type);
                            let ptr = self
                                .builder
                                .build_alloca(inferred_type, "match_result")
                                .map_err(|e| {
                                    format!("Failed to create match result variable: {}", e)
                                })?;
                            result_ptr = Some(ptr);
                        }

                        self.builder
                            .build_store(result_ptr.unwrap(), arm_result)
                            .map_err(|e| format!("Failed to store match result: {}", e))?;
                        self.builder
                            .build_unconditional_branch(merge_block)
                            .map_err(|e| format!("Failed to branch to merge: {}", e))?;

                        current_block = else_block;
                        continue; // Skip the normal arm body compilation below
                    } else {
                        let guard_val = self.compile_expression(guard)?;
                        let guard_bool = guard_val.into_int_value();

                        // Combine pattern match with guard: matches && guard
                        self.builder
                            .build_and(matches, guard_bool, "match_and_guard")
                            .map_err(|e| format!("Failed to build guard AND: {}", e))?
                    }
                } else {
                    matches
                };

                self.builder
                    .build_conditional_branch(final_condition, then_block, else_block)
                    .map_err(|e| format!("Failed to build match branch: {}", e))?;

                self.builder.position_at_end(then_block);

                // Now do the binding in then_block
                self.compile_pattern_binding(&arm.pattern, match_value)?;
            }

            let arm_result = self.compile_expression(&arm.body)?;

            // On first arm, infer the result type and create the result variable IN FUNCTION ENTRY
            if result_ptr.is_none() {
                let inferred_type = arm_result.get_type();
                result_type = Some(inferred_type);

                // CRITICAL: Create alloca at the BEGINNING of function entry block
                // This ensures it dominates all uses
                let current_pos = self.builder.get_insert_block().unwrap();
                let func = self.current_function.unwrap();
                let func_entry = func.get_first_basic_block().unwrap();

                // Position at the START of function entry block (before any instructions)
                if let Some(first_instr) = func_entry.get_first_instruction() {
                    self.builder.position_before(&first_instr);
                } else {
                    self.builder.position_at_end(func_entry);
                }

                let ptr = self
                    .builder
                    .build_alloca(inferred_type, "match_result")
                    .map_err(|e| format!("Failed to create match result variable: {}", e))?;
                result_ptr = Some(ptr);

                // Restore position to current arm's then_block
                self.builder.position_at_end(current_pos);
            }

            self.builder
                .build_store(result_ptr.unwrap(), arm_result)
                .map_err(|e| format!("Failed to store match result: {}", e))?;
            self.builder
                .build_unconditional_branch(merge_block)
                .map_err(|e| format!("Failed to branch to merge: {}", e))?;

            current_block = else_block;
            // CRITICAL: Position builder at else_block for next arm's check
            self.builder.position_at_end(else_block);
        }

        // Position at merge block and load result
        self.builder.position_at_end(merge_block);
        self.builder
            .build_load(result_type.unwrap(), result_ptr.unwrap(), "match_result")
            .map_err(|e| format!("Failed to load match result: {}", e))
    }

    /// Check if a pattern matches a value (WITHOUT binding - no side effects)
    fn compile_pattern_check(
        &mut self,
        pattern: &Pattern,
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        match pattern {
            Pattern::Wildcard => {
                // Wildcard always matches
                Ok(self.context.bool_type().const_int(1, false))
            }

            Pattern::Literal(lit_expr) => {
                // Compare value with literal
                let literal_val = self.compile_expression(lit_expr)?;
                self.compile_equality_comparison(value, literal_val)
            }

            Pattern::Ident(name) => {
                // Check if this identifier is actually an enum variant (unit variant)
                // If so, treat it as an enum pattern instead of a binding
                let mut is_enum_variant = false;
                let mut variant_index = 0;

                for (_enum_name, enum_def) in &self.enum_ast_defs {
                    if let Some(idx) = enum_def.variants.iter().position(|v| v.name == *name) {
                        is_enum_variant = true;
                        variant_index = idx;
                        break;
                    }
                }

                if is_enum_variant {
                    // Treat as unit enum variant pattern
                    if !value.is_int_value() {
                        return Ok(self.context.bool_type().const_int(0, false));
                    }

                    let enum_val = value.into_int_value();
                    let expected_tag = self
                        .context
                        .i32_type()
                        .const_int(variant_index as u64, false);
                    let tag_matches = self
                        .builder
                        .build_int_compare(
                            IntPredicate::EQ,
                            enum_val,
                            expected_tag,
                            "enum_tag_check",
                        )
                        .map_err(|e| format!("Failed to compare enum tags: {}", e))?;

                    Ok(tag_matches)
                } else {
                    // Regular identifier pattern always matches (binding will happen later)
                    Ok(self.context.bool_type().const_int(1, false))
                }
            }

            Pattern::Tuple(patterns) => {
                // For tuple patterns: value should now be a struct value (loaded in match expression)
                eprintln!(
                    "[DEBUG TUPLE CHECK] value.is_struct: {}",
                    value.is_struct_value()
                );
                if !value.is_struct_value() {
                    // Not a struct value, pattern doesn't match
                    eprintln!("[DEBUG TUPLE CHECK] Not a struct, returning false");
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                let struct_val = value.into_struct_value();
                let struct_type = struct_val.get_type();

                eprintln!(
                    "[DEBUG TUPLE CHECK] Element count: {}, pattern count: {}",
                    struct_type.count_fields(),
                    patterns.len()
                );

                // Check element count matches
                if struct_type.count_fields() as usize != patterns.len() {
                    eprintln!("[DEBUG TUPLE CHECK] Count mismatch, returning false");
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                // Recursively check each sub-pattern
                let mut combined_result = self.context.bool_type().const_int(1, false); // Start with true

                for (i, sub_pattern) in patterns.iter().enumerate() {
                    // Extract tuple element
                    let element = self
                        .builder
                        .build_extract_value(struct_val, i as u32, &format!("tuple_check_{}", i))
                        .map_err(|e| format!("Failed to extract tuple element for check: {}", e))?;

                    eprintln!("[DEBUG TUPLE CHECK] Element {}: {:?}", i, element);

                    // Recursively check sub-pattern
                    let sub_matches = self.compile_pattern_check(sub_pattern, element)?;

                    eprintln!(
                        "[DEBUG TUPLE CHECK] Sub-pattern {} result: {:?}",
                        i, sub_matches
                    );

                    // Combine with AND: combined_result = combined_result && sub_matches
                    combined_result = self
                        .builder
                        .build_and(combined_result, sub_matches, &format!("tuple_and_{}", i))
                        .map_err(|e| format!("Failed to combine tuple pattern checks: {}", e))?;
                }

                eprintln!("[DEBUG TUPLE CHECK] Combined result: {:?}", combined_result);
                Ok(combined_result)
            }

            Pattern::Struct { name, fields } => {
                // For struct patterns: value should be a struct value
                if !value.is_struct_value() {
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                let struct_val = value.into_struct_value();

                // Get struct definition to map field names to indices
                let struct_def = self
                    .struct_defs
                    .get(name)
                    .ok_or_else(|| format!("Struct '{}' not found", name))?
                    .clone();

                // Check all pattern fields match
                let mut combined_result = self.context.bool_type().const_int(1, false);

                for (field_name, field_pattern) in fields.iter() {
                    // Find field index in struct definition
                    let field_index = struct_def
                        .fields
                        .iter()
                        .position(|(name, _)| name == field_name)
                        .ok_or_else(|| {
                            format!("Field '{}' not found in struct '{}'", field_name, name)
                        })?;

                    // Extract field value
                    let field_val = self
                        .builder
                        .build_extract_value(
                            struct_val,
                            field_index as u32,
                            &format!("struct_field_{}", field_name),
                        )
                        .map_err(|e| format!("Failed to extract struct field: {}", e))?;

                    // Recursively check field pattern
                    let field_matches = self.compile_pattern_check(field_pattern, field_val)?;

                    // Combine with AND
                    combined_result = self
                        .builder
                        .build_and(
                            combined_result,
                            field_matches,
                            &format!("struct_and_{}", field_name),
                        )
                        .map_err(|e| format!("Failed to combine struct pattern checks: {}", e))?;
                }

                Ok(combined_result)
            }

            Pattern::Enum {
                name,
                variant,
                data,
            } => {
                // Enum patterns: check variant tag
                // Unit variants: value is i32 tag
                // Data-carrying variants: value is struct { i32, T }

                // Check if value is struct (data-carrying) or int (unit)
                let is_data_carrying = value.is_struct_value();

                if !is_data_carrying && !value.is_int_value() {
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                let enum_val = if is_data_carrying {
                    // Extract tag from struct { tag, data }
                    let struct_val = value.into_struct_value();
                    self.builder
                        .build_extract_value(struct_val, 0, "enum_tag")
                        .map_err(|e| format!("Failed to extract enum tag: {}", e))?
                        .into_int_value()
                } else {
                    value.into_int_value()
                };

                // Find enum definition and variant index
                let enum_name = if name.is_empty() {
                    // Infer enum name from variant by searching all enums
                    let mut found_enum = None;
                    for (e_name, e_def) in &self.enum_ast_defs {
                        if e_def.variants.iter().any(|v| v.name == *variant) {
                            found_enum = Some(e_name.clone());
                            break;
                        }
                    }
                    found_enum
                        .ok_or_else(|| format!("Cannot find enum for variant '{}'", variant))?
                } else {
                    name.clone()
                };

                let enum_def = self
                    .enum_ast_defs
                    .get(&enum_name)
                    .ok_or_else(|| format!("Enum '{}' not found", enum_name))?;

                let variant_index = enum_def
                    .variants
                    .iter()
                    .position(|v| v.name == *variant)
                    .ok_or_else(|| {
                        format!("Variant '{}' not found in enum '{}'", variant, enum_name)
                    })?;

                // Compare tag value
                let expected_tag = self
                    .context
                    .i32_type()
                    .const_int(variant_index as u64, false);
                let tag_matches = self
                    .builder
                    .build_int_compare(IntPredicate::EQ, enum_val, expected_tag, "enum_tag_check")
                    .map_err(|e| format!("Failed to compare enum tags: {}", e))?;

                // If variant has data, also check inner pattern
                if let Some(inner_pattern) = data {
                    if is_data_carrying {
                        // Extract data from struct { tag, data }
                        let struct_val = value.into_struct_value();
                        let data_val = self
                            .builder
                            .build_extract_value(struct_val, 1, "enum_data")
                            .map_err(|e| format!("Failed to extract enum data: {}", e))?;

                        // Recursively check inner pattern
                        let data_matches = self.compile_pattern_check(inner_pattern, data_val)?;

                        // Combine: tag matches AND data matches
                        let combined = self
                            .builder
                            .build_and(tag_matches, data_matches, "enum_full_match")
                            .map_err(|e| format!("Failed to combine enum pattern checks: {}", e))?;

                        return Ok(combined);
                    } else {
                        // Pattern expects data but value is unit variant
                        return Ok(self.context.bool_type().const_int(0, false));
                    }
                }

                Ok(tag_matches)
            }

            Pattern::Or(patterns) => {
                // Or pattern: 1 | 2 | 3 | 4 | 5
                // Check each pattern and combine with OR
                // This will be vectorized with SIMD in future optimizations

                if patterns.is_empty() {
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                // Check first pattern
                let mut combined_result = self.compile_pattern_check(&patterns[0], value)?;

                // OR with remaining patterns
                for (i, pattern) in patterns.iter().enumerate().skip(1) {
                    let pattern_matches = self.compile_pattern_check(pattern, value)?;
                    combined_result = self
                        .builder
                        .build_or(
                            combined_result,
                            pattern_matches,
                            &format!("or_pattern_{}", i),
                        )
                        .map_err(|e| format!("Failed to combine OR patterns: {}", e))?;
                }

                Ok(combined_result)
            }
        }
    }

    /// Bind pattern variables AFTER pattern has matched
    fn compile_pattern_binding(
        &mut self,
        pattern: &Pattern,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), String> {
        match pattern {
            Pattern::Wildcard => {
                // Wildcard doesn't bind anything
                Ok(())
            }

            Pattern::Literal(_) => {
                // Literals don't bind anything
                Ok(())
            }

            Pattern::Ident(name) => {
                // Bind the value to the identifier
                let value_type = value.get_type();
                let ptr = self
                    .builder
                    .build_alloca(value_type, name)
                    .map_err(|e| format!("Failed to allocate for pattern binding: {}", e))?;
                self.builder
                    .build_store(ptr, value)
                    .map_err(|e| format!("Failed to store pattern binding: {}", e))?;
                self.variables.insert(name.clone(), ptr);
                self.variable_types.insert(name.clone(), value_type);
                Ok(())
            }

            Pattern::Tuple(patterns) => {
                // Destructure tuple and bind each element
                if !value.is_struct_value() {
                    return Err("Expected struct value for tuple pattern".to_string());
                }

                let struct_val = value.into_struct_value();
                for (i, sub_pattern) in patterns.iter().enumerate() {
                    let element = self
                        .builder
                        .build_extract_value(struct_val, i as u32, &format!("tuple_{}", i))
                        .map_err(|e| format!("Failed to extract tuple element: {}", e))?;

                    self.compile_pattern_binding(sub_pattern, element)?;
                }
                Ok(())
            }

            Pattern::Struct { name, fields } => {
                // Destructure struct and bind each field
                if !value.is_struct_value() {
                    return Err("Expected struct value for struct pattern".to_string());
                }

                // Get struct definition to map field names to indices
                let struct_def = self
                    .struct_defs
                    .get(name)
                    .ok_or_else(|| format!("Struct '{}' not found", name))?
                    .clone();

                let struct_val = value.into_struct_value();

                for (field_name, sub_pattern) in fields {
                    // Find the field index
                    let field_idx = struct_def
                        .fields
                        .iter()
                        .position(|(fname, _)| fname == field_name)
                        .ok_or_else(|| {
                            format!("Field '{}' not found in struct '{}'", field_name, name)
                        })?;

                    // Extract the field value
                    let field_value = self
                        .builder
                        .build_extract_value(
                            struct_val,
                            field_idx as u32,
                            &format!("{}_{}", name, field_name),
                        )
                        .map_err(|e| format!("Failed to extract struct field: {}", e))?;

                    // Recursively bind the field pattern
                    self.compile_pattern_binding(sub_pattern, field_value)?;
                }
                Ok(())
            }

            Pattern::Enum {
                name: enum_name,
                variant,
                data,
            } => {
                eprintln!(
                    "🔵 Pattern binding - Enum: {}::{}, has_data={}",
                    enum_name,
                    variant,
                    data.is_some()
                );
                // For unit variants (no data), no binding needed
                // For data-carrying variants, extract and bind the data
                if let Some(inner_pattern) = data {
                    eprintln!("  → Has inner pattern: {:?}", inner_pattern);
                    eprintln!("  → Value is struct: {}", value.is_struct_value());
                    eprintln!("  → Value type: {:?}", value.get_type());
                    // Check if value is struct (data-carrying enum)
                    if value.is_struct_value() {
                        // Extract data from struct { tag, data }
                        let struct_val = value.into_struct_value();
                        let data_val = self
                            .builder
                            .build_extract_value(struct_val, 1, "enum_data_bind")
                            .map_err(|e| {
                                format!("Failed to extract enum data for binding: {}", e)
                            })?;

                        eprintln!("  ✅ Extracted enum data, binding to pattern...");
                        // Recursively bind inner pattern
                        self.compile_pattern_binding(inner_pattern, data_val)?;
                    } else {
                        eprintln!("  ⚠️ Value is not struct, skipping data extraction");
                    }
                }
                Ok(())
            }

            Pattern::Or(patterns) => {
                // Or patterns don't bind variables (they only match)
                // If needed, bind variables from the first matching pattern
                // For now, or patterns are only used for literals (1 | 2 | 3)
                // which don't have bindings
                if patterns.is_empty() {
                    return Ok(());
                }

                // Check if any pattern has bindings (identifiers)
                // For now, we only support Or patterns with literals
                for pattern in patterns {
                    if matches!(pattern, Pattern::Ident(_)) {
                        return Err(
                            "Or patterns with identifier bindings not yet supported".to_string()
                        );
                    }
                }

                Ok(())
            }
        }
    }

    /// Old compile_pattern_match (DEPRECATED - kept for compatibility with other code)
    /// Use compile_pattern_check + compile_pattern_binding instead
    fn compile_pattern_match(
        &mut self,
        pattern: &Pattern,
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        match pattern {
            Pattern::Wildcard => {
                // Wildcard always matches
                Ok(self.context.bool_type().const_int(1, false))
            }

            Pattern::Literal(lit_expr) => {
                // Compare value with literal
                let literal_val = self.compile_expression(lit_expr)?;
                self.compile_equality_comparison(value, literal_val)
            }

            Pattern::Ident(name) => {
                // Bind the value to the identifier
                let value_type = value.get_type();
                let ptr = self
                    .builder
                    .build_alloca(value_type, name)
                    .map_err(|e| format!("Failed to allocate for pattern binding: {}", e))?;
                self.builder
                    .build_store(ptr, value)
                    .map_err(|e| format!("Failed to store pattern binding: {}", e))?;
                self.variables.insert(name.clone(), ptr);
                self.variable_types.insert(name.clone(), value_type);
                // Identifier pattern always matches
                Ok(self.context.bool_type().const_int(1, false))
            }

            Pattern::Tuple(patterns) => {
                // Destructure tuple and match each element
                if !value.is_struct_value() {
                    return Err("Cannot match tuple pattern on non-tuple value".to_string());
                }

                let tuple_val = value.into_struct_value();
                let mut all_match = self.context.bool_type().const_int(1, false);

                for (i, sub_pattern) in patterns.iter().enumerate() {
                    // Extract tuple element
                    let elem_val = self
                        .builder
                        .build_extract_value(tuple_val, i as u32, &format!("tuple_{}", i))
                        .map_err(|e| format!("Failed to extract tuple element: {}", e))?;

                    // Recursively match sub-pattern (this will bind identifiers)
                    let elem_matches = self.compile_pattern_match(sub_pattern, elem_val)?;

                    // AND with accumulated result
                    all_match = self
                        .builder
                        .build_and(all_match, elem_matches, "tuple_match_and")
                        .map_err(|e| format!("Failed to AND tuple matches: {}", e))?;
                }

                // Tuple pattern always matches if all elements match
                Ok(all_match)
            }

            Pattern::Struct { name, fields } => {
                // TODO: Implement struct pattern matching
                // For now, always match and bind fields if variable names provided
                let _ = (name, fields);
                Ok(self.context.bool_type().const_int(1, false))
            }

            Pattern::Enum { .. } => {
                // TODO: Implement enum pattern matching
                Ok(self.context.bool_type().const_int(1, false))
            }

            Pattern::Or(patterns) => {
                // Or pattern in old compile_pattern_match
                // Check each pattern and combine with OR
                if patterns.is_empty() {
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                let mut combined_result = self.compile_pattern_match(&patterns[0], value)?;

                for (i, pattern) in patterns.iter().enumerate().skip(1) {
                    let pattern_matches = self.compile_pattern_match(pattern, value)?;
                    combined_result = self
                        .builder
                        .build_or(combined_result, pattern_matches, &format!("or_match_{}", i))
                        .map_err(|e| format!("Failed to combine OR patterns: {}", e))?;
                }

                Ok(combined_result)
            }
        }
    }

    /// Helper to compile equality comparison between two values
    fn compile_equality_comparison(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => self
                .builder
                .build_int_compare(inkwell::IntPredicate::EQ, l, r, "eq_cmp")
                .map_err(|e| format!("Failed to build int comparison: {}", e)),

            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eq_cmp")
                .map_err(|e| format!("Failed to build float comparison: {}", e)),

            _ => Err("Type mismatch in pattern comparison".to_string()),
        }
    }
}
