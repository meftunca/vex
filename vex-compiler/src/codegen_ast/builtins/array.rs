// Runtime array functions: length, get, set, append

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// array_len(arr) - Get array length
pub fn builtin_array_len<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("array_len() takes exactly one argument".to_string());
    }

    let arr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("array_len() argument must be a pointer".to_string()),
    };

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_array_len = codegen.declare_runtime_fn(
        "vex_array_len",
        &[i8_ptr.into()],
        codegen.context.i64_type().into(),
    );

    let result = codegen
        .builder
        .build_call(vex_array_len, &[arr.into()], "array_len_call")
        .map_err(|e| format!("Failed to call array_len: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("array_len didn't return a value".to_string())
}

/// array_get(arr, index, elem_size) - Get array element
pub fn builtin_array_get<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err("array_get() takes exactly three arguments".to_string());
    }

    let arr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("array_get() first argument must be a pointer".to_string()),
    };

    let index = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("array_get() second argument must be an integer".to_string()),
    };

    let elem_size = match args[2] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("array_get() third argument must be an integer".to_string()),
    };

    let index_i64 = codegen
        .builder
        .build_int_z_extend(index, codegen.context.i64_type(), "index_cast")
        .map_err(|e| format!("Failed to cast index: {}", e))?;

    let elem_size_i64 = codegen
        .builder
        .build_int_z_extend(elem_size, codegen.context.i64_type(), "elem_size_cast")
        .map_err(|e| format!("Failed to cast elem_size: {}", e))?;

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_array_get = codegen.declare_runtime_fn(
        "vex_array_get",
        &[
            i8_ptr.into(),
            codegen.context.i64_type().into(),
            codegen.context.i64_type().into(),
        ],
        i8_ptr.into(),
    );

    let result = codegen
        .builder
        .build_call(
            vex_array_get,
            &[arr.into(), index_i64.into(), elem_size_i64.into()],
            "array_get_call",
        )
        .map_err(|e| format!("Failed to call array_get: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("array_get didn't return a value".to_string())
}

/// array_set(arr, index, elem, elem_size) - Set array element
pub fn builtin_array_set<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 4 {
        return Err("array_set() takes exactly four arguments".to_string());
    }

    let arr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("array_set() first argument must be a pointer".to_string()),
    };

    let index = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("array_set() second argument must be an integer".to_string()),
    };

    let elem = match args[2] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("array_set() third argument must be a pointer".to_string()),
    };

    let elem_size = match args[3] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("array_set() fourth argument must be an integer".to_string()),
    };

    let index_i64 = codegen
        .builder
        .build_int_z_extend(index, codegen.context.i64_type(), "index_cast")
        .map_err(|e| format!("Failed to cast index: {}", e))?;

    let elem_size_i64 = codegen
        .builder
        .build_int_z_extend(elem_size, codegen.context.i64_type(), "elem_size_cast")
        .map_err(|e| format!("Failed to cast elem_size: {}", e))?;

    // For void return type, use declare_runtime_fn_void
    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_array_set = codegen.declare_runtime_fn_void(
        "vex_array_set",
        &[
            i8_ptr.into(),
            codegen.context.i64_type().into(),
            i8_ptr.into(),
            codegen.context.i64_type().into(),
        ],
    );

    codegen
        .builder
        .build_call(
            vex_array_set,
            &[
                arr.into(),
                index_i64.into(),
                elem.into(),
                elem_size_i64.into(),
            ],
            "array_set_call",
        )
        .map_err(|e| format!("Failed to call array_set: {}", e))?;

    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// array_append(arr, elem, elem_size) - Append to array
pub fn builtin_array_append<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err("array_append() takes exactly three arguments".to_string());
    }

    let arr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("array_append() first argument must be a pointer".to_string()),
    };

    let elem = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("array_append() second argument must be a pointer".to_string()),
    };

    let elem_size = match args[2] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("array_append() third argument must be an integer".to_string()),
    };

    let elem_size_i64 = codegen
        .builder
        .build_int_z_extend(elem_size, codegen.context.i64_type(), "elem_size_cast")
        .map_err(|e| format!("Failed to cast elem_size: {}", e))?;

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_array_append = codegen.declare_runtime_fn(
        "vex_array_append",
        &[
            i8_ptr.into(),
            i8_ptr.into(),
            codegen.context.i64_type().into(),
        ],
        i8_ptr.into(),
    );

    let result = codegen
        .builder
        .build_call(
            vex_array_append,
            &[arr.into(), elem.into(), elem_size_i64.into()],
            "array_append_call",
        )
        .map_err(|e| format!("Failed to call array_append: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("array_append didn't return a value".to_string())
}
