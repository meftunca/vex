// Runtime string functions: strlen, strcmp, strcpy, strcat, strdup

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// strlen(s) - Get string length
pub fn builtin_strlen<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("strlen() takes exactly one argument".to_string());
    }

    let str_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("strlen() argument must be a string pointer".to_string()),
    };

    let vex_strlen = codegen.declare_runtime_fn(
        "vex_strlen",
        &[codegen
            .context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .into()],
        codegen.context.i64_type().into(),
    );

    let result = codegen
        .builder
        .build_call(vex_strlen, &[str_ptr.into()], "strlen_call")
        .map_err(|e| format!("Failed to call strlen: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// strcmp(s1, s2) - Compare strings
pub fn builtin_strcmp<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("strcmp() takes exactly two arguments".to_string());
    }

    let s1 = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("strcmp() first argument must be a string pointer".to_string()),
    };

    let s2 = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("strcmp() second argument must be a string pointer".to_string()),
    };

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_strcmp = codegen.declare_runtime_fn(
        "vex_strcmp",
        &[i8_ptr.into(), i8_ptr.into()],
        codegen.context.i32_type().into(),
    );

    let result = codegen
        .builder
        .build_call(vex_strcmp, &[s1.into(), s2.into()], "strcmp_call")
        .map_err(|e| format!("Failed to call strcmp: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// strcpy(dest, src) - Copy string
pub fn builtin_strcpy<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("strcpy() takes exactly two arguments".to_string());
    }

    let dest = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("strcpy() first argument must be a pointer".to_string()),
    };

    let src = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("strcpy() second argument must be a pointer".to_string()),
    };

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_strcpy =
        codegen.declare_runtime_fn("vex_strcpy", &[i8_ptr.into(), i8_ptr.into()], i8_ptr.into());

    let result = codegen
        .builder
        .build_call(vex_strcpy, &[dest.into(), src.into()], "strcpy_call")
        .map_err(|e| format!("Failed to call strcpy: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// strcat(dest, src) - Concatenate strings
pub fn builtin_strcat<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("strcat() takes exactly two arguments".to_string());
    }

    let dest = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("strcat() first argument must be a pointer".to_string()),
    };

    let src = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("strcat() second argument must be a pointer".to_string()),
    };

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_strcat =
        codegen.declare_runtime_fn("vex_strcat", &[i8_ptr.into(), i8_ptr.into()], i8_ptr.into());

    let result = codegen
        .builder
        .build_call(vex_strcat, &[dest.into(), src.into()], "strcat_call")
        .map_err(|e| format!("Failed to call strcat: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// strdup(s) - Duplicate string
pub fn builtin_strdup<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("strdup() takes exactly one argument".to_string());
    }

    let str_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("strdup() argument must be a string pointer".to_string()),
    };

    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_strdup = codegen.declare_runtime_fn("vex_strdup", &[i8_ptr.into()], i8_ptr.into());

    let result = codegen
        .builder
        .build_call(vex_strdup, &[str_ptr.into()], "strdup_call")
        .map_err(|e| format!("Failed to call strdup: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_string_as_cstr(s: *String): *u8 - Get raw C string pointer from String
pub fn builtin_string_as_cstr<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("vex_string_as_cstr() takes exactly one argument".to_string());
    }

    let str_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_string_as_cstr() argument must be a String pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_string_as_cstr =
        codegen.declare_runtime_fn("vex_string_as_cstr", &[ptr_type.into()], ptr_type.into());

    let result = codegen
        .builder
        .build_call(vex_string_as_cstr, &[str_ptr.into()], "string_as_cstr")
        .map_err(|e| format!("Failed to call vex_string_as_cstr: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_string_len(s: *String): u64 - Get length of String in bytes
pub fn builtin_string_len<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("vex_string_len() takes exactly one argument".to_string());
    }

    let str_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_string_len() argument must be a String pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let i64_type = codegen.context.i64_type();
    let vex_string_len =
        codegen.declare_runtime_fn("vex_string_len", &[ptr_type.into()], i64_type.into());

    let result = codegen
        .builder
        .build_call(vex_string_len, &[str_ptr.into()], "string_len")
        .map_err(|e| format!("Failed to call vex_string_len: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}
