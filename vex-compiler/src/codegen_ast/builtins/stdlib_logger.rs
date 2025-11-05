// Vex stdlib builtins - logger module
// Direct wrappers for vex_io.c functions

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// logger.debug(msg: string)
pub fn stdlib_logger_debug<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("logger.debug expects 1 argument".to_string());
    }

    // Get vex_print and vex_println functions
    let print_fn = codegen
        .module
        .get_function("vex_print")
        .ok_or("vex_print not declared")?;
    let println_fn = codegen
        .module
        .get_function("vex_println")
        .ok_or("vex_println not declared")?;

    // Print "[DEBUG] "
    let debug_prefix = codegen
        .builder
        .build_global_string_ptr("[DEBUG] ", "debug_prefix")
        .map_err(|e| format!("Failed to build string: {}", e))?;
    codegen
        .builder
        .build_call(
            print_fn,
            &[debug_prefix.as_pointer_value().into()],
            "print_debug_prefix",
        )
        .map_err(|e| format!("Failed to call print: {}", e))?;

    // Print message + newline
    codegen
        .builder
        .build_call(println_fn, &[args[0].into()], "print_msg")
        .map_err(|e| format!("Failed to call println: {}", e))?;

    // Return void (unit type)
    Ok(codegen.context.i8_type().const_zero().into())
}

/// logger.info(msg: string)
pub fn stdlib_logger_info<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("logger.info expects 1 argument".to_string());
    }

    let print_fn = codegen
        .module
        .get_function("vex_print")
        .ok_or("vex_print not declared")?;
    let println_fn = codegen
        .module
        .get_function("vex_println")
        .ok_or("vex_println not declared")?;

    let info_prefix = codegen
        .builder
        .build_global_string_ptr("[INFO] ", "info_prefix")
        .map_err(|e| format!("Failed to build string: {}", e))?;
    codegen
        .builder
        .build_call(
            print_fn,
            &[info_prefix.as_pointer_value().into()],
            "print_info_prefix",
        )
        .map_err(|e| format!("Failed to call print: {}", e))?;

    codegen
        .builder
        .build_call(println_fn, &[args[0].into()], "print_msg")
        .map_err(|e| format!("Failed to call println: {}", e))?;

    Ok(codegen.context.i8_type().const_zero().into())
}

/// logger.warn(msg: string)
pub fn stdlib_logger_warn<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("logger.warn expects 1 argument".to_string());
    }

    let print_fn = codegen
        .module
        .get_function("vex_print")
        .ok_or("vex_print not declared")?;
    let println_fn = codegen
        .module
        .get_function("vex_println")
        .ok_or("vex_println not declared")?;

    let warn_prefix = codegen
        .builder
        .build_global_string_ptr("[WARN] ", "warn_prefix")
        .map_err(|e| format!("Failed to build string: {}", e))?;
    codegen
        .builder
        .build_call(
            print_fn,
            &[warn_prefix.as_pointer_value().into()],
            "print_warn_prefix",
        )
        .map_err(|e| format!("Failed to call print: {}", e))?;

    codegen
        .builder
        .build_call(println_fn, &[args[0].into()], "print_msg")
        .map_err(|e| format!("Failed to call println: {}", e))?;

    Ok(codegen.context.i8_type().const_zero().into())
}

/// logger.error(msg: string)
pub fn stdlib_logger_error<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("logger.error expects 1 argument".to_string());
    }

    let print_fn = codegen
        .module
        .get_function("vex_print")
        .ok_or("vex_print not declared")?;
    let println_fn = codegen
        .module
        .get_function("vex_println")
        .ok_or("vex_println not declared")?;

    let error_prefix = codegen
        .builder
        .build_global_string_ptr("[ERROR] ", "error_prefix")
        .map_err(|e| format!("Failed to build string: {}", e))?;
    codegen
        .builder
        .build_call(
            print_fn,
            &[error_prefix.as_pointer_value().into()],
            "print_error_prefix",
        )
        .map_err(|e| format!("Failed to call print: {}", e))?;

    codegen
        .builder
        .build_call(println_fn, &[args[0].into()], "print_msg")
        .map_err(|e| format!("Failed to call println: {}", e))?;

    Ok(codegen.context.i8_type().const_zero().into())
}
