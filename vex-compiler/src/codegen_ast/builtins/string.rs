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
        &[codegen.context.ptr_type(AddressSpace::default()).into()],
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

    let i8_ptr = codegen.context.ptr_type(AddressSpace::default());
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

    let i8_ptr = codegen.context.ptr_type(AddressSpace::default());
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

    let i8_ptr = codegen.context.ptr_type(AddressSpace::default());
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

    let i8_ptr = codegen.context.ptr_type(AddressSpace::default());
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

/// vex_str_contains(s: str, substr: str): bool - Check if string contains substring
pub fn builtin_str_contains<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("vex_str_contains() takes exactly two arguments".to_string());
    }

    let s = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_contains() first argument must be a string pointer".to_string()),
    };

    let substr = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_contains() second argument must be a string pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_str_contains = codegen.declare_runtime_fn(
        "vex_str_contains",
        &[ptr_type.into(), ptr_type.into()],
        codegen.context.bool_type().into(),
    );

    let result = codegen
        .builder
        .build_call(vex_str_contains, &[s.into(), substr.into()], "str_contains")
        .map_err(|e| format!("Failed to call vex_str_contains: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_str_has_prefix(s: str, prefix: str): bool - Check if string starts with prefix
pub fn builtin_str_has_prefix<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("vex_str_has_prefix() takes exactly two arguments".to_string());
    }

    let s = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_has_prefix() first argument must be a string pointer".to_string()),
    };

    let prefix = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_has_prefix() second argument must be a string pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_str_has_prefix = codegen.declare_runtime_fn(
        "vex_str_has_prefix",
        &[ptr_type.into(), ptr_type.into()],
        codegen.context.bool_type().into(),
    );

    let result = codegen
        .builder
        .build_call(vex_str_has_prefix, &[s.into(), prefix.into()], "str_has_prefix")
        .map_err(|e| format!("Failed to call vex_str_has_prefix: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_str_has_suffix(s: str, suffix: str): bool - Check if string ends with suffix
pub fn builtin_str_has_suffix<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("vex_str_has_suffix() takes exactly two arguments".to_string());
    }

    let s = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_has_suffix() first argument must be a string pointer".to_string()),
    };

    let suffix = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_has_suffix() second argument must be a string pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_str_has_suffix = codegen.declare_runtime_fn(
        "vex_str_has_suffix",
        &[ptr_type.into(), ptr_type.into()],
        codegen.context.bool_type().into(),
    );

    let result = codegen
        .builder
        .build_call(vex_str_has_suffix, &[s.into(), suffix.into()], "str_has_suffix")
        .map_err(|e| format!("Failed to call vex_str_has_suffix: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_str_to_upper(s: str): str - Convert string to uppercase (ASCII only)
pub fn builtin_str_to_upper<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("vex_str_to_upper() takes exactly one argument".to_string());
    }

    let s = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_to_upper() argument must be a string pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_str_to_upper = codegen.declare_runtime_fn(
        "vex_str_to_upper",
        &[ptr_type.into()],
        ptr_type.into(),
    );

    let result = codegen
        .builder
        .build_call(vex_str_to_upper, &[s.into()], "str_to_upper")
        .map_err(|e| format!("Failed to call vex_str_to_upper: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_str_to_lower(s: str): str - Convert string to lowercase (ASCII only)
pub fn builtin_str_to_lower<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("vex_str_to_lower() takes exactly one argument".to_string());
    }

    let s = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_to_lower() argument must be a string pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_str_to_lower = codegen.declare_runtime_fn(
        "vex_str_to_lower",
        &[ptr_type.into()],
        ptr_type.into(),
    );

    let result = codegen
        .builder
        .build_call(vex_str_to_lower, &[s.into()], "str_to_lower")
        .map_err(|e| format!("Failed to call vex_str_to_lower: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_str_trim(s: str): str - Trim whitespace from both ends
pub fn builtin_str_trim<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("vex_str_trim() takes exactly one argument".to_string());
    }

    let s = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_trim() argument must be a string pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_str_trim = codegen.declare_runtime_fn(
        "vex_str_trim",
        &[ptr_type.into()],
        ptr_type.into(),
    );

    let result = codegen
        .builder
        .build_call(vex_str_trim, &[s.into()], "str_trim")
        .map_err(|e| format!("Failed to call vex_str_trim: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_str_replace(s: str, old: str, new: str): str - Replace all occurrences
pub fn builtin_str_replace<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err("vex_str_replace() takes exactly three arguments".to_string());
    }

    let s = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_replace() first argument must be a string pointer".to_string()),
    };

    let old = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_replace() second argument must be a string pointer".to_string()),
    };

    let new = match args[2] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_replace() third argument must be a string pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_str_replace = codegen.declare_runtime_fn(
        "vex_str_replace",
        &[ptr_type.into(), ptr_type.into(), ptr_type.into()],
        ptr_type.into(),
    );

    let result = codegen
        .builder
        .build_call(vex_str_replace, &[s.into(), old.into(), new.into()], "str_replace")
        .map_err(|e| format!("Failed to call vex_str_replace: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// vex_str_split(s: str, delim: str): ptr - Split string by delimiter
/// Returns char** (NULL-terminated array of strings)
pub fn builtin_str_split<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("vex_str_split() takes exactly two arguments".to_string());
    }

    let s = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_split() first argument must be a string pointer".to_string()),
    };

    let delim = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("vex_str_split() second argument must be a string pointer".to_string()),
    };

    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let i64_type = codegen.context.i64_type();
    
    // Allocate space for count output parameter
    let count_ptr = codegen
        .builder
        .build_alloca(i64_type, "split_count")
        .map_err(|e| format!("Failed to allocate count: {}", e))?;

    let vex_str_split = codegen.declare_runtime_fn(
        "vex_str_split",
        &[ptr_type.into(), ptr_type.into(), ptr_type.into()],
        ptr_type.into(),
    );

    let result = codegen
        .builder
        .build_call(vex_str_split, &[s.into(), delim.into(), count_ptr.into()], "str_split")
        .map_err(|e| format!("Failed to call vex_str_split: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}
