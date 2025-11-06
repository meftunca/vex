// Match expression and pattern matching code generation

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile match expression as a series of if-else comparisons
    pub(crate) fn compile_match_expression(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if arms.is_empty() {
            return Err("Match expression must have at least one arm".to_string());
        }

        // Debug: print patterns
        eprintln!("🔵 Match arms:");
        for (i, arm) in arms.iter().enumerate() {
            eprintln!("  Arm {}: pattern={:?}", i, arm.pattern);
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
        } else if matches!(value, Expression::EnumLiteral { .. }) && match_value.is_struct_value() {
            // Enum literals: Already struct values, no need to load
            // match_value is already correct
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
            // Only load if it's a pointer (data enum stored on stack)
            // Unit enums are already i32 values, no load needed
            if match_value.is_pointer_value() {
                if let Some(enum_name) = self.variable_enum_names.get(var_name) {
                    eprintln!("  → Found enum: {}, loading from pointer", enum_name);
                    // Use the stored type from variable_types
                    if let Some(enum_type) = self.variable_types.get(var_name) {
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
                    }
                }
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
    pub(crate) fn compile_pattern_check(
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
                    eprintln!(
                        "🔵 Pattern::Ident as enum variant: {}, variant_index={}",
                        name, variant_index
                    );
                    if !value.is_int_value() {
                        return Ok(self.context.bool_type().const_int(0, false));
                    }

                    let enum_val = value.into_int_value();
                    let expected_tag = self
                        .context
                        .i32_type()
                        .const_int(variant_index as u64, false);
                    eprintln!(
                        "🔵 Pattern::Ident check - expected_tag type: {:?}, enum_val type: {:?}",
                        expected_tag.get_type(),
                        enum_val.get_type()
                    );
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
                eprintln!(
                    "🔵 Pattern::Enum check called - enum={}, variant={}, has_data={}",
                    name,
                    variant,
                    data.is_some()
                );
                // Enum patterns: check variant tag
                // Unit variants: value is i8 tag
                // Data-carrying variants: value is struct { i8, T }

                // Check if value is struct (data-carrying) or int (unit)
                let is_data_carrying = value.is_struct_value();

                if !is_data_carrying && !value.is_int_value() {
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                let enum_val = if is_data_carrying {
                    // Extract tag from struct { i8, data }
                    let struct_val = value.into_struct_value();
                    let tag_val = self
                        .builder
                        .build_extract_value(struct_val, 0, "enum_tag")
                        .map_err(|e| format!("Failed to extract enum tag: {}", e))?
                        .into_int_value();

                    // Tag is i32 (unified type for all enums)
                    if tag_val.get_type().get_bit_width() != 32 {
                        self.builder
                            .build_int_z_extend(tag_val, self.context.i32_type(), "enum_tag_i32")
                            .map_err(|e| format!("Failed to cast enum tag to i32: {}", e))?
                    } else {
                        tag_val
                    }
                } else {
                    let int_val = value.into_int_value();
                    // Cast to i32 if needed
                    if int_val.get_type().get_bit_width() != 32 {
                        self.builder
                            .build_int_z_extend(int_val, self.context.i32_type(), "enum_tag_i32")
                            .map_err(|e| format!("Failed to cast enum tag to i32: {}", e))?
                    } else {
                        int_val
                    }
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

                // Compare tag value - use i32 for tag type (unified with unit enum literals)
                let expected_tag = self
                    .context
                    .i32_type()
                    .const_int(variant_index as u64, false);
                eprintln!(
                    "🔵 Pattern::Enum check - expected_tag type: {:?}, enum_val type: {:?}",
                    expected_tag.get_type(),
                    enum_val.get_type()
                );
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
    pub(crate) fn compile_pattern_binding(
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

    /// Helper to compile equality comparison between two values
    pub(crate) fn compile_equality_comparison(
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
