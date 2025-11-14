// String conversion and formatting utilities for ASTCodeGen
// Handles value-to-string conversion for F-string interpolation

use inkwell::values::{BasicValueEnum, FunctionValue};

impl<'ctx> super::ASTCodeGen<'ctx> {
    /// Convert any value to string using appropriate conversion function
    pub(crate) fn convert_value_to_string(
        &mut self,
        value: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            BasicValueEnum::IntValue(int_val) => {
                let bit_width = int_val.get_type().get_bit_width();
                match bit_width {
                    32 => {
                        // i32 → vex_i32_to_string
                        let to_str_fn = self.get_or_declare_i32_to_string()?;
                        let result = self
                            .builder
                            .build_call(to_str_fn, &[int_val.into()], "i32_to_str")
                            .map_err(|e| format!("Failed to call i32_to_string: {}", e))?;
                        Ok(result.try_as_basic_value().unwrap_basic())
                    }
                    64 => {
                        // i64 → vex_i64_to_string
                        let to_str_fn = self.get_or_declare_i64_to_string()?;
                        let result = self
                            .builder
                            .build_call(to_str_fn, &[int_val.into()], "i64_to_str")
                            .map_err(|e| format!("Failed to call i64_to_string: {}", e))?;
                        Ok(result.try_as_basic_value().unwrap_basic())
                    }
                    1 => {
                        // bool → "true" or "false"
                        let true_str = self
                            .builder
                            .build_global_string_ptr("true", "bool_true")
                            .map_err(|e| format!("Failed to create bool string: {}", e))?;
                        let false_str = self
                            .builder
                            .build_global_string_ptr("false", "bool_false")
                            .map_err(|e| format!("Failed to create bool string: {}", e))?;

                        let result = self
                            .builder
                            .build_select(
                                int_val,
                                true_str.as_pointer_value(),
                                false_str.as_pointer_value(),
                                "bool_to_str",
                            )
                            .map_err(|e| format!("Failed to select bool string: {}", e))?;

                        Ok(result)
                    }
                    _ => Err(format!(
                        "Unsupported integer bit width for string conversion: {}",
                        bit_width
                    )),
                }
            }
            BasicValueEnum::FloatValue(float_val) => {
                let float_type = float_val.get_type();
                if float_type == self.context.f32_type() {
                    // f32 → vex_f32_to_string
                    let to_str_fn = self.get_or_declare_f32_to_string()?;
                    let result = self
                        .builder
                        .build_call(to_str_fn, &[float_val.into()], "f32_to_str")
                        .map_err(|e| format!("Failed to call f32_to_string: {}", e))?;
                    Ok(result.try_as_basic_value().unwrap_basic())
                } else {
                    // f64 → vex_f64_to_string
                    let to_str_fn = self.get_or_declare_f64_to_string()?;
                    let result = self
                        .builder
                        .build_call(to_str_fn, &[float_val.into()], "f64_to_str")
                        .map_err(|e| format!("Failed to call f64_to_string: {}", e))?;
                    Ok(result.try_as_basic_value().unwrap_basic())
                }
            }
            BasicValueEnum::PointerValue(_) => {
                // Already a string pointer - return as-is
                Ok(value)
            }
            _ => Err(format!(
                "Cannot convert {:?} to string in F-string interpolation",
                value
            )),
        }
    }

    /// Get or declare vex_strcat_new function
    pub(crate) fn get_or_declare_vex_strcat_new(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(func) = self.module.get_function("vex_strcat_new") {
            return Ok(func);
        }

        // char* vex_strcat_new(const char* s1, const char* s2)
        let i8_ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);

        Ok(self.module.add_function("vex_strcat_new", fn_type, None))
    }

    /// Get or declare vex_i32_to_string function
    pub(crate) fn get_or_declare_i32_to_string(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(func) = self.module.get_function("vex_i32_to_string") {
            return Ok(func);
        }

        let i8_ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let fn_type = i8_ptr_type.fn_type(&[self.context.i32_type().into()], false);

        Ok(self.module.add_function("vex_i32_to_string", fn_type, None))
    }

    /// Get or declare vex_i64_to_string function
    pub(crate) fn get_or_declare_i64_to_string(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(func) = self.module.get_function("vex_i64_to_string") {
            return Ok(func);
        }

        let i8_ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let fn_type = i8_ptr_type.fn_type(&[self.context.i64_type().into()], false);

        Ok(self.module.add_function("vex_i64_to_string", fn_type, None))
    }

    /// Get or declare vex_f32_to_string function
    pub(crate) fn get_or_declare_f32_to_string(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(func) = self.module.get_function("vex_f32_to_string") {
            return Ok(func);
        }

        let i8_ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let fn_type = i8_ptr_type.fn_type(&[self.context.f32_type().into()], false);

        Ok(self.module.add_function("vex_f32_to_string", fn_type, None))
    }

    /// Get or declare vex_f64_to_string function
    pub(crate) fn get_or_declare_f64_to_string(&self) -> Result<FunctionValue<'ctx>, String> {
        if let Some(func) = self.module.get_function("vex_f64_to_string") {
            return Ok(func);
        }

        let i8_ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let fn_type = i8_ptr_type.fn_type(&[self.context.f64_type().into()], false);

        Ok(self.module.add_function("vex_f64_to_string", fn_type, None))
    }
}
