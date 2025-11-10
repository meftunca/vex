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
        // ⭐ NEW: Operator Overloading - Check if left operand has operator trait
        if let Ok(left_type) = self.infer_expression_type(left) {
            // ⭐ BUILTIN: Check for Vec + Vec (if both are Vec)
            if let Type::Generic { ref name, .. } = left_type {
                if name == "Vec" && matches!(op, BinaryOp::Add) {
                    if let Ok(right_type) = self.infer_expression_type(right) {
                        if let Type::Generic {
                            name: right_name, ..
                        } = right_type
                        {
                            if right_name == "Vec" {
                                let left_val = self.compile_expression(left)?;
                                let right_val = self.compile_expression(right)?;

                                let concat_fn = self.get_vex_vec_concat();
                                let result = self
                                    .builder
                                    .build_call(
                                        concat_fn,
                                        &[left_val.into(), right_val.into()],
                                        "vec_concat",
                                    )
                                    .map_err(|e| format!("Failed to call vex_vec_concat: {}", e))?;

                                return result
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or("vex_vec_concat didn't return a value".to_string());
                            }
                        }
                    }
                }
            }

            if let Type::Named(type_name) = left_type {
                let (trait_name, method_name) = self.binary_op_to_trait(op);
                if !trait_name.is_empty() {
                    if let Some(_) = self.has_operator_trait(&type_name, trait_name) {
                        // Desugar to method call: left.add(right)
                        eprintln!("🎯 Operator overloading: {}.{}()", type_name, method_name);
                        return self.compile_method_call(
                            left,
                            method_name,
                            &[], // No generic type args for operator overloading
                            &vec![right.clone()],
                            false,
                        );
                    }
                }
            }
        }

        // Fallback: Builtin operations (int, float, pointer)
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
                    BinaryOp::BitAnd => self.builder.build_and(l, r, "bitand"),
                    BinaryOp::BitOr => self.builder.build_or(l, r, "bitor"),
                    BinaryOp::BitXor => self.builder.build_xor(l, r, "bitxor"),
                    BinaryOp::Shl => self.builder.build_left_shift(l, r, "shl"),
                    BinaryOp::Shr => self.builder.build_right_shift(l, r, true, "shr"),
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
                match op {
                    // String concatenation: s1 + s2 → vex_strcat_new
                    BinaryOp::Add => {
                        eprintln!("🔗 String concatenation: calling vex_strcat_new");

                        // Declare vex_strcat_new if not already declared
                        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                        let strcat_fn = self.declare_runtime_fn(
                            "vex_strcat_new",
                            &[ptr_type.into(), ptr_type.into()],
                            ptr_type.into(),
                        );

                        // Call vex_strcat_new(left, right) → returns new string
                        let concat_result = self
                            .builder
                            .build_call(strcat_fn, &[l.into(), r.into()], "strcat_result")
                            .map_err(|e| format!("Failed to call vex_strcat_new: {}", e))?;

                        let result_ptr = concat_result
                            .try_as_basic_value()
                            .left()
                            .ok_or("vex_strcat_new didn't return a value")?;

                        Ok(result_ptr)
                    }

                    // String comparison using vex_strcmp
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
