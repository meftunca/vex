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
            (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
                // String comparison using vex_strcmp
                match op {
                    BinaryOp::Eq | BinaryOp::NotEq => {
                        // Declare vex_strcmp if not already declared
                        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                        let strcmp_fn = self.declare_runtime_fn(
                            "vex_strcmp",
                            &[ptr_type.into(), ptr_type.into()],
                            self.context.i32_type().into(),
                        );

                        // Call vex_strcmp(left, right)
                        let cmp_result = self
                            .builder
                            .build_call(strcmp_fn, &[l.into(), r.into()], "strcmp_result")
                            .map_err(|e| format!("Failed to call vex_strcmp: {}", e))?;

                        let cmp_value = cmp_result
                            .try_as_basic_value()
                            .left()
                            .ok_or("vex_strcmp didn't return a value")?
                            .into_int_value();

                        // vex_strcmp returns 0 if equal
                        let zero = self.context.i32_type().const_int(0, false);
                        let result = if matches!(op, BinaryOp::Eq) {
                            self.builder.build_int_compare(
                                IntPredicate::EQ,
                                cmp_value,
                                zero,
                                "streq",
                            )
                        } else {
                            self.builder.build_int_compare(
                                IntPredicate::NE,
                                cmp_value,
                                zero,
                                "strne",
                            )
                        }
                        .map_err(|e| format!("Failed to compare strcmp result: {}", e))?;

                        Ok(result.into())
                    }
                    _ => Err("Only == and != are supported for string comparison".to_string()),
                }
            }
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }
}
