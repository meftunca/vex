// Compiler hints: assume, likely, unlikely, prefetch

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// assume(condition) - Optimization hint that condition is true
pub fn builtin_assume<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("assume() takes exactly one argument".to_string());
    }

    let condition = args[0];
    let cond_bool = match condition {
        BasicValueEnum::IntValue(int_val) => {
            let int_type = int_val.get_type();
            // Convert to i1 if not already
            if int_type.get_bit_width() == 1 {
                // Already i1 (bool)
                int_val
            } else {
                // Convert to i1 by comparing with 0
                codegen
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        int_val,
                        int_type.const_int(0, false),
                        "cond_bool",
                    )
                    .map_err(|e| format!("Failed to convert condition to bool: {}", e))?
            }
        }
        _ => return Err("assume() condition must be a boolean or integer".to_string()),
    };

    // Declare llvm.assume
    let intrinsic =
        codegen.declare_llvm_intrinsic_void("llvm.assume", &[codegen.context.bool_type().into()]);

    codegen
        .builder
        .build_call(intrinsic, &[cond_bool.into()], "assume_call")
        .map_err(|e| format!("Failed to call assume: {}", e))?;

    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// likely(x) - Hint that condition is likely true
pub fn builtin_likely<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("likely() takes exactly one argument".to_string());
    }

    let condition = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("likely() argument must be an integer or boolean".to_string()),
    };

    // Use llvm.expect.i32(val, 1) for "likely true"
    let int_type = condition.get_type();
    let intrinsic = codegen.declare_llvm_intrinsic(
        &format!("llvm.expect.i{}", int_type.get_bit_width()),
        &[int_type.into(), int_type.into()],
        int_type.into(),
    );

    let expected = int_type.const_int(1, false);
    let result = codegen
        .builder
        .build_call(
            intrinsic,
            &[condition.into(), expected.into()],
            "likely_call",
        )
        .map_err(|e| format!("Failed to call likely: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// unlikely(x) - Hint that condition is likely false
pub fn builtin_unlikely<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("unlikely() takes exactly one argument".to_string());
    }

    let condition = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("unlikely() argument must be an integer or boolean".to_string()),
    };

    // Use llvm.expect.i32(val, 0) for "likely false"
    let int_type = condition.get_type();
    let intrinsic = codegen.declare_llvm_intrinsic(
        &format!("llvm.expect.i{}", int_type.get_bit_width()),
        &[int_type.into(), int_type.into()],
        int_type.into(),
    );

    let expected = int_type.const_int(0, false);
    let result = codegen
        .builder
        .build_call(
            intrinsic,
            &[condition.into(), expected.into()],
            "unlikely_call",
        )
        .map_err(|e| format!("Failed to call unlikely: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// prefetch(addr, rw, locality, cache_type) - Memory prefetch hint
pub fn builtin_prefetch<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 4 {
        return Err(
            "prefetch() takes exactly 4 arguments (addr, rw, locality, cache_type)".to_string(),
        );
    }

    let addr = match args[0] {
        BasicValueEnum::PointerValue(ptr_val) => ptr_val,
        _ => return Err("prefetch() first argument must be a pointer".to_string()),
    };

    let rw = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("prefetch() rw must be an integer".to_string()),
    };

    let locality = match args[2] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("prefetch() locality must be an integer".to_string()),
    };

    let cache_type = match args[3] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("prefetch() cache_type must be an integer".to_string()),
    };

    // Declare llvm.prefetch
    let i8_ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let intrinsic = codegen.declare_llvm_intrinsic_void(
        "llvm.prefetch.p0",
        &[
            i8_ptr_type.into(),
            codegen.context.i32_type().into(),
            codegen.context.i32_type().into(),
            codegen.context.i32_type().into(),
        ],
    );

    codegen
        .builder
        .build_call(
            intrinsic,
            &[addr.into(), rw.into(), locality.into(), cache_type.into()],
            "prefetch_call",
        )
        .map_err(|e| format!("Failed to call prefetch: {}", e))?;

    Ok(codegen.context.i32_type().const_int(0, false).into())
}
