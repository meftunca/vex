// Core builtin functions: print, println, panic, assert, unreachable

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// print(value) - Output without newline
pub fn builtin_print<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("print() takes exactly one argument".to_string());
    }

    let val = args[0];

    // Determine format string based on type (NO newline)
    match val {
        BasicValueEnum::IntValue(int_val) => {
            // Check bit width: i32 vs i64
            let bit_width = int_val.get_type().get_bit_width();
            if bit_width == 64 {
                codegen.build_printf("%lld", &[val])?;
            } else {
                codegen.build_printf("%d", &[val])?;
            }
        }
        BasicValueEnum::FloatValue(float_val) => {
            // Check if f32 or f64
            let float_type = float_val.get_type();
            if float_type == codegen.context.f64_type() {
                codegen.build_printf("%g", &[val])?; // %g for cleaner output
            } else {
                codegen.build_printf("%g", &[val])?;
            }
        }
        BasicValueEnum::PointerValue(_) => {
            // String (i8* pointer)
            codegen.build_printf("%s", &[val])?;
        }
        _ => {
            return Err(format!("print() doesn't support this type yet: {:?}", val));
        }
    }

    // Return void (i32 0 as dummy)
    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// println(value) - Output with newline
pub fn builtin_println<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("println() takes exactly one argument".to_string());
    }

    let val = args[0];

    // Determine format string based on type (WITH newline)
    match val {
        BasicValueEnum::IntValue(int_val) => {
            // Check bit width: i32 vs i64
            let bit_width = int_val.get_type().get_bit_width();
            if bit_width == 64 {
                codegen.build_printf("%lld\n", &[val])?;
            } else {
                codegen.build_printf("%d\n", &[val])?;
            }
        }
        BasicValueEnum::FloatValue(float_val) => {
            // Check if f32 or f64
            let float_type = float_val.get_type();
            if float_type == codegen.context.f64_type() {
                codegen.build_printf("%g\n", &[val])?;
            } else {
                codegen.build_printf("%g\n", &[val])?;
            }
        }
        BasicValueEnum::PointerValue(_) => {
            // String (i8* pointer)
            codegen.build_printf("%s\n", &[val])?;
        }
        _ => {
            return Err(format!(
                "println() doesn't support this type yet: {:?}",
                val
            ));
        }
    }

    // Return void (i32 0 as dummy)
    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// panic(message) - Abort program with error message
pub fn builtin_panic<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("panic() requires at least one argument".to_string());
    }

    let message = args[0];

    // Print error message to stderr
    match message {
        BasicValueEnum::PointerValue(_) => {
            // Print "panic: <message>\n"
            codegen.build_printf("panic: %s\n", &[message])?;
        }
        _ => {
            codegen.build_printf("panic!\n", &[])?;
        }
    }

    // Call abort() to terminate
    let abort_fn = codegen.declare_abort();
    codegen
        .builder
        .build_call(abort_fn, &[], "abort_call")
        .map_err(|e| format!("Failed to call abort: {}", e))?;

    // Unreachable after abort
    codegen
        .builder
        .build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    // Return dummy value (never reached)
    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// assert(condition, message?) - Runtime assertion
pub fn builtin_assert<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("assert() requires at least one argument".to_string());
    }

    let condition = args[0];
    let message = args.get(1);

    // Check if condition is false
    let cond_bool = match condition {
        BasicValueEnum::IntValue(int_val) => {
            // Convert to i1
            codegen
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    int_val,
                    codegen.context.i32_type().const_int(0, false),
                    "assert_cond",
                )
                .map_err(|e| format!("Failed to compare: {}", e))?
        }
        _ => {
            return Err("assert() condition must be boolean".to_string());
        }
    };

    // Create basic blocks
    let current_fn = codegen.current_function.ok_or("No current function")?;
    let then_block = codegen
        .context
        .append_basic_block(current_fn, "assert_pass");
    let else_block = codegen
        .context
        .append_basic_block(current_fn, "assert_fail");

    // Branch on condition
    codegen
        .builder
        .build_conditional_branch(cond_bool, then_block, else_block)
        .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

    // Else block: assertion failed
    codegen.builder.position_at_end(else_block);
    if let Some(msg) = message {
        codegen.build_printf("assertion failed: %s\n", &[*msg])?;
    } else {
        codegen.build_printf("assertion failed\n", &[])?;
    }

    let abort_fn = codegen.declare_abort();
    codegen
        .builder
        .build_call(abort_fn, &[], "abort_call")
        .map_err(|e| format!("Failed to call abort: {}", e))?;
    codegen
        .builder
        .build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    // Then block: continue
    codegen.builder.position_at_end(then_block);

    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// unreachable() - Mark code as unreachable (optimization hint + runtime trap)
pub fn builtin_unreachable<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Build LLVM unreachable instruction
    codegen
        .builder
        .build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    // Return a dummy value (never reached)
    Ok(codegen.context.i32_type().const_int(0, false).into())
}
