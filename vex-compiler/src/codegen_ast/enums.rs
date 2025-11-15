// src/codegen/enums.rs
use super::*;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn generate_enum_constructors(&mut self, enum_def: &Enum) -> Result<(), String> {
        for (tag_index, variant) in enum_def.variants.iter().enumerate() {
            let constructor_name = format!("{}_{}", enum_def.name, variant.name);

            if !variant.data.is_empty() {
                // Multi-value tuple variant: create struct for data
                let data_llvm_types: Vec<BasicTypeEnum> = variant
                    .data
                    .iter()
                    .map(|ty| self.ast_type_to_llvm(ty))
                    .collect();

                let i32_type = self.context.i32_type();

                // For single-value, use direct type; for multi-value, use struct
                let data_type = if data_llvm_types.len() == 1 {
                    data_llvm_types[0]
                } else {
                    BasicTypeEnum::StructType(self.context.struct_type(&data_llvm_types, false))
                };

                let enum_struct_type = self
                    .context
                    .struct_type(&[i32_type.into(), data_type], false);

                let fn_type = enum_struct_type.fn_type(
                    &data_llvm_types
                        .iter()
                        .map(|t| (*t).into())
                        .collect::<Vec<_>>(),
                    false,
                );
                let function = self.module.add_function(&constructor_name, fn_type, None);

                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                // Build data value from parameters
                let data_value = if data_llvm_types.len() == 1 {
                    // Single parameter - use directly
                    function
                        .get_nth_param(0)
                        .ok_or_else(|| "Missing data parameter".to_string())?
                } else {
                    // Multiple parameters - pack into struct
                    let mut tuple_val = self
                        .context
                        .struct_type(&data_llvm_types, false)
                        .get_undef();
                    for (i, _) in data_llvm_types.iter().enumerate() {
                        let param_idx = crate::safe_field_index(i)
                            .map_err(|e| format!("Enum field index overflow: {}", e))?;
                        let param = function
                            .get_nth_param(param_idx)
                            .ok_or_else(|| format!("Missing parameter {}", i))?;
                        tuple_val = self
                            .builder
                            .build_insert_value(tuple_val, param, param_idx, &format!("field_{}", i))
                            .map_err(|e| format!("Failed to insert tuple field: {}", e))?
                            .into_struct_value();
                    }
                    tuple_val.into()
                };

                let undef_struct = enum_struct_type.get_undef();

                let tag_value = i32_type.const_int(tag_index as u64, false);
                let with_tag = self
                    .builder
                    .build_insert_value(undef_struct, tag_value, 0, "with_tag")
                    .map_err(|e| format!("Failed to insert tag: {}", e))?;

                let enum_value = self
                    .builder
                    .build_insert_value(with_tag, data_value, 1, "enum_value")
                    .map_err(|e| format!("Failed to insert data: {}", e))?;

                let enum_basic_value: BasicValueEnum = match enum_value {
                    inkwell::values::AggregateValueEnum::ArrayValue(v) => v.into(),
                    inkwell::values::AggregateValueEnum::StructValue(v) => v.into(),
                };

                self.builder
                    .build_return(Some(&enum_basic_value))
                    .map_err(|e| format!("Failed to build return: {}", e))?;

                self.functions.insert(constructor_name, function);
            } else {
                let i32_type = self.context.i32_type();
                let fn_type = i32_type.fn_type(&[], false);
                let function = self.module.add_function(&constructor_name, fn_type, None);

                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                let tag_value = i32_type.const_int(tag_index as u64, false);
                self.builder
                    .build_return(Some(&tag_value))
                    .map_err(|e| format!("Failed to build return: {}", e))?;

                self.functions.insert(constructor_name, function);
            }
        }
        Ok(())
    }
}
