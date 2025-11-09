// Match expression and pattern matching code generation

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
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
        eprintln!("🟣 Match expression type: {:?}", value);

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
                eprintln!(
                    "  🔍 Found struct variable: {} -> {}",
                    var_name, struct_name
                );
                eprintln!("  🔍 Is pointer: {}", match_value.is_pointer_value());
                // This variable holds a struct value, load it for pattern matching
                if match_value.is_pointer_value() {
                    eprintln!("  → Loading struct from pointer...");
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
                    eprintln!(
                        "  ✅ Loaded! Now is_struct: {}",
                        match_value.is_struct_value()
                    );
                }
            } else {
                eprintln!(
                    "  ⚠️ Struct variable '{}' NOT found in variable_struct_names!",
                    var_name
                );
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
                eprintln!("🔵 compile_pattern_check: Struct pattern '{}'", name);
                // For struct patterns: value should be a struct value
                // If it's a pointer, load it first
                let struct_value = if value.is_pointer_value() {
                    eprintln!("🔵 Struct pattern: value is pointer, loading...");
                    // Get struct definition
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

                    self.builder
                        .build_load(
                            struct_type,
                            value.into_pointer_value(),
                            "struct_pattern_loaded",
                        )
                        .map_err(|e| format!("Failed to load struct for pattern: {}", e))?
                } else {
                    value
                };

                if !struct_value.is_struct_value() {
                    eprintln!("  ⚠️ After load, still not struct value!");
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                eprintln!("  ✅ Struct value ready, extracting fields...");
                let struct_val = struct_value.into_struct_value();

                // Get struct definition to map field names to indices
                let struct_def = self
                    .struct_defs
                    .get(name)
                    .ok_or_else(|| format!("Struct '{}' not found", name))?
                    .clone();

                // Check all pattern fields match
                let mut combined_result = self.context.bool_type().const_int(1, false);

                for (field_name, field_pattern) in fields.iter() {
                    eprintln!(
                        "    → Checking field '{}', pattern={:?}",
                        field_name, field_pattern
                    );
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

                    eprintln!(
                        "    → Field extracted: index={}, value_type={:?}",
                        field_index,
                        field_val.get_type()
                    );

                    // Recursively check field pattern
                    let field_matches = self.compile_pattern_check(field_pattern, field_val)?;

                    eprintln!("    → Field match result: {:?}", field_matches);

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
                    !data.is_empty()
                );
                // Enum patterns: check variant tag
                // Unit variants: value is i8 tag
                // Data-carrying variants: value is struct { i8, T } or { i8, struct{T1,T2,...} }

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

                    // First check builtins
                    if *variant == "Some" || *variant == "None" {
                        found_enum = Some("Option".to_string());
                    } else if *variant == "Ok" || *variant == "Err" {
                        found_enum = Some("Result".to_string());
                    } else {
                        // Search user-defined enums
                        for (e_name, e_def) in &self.enum_ast_defs {
                            if e_def.variants.iter().any(|v| v.name == *variant) {
                                found_enum = Some(e_name.clone());
                                break;
                            }
                        }
                    }

                    found_enum
                        .ok_or_else(|| format!("Cannot find enum for variant '{}'", variant))?
                } else {
                    name.clone()
                };

                // Handle builtin enums (Result, Option) - they don't have AST definitions
                let is_builtin_enum = enum_name == "Result" || enum_name == "Option";

                let variant_index = if is_builtin_enum {
                    // Builtin enums have fixed variant indices
                    if enum_name == "Option" {
                        if *variant == "Some" {
                            0
                        } else {
                            1
                        } // None
                    } else {
                        if *variant == "Ok" {
                            0
                        } else {
                            1
                        } // Err
                    }
                } else {
                    let enum_def = self
                        .enum_ast_defs
                        .get(&enum_name)
                        .ok_or_else(|| format!("Enum '{}' not found", enum_name))?;

                    enum_def
                        .variants
                        .iter()
                        .position(|v| v.name == *variant)
                        .ok_or_else(|| {
                            format!("Variant '{}' not found in enum '{}'", variant, enum_name)
                        })?
                };

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

                // If variant has data patterns, check them
                if !data.is_empty() {
                    if is_data_carrying {
                        // Extract data from struct { tag, data }
                        let struct_val = value.into_struct_value();
                        let data_val = self
                            .builder
                            .build_extract_value(struct_val, 1, "enum_data")
                            .map_err(|e| format!("Failed to extract enum data: {}", e))?;

                        // For multi-value tuple: data is struct, need to extract each field
                        // For single-value: data is the value itself
                        if data.len() == 1 {
                            // Single-value tuple - check directly
                            let data_matches = self.compile_pattern_check(&data[0], data_val)?;

                            // Combine: tag matches AND data matches
                            let combined = self
                                .builder
                                .build_and(tag_matches, data_matches, "enum_full_match")
                                .map_err(|e| {
                                    format!("Failed to combine enum pattern checks: {}", e)
                                })?;

                            return Ok(combined);
                        } else {
                            // Multi-value tuple - extract each field from data struct
                            let mut combined_match = tag_matches;

                            // Data is a struct with multiple fields
                            let data_struct = data_val.into_struct_value();
                            for (i, pattern) in data.iter().enumerate() {
                                let field_val = self
                                    .builder
                                    .build_extract_value(
                                        data_struct,
                                        i as u32,
                                        &format!("field_{}", i),
                                    )
                                    .map_err(|e| {
                                        format!("Failed to extract tuple field {}: {}", i, e)
                                    })?;

                                let field_matches =
                                    self.compile_pattern_check(pattern, field_val)?;

                                combined_match = self
                                    .builder
                                    .build_and(
                                        combined_match,
                                        field_matches,
                                        &format!("tuple_match_{}", i),
                                    )
                                    .map_err(|e| {
                                        format!("Failed to combine tuple field match: {}", e)
                                    })?;
                            }

                            return Ok(combined_match);
                        }
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

            Pattern::Array { elements, rest } => {
                // Array/Slice pattern: [a, b, ..rest] or [x, y, z]
                // Extract elements from array value and check patterns

                eprintln!(
                    "🔵 Pattern::Array check - elements={}, rest={:?}",
                    elements.len(),
                    rest
                );

                // Value should be an array or vector
                if !value.is_array_value() && !value.is_struct_value() {
                    eprintln!("  ⚠️ Value is not an array, pattern fails");
                    return Ok(self.context.bool_type().const_int(0, false));
                }

                // Start with true - all patterns must match
                let mut all_match = self.context.bool_type().const_int(1, false);

                if value.is_array_value() {
                    let array_val = value.into_array_value();
                    let array_type = array_val.get_type();
                    let array_len = array_type.len() as usize;

                    eprintln!(
                        "  → Array length={}, pattern elements={}",
                        array_len,
                        elements.len()
                    );

                    // If no rest pattern, length must match exactly
                    if rest.is_none() && array_len != elements.len() {
                        eprintln!("  ⚠️ Length mismatch, pattern fails");
                        return Ok(self.context.bool_type().const_int(0, false));
                    }

                    // If rest pattern exists, we need at least as many elements as patterns
                    if rest.is_some() && array_len < elements.len() {
                        eprintln!("  ⚠️ Not enough elements for pattern, fails");
                        return Ok(self.context.bool_type().const_int(0, false));
                    }

                    // Check each element pattern
                    for (i, elem_pattern) in elements.iter().enumerate() {
                        // Extract array element using build_extract_value
                        let elem_val = self
                            .builder
                            .build_extract_value(array_val, i as u32, &format!("array_elem_{}", i))
                            .map_err(|e| format!("Failed to extract array element {}: {}", i, e))?;

                        eprintln!("  → Checking element {} against pattern", i);

                        // Recursively check sub-pattern
                        let elem_matches = self.compile_pattern_check(elem_pattern, elem_val)?;

                        all_match = self
                            .builder
                            .build_and(all_match, elem_matches, &format!("array_and_{}", i))
                            .map_err(|e| format!("Failed to AND array element checks: {}", e))?;
                    }
                } else if value.is_struct_value() {
                    // Vector/dynamic array represented as struct
                    // For now, treat as array-like
                    let struct_val = value.into_struct_value();
                    let struct_type = struct_val.get_type();
                    let field_count = struct_type.count_fields() as usize;

                    eprintln!(
                        "  → Struct/vector field_count={}, pattern elements={}",
                        field_count,
                        elements.len()
                    );

                    // If no rest pattern, must have exact number of fields
                    if rest.is_none() && field_count != elements.len() {
                        return Ok(self.context.bool_type().const_int(0, false));
                    }

                    // Check each element pattern
                    for (i, elem_pattern) in elements.iter().enumerate() {
                        if i >= field_count {
                            break;
                        }

                        let elem_val = self
                            .builder
                            .build_extract_value(struct_val, i as u32, &format!("vec_elem_{}", i))
                            .map_err(|e| {
                                format!("Failed to extract vector element {}: {}", i, e)
                            })?;

                        let elem_matches = self.compile_pattern_check(elem_pattern, elem_val)?;

                        all_match = self
                            .builder
                            .build_and(all_match, elem_matches, &format!("vec_and_{}", i))
                            .map_err(|e| format!("Failed to AND vector element checks: {}", e))?;
                    }
                }

                Ok(all_match)
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
                eprintln!("🟢 compile_pattern_binding: Struct pattern '{}'", name);
                // Destructure struct and bind each field
                // If it's a pointer, load it first
                let struct_value = if value.is_pointer_value() {
                    eprintln!("🔵 Struct binding: value is pointer, loading...");
                    // Get struct definition
                    let struct_def_temp = self
                        .struct_defs
                        .get(name)
                        .ok_or_else(|| format!("Struct '{}' not found", name))?
                        .clone();

                    let field_types: Vec<_> = struct_def_temp
                        .fields
                        .iter()
                        .map(|(_, ty)| self.ast_type_to_llvm(ty))
                        .collect();
                    let struct_type = self.context.struct_type(&field_types, false);

                    self.builder
                        .build_load(
                            struct_type,
                            value.into_pointer_value(),
                            "struct_binding_loaded",
                        )
                        .map_err(|e| format!("Failed to load struct for binding: {}", e))?
                } else {
                    eprintln!("🔵 Struct binding: value is already loaded");
                    value
                };

                if !struct_value.is_struct_value() {
                    eprintln!("  ⚠️ After load, binding still not struct value!");
                    return Err("Expected struct value for struct pattern".to_string());
                }

                eprintln!(
                    "  ✅ Struct binding ready, extracting {} fields",
                    fields.len()
                );
                // Get struct definition to map field names to indices
                let struct_def = self
                    .struct_defs
                    .get(name)
                    .ok_or_else(|| format!("Struct '{}' not found", name))?
                    .clone();

                let struct_val = struct_value.into_struct_value();

                for (field_name, sub_pattern) in fields {
                    eprintln!(
                        "    → Binding field '{}', pattern={:?}",
                        field_name, sub_pattern
                    );
                    // Find the field index
                    let field_idx = struct_def
                        .fields
                        .iter()
                        .position(|(fname, _)| fname == field_name)
                        .ok_or_else(|| {
                            format!("Field '{}' not found in struct '{}'", field_name, name)
                        })?;

                    eprintln!("    → Field index: {}", field_idx);

                    // Extract the field value
                    let field_value = self
                        .builder
                        .build_extract_value(
                            struct_val,
                            field_idx as u32,
                            &format!("{}_{}", name, field_name),
                        )
                        .map_err(|e| format!("Failed to extract struct field: {}", e))?;

                    eprintln!(
                        "    → Field value extracted, type={:?}",
                        field_value.get_type()
                    );

                    // Recursively bind the field pattern
                    eprintln!("    → Calling compile_pattern_binding recursively...");
                    self.compile_pattern_binding(sub_pattern, field_value)?;
                    eprintln!("    → Field binding complete");
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
                    !data.is_empty()
                );
                // For unit variants (no data), no binding needed
                // For data-carrying variants, extract and bind the data
                if !data.is_empty() {
                    eprintln!("  → Has {} data patterns", data.len());
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

                        // For single-value tuple: bind directly
                        // For multi-value tuple: extract each field from struct
                        if data.len() == 1 {
                            self.compile_pattern_binding(&data[0], data_val)?;
                        } else {
                            // Multi-value tuple - extract each field from data struct
                            let data_struct = data_val.into_struct_value();
                            for (i, pattern) in data.iter().enumerate() {
                                let field_val = self
                                    .builder
                                    .build_extract_value(
                                        data_struct,
                                        i as u32,
                                        &format!("tuple_field_{}", i),
                                    )
                                    .map_err(|e| {
                                        format!("Failed to extract tuple field {}: {}", i, e)
                                    })?;

                                self.compile_pattern_binding(pattern, field_val)?;
                            }
                        }
                    } else {
                        eprintln!("  ⚠️ Value is not struct, cannot extract data for bindings");
                        // For builtin enums that return int instead of struct, we need special handling
                        // Bind the value directly for now (single value case)
                        if data.len() == 1 {
                            self.compile_pattern_binding(&data[0], value)?;
                        }
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
                    if let Pattern::Ident(_) = pattern {
                        return Err("Or patterns with variable bindings not yet supported. Use separate match arms instead.".to_string());
                    }
                }

                // No bindings needed for literals
                Ok(())
            }

            Pattern::Array { elements, rest } => {
                // Array/Slice pattern binding: [a, b, ..rest]
                // Bind each element and optionally the rest slice
                eprintln!(
                    "🔵 Pattern::Array binding - elements={}, rest={:?}",
                    elements.len(),
                    rest
                );

                // Extract array elements and bind them
                if value.is_array_value() {
                    let array_val = value.into_array_value();

                    for (i, elem_pattern) in elements.iter().enumerate() {
                        // Extract array element
                        let elem_val = self
                            .builder
                            .build_extract_value(array_val, i as u32, &format!("array_bind_{}", i))
                            .map_err(|e| {
                                format!("Failed to extract array element for binding: {}", e)
                            })?;

                        eprintln!("  → Binding array element {} to pattern", i);

                        // Recursively bind sub-pattern
                        self.compile_pattern_binding(elem_pattern, elem_val)?;
                    }

                    // If there's a rest pattern, bind the remaining slice
                    if let Some(rest_name) = rest {
                        if rest_name != "_" {
                            eprintln!(
                                "  → Rest pattern '{}' - creating slice from remaining elements",
                                rest_name
                            );

                            // Create a slice/array with remaining elements
                            let array_len = array_val.get_type().len() as usize;
                            let remaining_count = array_len - elements.len();

                            if remaining_count > 0 {
                                // Allocate array for rest elements
                                let elem_type = array_val.get_type().get_element_type();

                                // Convert BasicTypeEnum to concrete type for array creation
                                let rest_array_type: BasicTypeEnum = match elem_type {
                                    BasicTypeEnum::IntType(t) => {
                                        t.array_type(remaining_count as u32).into()
                                    }
                                    BasicTypeEnum::FloatType(t) => {
                                        t.array_type(remaining_count as u32).into()
                                    }
                                    BasicTypeEnum::PointerType(t) => {
                                        t.array_type(remaining_count as u32).into()
                                    }
                                    BasicTypeEnum::StructType(t) => {
                                        t.array_type(remaining_count as u32).into()
                                    }
                                    BasicTypeEnum::ArrayType(t) => {
                                        t.array_type(remaining_count as u32).into()
                                    }
                                    _ => {
                                        return Err(
                                            "Unsupported element type for rest pattern".to_string()
                                        )
                                    }
                                };

                                let rest_ptr = self
                                    .builder
                                    .build_alloca(rest_array_type, &format!("{}_rest", rest_name))
                                    .map_err(|e| format!("Failed to allocate rest array: {}", e))?;

                                // Copy remaining elements
                                for i in 0..remaining_count {
                                    let src_idx = elements.len() + i;
                                    let elem_val = self
                                        .builder
                                        .build_extract_value(
                                            array_val,
                                            src_idx as u32,
                                            &format!("rest_elem_{}", i),
                                        )
                                        .map_err(|e| {
                                            format!("Failed to extract rest element: {}", e)
                                        })?;

                                    let dest_ptr = unsafe {
                                        self.builder
                                            .build_gep(
                                                rest_array_type,
                                                rest_ptr,
                                                &[
                                                    self.context.i32_type().const_int(0, false),
                                                    self.context
                                                        .i32_type()
                                                        .const_int(i as u64, false),
                                                ],
                                                &format!("rest_gep_{}", i),
                                            )
                                            .map_err(|e| {
                                                format!("Failed to GEP rest array: {}", e)
                                            })?
                                    };

                                    self.builder.build_store(dest_ptr, elem_val).map_err(|e| {
                                        format!("Failed to store rest element: {}", e)
                                    })?;
                                }

                                // Bind rest array to variable
                                self.variables.insert(rest_name.clone(), rest_ptr);
                                self.variable_types
                                    .insert(rest_name.clone(), rest_array_type.into());

                                eprintln!(
                                    "  ✅ Bound rest pattern '{}' with {} elements",
                                    rest_name, remaining_count
                                );
                            } else {
                                eprintln!("  ⚠️ No remaining elements for rest pattern");
                            }
                        }
                    }
                } else if value.is_struct_value() {
                    // Vector/dynamic array as struct
                    let struct_val = value.into_struct_value();

                    for (i, elem_pattern) in elements.iter().enumerate() {
                        let elem_val = self
                            .builder
                            .build_extract_value(struct_val, i as u32, &format!("vec_bind_{}", i))
                            .map_err(|e| {
                                format!("Failed to extract vector element for binding: {}", e)
                            })?;

                        self.compile_pattern_binding(elem_pattern, elem_val)?;
                    }

                    if let Some(rest_name) = rest {
                        if rest_name != "_" {
                            eprintln!(
                                "  → Rest pattern '{}' for vector (not yet implemented)",
                                rest_name
                            );
                            // TODO: Vector slice support
                        }
                    }
                } else {
                    return Err(format!(
                        "Cannot bind array pattern to non-array value: {:?}",
                        value.get_type()
                    ));
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
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                // Handle integer width mismatch by casting to wider type
                let l_width = l.get_type().get_bit_width();
                let r_width = r.get_type().get_bit_width();

                let (left_val, right_val) = if l_width != r_width {
                    if l_width > r_width {
                        // Cast right to left's type
                        let r_cast = self
                            .builder
                            .build_int_s_extend(r, l.get_type(), "pattern_cast")
                            .map_err(|e| format!("Failed to cast pattern literal: {}", e))?;
                        (l, r_cast)
                    } else {
                        // Cast left to right's type
                        let l_cast = self
                            .builder
                            .build_int_s_extend(l, r.get_type(), "pattern_cast")
                            .map_err(|e| format!("Failed to cast pattern value: {}", e))?;
                        (l_cast, r)
                    }
                } else {
                    (l, r)
                };

                self.builder
                    .build_int_compare(inkwell::IntPredicate::EQ, left_val, right_val, "eq_cmp")
                    .map_err(|e| format!("Failed to build int comparison: {}", e))
            }

            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eq_cmp")
                .map_err(|e| format!("Failed to build float comparison: {}", e)),

            _ => Err("Type mismatch in pattern comparison".to_string()),
        }
    }
}
