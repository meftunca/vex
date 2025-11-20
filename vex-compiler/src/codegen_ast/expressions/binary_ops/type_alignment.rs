//! Type alignment for binary operations
//!
//! Handles automatic type widening (safe) with downcast prevention

use super::super::super::ASTCodeGen;
use crate::type_system::coercion_rules::{
    classify_coercion, coercion_policy, format_coercion_error, format_coercion_warning,
    CoercionPolicy,
};
use inkwell::values::{FloatValue, IntValue};
use vex_ast::Type;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Align integer widths for binary operations with AST type awareness
    /// - Upcast (Safe): i8 → i64 ✓
    /// - Downcast (Unsafe): i64 → i32 ✗ (error), unless unsafe{} (warning)
    pub(crate) fn align_integer_widths_with_ast(
        &mut self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
        left_type: &Type,
        right_type: &Type,
    ) -> Result<(IntValue<'ctx>, IntValue<'ctx>), String> {
        let left_bits = left.get_type().get_bit_width();
        let right_bits = right.get_type().get_bit_width();

        if left_bits == right_bits {
            return Ok((left, right));
        }

        // Check coercion rules using AST types
        if left_bits < right_bits {
            // Widen left to match right
            let kind = classify_coercion(left_type, right_type);
            let policy = coercion_policy(kind, self.is_in_unsafe_block);

            match policy {
                CoercionPolicy::Allow => {
                    let widened = self
                        .builder
                        .build_int_s_extend(left, right.get_type(), "left_upcast")
                        .map_err(|e| format!("Failed to widen left operand: {}", e))?;
                    Ok((widened, right))
                }
                CoercionPolicy::Warn => {
                    // Emit warning in unsafe{}
                    eprintln!(
                        "⚠️  warning: {}",
                        format_coercion_warning(left_type, right_type)
                    );
                    let widened = self
                        .builder
                        .build_int_s_extend(left, right.get_type(), "left_unsafe_cast")
                        .map_err(|e| format!("Failed to cast left operand: {}", e))?;
                    Ok((widened, right))
                }
                CoercionPolicy::Error => Err(format_coercion_error(left_type, right_type, kind)),
            }
        } else {
            // Widen right to match left
            let kind = classify_coercion(right_type, left_type);
            let policy = coercion_policy(kind, self.is_in_unsafe_block);

            match policy {
                CoercionPolicy::Allow => {
                    let widened = self
                        .builder
                        .build_int_s_extend(right, left.get_type(), "right_upcast")
                        .map_err(|e| format!("Failed to widen right operand: {}", e))?;
                    Ok((left, widened))
                }
                CoercionPolicy::Warn => {
                    // Emit warning in unsafe{}
                    eprintln!(
                        "⚠️  warning: {}",
                        format_coercion_warning(right_type, left_type)
                    );
                    let widened = self
                        .builder
                        .build_int_s_extend(right, left.get_type(), "right_unsafe_cast")
                        .map_err(|e| format!("Failed to cast right operand: {}", e))?;
                    Ok((left, widened))
                }
                CoercionPolicy::Error => Err(format_coercion_error(right_type, left_type, kind)),
            }
        }
    }

    /// Align integer widths for binary operations with safe coercion rules
    /// - Upcast (Safe): i8 → i64 ✓
    /// - Downcast (Unsafe): i64 → i32 ✗ (error), unless unsafe{} (warning)
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

        // Determine AST types from LLVM bit widths
        let left_type = self.infer_int_type_from_bits(left_bits, true)?; // assume signed
        let right_type = self.infer_int_type_from_bits(right_bits, true)?;

        // Check coercion rules
        if left_bits < right_bits {
            // Widen left to match right
            let kind = classify_coercion(&left_type, &right_type);
            let policy = coercion_policy(kind, self.is_in_unsafe_block);

            match policy {
                CoercionPolicy::Allow => {
                    let widened = self
                        .builder
                        .build_int_s_extend(left, right.get_type(), "left_upcast")
                        .map_err(|e| format!("Failed to widen left operand: {}", e))?;
                    Ok((widened, right))
                }
                CoercionPolicy::Warn => {
                    // Emit warning in unsafe{}
                    eprintln!(
                        "⚠️  warning: {}",
                        format_coercion_warning(&left_type, &right_type)
                    );
                    let widened = self
                        .builder
                        .build_int_s_extend(left, right.get_type(), "left_unsafe_cast")
                        .map_err(|e| format!("Failed to cast left operand: {}", e))?;
                    Ok((widened, right))
                }
                CoercionPolicy::Error => {
                    Err(format_coercion_error(&left_type, &right_type, kind))
                }
            }
        } else {
            // Widen right to match left
            let kind = classify_coercion(&right_type, &left_type);
            let policy = coercion_policy(kind, self.is_in_unsafe_block);

            match policy {
                CoercionPolicy::Allow => {
                    let widened = self
                        .builder
                        .build_int_s_extend(right, left.get_type(), "right_upcast")
                        .map_err(|e| format!("Failed to widen right operand: {}", e))?;
                    Ok((left, widened))
                }
                CoercionPolicy::Warn => {
                    // Emit warning in unsafe{}
                    eprintln!(
                        "⚠️  warning: {}",
                        format_coercion_warning(&right_type, &left_type)
                    );
                    let widened = self
                        .builder
                        .build_int_s_extend(right, left.get_type(), "right_unsafe_cast")
                        .map_err(|e| format!("Failed to cast right operand: {}", e))?;
                    Ok((left, widened))
                }
                CoercionPolicy::Error => {
                    Err(format_coercion_error(&right_type, &left_type, kind))
                }
            }
        }
    }

    /// Infer AST Type from LLVM integer bit width
    /// Note: Cannot distinguish signed/unsigned from LLVM type alone
    fn infer_int_type_from_bits(&self, bits: u32, is_signed: bool) -> Result<Type, String> {
        if is_signed {
            match bits {
                8 => Ok(Type::I8),
                16 => Ok(Type::I16),
                32 => Ok(Type::I32),
                64 => Ok(Type::I64),
                128 => Ok(Type::I128),
                _ => Err(format!("Unsupported integer bit width: {}", bits)),
            }
        } else {
            match bits {
                8 => Ok(Type::U8),
                16 => Ok(Type::U16),
                32 => Ok(Type::U32),
                64 => Ok(Type::U64),
                128 => Ok(Type::U128),
                _ => Err(format!("Unsupported integer bit width: {}", bits)),
            }
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

        // Determine AST types for coercion check
        let left_ast_type = if left_type == f16_type {
            Type::F16
        } else if left_type == f32_type {
            Type::F32
        } else {
            Type::F64
        };

        let right_ast_type = if right_type == f16_type {
            Type::F16
        } else if right_type == f32_type {
            Type::F32
        } else {
            Type::F64
        };

        let target_ast_type = if target_type == f16_type {
            Type::F16
        } else if target_type == f32_type {
            Type::F32
        } else {
            Type::F64
        };

        // Widen left if needed
        let left = if left_type != target_type {
            let kind = classify_coercion(&left_ast_type, &target_ast_type);
            let policy = coercion_policy(kind, self.is_in_unsafe_block);

            match policy {
                CoercionPolicy::Allow => self
                    .builder
                    .build_float_ext(left, target_type, "left_fupcast")
                    .map_err(|e| format!("Failed to widen left float: {}", e))?,
                CoercionPolicy::Warn => {
                    eprintln!(
                        "⚠️  warning: {}",
                        format_coercion_warning(&left_ast_type, &target_ast_type)
                    );
                    self.builder
                        .build_float_ext(left, target_type, "left_funsafe")
                        .map_err(|e| format!("Failed to cast left float: {}", e))?
                }
                CoercionPolicy::Error => {
                    return Err(format_coercion_error(
                        &left_ast_type,
                        &target_ast_type,
                        kind,
                    ))
                }
            }
        } else {
            left
        };

        // Widen right if needed
        let right = if right_type != target_type {
            let kind = classify_coercion(&right_ast_type, &target_ast_type);
            let policy = coercion_policy(kind, self.is_in_unsafe_block);

            match policy {
                CoercionPolicy::Allow => self
                    .builder
                    .build_float_ext(right, target_type, "right_fupcast")
                    .map_err(|e| format!("Failed to widen right float: {}", e))?,
                CoercionPolicy::Warn => {
                    eprintln!(
                        "⚠️  warning: {}",
                        format_coercion_warning(&right_ast_type, &target_ast_type)
                    );
                    self.builder
                        .build_float_ext(right, target_type, "right_funsafe")
                        .map_err(|e| format!("Failed to cast right float: {}", e))?
                }
                CoercionPolicy::Error => {
                    return Err(format_coercion_error(
                        &right_ast_type,
                        &target_ast_type,
                        kind,
                    ))
                }
            }
        } else {
            right
        };

        Ok((left, right))
    }
}
