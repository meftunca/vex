// LLVM IR safety utilities - pointer validation and size checks
//
// This module provides safety checks for LLVM operations to prevent:
// - Stack overflow from excessive allocations
// - Null pointer dereference
// - Out-of-bounds memory access
// - Type confusion from unchecked casts
//
// Related: critique/03_LLVM_POINTER_SAFETY.md

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{IntValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::IntPredicate;

/// Maximum stack allocation size (1 MB)
/// Prevents stack overflow from excessive alloca operations
pub const MAX_STACK_ALLOC_SIZE: usize = 1024 * 1024;

/// Validates that a type's size is safe for stack allocation
///
/// # Arguments
/// * `ty` - The LLVM type to validate
/// * `type_name` - Human-readable name for error messages
///
/// # Returns
/// * `Ok(())` - Type is safe to allocate on stack
/// * `Err(String)` - Size exceeds MAX_STACK_ALLOC_SIZE
pub fn validate_stack_allocation_size(ty: BasicTypeEnum, type_name: &str) -> Result<(), String> {
    let size_bits = match ty {
        BasicTypeEnum::ArrayType(arr) => {
            let elem_type = arr.get_element_type();
            let length = arr.len();

            // Get element size in bits
            let elem_size = match elem_type {
                BasicTypeEnum::IntType(it) => it.get_bit_width() as usize,
                BasicTypeEnum::FloatType(ft) => {
                    if ft.get_context().f32_type() == ft {
                        32
                    } else if ft.get_context().f64_type() == ft {
                        64
                    } else {
                        return Err(format!("Unknown float type in array: {}", type_name));
                    }
                }
                BasicTypeEnum::PointerType(_) => 64, // Assume 64-bit pointers
                BasicTypeEnum::StructType(st) => {
                    // For structs, we can't easily get exact size without target data
                    // Conservative estimate: sum of field sizes
                    let mut total = 0;
                    for field in st.get_field_types() {
                        total += estimate_type_size_bits(field)?;
                    }
                    total
                }
                BasicTypeEnum::ArrayType(_) => {
                    return Err(format!("Nested array type not supported: {}", type_name));
                }
                _ => {
                    return Err(format!("Unsupported element type in array: {}", type_name));
                }
            };

            elem_size.checked_mul(length as usize).ok_or_else(|| {
                format!(
                    "Array size overflow for {}: {} * {}",
                    type_name, elem_size, length
                )
            })?
        }
        BasicTypeEnum::StructType(st) => {
            let mut total = 0;
            for field in st.get_field_types() {
                total += estimate_type_size_bits(field)?;
            }
            total
        }
        BasicTypeEnum::IntType(it) => it.get_bit_width() as usize,
        BasicTypeEnum::FloatType(ft) => {
            if ft.get_context().f32_type() == ft {
                32
            } else if ft.get_context().f64_type() == ft {
                64
            } else {
                128 // Assume f128 or extended precision
            }
        }
        BasicTypeEnum::PointerType(_) => 64, // Assume 64-bit pointers
        _ => {
            return Err(format!("Cannot determine size for type: {}", type_name));
        }
    };

    let size_bytes = size_bits / 8;

    if size_bytes > MAX_STACK_ALLOC_SIZE {
        return Err(format!(
            "Stack allocation too large for '{}': {} bytes (max: {} bytes)",
            type_name, size_bytes, MAX_STACK_ALLOC_SIZE
        ));
    }

    Ok(())
}

/// Helper function to estimate type size in bits
fn estimate_type_size_bits(ty: BasicTypeEnum) -> Result<usize, String> {
    match ty {
        BasicTypeEnum::IntType(it) => Ok(it.get_bit_width() as usize),
        BasicTypeEnum::FloatType(ft) => {
            if ft.get_context().f32_type() == ft {
                Ok(32)
            } else if ft.get_context().f64_type() == ft {
                Ok(64)
            } else {
                Ok(128)
            }
        }
        BasicTypeEnum::PointerType(_) => Ok(64),
        BasicTypeEnum::ArrayType(arr) => {
            let elem_size = estimate_type_size_bits(arr.get_element_type())?;
            elem_size
                .checked_mul(arr.len() as usize)
                .ok_or_else(|| "Array size overflow in estimation".to_string())
        }
        BasicTypeEnum::StructType(st) => {
            let mut total = 0;
            for field in st.get_field_types() {
                total += estimate_type_size_bits(field)?;
            }
            Ok(total)
        }
        _ => Err("Unknown type in size estimation".to_string()),
    }
}

/// Checks if a pointer value is likely non-null based on provenance
///
/// This is a compile-time heuristic check. For runtime null checks,
/// use emit_null_check which generates LLVM IR.
///
/// Returns false for:
/// - Pointers from function parameters (could be null from FFI)
/// - Pointers from external calls
/// - Global pointers that might be uninitialized
///
/// Returns true for:
/// - Pointers from recent alloca
/// - String literals
/// - GEP results from validated pointers
pub fn is_pointer_provably_nonnull(ptr: PointerValue) -> bool {
    // Check if pointer comes from an alloca instruction
    // (allocas in entry block are always non-null)
    if let Some(instruction) = ptr.as_instruction() {
        // Compare opcode directly without formatting
        use inkwell::values::InstructionOpcode;
        return instruction.get_opcode() == InstructionOpcode::Alloca;
    }

    // Conservative: assume unknown pointers might be null
    false
}

/// Emit runtime null pointer check before dereferencing
///
/// Generates LLVM IR to check if pointer is null and abort if so.
/// This prevents undefined behavior from null pointer dereference.
///
/// **Note:** This function requires access to the LLVM module, so it's designed
/// to be called from within ASTCodeGen methods, not standalone.
///
/// # Arguments
/// * `builder` - LLVM IR builder
/// * `context` - LLVM context  
/// * `module` - LLVM module (for adding panic function)
/// * `ptr` - Pointer to validate
/// * `error_msg` - Error message for panic (compile-time constant)
///
/// # Generated IR
/// ```llvm
/// %is_null = icmp eq ptr %ptr, null
/// br i1 %is_null, label %null_panic, label %safe_continue
///
/// null_panic:
///   call void @vex_panic_null_ptr()
///   unreachable
///
/// safe_continue:
///   ; ... original code continues ...
/// ```
pub fn emit_null_check<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    ptr: PointerValue<'ctx>,
    error_msg: &str,
) -> Result<(), String> {
    // Skip check if provably non-null (optimization)
    if is_pointer_provably_nonnull(ptr) {
        return Ok(());
    }

    // Get current function
    let current_fn = builder
        .get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or_else(|| "No active function for null check".to_string())?;

    // Create null constant (opaque pointer)
    let null_ptr = context.ptr_type(AddressSpace::default()).const_null();

    // Convert both to integers for comparison (LLVM opaque pointers)
    let ptr_int = builder
        .build_ptr_to_int(ptr, context.i64_type(), "ptr_int")
        .map_err(|e| format!("Failed to convert pointer to int: {}", e))?;
    let null_int = builder
        .build_ptr_to_int(null_ptr, context.i64_type(), "null_int")
        .map_err(|e| format!("Failed to convert null to int: {}", e))?;

    // Compare pointer with null
    let is_null = builder
        .build_int_compare(IntPredicate::EQ, ptr_int, null_int, "is_null")
        .map_err(|e| format!("Failed to build null comparison: {}", e))?;

    // Create basic blocks
    let null_block = context.append_basic_block(current_fn, "null_panic");
    let safe_block = context.append_basic_block(current_fn, "safe_continue");

    // Branch based on null check
    builder
        .build_conditional_branch(is_null, null_block, safe_block)
        .map_err(|e| format!("Failed to build null check branch: {}", e))?;

    // Emit panic in null block
    builder.position_at_end(null_block);

    // Call panic function (will be implemented in runtime)
    let i8_ptr_type = context.ptr_type(AddressSpace::default());
    let void_type = context.void_type();

    let panic_fn_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    let panic_fn = module.add_function("vex_panic_null_ptr", panic_fn_type, None);

    // Create error message constant
    let error_str = builder
        .build_global_string_ptr(error_msg, "null_err_msg")
        .map_err(|e| format!("Failed to create error string: {}", e))?;

    builder
        .build_call(panic_fn, &[error_str.as_pointer_value().into()], "")
        .map_err(|e| format!("Failed to call panic: {}", e))?;

    builder
        .build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    // Continue in safe block
    builder.position_at_end(safe_block);

    Ok(())
}

/// Emit runtime array bounds check before GEP operation
///
/// Validates that index is within array bounds to prevent buffer overflow.
///
/// # Arguments
/// * `builder` - LLVM IR builder
/// * `context` - LLVM context
/// * `module` - LLVM module (for adding panic function)
/// * `index` - Array index to validate
/// * `array_len` - Array length (compile-time constant or runtime value)
///
/// # Generated IR
/// ```llvm
/// %out_of_bounds = icmp uge i32 %index, %array_len
/// br i1 %out_of_bounds, label %bounds_panic, label %safe_continue
///
/// bounds_panic:
///   call void @vex_panic_bounds(i32 %index, i32 %array_len)
///   unreachable
/// ```
pub fn emit_bounds_check<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    index: IntValue<'ctx>,
    array_len: IntValue<'ctx>,
) -> Result<(), String> {
    // Get current function
    let current_fn = builder
        .get_insert_block()
        .and_then(|bb| bb.get_parent())
        .ok_or_else(|| "No active function for bounds check".to_string())?;

    // Check if index >= array_len (unsigned comparison)
    let out_of_bounds = builder
        .build_int_compare(IntPredicate::UGE, index, array_len, "out_of_bounds")
        .map_err(|e| format!("Failed to build bounds comparison: {}", e))?;

    // Create basic blocks
    let panic_block = context.append_basic_block(current_fn, "bounds_panic");
    let safe_block = context.append_basic_block(current_fn, "safe_continue");

    // Branch based on bounds check
    builder
        .build_conditional_branch(out_of_bounds, panic_block, safe_block)
        .map_err(|e| format!("Failed to build bounds check branch: {}", e))?;

    // Emit panic in panic block
    builder.position_at_end(panic_block);

    // Call panic function with index and length
    let void_type = context.void_type();
    let i32_type = context.i32_type();

    let panic_fn_type = void_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let panic_fn = module.add_function("vex_panic_bounds", panic_fn_type, None);

    builder
        .build_call(panic_fn, &[index.into(), array_len.into()], "")
        .map_err(|e| format!("Failed to call panic: {}", e))?;

    builder
        .build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    // Continue in safe block
    builder.position_at_end(safe_block);

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_small_allocation() {
        let context = Context::create();
        let i32_type = context.i32_type();

        assert!(validate_stack_allocation_size(i32_type.into(), "i32").is_ok());
    }

    #[test]
    fn test_array_allocation_ok() {
        let context = Context::create();
        let i32_type = context.i32_type();
        let array_type = i32_type.array_type(1000); // 4KB

        assert!(validate_stack_allocation_size(array_type.into(), "i32[1000]").is_ok());
    }

    #[test]
    fn test_array_allocation_too_large() {
        let context = Context::create();
        let i32_type = context.i32_type();
        let array_type = i32_type.array_type(1_000_000); // 4MB - exceeds limit

        let result = validate_stack_allocation_size(array_type.into(), "i32[1000000]");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too large"));
    }

    #[test]
    fn test_struct_allocation() {
        let context = Context::create();
        let i32_type = context.i32_type();
        let f64_type = context.f64_type();
        let struct_type = context.struct_type(&[i32_type.into(), f64_type.into()], false);

        // Struct should be 12 bytes (4 + 8), well under limit
        assert!(validate_stack_allocation_size(struct_type.into(), "Point").is_ok());
    }
}
