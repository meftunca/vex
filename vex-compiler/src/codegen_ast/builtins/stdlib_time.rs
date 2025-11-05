// Vex stdlib builtins - time module
// Direct wrappers for vex_time.c functions

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// time.now(): i64
pub fn stdlib_time_now<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if !args.is_empty() {
        return Err("time.now expects 0 arguments".to_string());
    }

    // Get vex_time_now function (returns i64 milliseconds)
    let time_now_fn = codegen
        .module
        .get_function("vex_time_now")
        .ok_or("vex_time_now not declared")?;

    let call = codegen
        .builder
        .build_call(time_now_fn, &[], "time_now_call")
        .map_err(|e| format!("Failed to call vex_time_now: {}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("vex_time_now should return i64")?;

    Ok(call)
}

/// time.high_res(): i64
pub fn stdlib_time_high_res<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if !args.is_empty() {
        return Err("time.high_res expects 0 arguments".to_string());
    }

    // Get vex_time_monotonic function (returns i64 nanoseconds)
    let monotonic_fn = codegen
        .module
        .get_function("vex_time_monotonic")
        .ok_or("vex_time_monotonic not declared")?;

    let call = codegen
        .builder
        .build_call(monotonic_fn, &[], "time_monotonic_call")
        .map_err(|e| format!("Failed to call vex_time_monotonic: {}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("vex_time_monotonic should return i64")?;

    Ok(call)
}

/// time.sleep_ms(ms: i64)
pub fn stdlib_time_sleep_ms<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("time.sleep_ms expects 1 argument".to_string());
    }

    // Get vex_time_sleep function
    let sleep_fn = codegen
        .module
        .get_function("vex_time_sleep")
        .ok_or("vex_time_sleep not declared")?;

    codegen
        .builder
        .build_call(sleep_fn, &[args[0].into()], "sleep_call")
        .map_err(|e| format!("Failed to call vex_time_sleep: {}", e))?;

    // Return void
    Ok(codegen.context.i8_type().const_zero().into())
}
