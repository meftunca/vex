// Expression compilation - structs and enums
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile struct literal expressions
    pub(crate) fn compile_struct_dispatch(
        &mut self,
        name: &str,
        type_args: &[vex_ast::Type],
        fields: &[(String, vex_ast::Expression)],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_struct_literal(name, type_args, fields)
    }

    /// Compile enum literal expressions
    pub(crate) fn compile_enum_dispatch(
        &mut self,
        enum_name: &str,
        variant: &str,
        data: &[vex_ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Phase 0.4: Handle builtin enums (Option, Result) specially
        if enum_name == "Option" || enum_name == "Result" {
            return self.compile_builtin_enum_literal(enum_name, variant, &data.to_vec());
        }

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
                .ok_or_else(|| format!("Variant {} not found in enum {}", variant, enum_name))?;

            // Check if enum has ANY data-carrying variants
            let enum_has_data = enum_def.variants.iter().any(|v| !v.data.is_empty());

            if data.is_empty() && !enum_has_data {
                // Pure unit enum (all variants are unit): return tag as i32 for compatibility
                // (Variables expect i32, return statements expect i32)
                let tag = self
                    .context
                    .i32_type()
                    .const_int(variant_index as u64, false);
                Ok(tag.into())
            } else {
                // Mixed enum (has data variants): create struct { tag: i32, data: T }
                // For unit variants in mixed enums, use enum's data type from definition

                let (data_value, actual_data_type) = if !data.is_empty() {
                    // Variant has data: compile all expressions
                    if data.len() == 1 {
                        // Single-value tuple: compile directly
                        let val = self.compile_expression(&data[0])?;
                        let ty = val.get_type();
                        (val, ty)
                    } else {
                        // Multi-value tuple: create struct with all values
                        let mut field_values = Vec::new();
                        let mut field_types = Vec::new();

                        for expr in data {
                            let val = self.compile_expression(expr)?;
                            let ty = val.get_type();
                            field_values.push(val);
                            field_types.push(ty);
                        }

                        // Create tuple struct type
                        let tuple_struct_type = self.context.struct_type(&field_types, false);

                        // Build tuple value
                        let mut tuple_val = tuple_struct_type.get_undef();
                        for (i, val) in field_values.iter().enumerate() {
                            tuple_val = self
                                .builder
                                .build_insert_value(
                                    tuple_val,
                                    *val,
                                    i as u32,
                                    &format!("tuple_field_{}", i),
                                )
                                .map_err(|e| format!("Failed to insert tuple field: {}", e))?
                                .into_struct_value();
                        }

                        (tuple_val.into(), tuple_struct_type.into())
                    }
                } else {
                    // Unit variant in mixed enum: use enum's largest data type with zero value
                    let enum_llvm_type =
                        self.ast_type_to_llvm(&vex_ast::Type::Named(enum_name.to_string()));
                    if let inkwell::types::BasicTypeEnum::StructType(struct_ty) = enum_llvm_type {
                        // Extract data field type (index 1)
                        let data_field_type = struct_ty
                            .get_field_type_at_index(1)
                            .ok_or_else(|| "Enum struct missing data field".to_string())?;
                        // Create zero/undef value for data field
                        let zero_value = match data_field_type {
                            inkwell::types::BasicTypeEnum::IntType(int_ty) => {
                                int_ty.const_zero().into()
                            }
                            inkwell::types::BasicTypeEnum::FloatType(float_ty) => {
                                float_ty.const_zero().into()
                            }
                            inkwell::types::BasicTypeEnum::PointerType(ptr_ty) => {
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
                        return Err(format!("Expected struct type for mixed enum {}", enum_name));
                    }
                };

                // Create struct type: { i32, T } (consistent with ast_type_to_llvm)
                let tag_type = self.context.i32_type();
                let struct_type = self
                    .context
                    .struct_type(&[tag_type.into(), actual_data_type], false);

                // Allocate struct on stack
                let struct_ptr = self
                    .builder
                    .build_alloca(struct_type, "enum_data_carrier")
                    .map_err(|e| format!("Failed to allocate enum struct: {}", e))?;

                // Store tag at index 0 (i32 type - consistent with struct definition)
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
            Err(format!("Enum {} not found in definitions", enum_name))
        }
    }
}
