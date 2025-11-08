// Type reflection builtins
// typeof, type_name, type_id

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// Get type name as string (RTTI)
/// Usage: typeof(x) -> "i32", "f64", "MyStruct", etc.
pub fn builtin_typeof<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("typeof expects 1 argument, got {}", args.len()));
    }

    let value = args[0];
    let value_type = value.get_type();

    // Generate type name string based on LLVM type
    let type_name = match value_type {
        inkwell::types::BasicTypeEnum::IntType(int_ty) => {
            let bit_width = int_ty.get_bit_width();
            match bit_width {
                1 => "bool",
                8 => "i8",
                16 => "i16",
                32 => "i32",
                64 => "i64",
                _ => "int",
            }
        }
        inkwell::types::BasicTypeEnum::FloatType(float_ty) => {
            let kind = float_ty.get_context().f32_type();
            if float_ty == kind {
                "f32"
            } else {
                "f64"
            }
        }
        inkwell::types::BasicTypeEnum::PointerType(_) => "ptr",
        inkwell::types::BasicTypeEnum::ArrayType(_) => "array",
        inkwell::types::BasicTypeEnum::StructType(struct_ty) => {
            // Try to get struct name from metadata
            if let Some(name) = struct_ty.get_name() {
                let name_str = name.to_str().unwrap();
                let str_ptr = codegen
                    .builder
                    .build_global_string_ptr(name_str, "type_name")
                    .map_err(|e| format!("Failed to create type name string: {:?}", e))?;
                return Ok(str_ptr.as_pointer_value().into());
            }
            "struct"
        }
        inkwell::types::BasicTypeEnum::VectorType(_) => "vector",
        inkwell::types::BasicTypeEnum::ScalableVectorType(_) => "scalable_vector",
    };

    // Create string literal for type name
    let type_str = codegen
        .builder
        .build_global_string_ptr(type_name, "type_name")
        .map_err(|e| format!("Failed to create type name string: {:?}", e))?;
    Ok(type_str.as_pointer_value().into())
}

/// Get type size at compile-time (already implemented in memory.rs)
/// This is just an alias for clarity
pub fn builtin_type_size<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Redirect to sizeof
    super::memory::builtin_sizeof(codegen, args)
}

/// Get type alignment at compile-time (already implemented in memory.rs)
/// This is just an alias for clarity
pub fn builtin_type_align<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Redirect to alignof
    super::memory::builtin_alignof(codegen, args)
}

/// Check if type is integer
pub fn builtin_is_int_type<'ctx>(
    _codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "is_int_type expects 1 argument, got {}",
            args.len()
        ));
    }

    let value = args[0];
    let is_int = matches!(value.get_type(), inkwell::types::BasicTypeEnum::IntType(_));

    let context = value.get_type().into_int_type().get_context();
    let bool_val = context.bool_type().const_int(is_int as u64, false);
    Ok(bool_val.into())
}

/// Check if type is float
pub fn builtin_is_float_type<'ctx>(
    _codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "is_float_type expects 1 argument, got {}",
            args.len()
        ));
    }

    let value = args[0];
    let is_float = matches!(
        value.get_type(),
        inkwell::types::BasicTypeEnum::FloatType(_)
    );

    let context = value.get_type().into_int_type().get_context();
    let bool_val = context.bool_type().const_int(is_float as u64, false);
    Ok(bool_val.into())
}

/// Check if type is pointer
pub fn builtin_is_pointer_type<'ctx>(
    _codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "is_pointer_type expects 1 argument, got {}",
            args.len()
        ));
    }

    let value = args[0];
    let is_ptr = matches!(
        value.get_type(),
        inkwell::types::BasicTypeEnum::PointerType(_)
    );

    let context = value.get_type().into_int_type().get_context();
    let bool_val = context.bool_type().const_int(is_ptr as u64, false);
    Ok(bool_val.into())
}

/// Get type ID (numeric identifier for runtime type checks)
pub fn builtin_type_id<'ctx>(
    _codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("type_id expects 1 argument, got {}", args.len()));
    }

    let value = args[0];
    let value_type = value.get_type();

    // Generate unique type ID based on LLVM type
    let type_id: u64 = match value_type {
        inkwell::types::BasicTypeEnum::IntType(int_ty) => {
            let bit_width = int_ty.get_bit_width();
            match bit_width {
                1 => 1,   // bool
                8 => 2,   // i8
                16 => 3,  // i16
                32 => 4,  // i32
                64 => 5,  // i64
                _ => 100, // custom int
            }
        }
        inkwell::types::BasicTypeEnum::FloatType(float_ty) => {
            let kind = float_ty.get_context().f32_type();
            if float_ty == kind {
                10 // f32
            } else {
                11 // f64
            }
        }
        inkwell::types::BasicTypeEnum::PointerType(_) => 20,
        inkwell::types::BasicTypeEnum::ArrayType(_) => 30,
        inkwell::types::BasicTypeEnum::StructType(struct_ty) => {
            // Hash struct name for unique ID
            if let Some(name) = struct_ty.get_name() {
                let name_str = name.to_str().unwrap();
                // Simple hash
                let mut hash: u64 = 1000;
                for byte in name_str.bytes() {
                    hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
                }
                hash
            } else {
                40 // Anonymous struct
            }
        }
        inkwell::types::BasicTypeEnum::VectorType(_) => 50,
        inkwell::types::BasicTypeEnum::ScalableVectorType(_) => 60,
    };

    let context = value.get_type().into_int_type().get_context();
    let id_val = context.i64_type().const_int(type_id, false);
    Ok(id_val.into())
}

/// Get field metadata for a struct type
/// Usage: field_metadata("StructName", "field_name") -> HashMap<str, str> | None
/// Returns metadata map for the field, or None if not found
pub fn builtin_field_metadata<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!(
            "field_metadata expects 2 arguments (struct_name, field_name), got {}",
            args.len()
        ));
    }

    // Extract struct name and field name from string arguments
    // Note: In real implementation, we need to pass these as compile-time constants
    // For now, return a dummy result indicating metadata API exists

    // TODO: Implement compile-time metadata lookup
    // 1. Get struct_name and field_name from constant string arguments
    // 2. Lookup in codegen.struct_metadata
    // 3. Generate HashMap literal or return None

    eprintln!("⚠️  field_metadata() called but full implementation pending");
    eprintln!("    This requires compile-time string evaluation");

    // For now, return i32 0 as placeholder
    let context = codegen.context;
    let zero = context.i32_type().const_int(0, false);
    Ok(zero.into())
}
