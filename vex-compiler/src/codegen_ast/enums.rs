// src/codegen/enums.rs
use super::*;
use vex_ast::*;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn generate_enum_constructors(&mut self, enum_def: &Enum) -> Result<(), String> {
        for (tag_index, variant) in enum_def.variants.iter().enumerate() {
            let constructor_name = format!("{}_{}", enum_def.name, variant.name);

            if let Some(ref data_type) = variant.data {
                let data_llvm_type = self.ast_type_to_llvm(data_type);

                let i32_type = self.context.i32_type();
                let enum_struct_type = self
                    .context
                    .struct_type(&[i32_type.into(), data_llvm_type], false);

                let fn_type = enum_struct_type.fn_type(&[data_llvm_type.into()], false);
                let function = self.module.add_function(&constructor_name, fn_type, None);

                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                let data_param = function
                    .get_nth_param(0)
                    .ok_or_else(|| "Missing data parameter".to_string())?;

                let undef_struct = enum_struct_type.get_undef();

                let tag_value = i32_type.const_int(tag_index as u64, false);
                let with_tag = self
                    .builder
                    .build_insert_value(undef_struct, tag_value, 0, "with_tag")
                    .map_err(|e| format!("Failed to insert tag: {}", e))?;

                let enum_value = self
                    .builder
                    .build_insert_value(with_tag, data_param, 1, "enum_value")
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
