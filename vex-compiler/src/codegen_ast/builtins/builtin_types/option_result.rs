// Option and Result type constructors

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// Builtin: Some(value: T) -> Option<T>
/// Creates Option<T> with Some variant (tag=1, value)
/// Memory layout: { u8 tag, T value }
pub fn builtin_option_some<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("Some() requires exactly 1 argument".to_string());
    }

    let value = args[0];
    let value_type = value.get_type();

    // Option<T> layout: { i32 tag, T value }

    // Allocate Option<T> on stack
    let option_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(), // tag
            value_type,                        // value
        ],
        false,
    );

    let option_ptr = codegen
        .builder
        .build_alloca(option_type, "option_some")
        .map_err(|e| format!("Failed to allocate Option<T>: {}", e))?;

    // Set tag = 1 (Some)
    let tag_ptr = codegen
        .builder
        .build_struct_gep(option_type, option_ptr, 0, "tag_ptr")
        .map_err(|e| format!("Failed to get tag pointer: {}", e))?;
    let tag_val = codegen.context.i32_type().const_int(1, false);
    codegen
        .builder
        .build_store(tag_ptr, tag_val)
        .map_err(|e| format!("Failed to store tag: {}", e))?;

    // Set value
    let value_ptr = codegen
        .builder
        .build_struct_gep(option_type, option_ptr, 1, "value_ptr")
        .map_err(|e| format!("Failed to get value pointer: {}", e))?;
    codegen
        .builder
        .build_store(value_ptr, value)
        .map_err(|e| format!("Failed to store value: {}", e))?;

    // Load and return Option<T> as value
    let option_val = codegen
        .builder
        .build_load(option_type, option_ptr, "option_val")
        .map_err(|e| format!("Failed to load Option<T>: {}", e))?;

    Ok(option_val)
}

/// Builtin: None -> Option<T>
/// Creates Option<T> with None variant (tag=0, no value)
/// Memory layout: { u8 tag, T padding }
pub fn builtin_option_none<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // None has no arguments, but we need to infer T from context
    // For now, create Option<i32> with tag=0
    // TODO: Type inference from context

    let value_type = codegen.context.i32_type(); // Default to i32

    // Allocate Option<T> on stack
    let option_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(), // tag
            value_type.into(),                 // padding (unused)
        ],
        false,
    );

    let option_ptr = codegen
        .builder
        .build_alloca(option_type, "option_none")
        .map_err(|e| format!("Failed to allocate Option<T>: {}", e))?;

    // Set tag = 0 (None)
    let tag_ptr = codegen
        .builder
        .build_struct_gep(option_type, option_ptr, 0, "tag_ptr")
        .map_err(|e| format!("Failed to get tag pointer: {}", e))?;
    let tag_val = codegen.context.i32_type().const_int(0, false);
    codegen
        .builder
        .build_store(tag_ptr, tag_val)
        .map_err(|e| format!("Failed to store tag: {}", e))?;

    // No need to initialize value (None has no value)

    // Load and return Option<T> as value
    let option_val = codegen
        .builder
        .build_load(option_type, option_ptr, "option_val")
        .map_err(|e| format!("Failed to load Option<T>: {}", e))?;

    Ok(option_val)
}

/// Builtin: Ok(value: T) -> Result<T, E>
/// Creates Result<T,E> with Ok variant (tag=1, ok_value)
/// Memory layout: { u8 tag, union { T ok, E err } }
pub fn builtin_result_ok<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("Ok() requires exactly 1 argument".to_string());
    }

    let ok_value = args[0];
    let ok_type = ok_value.get_type();

    // For now, assume error type is also same as ok type (simplification)
    // TODO: Infer error type from context
    let _err_type = ok_type;

    // Result<T,E> layout: { i32 tag, T ok_or_err }
    let result_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(), // tag
            ok_type,                           // ok value (union with err)
        ],
        false,
    );

    let result_ptr = codegen
        .builder
        .build_alloca(result_type, "result_ok")
        .map_err(|e| format!("Failed to allocate Result<T,E>: {}", e))?;

    // Set tag = 1 (Ok)
    let tag_ptr = codegen
        .builder
        .build_struct_gep(result_type, result_ptr, 0, "tag_ptr")
        .map_err(|e| format!("Failed to get tag pointer: {}", e))?;
    let tag_val = codegen.context.i32_type().const_int(1, false);
    codegen
        .builder
        .build_store(tag_ptr, tag_val)
        .map_err(|e| format!("Failed to store tag: {}", e))?;

    // Set ok value
    let ok_ptr = codegen
        .builder
        .build_struct_gep(result_type, result_ptr, 1, "ok_ptr")
        .map_err(|e| format!("Failed to get ok pointer: {}", e))?;
    codegen
        .builder
        .build_store(ok_ptr, ok_value)
        .map_err(|e| format!("Failed to store ok value: {}", e))?;

    // Load and return Result<T,E> as value
    let result_val = codegen
        .builder
        .build_load(result_type, result_ptr, "result_val")
        .map_err(|e| format!("Failed to load Result<T,E>: {}", e))?;

    Ok(result_val)
}

/// Builtin: Err(error: E) -> Result<T, E>
/// Creates Result<T,E> with Err variant (tag=0, err_value)
/// Memory layout: { u8 tag, union { T ok, E err } }
pub fn builtin_result_err<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("Err() requires exactly 1 argument".to_string());
    }

    let err_value = args[0];
    let err_type = err_value.get_type();

    // For now, assume ok type is also same as err type (simplification)
    // TODO: Infer ok type from context
    let value_type = err_type;

    // Result<T,E> layout: { i32 tag, T ok_or_err }
    let result_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(), // tag
            value_type,                        // err value (union with ok)
        ],
        false,
    );

    let result_ptr = codegen
        .builder
        .build_alloca(result_type, "result_err")
        .map_err(|e| format!("Failed to allocate Result<T,E>: {}", e))?;

    // Set tag = 0 (Err)
    let tag_ptr = codegen
        .builder
        .build_struct_gep(result_type, result_ptr, 0, "tag_ptr")
        .map_err(|e| format!("Failed to get tag pointer: {}", e))?;
    let tag_val = codegen.context.i32_type().const_int(0, false);
    codegen
        .builder
        .build_store(tag_ptr, tag_val)
        .map_err(|e| format!("Failed to store tag: {}", e))?;

    // Set err value
    let err_ptr = codegen
        .builder
        .build_struct_gep(result_type, result_ptr, 1, "err_ptr")
        .map_err(|e| format!("Failed to get err pointer: {}", e))?;
    codegen
        .builder
        .build_store(err_ptr, err_value)
        .map_err(|e| format!("Failed to store err value: {}", e))?;

    // Load and return Result<T,E> as value
    let result_val = codegen
        .builder
        .build_load(result_type, result_ptr, "result_val")
        .map_err(|e| format!("Failed to load Result<T,E>: {}", e))?;

    Ok(result_val)
}
