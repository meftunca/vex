//! Power operations for binary expressions
//!
//! Handles exponentiation for both integer and float types

use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile integer power: base ** exp using loop
    pub(crate) fn compile_int_power(
        &mut self,
        base: inkwell::values::IntValue<'ctx>,
        exp: inkwell::values::IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Fast path for constant exponents
        if let Some(exp_const) = exp.get_zero_extended_constant() {
            if exp_const == 0 {
                return Ok(self.context.i64_type().const_int(1, false).into());
            }
            if exp_const == 1 {
                return Ok(base.into());
            }
        }

        // result = 1
        let result_alloca = self
            .builder
            .build_alloca(base.get_type(), "pow_result")
            .map_err(|e| format!("Failed to allocate power result: {}", e))?;
        let one = base.get_type().const_int(1, false);
        self.builder
            .build_store(result_alloca, one)
            .map_err(|e| format!("Failed to store initial result: {}", e))?;

        // counter = exp
        let counter_alloca = self
            .builder
            .build_alloca(exp.get_type(), "pow_counter")
            .map_err(|e| format!("Failed to allocate counter: {}", e))?;
        self.builder
            .build_store(counter_alloca, exp)
            .map_err(|e| format!("Failed to store counter: {}", e))?;

        // Loop: while counter > 0
        let parent_fn = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let loop_block = self.context.append_basic_block(parent_fn, "pow_loop");
        let after_block = self.context.append_basic_block(parent_fn, "pow_after");

        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| format!("Failed to branch to loop: {}", e))?;
        self.builder.position_at_end(loop_block);

        // Load counter
        let counter = self
            .builder
            .build_load(exp.get_type(), counter_alloca, "counter")
            .map_err(|e| format!("Failed to load counter: {}", e))?
            .into_int_value();

        // Check if counter > 0
        let zero = exp.get_type().const_int(0, false);
        let cond = self
            .builder
            .build_int_compare(inkwell::IntPredicate::SGT, counter, zero, "pow_cond")
            .map_err(|e| format!("Failed to compare: {}", e))?;

        let loop_body = self.context.append_basic_block(parent_fn, "pow_body");
        self.builder
            .build_conditional_branch(cond, loop_body, after_block)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

        // Loop body: result *= base
        self.builder.position_at_end(loop_body);
        let current_result = self
            .builder
            .build_load(base.get_type(), result_alloca, "current_result")
            .map_err(|e| format!("Failed to load result: {}", e))?
            .into_int_value();
        let new_result = self
            .builder
            .build_int_mul(current_result, base, "new_result")
            .map_err(|e| format!("Failed to multiply: {}", e))?;
        self.builder
            .build_store(result_alloca, new_result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // counter -= 1
        let new_counter = self
            .builder
            .build_int_sub(counter, one, "new_counter")
            .map_err(|e| format!("Failed to decrement: {}", e))?;
        self.builder
            .build_store(counter_alloca, new_counter)
            .map_err(|e| format!("Failed to store counter: {}", e))?;

        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| format!("Failed to branch back: {}", e))?;

        // After loop
        self.builder.position_at_end(after_block);
        let final_result = self
            .builder
            .build_load(base.get_type(), result_alloca, "final_result")
            .map_err(|e| format!("Failed to load final result: {}", e))?;

        Ok(final_result)
    }

    /// Compile float power: base ** exp using llvm.pow intrinsic
    pub(crate) fn compile_float_power(
        &mut self,
        base: inkwell::values::FloatValue<'ctx>,
        exp: inkwell::values::FloatValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Declare llvm.pow.f64 intrinsic
        let pow_intrinsic = self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
            let f64_type = self.context.f64_type();
            let fn_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
            self.module.add_function("llvm.pow.f64", fn_type, None)
        });

        let result = self
            .builder
            .build_call(pow_intrinsic, &[base.into(), exp.into()], "pow_result")
            .map_err(|e| format!("Failed to call pow intrinsic: {}", e))?
            .try_as_basic_value()
            .unwrap_basic();

        Ok(result)
    }
}
