// Memory management builtins: alloc, free, realloc, sizeof, alignof

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// alloc(size) - Allocate memory
pub fn builtin_alloc<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("alloc() takes exactly one argument (size)".to_string());
    }

    let size = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("alloc() size must be an integer".to_string()),
    };

    // Cast size to i64 (size_t)
    let size_i64 = codegen
        .builder
        .build_int_z_extend(size, codegen.context.i64_type(), "size_cast")
        .map_err(|e| format!("Failed to cast size to i64: {}", e))?;

    // Declare vex_malloc from runtime
    let vex_malloc = codegen.declare_vex_malloc();

    // Call vex_malloc(size)
    let result = codegen
        .builder
        .build_call(vex_malloc, &[size_i64.into()], "alloc_call")
        .map_err(|e| format!("Failed to call vex_malloc: {}", e))?;

    let ptr = result
        .try_as_basic_value()
        .left()
        .ok_or("vex_malloc didn't return a value")?;

    Ok(ptr)
}

/// free(ptr) - Free memory
pub fn builtin_free<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("free() takes exactly one argument (pointer)".to_string());
    }

    let ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr_val) => ptr_val,
        _ => return Err("free() argument must be a pointer".to_string()),
    };

    // Declare vex_free from runtime
    let vex_free = codegen.declare_vex_free();

    // Call vex_free(ptr)
    codegen
        .builder
        .build_call(vex_free, &[ptr.into()], "free_call")
        .map_err(|e| format!("Failed to call vex_free: {}", e))?;

    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// realloc(ptr, new_size) - Reallocate memory
pub fn builtin_realloc<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("realloc() takes exactly two arguments (ptr, new_size)".to_string());
    }

    let ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr_val) => ptr_val,
        _ => return Err("realloc() first argument must be a pointer".to_string()),
    };

    let new_size = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("realloc() second argument must be an integer".to_string()),
    };

    // Cast new_size to i64 (size_t)
    let new_size_i64 = codegen
        .builder
        .build_int_z_extend(new_size, codegen.context.i64_type(), "new_size_cast")
        .map_err(|e| format!("Failed to cast new_size to i64: {}", e))?;

    // Declare vex_realloc from runtime
    let vex_realloc = codegen.declare_vex_realloc();

    // Call vex_realloc(ptr, new_size)
    let result = codegen
        .builder
        .build_call(
            vex_realloc,
            &[ptr.into(), new_size_i64.into()],
            "realloc_call",
        )
        .map_err(|e| format!("Failed to call vex_realloc: {}", e))?;

    let new_ptr = result
        .try_as_basic_value()
        .left()
        .ok_or("vex_realloc didn't return a value")?;

    Ok(new_ptr)
}

/// sizeof<T>() - Get size of type in bytes
pub fn builtin_sizeof<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // sizeof is typically handled at compile-time with type information
    // For now, if called with a value, return the size of its type
    if args.is_empty() {
        return Err("sizeof() requires a value to determine type size".to_string());
    }

    let value = args[0];
    let size = match value {
        BasicValueEnum::IntValue(int_val) => {
            let int_type = int_val.get_type();
            int_type.get_bit_width() / 8
        }
        BasicValueEnum::FloatValue(float_val) => match float_val.get_type() {
            ty if ty == codegen.context.f32_type() => 4,
            ty if ty == codegen.context.f64_type() => 8,
            _ => return Err("Unknown float type".to_string()),
        },
        BasicValueEnum::PointerValue(_) => 8, // 64-bit pointer
        BasicValueEnum::StructValue(struct_val) => {
            // For structs, sum up field sizes (simplified, no padding calculation)
            let struct_type = struct_val.get_type();
            let field_count = struct_type.count_fields();
            let mut total_size = 0u32;

            for i in 0..field_count {
                if let Some(field_type) = struct_type.get_field_type_at_index(i) {
                    let field_size = match field_type {
                        inkwell::types::BasicTypeEnum::IntType(int_type) => {
                            int_type.get_bit_width() / 8
                        }
                        inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                            if float_type == codegen.context.f32_type() {
                                4
                            } else {
                                8
                            }
                        }
                        inkwell::types::BasicTypeEnum::PointerType(_) => 8,
                        _ => 8, // Default to pointer size
                    };
                    total_size += field_size;
                }
            }
            total_size
        }
        BasicValueEnum::ArrayValue(arr_val) => {
            let arr_type = arr_val.get_type();
            let elem_type = arr_type.get_element_type();
            let len = arr_type.len();
            // Simplified: assume elem size based on type
            match elem_type {
                inkwell::types::BasicTypeEnum::IntType(int_type) => {
                    (int_type.get_bit_width() / 8) * len
                }
                inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                    if float_type == codegen.context.f32_type() {
                        4 * len
                    } else {
                        8 * len
                    }
                }
                _ => return Err("Cannot determine array element size".to_string()),
            }
        }
        _ => return Err("sizeof() cannot determine size of this type".to_string()),
    };

    Ok(codegen
        .context
        .i64_type()
        .const_int(size as u64, false)
        .into())
}

/// alignof<T>() - Get alignment of type in bytes
pub fn builtin_alignof<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("alignof() requires a value to determine type alignment".to_string());
    }

    let value = args[0];
    let alignment = match value {
        BasicValueEnum::IntValue(int_val) => {
            let int_type = int_val.get_type();
            let bit_width = int_type.get_bit_width();
            match bit_width {
                8 => 1,
                16 => 2,
                32 => 4,
                64 => 8,
                128 => 16,
                _ => (bit_width / 8).max(1),
            }
        }
        BasicValueEnum::FloatValue(float_val) => match float_val.get_type() {
            ty if ty == codegen.context.f32_type() => 4,
            ty if ty == codegen.context.f64_type() => 8,
            _ => return Err("Unknown float type".to_string()),
        },
        BasicValueEnum::PointerValue(_) => 8, // 64-bit pointer alignment
        BasicValueEnum::StructValue(struct_val) => {
            // For structs, alignment is the max alignment of all fields
            let struct_type = struct_val.get_type();
            let field_count = struct_type.count_fields();
            let mut max_align = 1u32;

            for i in 0..field_count {
                if let Some(field_type) = struct_type.get_field_type_at_index(i) {
                    let field_align = match field_type {
                        inkwell::types::BasicTypeEnum::IntType(int_type) => {
                            let bit_width = int_type.get_bit_width();
                            match bit_width {
                                8 => 1,
                                16 => 2,
                                32 => 4,
                                64 => 8,
                                128 => 16,
                                _ => (bit_width / 8).max(1),
                            }
                        }
                        inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                            if float_type == codegen.context.f32_type() {
                                4
                            } else {
                                8
                            }
                        }
                        inkwell::types::BasicTypeEnum::PointerType(_) => 8,
                        _ => 8, // Default to pointer alignment
                    };
                    max_align = max_align.max(field_align);
                }
            }
            max_align
        }
        BasicValueEnum::ArrayValue(arr_val) => {
            // Array alignment is same as element alignment
            let arr_type = arr_val.get_type();
            let elem_type = arr_type.get_element_type();
            match elem_type {
                inkwell::types::BasicTypeEnum::IntType(int_type) => {
                    let bit_width = int_type.get_bit_width();
                    match bit_width {
                        8 => 1,
                        16 => 2,
                        32 => 4,
                        64 => 8,
                        128 => 16,
                        _ => (bit_width / 8).max(1),
                    }
                }
                inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                    if float_type == codegen.context.f32_type() {
                        4
                    } else {
                        8
                    }
                }
                _ => return Err("Cannot determine array element alignment".to_string()),
            }
        }
        _ => return Err("alignof() cannot determine alignment of this type".to_string()),
    };

    Ok(codegen
        .context
        .i64_type()
        .const_int(alignment as u64, false)
        .into())
}
