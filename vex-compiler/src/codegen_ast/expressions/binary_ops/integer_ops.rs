//! Integer binary operations
//!
//! Handles arithmetic, comparison, bitwise, and power operations for integer types

use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operations for integer operands
    pub(crate) fn compile_integer_binary_op(
        &mut self,
        l: inkwell::values::IntValue<'ctx>,
        r: inkwell::values::IntValue<'ctx>,
        op: &BinaryOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // If operands have different bit widths, extend the smaller one
        let (l_final, r_final) =
            if l.get_type().get_bit_width() != r.get_type().get_bit_width() {
                if l.get_type().get_bit_width() < r.get_type().get_bit_width() {
                    // Extend left to match right
                    let l_ext = self
                        .builder
                        .build_int_s_extend(l, r.get_type(), "sext_l")
                        .map_err(|e| format!("Failed to extend operand: {}", e))?;
                    (l_ext, r)
                } else {
                    // Extend right to match left
                    let r_ext = self
                        .builder
                        .build_int_s_extend(r, l.get_type(), "sext_r")
                        .map_err(|e| format!("Failed to extend operand: {}", e))?;
                    (l, r_ext)
                }
            } else {
                (l, r)
            };

        let l = l_final;
        let r = r_final;
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