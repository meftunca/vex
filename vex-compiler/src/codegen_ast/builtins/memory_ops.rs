// Runtime memory operations: memcpy, memset, memcmp, memmove

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// memcpy(dest, src, n) - Copy memory
pub fn builtin_memcpy<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err("memcpy() takes exactly three arguments".to_string());
    }

    let dest = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("memcpy() first argument must be a pointer".to_string()),
    };

    let src = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("memcpy() second argument must be a pointer".to_string()),
    };

    let n = match args[2] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("memcpy() third argument must be an integer".to_string()),
    };

    // Cast to i64 if needed
    let n_i64 = codegen
        .builder
        .build_int_z_extend(n, codegen.context.i64_type(), "n_cast")
        .map_err(|e| format!("Failed to cast size: {}", e))?;

    let i8_ptr = codegen.context.ptr_type(AddressSpace::default());
    let vex_memcpy = codegen.declare_runtime_fn(
        "vex_memcpy",
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
            vex_memcpy,
            &[dest.into(), src.into(), n_i64.into()],
            "memcpy_call",
        )
        .map_err(|e| format!("Failed to call memcpy: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// memset(s, c, n) - Fill memory
pub fn builtin_memset<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err("memset() takes exactly three arguments".to_string());
    }

    let ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("memset() first argument must be a pointer".to_string()),
    };

    let c = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("memset() second argument must be an integer".to_string()),
    };

    let n = match args[2] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("memset() third argument must be an integer".to_string()),
    };

    let n_i64 = codegen
        .builder
        .build_int_z_extend(n, codegen.context.i64_type(), "n_cast")
        .map_err(|e| format!("Failed to cast size: {}", e))?;

    let i8_ptr = codegen.context.ptr_type(AddressSpace::default());
    let vex_memset = codegen.declare_runtime_fn(
        "vex_memset",
        &[
            i8_ptr.into(),
            codegen.context.i32_type().into(),
            codegen.context.i64_type().into(),
        ],
        i8_ptr.into(),
    );

    let result = codegen
        .builder
        .build_call(
            vex_memset,
            &[ptr.into(), c.into(), n_i64.into()],
            "memset_call",
        )
        .map_err(|e| format!("Failed to call memset: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// memcmp(s1, s2, n) - Compare memory
pub fn builtin_memcmp<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err("memcmp() takes exactly three arguments".to_string());
    }

    let s1 = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("memcmp() first argument must be a pointer".to_string()),
    };

    let s2 = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("memcmp() second argument must be a pointer".to_string()),
    };

    let n = match args[2] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("memcmp() third argument must be an integer".to_string()),
    };

    let n_i64 = codegen
        .builder
        .build_int_z_extend(n, codegen.context.i64_type(), "n_cast")
        .map_err(|e| format!("Failed to cast size: {}", e))?;

    let i8_ptr = codegen.context.ptr_type(AddressSpace::default());
    let vex_memcmp = codegen.declare_runtime_fn(
        "vex_memcmp",
        &[
            i8_ptr.into(),
            i8_ptr.into(),
            codegen.context.i64_type().into(),
        ],
        codegen.context.i32_type().into(),
    );

    let result = codegen
        .builder
        .build_call(
            vex_memcmp,
            &[s1.into(), s2.into(), n_i64.into()],
            "memcmp_call",
        )
        .map_err(|e| format!("Failed to call memcmp: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// memmove(dest, src, n) - Move memory (handles overlap)
pub fn builtin_memmove<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err("memmove() takes exactly three arguments".to_string());
    }

    let dest = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("memmove() first argument must be a pointer".to_string()),
    };

    let src = match args[1] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("memmove() second argument must be a pointer".to_string()),
    };

    let n = match args[2] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("memmove() third argument must be an integer".to_string()),
    };

    let n_i64 = codegen
        .builder
        .build_int_z_extend(n, codegen.context.i64_type(), "n_cast")
        .map_err(|e| format!("Failed to cast size: {}", e))?;

    let i8_ptr = codegen.context.ptr_type(AddressSpace::default());
    let vex_memmove = codegen.declare_runtime_fn(
        "vex_memmove",
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
            vex_memmove,
            &[dest.into(), src.into(), n_i64.into()],
            "memmove_call",
        )
        .map_err(|e| format!("Failed to call memmove: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}
