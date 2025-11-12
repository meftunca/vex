// Assertion and panic functions (panic, assert, unreachable)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;

pub fn builtin_print<'ctx>(
    _codegen: &mut ASTCodeGen<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    Err("print() should be handled by compile_print_call()".to_string())
}

/// println(...values) - DEPRECATED (use compile_print_call instead)
pub fn builtin_println<'ctx>(
    _codegen: &mut ASTCodeGen<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    Err("println() should be handled by compile_print_call()".to_string())
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
