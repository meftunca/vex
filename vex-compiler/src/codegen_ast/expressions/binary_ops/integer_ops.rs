//! Integer binary operations
//!
//! Handles arithmetic, comparison, bitwise, and power operations for integer types

use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operations for integer operands with expected type
    pub(crate) fn compile_integer_binary_op_with_expected(
        &mut self,
        l: inkwell::values::IntValue<'ctx>,
        r: inkwell::values::IntValue<'ctx>,
        op: &BinaryOp,
        expected_type: Option<&Type>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Determine target bit width from expected type
        let target_bit_width = if let Some(ty) = expected_type {
            match ty {
                Type::I8 | Type::U8 => 8,
                Type::I16 | Type::U16 => 16,
                Type::I32 | Type::U32 => 32,
                Type::I64 | Type::U64 => 64,
                Type::I128 | Type::U128 => 128,
                _ => l.get_type().get_bit_width(),
            }
        } else {
            l.get_type().get_bit_width()
        };

        // Call the original implementation but with target bit width for overflow checks
        self.compile_integer_binary_op_internal(l, r, op, target_bit_width)
    }

    /// Compile binary operations for integer operands (backward compat)
    pub(crate) fn compile_integer_binary_op(
        &mut self,
        l: inkwell::values::IntValue<'ctx>,
        r: inkwell::values::IntValue<'ctx>,
        op: &BinaryOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let target_bit_width = l.get_type().get_bit_width();
        self.compile_integer_binary_op_internal(l, r, op, target_bit_width)
    }

    /// Internal implementation with explicit target bit width
    fn compile_integer_binary_op_internal(
        &mut self,
        l: inkwell::values::IntValue<'ctx>,
        r: inkwell::values::IntValue<'ctx>,
        op: &BinaryOp,
        target_bit_width: u32,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Check if we are in a constant context (both operands are constant)
        let is_const_context = l.is_const() && r.is_const();

        // If operands have different bit widths, extend the smaller one
        let (l_final, r_final) = if l.get_type().get_bit_width() != r.get_type().get_bit_width() {
            if l.get_type().get_bit_width() < r.get_type().get_bit_width() {
                // Extend left to match right
                let l_ext = if is_const_context {
                    // Try to get constant value and create new constant with target type
                    if let Some(val) = l.get_sign_extended_constant() {
                        r.get_type().const_int(val as u64, true)
                    } else {
                        return Err(
                            "Cannot cast complex constant expression (only literals supported)"
                                .to_string(),
                        );
                    }
                } else {
                    self.builder
                        .build_int_s_extend(l, r.get_type(), "sext_l")
                        .map_err(|e| format!("Failed to extend operand: {}", e))?
                };
                (l_ext, r)
            } else {
                // Extend right to match left
                let r_ext = if is_const_context {
                    // Try to get constant value and create new constant with target type
                    if let Some(val) = r.get_sign_extended_constant() {
                        l.get_type().const_int(val as u64, true)
                    } else {
                        return Err(
                            "Cannot cast complex constant expression (only literals supported)"
                                .to_string(),
                        );
                    }
                } else {
                    self.builder
                        .build_int_s_extend(r, l.get_type(), "sext_r")
                        .map_err(|e| format!("Failed to extend operand: {}", e))?
                };
                (l, r_ext)
            }
        } else {
            (l, r)
        };

        let l = l_final;
        let r = r_final;

        // If in constant context, use constant folding
        if is_const_context {
            let result = match op {
                BinaryOp::Add => {
                    if let (Some(l_val), Some(r_val)) = (
                        l.get_sign_extended_constant(),
                        r.get_sign_extended_constant(),
                    ) {
                        let (res, overflow) = l_val.overflowing_add(r_val);
                        if overflow {
                            return Err("Integer overflow in constant addition".to_string());
                        }

                        // Check if result fits in the target bit width
                        let bit_width = l.get_type().get_bit_width();
                        if bit_width < 64 {
                            let min = -(1i64 << (bit_width - 1));
                            let max = (1i64 << (bit_width - 1)) - 1;
                            if res < min || res > max {
                                return Err(format!("Integer overflow in constant addition: value {} does not fit in {} bits", res, bit_width));
                            }
                        }

                        l.get_type().const_int(res as u64, true)
                    } else {
                        l.const_add(r)
                    }
                }
                BinaryOp::Sub => {
                    if let (Some(l_val), Some(r_val)) = (
                        l.get_sign_extended_constant(),
                        r.get_sign_extended_constant(),
                    ) {
                        let (res, overflow) = l_val.overflowing_sub(r_val);
                        if overflow {
                            return Err("Integer overflow in constant subtraction".to_string());
                        }

                        // ⭐ FIX: Use target_bit_width instead of l.get_type().get_bit_width()
                        if target_bit_width < 64 {
                            let min = -(1i64 << (target_bit_width - 1));
                            let max = (1i64 << (target_bit_width - 1)) - 1;
                            if res < min || res > max {
                                return Err(format!("Integer overflow in constant subtraction: value {} does not fit in {} bits", res, target_bit_width));
                            }
                        }

                        l.get_type().const_int(res as u64, true)
                    } else {
                        l.const_sub(r)
                    }
                }
                BinaryOp::Mul => {
                    // Manual constant folding for Mul to avoid linker errors with LLVMConstMul
                    if let (Some(l_val), Some(r_val)) = (
                        l.get_sign_extended_constant(),
                        r.get_sign_extended_constant(),
                    ) {
                        let (res, overflow) = l_val.overflowing_mul(r_val);
                        if overflow {
                            return Err("Integer overflow in constant multiplication".to_string());
                        }

                        // ⭐ FIX: Use target_bit_width instead of l.get_type().get_bit_width()
                        if target_bit_width < 64 {
                            let min = -(1i64 << (target_bit_width - 1));
                            let max = (1i64 << (target_bit_width - 1)) - 1;
                            if res < min || res > max {
                                return Err(format!("Integer overflow in constant multiplication: value {} does not fit in {} bits", res, target_bit_width));
                            }
                        }

                        l.get_type().const_int(res as u64, true)
                    } else {
                        return Err("Constant multiplication requires literal operands".to_string());
                    }
                }
                BinaryOp::Div => {
                    if let (Some(l_val), Some(r_val)) = (
                        l.get_sign_extended_constant(),
                        r.get_sign_extended_constant(),
                    ) {
                        if r_val == 0 {
                            return Err("Division by zero in constant expression".to_string());
                        }
                        let res = l_val.wrapping_div(r_val);
                        l.get_type().const_int(res as u64, true)
                    } else {
                        return Err("Constant division requires literal operands".to_string());
                    }
                }
                _ => {
                    return Err(format!(
                        "Operator {:?} not supported in constant expressions yet",
                        op
                    ))
                }
            };
            return Ok(result.into());
        }

        let result = match op {
            BinaryOp::Add => self.builder.build_int_add(l, r, "add"),
            BinaryOp::Sub => self.builder.build_int_sub(l, r, "sub"),
            BinaryOp::Mul => self.builder.build_int_mul(l, r, "mul"),
            BinaryOp::Div => self.builder.build_int_signed_div(l, r, "div"),
            BinaryOp::Mod => self.builder.build_int_signed_rem(l, r, "mod"),
            BinaryOp::Pow => {
                // Integer power: loop-based exponentiation
                return self.compile_int_power(l, r);
            }
            BinaryOp::Eq => {
                return Ok(self
                    .builder
                    .build_int_compare(IntPredicate::EQ, l, r, "eq")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::NotEq => {
                return Ok(self
                    .builder
                    .build_int_compare(IntPredicate::NE, l, r, "ne")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::Lt => {
                return Ok(self
                    .builder
                    .build_int_compare(IntPredicate::SLT, l, r, "lt")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::LtEq => {
                return Ok(self
                    .builder
                    .build_int_compare(IntPredicate::SLE, l, r, "le")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::Gt => {
                return Ok(self
                    .builder
                    .build_int_compare(IntPredicate::SGT, l, r, "gt")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::GtEq => {
                return Ok(self
                    .builder
                    .build_int_compare(IntPredicate::SGE, l, r, "ge")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::And => self.builder.build_and(l, r, "and"),
            BinaryOp::Or => self.builder.build_or(l, r, "or"),
            BinaryOp::BitAnd => self.builder.build_and(l, r, "bitand"),
            BinaryOp::BitOr => self.builder.build_or(l, r, "bitor"),
            BinaryOp::BitXor => self.builder.build_xor(l, r, "bitxor"),
            BinaryOp::Shl => self.builder.build_left_shift(l, r, "shl"),
            BinaryOp::Shr => self.builder.build_right_shift(l, r, true, "shr"),
            BinaryOp::Range | BinaryOp::RangeInclusive => {
                return Err("Range operators not yet implemented".to_string());
            }
            BinaryOp::NullCoalesce => {
                return Err("Null coalesce operator not yet implemented".to_string());
            }
        }
        .map_err(|e| format!("Failed to build operation: {}", e))?;
        Ok(result.into())
    }
}
