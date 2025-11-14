// Runtime UTF-8 functions: validation and character operations

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// utf8_valid(s, len) - Validate UTF-8 string
pub fn builtin_utf8_valid<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("utf8_valid() takes exactly two arguments".to_string());
    }

    let str_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("utf8_valid() first argument must be a pointer".to_string()),
    };

    let len = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("utf8_valid() second argument must be an integer".to_string()),
    };

    let len_i64 = codegen
        .builder
        .build_int_z_extend(len, codegen.context.i64_type(), "len_cast")
        .map_err(|e| format!("Failed to cast length: {}", e))?;

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_utf8_valid = codegen.declare_runtime_fn(
        "vex_utf8_valid",
        &[i8_ptr.into(), codegen.context.i64_type().into()],
        codegen.context.bool_type().into(),
    );

    let result = codegen
        .builder
        .build_call(
            vex_utf8_valid,
            &[str_ptr.into(), len_i64.into()],
            "utf8_valid_call",
        )
        .map_err(|e| format!("Failed to call utf8_valid: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// utf8_char_count(s) - Count UTF-8 characters
pub fn builtin_utf8_char_count<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("utf8_char_count() takes exactly one argument".to_string());
    }

    let str_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("utf8_char_count() argument must be a pointer".to_string()),
    };

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_utf8_char_count = codegen.declare_runtime_fn(
        "vex_utf8_char_count",
        &[i8_ptr.into()],
        codegen.context.i64_type().into(),
    );

    let result = codegen
        .builder
        .build_call(
            vex_utf8_char_count,
            &[str_ptr.into()],
            "utf8_char_count_call",
        )
        .map_err(|e| format!("Failed to call utf8_char_count: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// utf8_char_at(s, index) - Get character at index
pub fn builtin_utf8_char_at<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("utf8_char_at() takes exactly two arguments".to_string());
    }

    let str_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("utf8_char_at() first argument must be a pointer".to_string()),
    };

    let index = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("utf8_char_at() second argument must be an integer".to_string()),
    };

    let index_i64 = codegen
        .builder
        .build_int_z_extend(index, codegen.context.i64_type(), "index_cast")
        .map_err(|e| format!("Failed to cast index: {}", e))?;

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_utf8_char_at = codegen.declare_runtime_fn(
        "vex_utf8_char_at",
        &[i8_ptr.into(), codegen.context.i64_type().into()],
        i8_ptr.into(),
    );

    let result = codegen
        .builder
        .build_call(
            vex_utf8_char_at,
            &[str_ptr.into(), index_i64.into()],
            "utf8_char_at_call",
        )
        .map_err(|e| format!("Failed to call utf8_char_at: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}
