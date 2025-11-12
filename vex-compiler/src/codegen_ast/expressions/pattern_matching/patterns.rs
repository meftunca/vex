//! Pattern matching: checking logic
use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::Pattern;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Check if a pattern matches a value (WITHOUT binding - no side effects)
    pub(crate) fn compile_pattern_check(
        &mut self,
        pattern: &Pattern,
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        match pattern {
            Pattern::Wildcard => Ok(self.context.bool_type().const_int(1, false)),
            Pattern::Literal(lit_expr) => {
                let literal_val = self.compile_expression(lit_expr)?;
                self.compile_equality_comparison(value, literal_val)
            }
            Pattern::Ident(name) => {
                if self.is_enum_variant(name) {
                    self.check_unit_enum_variant(name, value)
                } else {
                    // Regular identifier pattern always matches
                    Ok(self.context.bool_type().const_int(1, false))
                }
            }
            Pattern::Tuple(patterns) => self.check_tuple_pattern(patterns, value),
            Pattern::Struct { name, fields } => self.check_struct_pattern(name, fields, value),
            Pattern::Enum { name, variant, data } => {
                self.check_enum_pattern(name, variant, data, value)
            }
            Pattern::Or(patterns) => self.check_or_pattern(patterns, value),
            Pattern::Array { elements, rest } => self.check_array_pattern(elements, rest, value),
        }
    }

    /// Helper for `Pattern::Ident` that could be a unit enum variant.
    fn check_unit_enum_variant(
        &mut self,
        name: &str,
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let (_enum_name, variant_index) = self
            .find_enum_and_variant_index(name)
            .ok_or_else(|| format!("Enum variant '{}' not found", name))?;

        let enum_val = self.extract_enum_tag(value)?;
        let expected_tag = self.context.i32_type().const_int(variant_index as u64, false);

        self.builder
            .build_int_compare(IntPredicate::EQ, enum_val, expected_tag, "enum_tag_check")
            .map_err(|e| format!("Failed to compare enum tags: {}", e))
    }

    /// Helper for `Pattern::Tuple`.
    fn check_tuple_pattern(
        &mut self,
        patterns: &[Pattern],
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        if !value.is_struct_value() {
            return Ok(self.context.bool_type().const_int(0, false));
        }
        let struct_val = value.into_struct_value();
        if struct_val.get_type().count_fields() as usize != patterns.len() {
            return Ok(self.context.bool_type().const_int(0, false));
        }

        let mut combined_result = self.context.bool_type().const_int(1, false);
        for (i, sub_pattern) in patterns.iter().enumerate() {
            let element = self
                .builder
                .build_extract_value(struct_val, i as u32, &format!("tuple_check_{}", i))
                .map_err(|e| format!("Failed to extract tuple element for check: {}", e))?;
            let sub_matches = self.compile_pattern_check(sub_pattern, element)?;
            combined_result = self
                .builder
                .build_and(
                    combined_result,
                    sub_matches,
                    &format!("tuple_and_{}", i),
                )
                .map_err(|e| format!("Failed to combine tuple pattern checks: {}", e))?;
        }
        Ok(combined_result)
    }

    /// Helper for `Pattern::Struct`.
    fn check_struct_pattern(
        &mut self,
        name: &str,
        fields: &[(String, Pattern)],
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let struct_value = if value.is_pointer_value() {
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
            return Ok(self.context.bool_type().const_int(0, false));
        }
        let struct_val = struct_value.into_struct_value();
        let struct_def = self
            .struct_defs
            .get(name)
            .ok_or_else(|| format!("Struct '{}' not found", name))?
            .clone();

        let mut combined_result = self.context.bool_type().const_int(1, false);
        for (field_name, field_pattern) in fields.iter() {
            let field_index = struct_def
                .fields
                .iter()
                .position(|(name, _)| name == field_name)
                .ok_or_else(|| format!("Field '{}' not found in struct '{}'", field_name, name))?;
            let field_val = self
                .builder
                .build_extract_value(
                    struct_val,
                    field_index as u32,
                    &format!("struct_field_{}", field_name),
                )
                .map_err(|e| format!("Failed to extract struct field: {}", e))?;
            let field_matches = self.compile_pattern_check(field_pattern, field_val)?;
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

    /// Helper for `Pattern::Enum`.
    fn check_enum_pattern(
        &mut self,
        name: &str,
        variant: &str,
        data: &[Pattern],
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let (_enum_name, variant_index) =
            self.find_enum_and_variant_index_by_name(name, variant)?;
        let enum_tag = self.extract_enum_tag(value)?;
        let expected_tag = self.context.i32_type().const_int(variant_index as u64, false);
        let tag_matches = self
            .builder
            .build_int_compare(IntPredicate::EQ, enum_tag, expected_tag, "enum_tag_check")
            .map_err(|e| format!("Failed to compare enum tags: {}", e))?;

        if data.is_empty() {
            return Ok(tag_matches);
        }

        if !value.is_struct_value() {
            return Ok(self.context.bool_type().const_int(0, false));
        }

        let struct_val = value.into_struct_value();
        let data_val = self
            .builder
            .build_extract_value(struct_val, 1, "enum_data")
            .map_err(|e| format!("Failed to extract enum data: {}", e))?;

        let data_matches = if data.len() == 1 {
            self.compile_pattern_check(&data[0], data_val)?
        } else {
            self.check_tuple_pattern(data, data_val)?
        };

        self.builder
            .build_and(tag_matches, data_matches, "enum_full_match")
            .map_err(|e| format!("Failed to combine enum pattern checks: {}", e))
    }

    /// Helper for `Pattern::Or`.
    fn check_or_pattern(
        &mut self,
        patterns: &[Pattern],
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        if patterns.is_empty() {
            return Ok(self.context.bool_type().const_int(0, false));
        }
        let mut combined_result = self.compile_pattern_check(&patterns[0], value)?;
        for pattern in patterns.iter().skip(1) {
            let pattern_matches = self.compile_pattern_check(pattern, value)?;
            combined_result = self
                .builder
                .build_or(combined_result, pattern_matches, "or_pattern")
                .map_err(|e| format!("Failed to combine OR patterns: {}", e))?;
        }
        Ok(combined_result)
    }

    /// Helper for `Pattern::Array`.
    fn check_array_pattern(
        &mut self,
        elements: &[Pattern],
        rest: &Option<String>,
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        if !value.is_array_value() {
            return Ok(self.context.bool_type().const_int(0, false));
        }
        let array_val = value.into_array_value();
        let array_len = array_val.get_type().len() as usize;

        if (rest.is_none() && array_len != elements.len())
            || (rest.is_some() && array_len < elements.len())
        {
            return Ok(self.context.bool_type().const_int(0, false));
        }

        let mut all_match = self.context.bool_type().const_int(1, false);
        for (i, elem_pattern) in elements.iter().enumerate() {
            let elem_val = self
                .builder
                .build_extract_value(array_val, i as u32, &format!("array_elem_{}", i))
                .map_err(|e| format!("Failed to extract array element {}: {}", i, e))?;
            let elem_matches = self.compile_pattern_check(elem_pattern, elem_val)?;
            all_match = self
                .builder
                .build_and(all_match, elem_matches, &format!("array_and_{}", i))
                .map_err(|e| format!("Failed to AND array element checks: {}", e))?;
        }
        Ok(all_match)
    }

    /// Extracts the i32 tag from an enum value, which can be an int or a struct.
    pub(crate) fn extract_enum_tag(
        &self,
        value: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        let tag_val = if value.is_struct_value() {
            self.builder
                .build_extract_value(value.into_struct_value(), 0, "enum_tag")
                .map_err(|e| format!("Failed to extract enum tag: {}", e))?
                .into_int_value()
        } else if value.is_int_value() {
            value.into_int_value()
        } else {
            return Err("Enum pattern expected an integer or struct value".to_string());
        };

        if tag_val.get_type().get_bit_width() != 32 {
            self.builder
                .build_int_z_extend(tag_val, self.context.i32_type(), "enum_tag_i32")
                .map_err(|e| format!("Failed to cast enum tag to i32: {}", e))
        } else {
            Ok(tag_val)
        }
    }

    /// Finds the enum definition and variant index for a given variant name.
    pub(crate) fn find_enum_and_variant_index(
        &self,
        variant_name: &str,
    ) -> Option<(String, usize)> {
        if variant_name == "Some" || variant_name == "None" {
            return Some((
                "Option".to_string(),
                if variant_name == "Some" { 0 } else { 1 },
            ));
        }
        if variant_name == "Ok" || variant_name == "Err" {
            return Some(("Result".to_string(), if variant_name == "Ok" { 0 } else { 1 }));
        }
        for (e_name, e_def) in &self.enum_ast_defs {
            if let Some(idx) = e_def.variants.iter().position(|v| v.name == variant_name) {
                return Some((e_name.clone(), idx));
            }
        }
        None
    }

    /// Finds enum and variant index, but with an explicit enum name.
    pub(crate) fn find_enum_and_variant_index_by_name(
        &self,
        enum_name: &str,
        variant_name: &str,
    ) -> Result<(String, usize), String> {
        let e_name = if enum_name.is_empty() {
            self.find_enum_and_variant_index(variant_name)
                .ok_or_else(|| format!("Cannot find enum for variant '{}'", variant_name))?
                .0
        } else {
            enum_name.to_string()
        };

        let is_builtin = e_name == "Option" || e_name == "Result";
        let variant_index = if is_builtin {
            match (e_name.as_str(), variant_name) {
                ("Option", "Some") => 0,
                ("Option", "None") => 1,
                ("Result", "Ok") => 0,
                ("Result", "Err") => 1,
                _ => {
                    return Err(format!(
                        "Unknown variant '{}' for builtin enum '{}'",
                        variant_name, e_name
                    ))
                }
            }
        } else {
            let enum_def = self
                .enum_ast_defs
                .get(&e_name)
                .ok_or_else(|| format!("Enum '{}' not found", e_name))?;
            enum_def
                .variants
                .iter()
                .position(|v| v.name == variant_name)
                .ok_or_else(|| {
                    format!("Variant '{}' not found in enum '{}'", variant_name, e_name)
                })?
        };

        Ok((e_name, variant_index))
    }

    /// Helper to compile equality comparison between two values
    pub(crate) fn compile_equality_comparison(
        &mut self,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<inkwell::values::IntValue<'ctx>, String> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                let l_width = l.get_type().get_bit_width();
                let r_width = r.get_type().get_bit_width();
                let (left_val, right_val) = if l_width > r_width {
                    (
                        l,
                        self.builder
                            .build_int_s_extend(r, l.get_type(), "pattern_cast")
                            .map_err(|e| e.to_string())?,
                    )
                } else if r_width > l_width {
                    (
                        self.builder
                            .build_int_s_extend(l, r.get_type(), "pattern_cast")
                            .map_err(|e| e.to_string())?,
                        r,
                    )
                } else {
                    (l, r)
                };
                self.builder
                    .build_int_compare(IntPredicate::EQ, left_val, right_val, "eq_cmp")
                    .map_err(|e| e.to_string())
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => self
                .builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "eq_cmp")
                .map_err(|e| e.to_string()),
            _ => Err(format!(
                "Type mismatch in pattern comparison: {:?} vs {:?}",
                left.get_type(),
                right.get_type()
            )),
        }
    }
}

