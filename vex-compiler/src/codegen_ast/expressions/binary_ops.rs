// Binary operations (arithmetic, comparison, logical)

use super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::{FloatPredicate, IntPredicate};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operation
    pub(crate) fn compile_binary_op(
        &mut self,
        left: &Expression,
        op: &BinaryOp,
        right: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let lhs = self.compile_expression(left)?;
        let rhs = self.compile_expression(right)?;

        match (lhs, rhs) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                let result = match op {
                    BinaryOp::Add => self.builder.build_int_add(l, r, "add"),
                    BinaryOp::Sub => self.builder.build_int_sub(l, r, "sub"),
                    BinaryOp::Mul => self.builder.build_int_mul(l, r, "mul"),
                    BinaryOp::Div => self.builder.build_int_signed_div(l, r, "div"),
                    BinaryOp::Mod => self.builder.build_int_signed_rem(l, r, "mod"),
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
                }
                .map_err(|e| format!("Failed to build operation: {}", e))?;
                Ok(result.into())
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                let result = match op {
                    BinaryOp::Add => self.builder.build_float_add(l, r, "fadd"),
                    BinaryOp::Sub => self.builder.build_float_sub(l, r, "fsub"),
                    BinaryOp::Mul => self.builder.build_float_mul(l, r, "fmul"),
                    BinaryOp::Div => self.builder.build_float_div(l, r, "fdiv"),
                    BinaryOp::Mod => self.builder.build_float_rem(l, r, "fmod"),
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
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }
}
