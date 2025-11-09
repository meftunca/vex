// Type casting operations

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile type cast: expr as TargetType
    /// Supports:
    /// - Numeric casts: i32 -> i64, f64 -> i32, i32 -> f32, etc.
    /// - Pointer casts: *T -> *U, &T -> *T
    /// - Sign changes: i32 -> u32, u64 -> i64
    pub(crate) fn compile_cast_expression(
        &mut self,
        expr: &Expression,
        target_type: &Type,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let value = self.compile_expression(expr)?;
        let target_llvm = self.ast_type_to_llvm(target_type);

        // Handle integer -> integer casts
        if let BasicValueEnum::IntValue(int_val) = value {
            if let inkwell::types::BasicTypeEnum::IntType(target_int) = target_llvm {
                let source_width = int_val.get_type().get_bit_width();
                let target_width = target_int.get_bit_width();

                if source_width < target_width {
                    // Widening cast: i32 -> i64 (safe, use sign extension)
                    return Ok(self
                        .builder
                        .build_int_s_extend(int_val, target_int, "cast_sext")
                        .map_err(|e| format!("Failed to sign-extend: {}", e))?
                        .into());
                } else if source_width > target_width {
                    // Narrowing cast: i64 -> i32 (lossy, truncate)
                    return Ok(self
                        .builder
                        .build_int_truncate(int_val, target_int, "cast_trunc")
                        .map_err(|e| format!("Failed to truncate: {}", e))?
                        .into());
                } else {
                    // Same width: i32 -> u32 or u32 -> i32 (bitcast)
                    return Ok(int_val.into());
                }
            }
        }

        // Handle float -> float casts
        if let BasicValueEnum::FloatValue(float_val) = value {
            if let inkwell::types::BasicTypeEnum::FloatType(target_float) = target_llvm {
                // LLVM doesn't expose size for floats, infer from types
                // f32 = 32bit, f64 = 64bit, f16 = 16bit, f128 = 128bit
                let source_is_double = float_val.get_type() == self.context.f64_type();
                let target_is_double = target_float == self.context.f64_type();

                if !source_is_double && target_is_double {
                    // f32 -> f64 (safe, extend)
                    return Ok(self
                        .builder
                        .build_float_ext(float_val, target_float, "cast_fext")
                        .map_err(|e| format!("Failed to extend float: {}", e))?
                        .into());
                } else if source_is_double && !target_is_double {
                    // f64 -> f32 (lossy, truncate)
                    return Ok(self
                        .builder
                        .build_float_trunc(float_val, target_float, "cast_ftrunc")
                        .map_err(|e| format!("Failed to truncate float: {}", e))?
                        .into());
                } else {
                    // Same type
                    return Ok(float_val.into());
                }
            }
        }

        // Handle int -> float
        // Need to determine if source is signed or unsigned to choose correct LLVM instruction
        if let BasicValueEnum::IntValue(int_val) = value {
            if let inkwell::types::BasicTypeEnum::FloatType(target_float) = target_llvm {
                // Check target_type to see if we're casting FROM unsigned
                // For now, use signed (most common case)
                // TODO: Track source type to distinguish signed vs unsigned
                // - Use uitofp for u8/u16/u32/u64 -> float
                // - Use sitofp for i8/i16/i32/i64 -> float
                return Ok(self
                    .builder
                    .build_signed_int_to_float(int_val, target_float, "cast_itof")
                    .map_err(|e| format!("Failed to convert int to float: {}", e))?
                    .into());
            }
        }

        // Handle float -> int
        // Similarly, need to know if target is signed or unsigned
        if let BasicValueEnum::FloatValue(float_val) = value {
            if let inkwell::types::BasicTypeEnum::IntType(target_int) = target_llvm {
                // Check target_type for unsigned types
                let is_unsigned = matches!(
                    target_type,
                    Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128
                );

                if is_unsigned {
                    // Float to unsigned int
                    return Ok(self
                        .builder
                        .build_float_to_unsigned_int(float_val, target_int, "cast_ftou")
                        .map_err(|e| format!("Failed to convert float to uint: {}", e))?
                        .into());
                } else {
                    // Float to signed int
                    return Ok(self
                        .builder
                        .build_float_to_signed_int(float_val, target_int, "cast_ftoi")
                        .map_err(|e| format!("Failed to convert float to int: {}", e))?
                        .into());
                }
            }
        }

        // Handle pointer casts: *T -> *U
        if let BasicValueEnum::PointerValue(ptr_val) = value {
            if let inkwell::types::BasicTypeEnum::PointerType(target_ptr) = target_llvm {
                return Ok(self
                    .builder
                    .build_pointer_cast(ptr_val, target_ptr, "cast_ptr")
                    .map_err(|e| format!("Failed to cast pointer: {}", e))?
                    .into());
            }
        }

        // ⭐ NEW: Handle int -> pointer cast (e.g., 0 as *u8 for null pointer)
        if let BasicValueEnum::IntValue(int_val) = value {
            if let inkwell::types::BasicTypeEnum::PointerType(target_ptr) = target_llvm {
                return Ok(self
                    .builder
                    .build_int_to_ptr(int_val, target_ptr, "cast_itop")
                    .map_err(|e| format!("Failed to cast int to pointer: {}", e))?
                    .into());
            }
        }

        // ⭐ NEW: Handle pointer -> int cast (e.g., ptr as i64 for address)
        if let BasicValueEnum::PointerValue(ptr_val) = value {
            if let inkwell::types::BasicTypeEnum::IntType(target_int) = target_llvm {
                return Ok(self
                    .builder
                    .build_ptr_to_int(ptr_val, target_int, "cast_ptoi")
                    .map_err(|e| format!("Failed to cast pointer to int: {}", e))?
                    .into());
            }
        }

        Err(format!(
            "Unsupported cast from {:?} to {:?}",
            value.get_type(),
            target_llvm
        ))
    }
}
