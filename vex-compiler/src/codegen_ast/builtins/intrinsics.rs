// LLVM intrinsics: bit manipulation and overflow checking

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;

// ============================================================================
// BIT MANIPULATION INTRINSICS
// ============================================================================

/// ctlz(x) - Count leading zeros
pub fn builtin_ctlz<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("ctlz() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("ctlz() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.ctlz.i{}", int_type.get_bit_width());

    // Declare LLVM intrinsic - takes (value, is_zero_undef)
    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), codegen.context.bool_type().into()],
        int_type.into(),
    );

    // Call with is_zero_undef=false (return bit_width for zero input)
    let is_zero_undef = codegen.context.bool_type().const_int(0, false);
    let result = codegen
        .builder
        .build_call(
            intrinsic,
            &[value.into(), is_zero_undef.into()],
            "ctlz_call",
        )
        .map_err(|e| format!("Failed to call ctlz: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("ctlz didn't return a value".to_string())
}

/// cttz(x) - Count trailing zeros
pub fn builtin_cttz<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("cttz() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("cttz() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.cttz.i{}", int_type.get_bit_width());

    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), codegen.context.bool_type().into()],
        int_type.into(),
    );

    let is_zero_undef = codegen.context.bool_type().const_int(0, false);
    let result = codegen
        .builder
        .build_call(
            intrinsic,
            &[value.into(), is_zero_undef.into()],
            "cttz_call",
        )
        .map_err(|e| format!("Failed to call cttz: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("cttz didn't return a value".to_string())
}

/// ctpop(x) - Count population (number of 1 bits)
pub fn builtin_ctpop<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("ctpop() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("ctpop() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.ctpop.i{}", int_type.get_bit_width());

    let intrinsic =
        codegen.declare_llvm_intrinsic(&intrinsic_name, &[int_type.into()], int_type.into());

    let result = codegen
        .builder
        .build_call(intrinsic, &[value.into()], "ctpop_call")
        .map_err(|e| format!("Failed to call ctpop: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("ctpop didn't return a value".to_string())
}

/// bswap(x) - Byte swap (reverse byte order)
pub fn builtin_bswap<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("bswap() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("bswap() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.bswap.i{}", int_type.get_bit_width());

    let intrinsic =
        codegen.declare_llvm_intrinsic(&intrinsic_name, &[int_type.into()], int_type.into());

    let result = codegen
        .builder
        .build_call(intrinsic, &[value.into()], "bswap_call")
        .map_err(|e| format!("Failed to call bswap: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("bswap didn't return a value".to_string())
}

/// bitreverse(x) - Reverse all bits
pub fn builtin_bitreverse<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("bitreverse() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("bitreverse() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.bitreverse.i{}", int_type.get_bit_width());

    let intrinsic =
        codegen.declare_llvm_intrinsic(&intrinsic_name, &[int_type.into()], int_type.into());

    let result = codegen
        .builder
        .build_call(intrinsic, &[value.into()], "bitreverse_call")
        .map_err(|e| format!("Failed to call bitreverse: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("bitreverse didn't return a value".to_string())
}

// ============================================================================
// OVERFLOW CHECKING INTRINSICS
// ============================================================================

/// sadd_overflow(a, b) - Signed add with overflow check
pub fn builtin_sadd_overflow<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("sadd_overflow() takes exactly two arguments".to_string());
    }

    let a = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("sadd_overflow() first argument must be an integer".to_string()),
    };

    let b = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("sadd_overflow() second argument must be an integer".to_string()),
    };

    let int_type = a.get_type();
    let intrinsic_name = format!("llvm.sadd.with.overflow.i{}", int_type.get_bit_width());

    // Result type is {i32, i1} struct
    let result_struct_type = codegen.context.struct_type(
        &[int_type.into(), codegen.context.bool_type().into()],
        false,
    );

    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), int_type.into()],
        result_struct_type.into(),
    );

    let result = codegen
        .builder
        .build_call(intrinsic, &[a.into(), b.into()], "sadd_overflow_call")
        .map_err(|e| format!("Failed to call sadd_overflow: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("sadd_overflow didn't return a value".to_string())
}

/// ssub_overflow(a, b) - Signed subtract with overflow check
pub fn builtin_ssub_overflow<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("ssub_overflow() takes exactly two arguments".to_string());
    }

    let a = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("ssub_overflow() first argument must be an integer".to_string()),
    };

    let b = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("ssub_overflow() second argument must be an integer".to_string()),
    };

    let int_type = a.get_type();
    let intrinsic_name = format!("llvm.ssub.with.overflow.i{}", int_type.get_bit_width());

    let result_struct_type = codegen.context.struct_type(
        &[int_type.into(), codegen.context.bool_type().into()],
        false,
    );

    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), int_type.into()],
        result_struct_type.into(),
    );

    let result = codegen
        .builder
        .build_call(intrinsic, &[a.into(), b.into()], "ssub_overflow_call")
        .map_err(|e| format!("Failed to call ssub_overflow: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("ssub_overflow didn't return a value".to_string())
}

/// smul_overflow(a, b) - Signed multiply with overflow check
pub fn builtin_smul_overflow<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("smul_overflow() takes exactly two arguments".to_string());
    }

    let a = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("smul_overflow() first argument must be an integer".to_string()),
    };

    let b = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("smul_overflow() second argument must be an integer".to_string()),
    };

    let int_type = a.get_type();
    let intrinsic_name = format!("llvm.smul.with.overflow.i{}", int_type.get_bit_width());

    let result_struct_type = codegen.context.struct_type(
        &[int_type.into(), codegen.context.bool_type().into()],
        false,
    );

    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), int_type.into()],
        result_struct_type.into(),
    );

    let result = codegen
        .builder
        .build_call(intrinsic, &[a.into(), b.into()], "smul_overflow_call")
        .map_err(|e| format!("Failed to call smul_overflow: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("smul_overflow didn't return a value".to_string())
}
