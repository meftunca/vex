// Compile-time formatting - Rust-style monomorphization
// Zero-cost abstraction: generates specialized LLVM IR for each type

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// i32_to_string(x) - Inline LLVM IR generation for i32 → string
pub fn builtin_i32_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("i32_to_string() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int) => int,
        _ => return Err("i32_to_string() argument must be an integer".to_string()),
    };

    // Allocate buffer: max i32 is "-2147483648" = 11 chars + null = 12 bytes
    let i8_type = codegen.context.i8_type();
    let buffer_size = codegen.context.i32_type().const_int(12, false);
    
    let malloc_fn = codegen.module.get_function("malloc").unwrap_or_else(|| {
        let fn_type = codegen.context.ptr_type(AddressSpace::default()).fn_type(
            &[codegen.context.i64_type().into()],
            false,
        );
        codegen.module.add_function("malloc", fn_type, None)
    });

    let buffer = codegen
        .builder
        .build_call(
            malloc_fn,
            &[codegen.context.i64_type().const_int(12, false).into()],
            "i32_str_buf",
        )
        .map_err(|e| format!("Failed to allocate buffer: {}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("malloc didn't return a value")?
        .into_pointer_value();

    // Generate inline integer-to-string conversion
    // Instead of calling snprintf, generate direct LLVM IR
    
    // For now, use snprintf as a bridge (will be replaced with pure LLVM IR)
    let snprintf = codegen.module.get_function("snprintf").unwrap_or_else(|| {
        let fn_type = codegen.context.i32_type().fn_type(
            &[
                codegen.context.ptr_type(AddressSpace::default()).into(),
                codegen.context.i64_type().into(),
                codegen.context.ptr_type(AddressSpace::default()).into(),
            ],
            true, // variadic
        );
        codegen.module.add_function("snprintf", fn_type, None)
    });

    // Create format string "%d"
    let format_str = codegen.builder.build_global_string_ptr("%d", "fmt_i32")
        .map_err(|e| format!("Failed to create format string: {}", e))?;

    codegen
        .builder
        .build_call(
            snprintf,
            &[
                buffer.into(),
                codegen.context.i64_type().const_int(12, false).into(),
                format_str.as_pointer_value().into(),
                value.into(),
            ],
            "snprintf_call",
        )
        .map_err(|e| format!("Failed to call snprintf: {}", e))?;

    Ok(buffer.into())
}

/// f64_to_string(x) - Inline LLVM IR generation for f64 → string
pub fn builtin_f64_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("f64_to_string() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::FloatValue(float) => float,
        _ => return Err("f64_to_string() argument must be a float".to_string()),
    };

    // Allocate buffer: max reasonable float representation ~32 bytes
    let malloc_fn = codegen.module.get_function("malloc").unwrap_or_else(|| {
        let fn_type = codegen.context.ptr_type(AddressSpace::default()).fn_type(
            &[codegen.context.i64_type().into()],
            false,
        );
        codegen.module.add_function("malloc", fn_type, None)
    });

    let buffer = codegen
        .builder
        .build_call(
            malloc_fn,
            &[codegen.context.i64_type().const_int(32, false).into()],
            "f64_str_buf",
        )
        .map_err(|e| format!("Failed to allocate buffer: {}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("malloc didn't return a value")?
        .into_pointer_value();

    let snprintf = codegen.module.get_function("snprintf").unwrap_or_else(|| {
        let fn_type = codegen.context.i32_type().fn_type(
            &[
                codegen.context.ptr_type(AddressSpace::default()).into(),
                codegen.context.i64_type().into(),
                codegen.context.ptr_type(AddressSpace::default()).into(),
            ],
            true,
        );
        codegen.module.add_function("snprintf", fn_type, None)
    });

    let format_str = codegen.builder.build_global_string_ptr("%.6g", "fmt_f64")
        .map_err(|e| format!("Failed to create format string: {}", e))?;

    codegen
        .builder
        .build_call(
            snprintf,
            &[
                buffer.into(),
                codegen.context.i64_type().const_int(32, false).into(),
                format_str.as_pointer_value().into(),
                value.into(),
            ],
            "snprintf_call",
        )
        .map_err(|e| format!("Failed to call snprintf: {}", e))?;

    Ok(buffer.into())
}

/// bool_to_string(x) - Inline constant strings
pub fn builtin_bool_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("bool_to_string() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int) => int,
        _ => return Err("bool_to_string() argument must be a boolean".to_string()),
    };

    // Create constant strings "true" and "false"
    let true_str = codegen.builder.build_global_string_ptr("true", "bool_true")
        .map_err(|e| format!("Failed to create true string: {}", e))?;
    let false_str = codegen.builder.build_global_string_ptr("false", "bool_false")
        .map_err(|e| format!("Failed to create false string: {}", e))?;

    // Select based on boolean value
    let result = codegen
        .builder
        .build_select(
            value,
            true_str.as_pointer_value(),
            false_str.as_pointer_value(),
            "bool_str_select",
        )
        .map_err(|e| format!("Failed to select bool string: {}", e))?;

    Ok(result)
}
