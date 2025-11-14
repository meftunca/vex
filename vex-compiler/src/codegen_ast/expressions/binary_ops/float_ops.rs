//! Float binary operations
//!
//! Handles arithmetic, comparison, and power operations for float types

use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::FloatPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operations for float operands
    pub(crate) fn compile_float_binary_op(
        &mut self,
        l: inkwell::values::FloatValue<'ctx>,
        r: inkwell::values::FloatValue<'ctx>,
        op: &BinaryOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let result = match op {
            BinaryOp::Add => self.builder.build_float_add(l, r, "fadd"),
            BinaryOp::Sub => self.builder.build_float_sub(l, r, "fsub"),
            BinaryOp::Mul => self.builder.build_float_mul(l, r, "fmul"),
            BinaryOp::Div => self.builder.build_float_div(l, r, "fdiv"),
            BinaryOp::Mod => self.builder.build_float_rem(l, r, "fmod"),
            BinaryOp::Pow => {
                // Float power: call llvm.pow intrinsic
                return self.compile_float_power(l, r);
            }
            BinaryOp::Range | BinaryOp::RangeInclusive | BinaryOp::NullCoalesce => {
                return Err("Range/NullCoalesce operators not implemented for floats".to_string());
            }
            BinaryOp::Eq => {
                return Ok(self
                    .builder
                    .build_float_compare(FloatPredicate::OEQ, l, r, "feq")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::NotEq => {
                return Ok(self
                    .builder
                    .build_float_compare(FloatPredicate::ONE, l, r, "fne")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::Lt => {
                return Ok(self
                    .builder
                    .build_float_compare(FloatPredicate::OLT, l, r, "flt")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::LtEq => {
                return Ok(self
                    .builder
                    .build_float_compare(FloatPredicate::OLE, l, r, "fle")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::Gt => {
                return Ok(self
                    .builder
                    .build_float_compare(FloatPredicate::OGT, l, r, "fgt")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            BinaryOp::GtEq => {
                return Ok(self
                    .builder
                    .build_float_compare(FloatPredicate::OGE, l, r, "fge")
                    .map_err(|e| format!("Failed to compare: {}", e))?
                    .into())
            }
            _ => return Err("Invalid float operation".to_string()),
        }
        .map_err(|e| format!("Failed to build operation: {}", e))?;
        Ok(result.into())
    }
}
