// Expression compilation - literals (numbers, strings, booleans, nil)
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile literal expressions (numbers, strings, booleans, nil)
    pub(crate) fn compile_literal(
        &mut self,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            vex_ast::Expression::IntLiteral(n) => {
                Ok(self.context.i32_type().const_int(*n as u64, false).into())
            }

            vex_ast::Expression::BigIntLiteral(s) => {
                // Parse large integer literals for i128/u128
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
