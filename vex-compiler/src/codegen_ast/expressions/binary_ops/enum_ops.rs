//! Enum binary operations
//!
//! Handles tag + data comparison for enum equality operations

use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::{FloatPredicate, IntPredicate};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operations for enum operands (tag + data comparison)
    pub(crate) fn compile_enum_binary_op(
        &mut self,
        l: inkwell::values::StructValue<'ctx>,
        r: inkwell::values::StructValue<'ctx>,
        op: &BinaryOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match op {
            BinaryOp::Eq | BinaryOp::NotEq => {
                // Compare tag first
                let l_tag = self
                    .builder
                    .build_extract_value(l, 0, "l_tag")
                    .map_err(|e| format!("Failed to extract left tag: {}", e))?
                    .into_int_value();
                let r_tag = self
                    .builder
                    .build_extract_value(r, 0, "r_tag")
                    .map_err(|e| format!("Failed to extract right tag: {}", e))?
                    .into_int_value();

                let tags_equal = self
                    .builder
                    .build_int_compare(IntPredicate::EQ, l_tag, r_tag, "tags_eq")
                    .map_err(|e| format!("Failed to compare tags: {}", e))?;

                // If tags are different, enums are not equal
                // If tags are same, also compare data field (index 1)
                let l_data = self
                    .builder
                    .build_extract_value(l, 1, "l_data")
                    .map_err(|e| format!("Failed to extract left data: {}", e))?;
                let r_data = self
                    .builder
                    .build_extract_value(r, 1, "r_data")
                    .map_err(|e| format!("Failed to extract right data: {}", e))?;

                // Compare data fields based on type
                let data_equal = match (l_data, r_data) {
                    (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => self
                        .builder
                        .build_int_compare(IntPredicate::EQ, li, ri, "data_eq")
                        .map_err(|e| format!("Failed to compare enum data: {}", e))?,
                    (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => self
                        .builder
                        .build_float_compare(FloatPredicate::OEQ, lf, rf, "data_eq")
                        .map_err(|e| format!("Failed to compare enum data: {}", e))?,
                    (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) => {
                        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                        let strcmp_fn = self.declare_runtime_fn(
                            "vex_strcmp",
                            &[ptr_type.into(), ptr_type.into()],
                            self.context.i32_type().into(),
                        );

                        let cmp_result = self
                            .builder
                            .build_call(strcmp_fn, &[lp.into(), rp.into()], "strcmp_result")
                            .map_err(|e| format!("Failed to call vex_strcmp: {}", e))?;

                        let cmp_value = cmp_result
                            .try_as_basic_value()
                            .unwrap_basic()
                            .into_int_value();

                        let zero = self.context.i32_type().const_int(0, false);
                        self.builder
                            .build_int_compare(IntPredicate::EQ, cmp_value, zero, "data_eq")
                            .map_err(|e| format!("Failed to compare string data: {}", e))?
                    }
                    (BasicValueEnum::StructValue(ls), BasicValueEnum::StructValue(rs)) => {
                        // Nested struct comparison (for multi-field enum data)
                        // Recursively compare all fields
                        let struct_type = ls.get_type();
                        let field_count = struct_type.count_fields();
                        let mut all_equal = self.context.bool_type().const_int(1, false);

                        for i in 0..field_count {
                            let lf = self
                                .builder
                                .build_extract_value(ls, i, &format!("ls_f{}", i))
                                .map_err(|e| format!("Failed to extract: {}", e))?;
                            let rf = self
                                .builder
                                .build_extract_value(rs, i, &format!("rs_f{}", i))
                                .map_err(|e| format!("Failed to extract: {}", e))?;

                            let field_eq = match (lf, rf) {
                                (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => {
                                    self.builder
                                        .build_int_compare(IntPredicate::EQ, li, ri, "feq")
                                        .map_err(|e| format!("Failed to compare: {}", e))?
                                }
                                (
                                    BasicValueEnum::FloatValue(lf),
                                    BasicValueEnum::FloatValue(rf),
                                ) => self
                                    .builder
                                    .build_float_compare(FloatPredicate::OEQ, lf, rf, "feq")
                                    .map_err(|e| format!("Failed to compare: {}", e))?,
                                _ => self.context.bool_type().const_int(1, false),
                            };

                            all_equal = self
                                .builder
                                .build_and(all_equal, field_eq, "and")
                                .map_err(|e| format!("Failed to AND: {}", e))?;
                        }

                        all_equal
                    }
                    _ => {
                        // For other types, assume equal if tags are equal
                        // This handles None case where data is just zero/undef
                        self.context.bool_type().const_int(1, false)
                    }
                };

                // Both tag and data must be equal
                let both_equal = self
                    .builder
                    .build_and(tags_equal, data_equal, "enum_eq")
                    .map_err(|e| format!("Failed to AND tag and data: {}", e))?;

                let result = if matches!(op, BinaryOp::Eq) {
                    both_equal
                } else {
                    self.builder
                        .build_not(both_equal, "enum_neq")
                        .map_err(|e| format!("Failed to negate: {}", e))?
                };

                Ok(result.into())
            }
            _ => Err("Only == and != are supported for enum comparison".to_string()),
        }
    }
}
