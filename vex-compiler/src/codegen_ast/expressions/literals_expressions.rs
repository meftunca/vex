// Expression compilation - literals (numbers, strings, booleans, nil)
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile literal expressions (numbers, strings, booleans, nil)
    /// Optional expected_type for target-typed inference
    pub(crate) fn compile_literal(
        &mut self,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_literal_with_type(expr, None)
    }

    /// Compile literal with optional expected type for context-dependent inference
    pub(crate) fn compile_literal_with_type(
        &mut self,
        expr: &vex_ast::Expression,
        expected_type: Option<&vex_ast::Type>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            vex_ast::Expression::IntLiteral(n) => {
                // Target-typed: infer from expected type if available
                if let Some(expected) = expected_type {
                    // Validate that literal value fits in target type
                    // Note: For negative literals like -128, the unary negation is handled separately
                    // This validates the positive part (128), and negation applies after
                    let value = *n;
                    match expected {
                        vex_ast::Type::I8 => {
                            // Allow both positive values in range and values that become valid after negation
                            if value > 127 && value != 128 {
                                return Err(format!(
                                    "integer literal {} is out of range for type i8 (range: -128 to 127)",
                                    value
                                ));
                            }
                            return Ok(self.context.i8_type().const_int(value as u64, true).into());
                        }
                        vex_ast::Type::I16 => {
                            if value > 32767 && value != 32768 {
                                return Err(format!(
                                    "integer literal {} is out of range for type i16 (range: -32768 to 32767)",
                                    value
                                ));
                            }
                            return Ok(self.context.i16_type().const_int(value as u64, true).into());
                        }
                        vex_ast::Type::I32 => {
                            return Ok(self.context.i32_type().const_int(value as u64, true).into())
                        }
                        vex_ast::Type::I64 => {
                            return Ok(self.context.i64_type().const_int(value as u64, true).into())
                        }
                        vex_ast::Type::I128 => {
                            return Ok(self
                                .context
                                .i128_type()
                                .const_int(value as u64, true)
                                .into())
                        }
                        vex_ast::Type::U8 => {
                            if value < 0 || value > 255 {
                                return Err(format!(
                                    "integer literal {} is out of range for type u8 (range: 0 to 255)",
                                    value
                                ));
                            }
                            return Ok(self.context.i8_type().const_int(value as u64, false).into());
                        }
                        vex_ast::Type::U16 => {
                            if value < 0 || value > 65535 {
                                return Err(format!(
                                    "integer literal {} is out of range for type u16 (range: 0 to 65535)",
                                    value
                                ));
                            }
                            return Ok(self
                                .context
                                .i16_type()
                                .const_int(value as u64, false)
                                .into());
                        }
                        vex_ast::Type::U32 => {
                            if value < 0 {
                                return Err(format!(
                                    "integer literal {} is out of range for type u32 (range: 0 to 4294967295)",
                                    value
                                ));
                            }
                            return Ok(self
                                .context
                                .i32_type()
                                .const_int(value as u64, false)
                                .into());
                        }
                        vex_ast::Type::U64 => {
                            if value < 0 {
                                return Err(format!(
                                    "integer literal {} is out of range for type u64 (range: 0 to 18446744073709551615)",
                                    value
                                ));
                            }
                            return Ok(self
                                .context
                                .i64_type()
                                .const_int(value as u64, false)
                                .into());
                        }
                        vex_ast::Type::U128 => {
                            if value < 0 {
                                return Err(format!(
                                    "integer literal {} is out of range for type u128 (must be non-negative)",
                                    value
                                ));
                            }
                            return Ok(self
                                .context
                                .i128_type()
                                .const_int(value as u64, false)
                                .into());
                        }
                        _ => {} // Fall through to default i32
                    }
                }
                // Default: i32
                Ok(self.context.i32_type().const_int(*n as u64, false).into())
            }

            vex_ast::Expression::TypedIntLiteral { value, type_suffix } => {
                // Compile typed integer literal with explicit type suffix
                match type_suffix.as_str() {
                    "i8" => Ok(self.context.i8_type().const_int(*value as u64, true).into()),
                    "i16" => Ok(self
                        .context
                        .i16_type()
                        .const_int(*value as u64, true)
                        .into()),
                    "i32" => Ok(self
                        .context
                        .i32_type()
                        .const_int(*value as u64, true)
                        .into()),
                    "i64" => Ok(self
                        .context
                        .i64_type()
                        .const_int(*value as u64, true)
                        .into()),
                    "u8" => Ok(self
                        .context
                        .i8_type()
                        .const_int(*value as u64, false)
                        .into()),
                    "u16" => Ok(self
                        .context
                        .i16_type()
                        .const_int(*value as u64, false)
                        .into()),
                    "u32" => Ok(self
                        .context
                        .i32_type()
                        .const_int(*value as u64, false)
                        .into()),
                    "u64" => Ok(self
                        .context
                        .i64_type()
                        .const_int(*value as u64, false)
                        .into()),
                    _ => Err(format!("Unsupported type suffix: {}", type_suffix)),
                }
            }

            vex_ast::Expression::TypedBigIntLiteral { value, type_suffix } => {
                // Compile big integer with explicit type suffix (i128/u128)
                match type_suffix.as_str() {
                    "i128" => {
                        let radix = if value.starts_with("0x") || value.starts_with("0X") {
                            inkwell::types::StringRadix::Hexadecimal
                        } else if value.starts_with("0b") || value.starts_with("0B") {
                            inkwell::types::StringRadix::Binary
                        } else if value.starts_with("0o") || value.starts_with("0O") {
                            inkwell::types::StringRadix::Octal
                        } else {
                            inkwell::types::StringRadix::Decimal
                        };
                        let num_str = if radix != inkwell::types::StringRadix::Decimal {
                            &value[2..]
                        } else {
                            value.as_str()
                        };
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(num_str, radix)
                            .unwrap()
                            .into())
                    }
                    "u128" => {
                        let radix = if value.starts_with("0x") || value.starts_with("0X") {
                            inkwell::types::StringRadix::Hexadecimal
                        } else if value.starts_with("0b") || value.starts_with("0B") {
                            inkwell::types::StringRadix::Binary
                        } else if value.starts_with("0o") || value.starts_with("0O") {
                            inkwell::types::StringRadix::Octal
                        } else {
                            inkwell::types::StringRadix::Decimal
                        };
                        let num_str = if radix != inkwell::types::StringRadix::Decimal {
                            &value[2..]
                        } else {
                            value.as_str()
                        };
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(num_str, radix)
                            .unwrap()
                            .into())
                    }
                    _ => Err(format!(
                        "Unsupported big integer type suffix: {}",
                        type_suffix
                    )),
                }
            }

            vex_ast::Expression::BigIntLiteral(s) => {
                // Handle large integer literals that don't fit in i64
                // These can be target-typed to u64 or i128/u128
                if let Some(expected) = expected_type {
                    match expected {
                        vex_ast::Type::U64 => {
                            // Parse as u64 for large unsigned values
                            if let Ok(value) = s.parse::<u64>() {
                                return Ok(self.context.i64_type().const_int(value, false).into());
                            }
                        }
                        vex_ast::Type::I128 | vex_ast::Type::U128 => {
                            // Parse as i128/u128
                            // Fall through to default i128 handling below
                        }
                        _ => {}
                    }
                }

                // Default: i128
                // Parse the bigint string
                // Remove any prefix (0x, 0b, 0o) and parse accordingly
                if s.starts_with("0x") || s.starts_with("0X") {
                    // Hexadecimal
                    let hex_str = &s[2..];
                    if u128::from_str_radix(hex_str, 16).is_ok() {
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(
                                hex_str,
                                inkwell::types::StringRadix::Hexadecimal,
                            )
                            .unwrap()
                            .into())
                    } else {
                        Err(format!("Invalid hexadecimal BigIntLiteral: {}", s))
                    }
                } else if s.starts_with("0b") || s.starts_with("0B") {
                    // Binary
                    let bin_str = &s[2..];
                    if u128::from_str_radix(bin_str, 2).is_ok() {
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(bin_str, inkwell::types::StringRadix::Binary)
                            .unwrap()
                            .into())
                    } else {
                        Err(format!("Invalid binary BigIntLiteral: {}", s))
                    }
                } else if s.starts_with("0o") || s.starts_with("0O") {
                    // Octal
                    let oct_str = &s[2..];
                    if u128::from_str_radix(oct_str, 8).is_ok() {
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(oct_str, inkwell::types::StringRadix::Octal)
                            .unwrap()
                            .into())
                    } else {
                        Err(format!("Invalid octal BigIntLiteral: {}", s))
                    }
                } else {
                    // Decimal
                    if s.parse::<u128>().is_ok() {
                        Ok(self
                            .context
                            .i128_type()
                            .const_int_from_string(s, inkwell::types::StringRadix::Decimal)
                            .unwrap()
                            .into())
                    } else {
                        Err(format!("Invalid decimal BigIntLiteral: {}", s))
                    }
                }
            }

            vex_ast::Expression::FloatLiteral(f) => {
                // Target-typed: compile to f32 if expected type is f32
                if let Some(expected) = expected_type {
                    match expected {
                        vex_ast::Type::F32 => {
                            return Ok(self.context.f32_type().const_float(*f).into())
                        }
                        vex_ast::Type::F64 => {
                            return Ok(self.context.f64_type().const_float(*f).into())
                        }
                        _ => {} // Fall through to default f64
                    }
                }
                // Default: f64
                Ok(self.context.f64_type().const_float(*f).into())
            }

            vex_ast::Expression::BoolLiteral(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }

            vex_ast::Expression::StringLiteral(s) => {
                // Create global string constant
                let global_str = self
                    .builder
                    .build_global_string_ptr(s, "str")
                    .map_err(|e| format!("Failed to create string: {}", e))?;
                Ok(global_str.as_pointer_value().into())
            }

            vex_ast::Expression::FStringLiteral(s) => {
                // For now, handle F-strings as formatted strings with interpolation
                self.compile_fstring(s)
            }

            vex_ast::Expression::Nil => {
                // Return zero/null for nil
                Ok(self.context.i8_type().const_int(0, false).into())
            }

            _ => Err(format!("Not a literal expression: {:?}", expr)),
        }
    }

    /// Compile array literal expressions
    pub(crate) fn compile_array_dispatch(
        &mut self,
        elements: &[vex_ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_array_literal(elements)
    }

    /// Compile array repeat literal expressions
    pub(crate) fn compile_array_repeat_dispatch(
        &mut self,
        value: &vex_ast::Expression,
        count: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_array_repeat_literal(value, count)
    }

    /// Compile map literal expressions
    pub(crate) fn compile_map_dispatch(
        &mut self,
        entries: &[(vex_ast::Expression, vex_ast::Expression)],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_map_literal(entries)
    }

    /// Compile tuple literal expressions
    pub(crate) fn compile_tuple_dispatch(
        &mut self,
        elements: &[vex_ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_tuple_literal(elements)
    }
}
