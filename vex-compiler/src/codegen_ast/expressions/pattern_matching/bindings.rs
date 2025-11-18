//! Pattern matching: variable binding logic
use crate::codegen_ast::ASTCodeGen;
use inkwell::types::{BasicType, BasicTypeEnum};
use inkwell::values::BasicValueEnum;
use vex_ast::Pattern;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Bind pattern variables AFTER pattern has matched
    pub(crate) fn compile_pattern_binding(
        &mut self,
        pattern: &Pattern,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), String> {
        match pattern {
            Pattern::Wildcard | Pattern::Literal(_) => Ok(()), // No bindings
            Pattern::Ident(name) => {
                // Don't bind if it's a unit enum variant
                if self.is_enum_variant(name) {
                    return Ok(());
                }
                self.bind_variable(name, value)
            }
            Pattern::Tuple(patterns) => {
                if !value.is_struct_value() {
                    return Err("Expected struct value for tuple pattern binding".to_string());
                }
                let struct_val = value.into_struct_value();
                for (i, sub_pattern) in patterns.iter().enumerate() {
                    let element = self
                        .builder
                        .build_extract_value(struct_val, i as u32, &format!("tuple_bind_{}", i))
                        .map_err(|e| {
                            format!("Failed to extract tuple element for binding: {}", e)
                        })?;
                    self.compile_pattern_binding(sub_pattern, element)?;
                }
                Ok(())
            }
            Pattern::Struct { name, fields } => {
                let struct_value = self.load_if_pointer(value, name)?;
                if !struct_value.is_struct_value() {
                    return Err("Expected struct value for struct pattern binding".to_string());
                }
                let struct_val = struct_value.into_struct_value();
                let struct_def = self
                    .struct_defs
                    .get(name)
                    .ok_or_else(|| format!("Struct '{}' not found", name))?
                    .clone();

                for (field_name, sub_pattern) in fields {
                    let field_idx = struct_def
                        .fields
                        .iter()
                        .position(|(fname, _)| fname == field_name)
                        .ok_or_else(|| {
                            format!("Field '{}' not found in struct '{}'", field_name, name)
                        })?;
                    let field_value = self
                        .builder
                        .build_extract_value(
                            struct_val,
                            field_idx as u32,
                            &format!("{}_{}", name, field_name),
                        )
                        .map_err(|e| {
                            format!("Failed to extract struct field for binding: {}", e)
                        })?;
                    self.compile_pattern_binding(sub_pattern, field_value)?;
                }
                Ok(())
            }
            Pattern::Enum { variant, data, .. } => {
                if !data.is_empty() {
                    // Value is the full enum struct: { i32 discriminant, T data }
                    let data_val = if value.is_struct_value() {
                        let enum_struct = value.into_struct_value();
                        self.builder
                            .build_extract_value(enum_struct, 1, "enum_data_bind")
                            .map_err(|e| {
                                format!("Failed to extract enum data for binding: {}", e)
                            })?
                    } else {
                        // For pointers or other types, treat as the data value directly
                        value
                    };

                    if data.len() == 1 {
                        self.compile_pattern_binding(&data[0], data_val)?;
                    } else {
                        // Multi-value tuple data
                        if !data_val.is_struct_value() {
                            return Err(format!(
                                "Expected struct for multi-value enum data in variant '{}'",
                                variant
                            ));
                        }
                        let data_struct = data_val.into_struct_value();
                        for (i, pattern) in data.iter().enumerate() {
                            let field_val = self
                                .builder
                                .build_extract_value(
                                    data_struct,
                                    i as u32,
                                    &format!("tuple_field_bind_{}", i),
                                )
                                .map_err(|e| {
                                    format!("Failed to extract tuple field for binding: {}", e)
                                })?;
                            self.compile_pattern_binding(pattern, field_val)?;
                        }
                    }
                }
                Ok(())
            }
            Pattern::Or(patterns) => {
                // Or-patterns cannot introduce bindings. This should be validated by the parser/type-checker.
                // We assume here that if bindings exist, they are consistent across all alternatives.
                // We will bind based on the first pattern that could have bindings.
                for p in patterns {
                    // Attempt to bind, but ignore errors if a branch doesn't match.
                    // This is a simplification. A full implementation requires more complex flow control.
                    let _ = self.compile_pattern_binding(p, value);
                }
                Ok(())
            }
            Pattern::Array { elements, rest } => {
                if !value.is_array_value() {
                    return Err("Cannot bind array pattern to non-array value".to_string());
                }
                let array_val = value.into_array_value();
                for (i, elem_pattern) in elements.iter().enumerate() {
                    let elem_idx = crate::safe_field_index(i)
                        .map_err(|e| format!("Array binding index overflow: {}", e))?;
                    let elem_val = self
                        .builder
                        .build_extract_value(array_val, elem_idx, &format!("array_bind_{}", i))
                        .map_err(|e| {
                            format!("Failed to extract array element for binding: {}", e)
                        })?;
                    self.compile_pattern_binding(elem_pattern, elem_val)?;
                }

                if let Some(rest_name) = rest {
                    if rest_name != "_" {
                        self.bind_rest_of_array(rest_name, array_val, elements.len())?;
                    }
                }
                Ok(())
            }
        }
    }

    /// Binds a value to a variable name in the current scope.
    fn bind_variable(&mut self, name: &str, value: BasicValueEnum<'ctx>) -> Result<(), String> {
        let value_type = value.get_type();
        let ptr = self
            .builder
            .build_alloca(value_type, name)
            .map_err(|e| format!("Failed to allocate for pattern binding: {}", e))?;
        self.builder
            .build_store(ptr, value)
            .map_err(|e| format!("Failed to store pattern binding: {}", e))?;
        self.variables.insert(name.to_string(), ptr);
        self.variable_types.insert(name.to_string(), value_type);
        Ok(())
    }

    /// Loads a struct value if the given value is a pointer to it.
    fn load_if_pointer(
        &self,
        value: BasicValueEnum<'ctx>,
        struct_name: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if value.is_pointer_value() {
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
            self.builder
                .build_load(
                    struct_type,
                    value.into_pointer_value(),
                    "struct_binding_loaded",
                )
                .map_err(|e| format!("Failed to load struct for binding: {}", e))
        } else {
            Ok(value)
        }
    }

    /// Binds the rest of an array to a slice.
    fn bind_rest_of_array(
        &mut self,
        rest_name: &str,
        array_val: inkwell::values::ArrayValue<'ctx>,
        start_index: usize,
    ) -> Result<(), String> {
        let array_len = array_val.get_type().len() as usize;
        let remaining_count = array_len - start_index;

        if remaining_count > 0 {
            let elem_type = array_val.get_type().get_element_type();
            let rest_count_u32 = crate::safe_array_size(remaining_count)
                .map_err(|e| format!("Rest array size overflow: {}", e))?;
            let rest_array_type: BasicTypeEnum = match elem_type {
                BasicTypeEnum::IntType(t) => t.array_type(rest_count_u32).as_basic_type_enum(),
                BasicTypeEnum::FloatType(t) => t.array_type(rest_count_u32).as_basic_type_enum(),
                BasicTypeEnum::PointerType(t) => t.array_type(rest_count_u32).as_basic_type_enum(),
                BasicTypeEnum::StructType(t) => t.array_type(rest_count_u32).as_basic_type_enum(),
                BasicTypeEnum::ArrayType(t) => t.array_type(rest_count_u32).as_basic_type_enum(),
                _ => return Err("Unsupported element type for rest pattern".to_string()),
            };

            let rest_ptr = self
                .builder
                .build_alloca(rest_array_type, &format!("{}_rest", rest_name))
                .map_err(|e| format!("Failed to allocate rest array: {}", e))?;

            for i in 0..remaining_count {
                let src_idx = start_index + i;
                let src_idx_u32 = crate::safe_field_index(src_idx)
                    .map_err(|e| format!("Rest array index overflow: {}", e))?;
                let src_val = self
                    .builder
                    .build_extract_value(array_val, src_idx_u32, &format!("rest_elem_{}", i))
                    .map_err(|e| format!("Failed to extract rest element: {}", e))?;
                let dest_ptr = unsafe {
                    self.builder
                        .build_gep(
                            rest_array_type,
                            rest_ptr,
                            &[
                                self.context.i32_type().const_int(0, false),
                                self.context.i32_type().const_int(i as u64, false),
                            ],
                            &format!("rest_gep_{}", i),
                        )
                        .map_err(|e| format!("Failed to GEP rest array: {}", e))?
                };
                self.builder
                    .build_store(dest_ptr, src_val)
                    .map_err(|e| format!("Failed to store rest element: {}", e))?;
            }

            self.variables.insert(rest_name.to_string(), rest_ptr);
            self.variable_types
                .insert(rest_name.to_string(), rest_array_type);
        }
        Ok(())
    }
}
