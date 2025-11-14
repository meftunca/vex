//! Pointer binary operations
//!
//! Handles string concatenation and comparison operations for pointer types

use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operations for pointer operands (strings)
    pub(crate) fn compile_pointer_binary_op(
        &mut self,
        l: inkwell::values::PointerValue<'ctx>,
        r: inkwell::values::PointerValue<'ctx>,
        op: &BinaryOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
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

                let result_ptr = concat_result.try_as_basic_value().unwrap_basic();

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
                    .unwrap_basic()
                    .into_int_value();

                // vex_strcmp returns 0 if equal
                let zero = self.context.i32_type().const_int(0, false);
                let result = if matches!(op, BinaryOp::Eq) {
                    self.builder
                        .build_int_compare(IntPredicate::EQ, cmp_value, zero, "streq")
                } else {
                    self.builder
                        .build_int_compare(IntPredicate::NE, cmp_value, zero, "strne")
                }
                .map_err(|e| format!("Failed to compare strcmp result: {}", e))?;

                Ok(result.into())
            }
            _ => Err("Only == and != are supported for string comparison".to_string()),
        }
    }
}
