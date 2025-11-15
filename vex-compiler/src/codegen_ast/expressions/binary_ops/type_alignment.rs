//! Type alignment for binary operations
//!
//! Handles automatic type widening/narrowing for integer and float operands

use super::super::super::ASTCodeGen;
use inkwell::values::{FloatValue, IntValue};

impl<'ctx> ASTCodeGen<'ctx> {
    /// Align integer widths for binary operations
    /// Widens the narrower operand to match the wider one
    pub(crate) fn align_integer_widths(
        &mut self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> Result<(IntValue<'ctx>, IntValue<'ctx>), String> {
        let left_bits = left.get_type().get_bit_width();
        let right_bits = right.get_type().get_bit_width();

        if left_bits == right_bits {
            return Ok((left, right));
        }

        if left_bits < right_bits {
            // Widen left to match right
            let widened = self
                .builder
                .build_int_s_extend(left, right.get_type(), "left_sext")
                .map_err(|e| format!("Failed to widen left operand: {}", e))?;
            Ok((widened, right))
        } else {
            // Widen right to match left
            let widened = self
                .builder
                .build_int_s_extend(right, left.get_type(), "right_sext")
                .map_err(|e| format!("Failed to widen right operand: {}", e))?;
            Ok((left, widened))
        }
    }

    /// Align float widths for binary operations
    /// Widens narrower floats to match wider ones (f16 < f32 < f64)
    pub(crate) fn align_float_widths(
        &mut self,
        left: FloatValue<'ctx>,
        right: FloatValue<'ctx>,
    ) -> Result<(FloatValue<'ctx>, FloatValue<'ctx>), String> {
        let left_type = left.get_type();
        let right_type = right.get_type();

        // Check if both are the same type
        if left_type == right_type {
            return Ok((left, right));
        }

        let f16_type = self.context.f16_type();
        let f32_type = self.context.f32_type();
        let f64_type = self.context.f64_type();

        // Determine target type (widest one)
        let target_type = if left_type == f64_type || right_type == f64_type {
            f64_type
        } else if left_type == f32_type || right_type == f32_type {
            f32_type
        } else if left_type == f16_type || right_type == f16_type {
            f16_type
        } else {
            return Err(format!(
                "Unsupported float types: {:?} vs {:?}",
                left_type, right_type
            ));
        };

        // Widen left if needed
        let left = if left_type != target_type {
            self.builder
                .build_float_ext(left, target_type, "left_fext")
                .map_err(|e| format!("Failed to widen left float: {}", e))?
        } else {
            left
        };

        // Widen right if needed
        let right = if right_type != target_type {
            self.builder
                .build_float_ext(right, target_type, "right_fext")
                .map_err(|e| format!("Failed to widen right float: {}", e))?
        } else {
            right
        };

        Ok((left, right))
    }
}
