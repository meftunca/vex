// Vex stdlib builtins - testing module
// Direct wrappers for vex_testing.c functions

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// testing.assert(condition: bool)
pub fn stdlib_testing_assert<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("testing.assert expects 1 argument".to_string());
    }

    // Get panic builtin
    let panic_fn = codegen
        .module
        .get_function("vex_panic")
        .ok_or("vex_panic not declared")?;

    let condition = args[0].into_int_value();

    // If condition is false, panic
    let then_block = codegen
        .context
        .append_basic_block(
            codegen.current_function.ok_or("No current function context")?,
            "assert_pass",
        );
    let else_block = codegen
        .context
        .append_basic_block(
            codegen.current_function.ok_or("No current function context")?,
            "assert_fail",
        );

    codegen
        .builder
        .build_conditional_branch(condition, then_block, else_block)
        .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

    // assert_fail block
    codegen.builder.position_at_end(else_block);
    let msg = codegen
        .builder
        .build_global_string_ptr("Assertion failed", "assert_msg")
        .map_err(|e| format!("Failed to build string: {}", e))?;
    codegen
        .builder
        .build_call(panic_fn, &[msg.as_pointer_value().into()], "panic_call")
        .map_err(|e| format!("Failed to call panic: {}", e))?;
    codegen
        .builder
        .build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    // assert_pass block
    codegen.builder.position_at_end(then_block);

    Ok(codegen.context.i8_type().const_zero().into())
}

/// testing.assert_eq<T>(left: T, right: T)
pub fn stdlib_testing_assert_eq<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("testing.assert_eq expects 2 arguments".to_string());
    }

    // Compare values (works for integers, floats need fcmp)
    let left = args[0];
    let right = args[1];

    let cmp = if left.is_int_value() {
        codegen
            .builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                left.into_int_value(),
                right.into_int_value(),
                "eq_cmp",
            )
            .map_err(|e| format!("Failed to build int compare: {}", e))?
    } else if left.is_float_value() {
        codegen
            .builder
            .build_float_compare(
                inkwell::FloatPredicate::OEQ,
                left.into_float_value(),
                right.into_float_value(),
                "eq_cmp",
            )
            .map_err(|e| format!("Failed to build float compare: {}", e))?
    } else {
        return Err("assert_eq only supports int/float types".to_string());
    };

    // If not equal, panic
    let panic_fn = codegen
        .module
        .get_function("vex_panic")
        .ok_or("vex_panic not declared")?;

    let then_block = codegen
        .context
        .append_basic_block(
            codegen.current_function.ok_or("No current function context")?,
            "assert_eq_pass",
        );
    let else_block = codegen
        .context
        .append_basic_block(
            codegen.current_function.ok_or("No current function context")?,
            "assert_eq_fail",
        );

    codegen
        .builder
        .build_conditional_branch(cmp, then_block, else_block)
        .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

    // assert_eq_fail block
    codegen.builder.position_at_end(else_block);
    let msg = codegen
        .builder
        .build_global_string_ptr("Assertion failed: values not equal", "assert_eq_msg")
        .map_err(|e| format!("Failed to build string: {}", e))?;
    codegen
        .builder
        .build_call(panic_fn, &[msg.as_pointer_value().into()], "panic_call")
        .map_err(|e| format!("Failed to call panic: {}", e))?;
    codegen
        .builder
        .build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    // assert_eq_pass block
    codegen.builder.position_at_end(then_block);

    Ok(codegen.context.i8_type().const_zero().into())
}

/// testing.assert_ne<T>(left: T, right: T)
pub fn stdlib_testing_assert_ne<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("testing.assert_ne expects 2 arguments".to_string());
    }

    let left = args[0];
    let right = args[1];

    let cmp = if left.is_int_value() {
        codegen
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                left.into_int_value(),
                right.into_int_value(),
                "ne_cmp",
            )
            .map_err(|e| format!("Failed to build int compare: {}", e))?
    } else if left.is_float_value() {
        codegen
            .builder
            .build_float_compare(
                inkwell::FloatPredicate::ONE,
                left.into_float_value(),
                right.into_float_value(),
                "ne_cmp",
            )
            .map_err(|e| format!("Failed to build float compare: {}", e))?
    } else {
        return Err("assert_ne only supports int/float types".to_string());
    };

    let panic_fn = codegen
        .module
        .get_function("vex_panic")
        .ok_or("vex_panic not declared")?;

    let then_block = codegen
        .context
        .append_basic_block(
            codegen.current_function.ok_or("No current function context")?,
            "assert_ne_pass",
        );
    let else_block = codegen
        .context
        .append_basic_block(
            codegen.current_function.ok_or("No current function context")?,
            "assert_ne_fail",
        );

    codegen
        .builder
        .build_conditional_branch(cmp, then_block, else_block)
        .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

    codegen.builder.position_at_end(else_block);
    let msg = codegen
        .builder
        .build_global_string_ptr("Assertion failed: values are equal", "assert_ne_msg")
        .map_err(|e| format!("Failed to build string: {}", e))?;
    codegen
        .builder
        .build_call(panic_fn, &[msg.as_pointer_value().into()], "panic_call")
        .map_err(|e| format!("Failed to call panic: {}", e))?;
    codegen
        .builder
        .build_unreachable()
        .map_err(|e| format!("Failed to build unreachable: {}", e))?;

    codegen.builder.position_at_end(then_block);

    Ok(codegen.context.i8_type().const_zero().into())
}
