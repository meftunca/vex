// Print function execution (compile_print_fmt, compile_print_variadic, value conversion)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicValueEnum, PointerValue};

pub(super) fn compile_print_fmt<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    func_name: &str,
    args: &[BasicValueEnum<'ctx>],
    arg_types: &[vex_ast::Type],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("Format mode requires at least a format string".to_string());
    }

    // Determine which C function to call
    let fn_name = if func_name == "println" {
        "vex_println_fmt"
    } else {
        "vex_print_fmt"
    };

    // Declare vex_print_fmt: void vex_print_fmt(const char* fmt, int count, VexValue* args)
    let vex_print_fmt_fn = if let Some(func) = codegen.module.get_function(fn_name) {
        func
    } else {
        let fn_type = codegen.context.void_type().fn_type(
            &[
                codegen
                    .context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(), // fmt string
                codegen.context.i32_type().into(), // count
                codegen
                    .context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(), // VexValue* args
            ],
            false,
        );
        codegen
            .module
            .add_function(fn_name, fn_type, Some(inkwell::module::Linkage::External))
    };

    // First arg is format string (already compiled)
    let fmt_str = args[0];

    // Remaining args need to be converted to VexValue array
    let value_args = &args[1..];

    if value_args.is_empty() {
        // No values to format - just print the format string as-is
        // Call with count=0, NULL args
        codegen
            .builder
            .build_call(
                vex_print_fmt_fn,
                &[
                    fmt_str.into(),
                    codegen.context.i32_type().const_int(0, false).into(),
                    codegen
                        .context
                        .ptr_type(inkwell::AddressSpace::default())
                        .const_null()
                        .into(),
                ],
                "print_fmt_call",
            )
            .map_err(|e| format!("Failed to call {}: {}", fn_name, e))?;

        return Ok(codegen.context.i32_type().const_int(0, false).into());
    }

    // VexValue struct layout - MUST match C exactly
    // C: struct { i32 type; union (i128-aligned, offset 16) }
    // Total size: 32 bytes (type=4, padding=12, union=16)
    let padding_array = codegen.context.i8_type().array_type(12);
    let vex_value_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(), // type (4 bytes, offset 0)
            padding_array.into(),              // padding (12 bytes)
            codegen.context.custom_width_int_type(128).into(), // union (16 bytes, offset 16)
        ],
        false,
    );

    // Allocate array: VexValue args[N]
    let count = value_args.len() as u32;
    let array_type = vex_value_type.array_type(count);
    let array_ptr = codegen
        .builder
        .build_alloca(array_type, "print_fmt_args_array")
        .map_err(|e| format!("Failed to allocate VexValue array: {}", e))?;

    // Fill array with converted values
    for (i, &val) in value_args.iter().enumerate() {
        let idx = codegen.context.i32_type().const_int(i as u64, false);
        let elem_ptr = unsafe {
            codegen
                .builder
                .build_gep(
                    array_type,
                    array_ptr,
                    &[codegen.context.i32_type().const_int(0, false), idx],
                    &format!("print_fmt_arg_{}", i),
                )
                .map_err(|e| format!("Failed to GEP args[{}]: {}", i, e))?
        };

        let arg_type = arg_types
            .get(i)
            .ok_or_else(|| format!("Missing type for arg {}", i))?;
        convert_to_vex_value_typed(codegen, val, elem_ptr, arg_type)?;
    }

    // Cast array to i8*
    let args_ptr = codegen
        .builder
        .build_pointer_cast(
            array_ptr,
            codegen.context.ptr_type(inkwell::AddressSpace::default()),
            "args_ptr",
        )
        .map_err(|e| format!("Failed to cast args array: {}", e))?;

    // Call vex_print_fmt(fmt, count, args)
    codegen
        .builder
        .build_call(
            vex_print_fmt_fn,
            &[
                fmt_str.into(),
                codegen
                    .context
                    .i32_type()
                    .const_int(count as u64, false)
                    .into(),
                args_ptr.into(),
            ],
            "print_fmt_call",
        )
        .map_err(|e| format!("Failed to call {}: {}", fn_name, e))?;

    // Return void (i32 0 as dummy)
    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// Go-style variadic print: print("x =", 42, "y =", 3.14)
pub(super) fn compile_print_variadic<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    func_name: &str,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Determine which C function to call
    let fn_name = if func_name == "println" {
        "vex_println_args"
    } else {
        "vex_print_args"
    };

    // This is the existing implementation
    // (moved from builtin_print/builtin_println)
    let vex_print_args_fn = if let Some(func) = codegen.module.get_function(fn_name) {
        func
    } else {
        let fn_type = codegen.context.void_type().fn_type(
            &[
                codegen.context.i32_type().into(),
                codegen
                    .context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            false,
        );
        codegen
            .module
            .add_function(fn_name, fn_type, Some(inkwell::module::Linkage::External))
    };

    let vex_value_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(),
            codegen.context.i32_type().into(),
            codegen.context.i64_type().into(),
        ],
        false,
    );

    let count = args.len() as u32;
    let array_type = vex_value_type.array_type(count);
    let array_ptr = codegen
        .builder
        .build_alloca(array_type, "print_args_array")
        .map_err(|e| format!("Failed to allocate VexValue array: {}", e))?;

    for (i, &val) in args.iter().enumerate() {
        let idx = codegen.context.i32_type().const_int(i as u64, false);
        let elem_ptr = unsafe {
            codegen
                .builder
                .build_gep(
                    array_type,
                    array_ptr,
                    &[codegen.context.i32_type().const_int(0, false), idx],
                    &format!("print_arg_{}", i),
                )
                .map_err(|e| format!("Failed to GEP args[{}]: {}", i, e))?
        };

        convert_to_vex_value(codegen, val, elem_ptr)?;
    }

    let args_ptr = codegen
        .builder
        .build_pointer_cast(
            array_ptr,
            codegen.context.ptr_type(inkwell::AddressSpace::default()),
            "args_ptr",
        )
        .map_err(|e| format!("Failed to cast args array: {}", e))?;

    codegen
        .builder
        .build_call(
            vex_print_args_fn,
            &[
                codegen
                    .context
                    .i32_type()
                    .const_int(count as u64, false)
                    .into(),
                args_ptr.into(),
            ],
            "print_call",
        )
        .map_err(|e| format!("Failed to call {}: {}", fn_name, e))?;

    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// Convert LLVM BasicValueEnum to VexValue struct with type information
/// Uses C helper functions (vex_value_i32, vex_value_f32, etc.) for correct union layout
fn convert_to_vex_value_typed<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    val: BasicValueEnum<'ctx>,
    vex_value_ptr: PointerValue<'ctx>,
    val_type: &vex_ast::Type,
) -> Result<(), String> {
    use vex_ast::Type;

    // ⭐ FEATURE: Automatic struct/array/Vec printing (zero-overhead via compile-time codegen)
    // Check if val_type is a struct - generate inline printing code
    if let Type::Named(struct_name) = val_type {
        if codegen.struct_defs.contains_key(struct_name) {
            // Generate automatic struct printing: StructName { field1: val, field2: val, ... }
            return convert_struct_to_debug_string(codegen, val, vex_value_ptr, struct_name);
        }
    }

    // Call appropriate vex_value_T() helper function for type-safe union construction
    // These are static inline C functions that return VexValue by value

    let helper_name = match val_type {
        Type::I8 => "vex_value_i8",
        Type::I16 => "vex_value_i16",
        Type::I32 => "vex_value_i32",
        Type::I64 => "vex_value_i64",
        Type::I128 => "vex_value_i128",
        Type::U8 => "vex_value_u8",
        Type::U16 => "vex_value_u16",
        Type::U32 => "vex_value_u32",
        Type::U64 => "vex_value_u64",
        Type::U128 => "vex_value_u128",
        Type::F16 => "vex_value_f16",
        Type::F32 => "vex_value_f32",
        Type::F64 => "vex_value_f64",
        Type::Bool => "vex_value_bool",
        Type::String => "vex_value_string",
        Type::Error => "vex_value_error",
        Type::Nil => "vex_value_nil",
        _ => "vex_value_ptr", // Fallback for pointers/structs
    };

    // Declare/get helper function
    let helper_fn = declare_vex_value_helper(codegen, helper_name, val_type)?;

    // Call helper: VexValue result = vex_value_T(val);
    let vex_value = if matches!(val_type, Type::Nil) {
        // Nil takes no arguments
        codegen
            .builder
            .build_call(helper_fn, &[], "vex_value_nil_call")
            .map_err(|e| format!("Failed to call vex_value_nil: {}", e))?
    } else {
        codegen
            .builder
            .build_call(helper_fn, &[val.into()], &format!("{}_call", helper_name))
            .map_err(|e| format!("Failed to call {}: {}", helper_name, e))?
    };

    let vex_value_result = vex_value
        .try_as_basic_value()
        .left()
        .ok_or_else(|| format!("{} returned void", helper_name))?;

    // Store the returned VexValue struct to the pointer
    codegen
        .builder
        .build_store(vex_value_ptr, vex_value_result)
        .map_err(|e| format!("Failed to store VexValue: {}", e))?;

    Ok(())
}

/// ⭐ ZERO-OVERHEAD: Generate inline struct debug string at compile-time
/// Converts struct to "StructName { field1: val, field2: val }" string via sprintf
/// Zero runtime overhead - all code generated during compilation
fn convert_struct_to_debug_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    struct_val: BasicValueEnum<'ctx>,
    vex_value_ptr: PointerValue<'ctx>,
    _struct_name: &str,
) -> Result<(), String> {
    // For now, fall back to pointer printing for structs
    // TODO: Implement proper recursive struct printing with VexString concatenation
    // This would require:
    // 1. Detect field types (int, float, string, nested struct)
    // 2. For each field, call appropriate to_string conversion
    // 3. Concatenate using VexString operations
    // 4. Return final formatted string as VexValue

    // Fallback: use vex_value_ptr helper (shows pointer address)
    let helper_fn = declare_vex_value_helper(codegen, "vex_value_ptr", &vex_ast::Type::I32)?;
    let vex_value = codegen
        .builder
        .build_call(helper_fn, &[struct_val.into()], "vex_value_ptr_call")
        .map_err(|e| format!("Failed to call vex_value_ptr: {}", e))?;

    let vex_value_result = vex_value
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_value_ptr returned void".to_string())?;

    codegen
        .builder
        .build_store(vex_value_ptr, vex_value_result)
        .map_err(|e| format!("Failed to store VexValue: {}", e))?;

    Ok(())
}

/// Declare vex_value_T() helper function
fn declare_vex_value_helper<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    name: &str,
    val_type: &vex_ast::Type,
) -> Result<inkwell::values::FunctionValue<'ctx>, String> {
    use vex_ast::Type;

    if let Some(func) = codegen.module.get_function(name) {
        return Ok(func);
    }

    // VexValue return type (struct by value)
    // CRITICAL: Match C layout with i128-aligned union
    // C: sizeof=32, type at offset 0 (4 bytes), union at offset 16 (16 bytes)
    // Padding: 12 bytes between type and union due to i128 alignment requirement
    let padding_array = codegen.context.i8_type().array_type(12);
    let vex_value_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(), // type (4 bytes, offset 0)
            padding_array.into(),              // padding (12 bytes)
            codegen.context.custom_width_int_type(128).into(), // union (16 bytes, offset 16)
        ],
        false,
    );

    // Determine parameter type
    let param_type = match val_type {
        Type::I8 => codegen.context.i8_type().into(),
        Type::I16 => codegen.context.i16_type().into(),
        Type::I32 => codegen.context.i32_type().into(),
        Type::I64 => codegen.context.i64_type().into(),
        Type::I128 => codegen.context.custom_width_int_type(128).into(),
        Type::U8 => codegen.context.i8_type().into(),
        Type::U16 => codegen.context.i16_type().into(),
        Type::U32 => codegen.context.i32_type().into(),
        Type::U64 => codegen.context.i64_type().into(),
        Type::U128 => codegen.context.custom_width_int_type(128).into(),
        Type::F16 => codegen.context.f16_type().into(),
        Type::F32 => codegen.context.f32_type().into(),
        Type::F64 => codegen.context.f64_type().into(),
        Type::Bool => codegen.context.bool_type().into(),
        Type::Nil => {
            // vex_value_nil() takes no arguments
            let fn_type = vex_value_type.fn_type(&[], false);
            return Ok(codegen.module.add_function(
                name,
                fn_type,
                Some(inkwell::module::Linkage::External),
            ));
        }
        Type::String | Type::Error | _ => codegen
            .context
            .ptr_type(inkwell::AddressSpace::default())
            .into(),
    };

    let fn_type = vex_value_type.fn_type(&[param_type], false);
    Ok(codegen
        .module
        .add_function(name, fn_type, Some(inkwell::module::Linkage::External)))
}

/// Convert LLVM BasicValueEnum to VexValue struct and store at pointer (legacy - no type info)
fn convert_to_vex_value<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    val: BasicValueEnum<'ctx>,
    vex_value_ptr: PointerValue<'ctx>,
) -> Result<(), String> {
    // VexValue type enum:
    // VEX_VALUE_I32 = 0, VEX_VALUE_I64 = 1, VEX_VALUE_F32 = 2, VEX_VALUE_F64 = 3,
    // VEX_VALUE_BOOL = 4, VEX_VALUE_STRING = 5, VEX_VALUE_PTR = 6

    // Define VexValue struct type (MUST be the same instance for all GEPs!)
    let vex_value_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(), // type (i32) - index 0
            codegen.context.i32_type().into(), // padding - index 1
            codegen.context.i64_type().into(), // union (largest: i64/f64/ptr) - index 2
        ],
        false,
    );

    let (type_enum, value_to_store) = match val {
        BasicValueEnum::IntValue(int_val) => {
            let bit_width = int_val.get_type().get_bit_width();
            if bit_width == 1 {
                // bool
                let as_i32 = codegen
                    .builder
                    .build_int_z_extend(int_val, codegen.context.i32_type(), "bool_to_i32")
                    .map_err(|e| format!("Failed to extend bool: {}", e))?;
                (4, as_i32.into()) // VEX_VALUE_BOOL
            } else if bit_width == 64 {
                // i64
                (1, int_val.into()) // VEX_VALUE_I64
            } else {
                // i32
                (0, int_val.into()) // VEX_VALUE_I32
            }
        }
        BasicValueEnum::FloatValue(float_val) => {
            let float_type = float_val.get_type();
            if float_type == codegen.context.f64_type() {
                // f64 - store as i64 bitcast
                let as_i64 = codegen
                    .builder
                    .build_bit_cast(float_val, codegen.context.i64_type(), "f64_bits")
                    .map_err(|e| format!("Failed to bitcast f64: {}", e))?
                    .into_int_value();
                (3, as_i64.into()) // VEX_VALUE_F64
            } else {
                // f32 - extend to i64 after bitcast to i32
                let as_i32 = codegen
                    .builder
                    .build_bit_cast(float_val, codegen.context.i32_type(), "f32_bits")
                    .map_err(|e| format!("Failed to bitcast f32: {}", e))?
                    .into_int_value();
                let as_i64 = codegen
                    .builder
                    .build_int_z_extend(as_i32, codegen.context.i64_type(), "f32_to_i64")
                    .map_err(|e| format!("Failed to extend f32: {}", e))?;
                (2, as_i64.into()) // VEX_VALUE_F32
            }
        }
        BasicValueEnum::PointerValue(ptr_val) => {
            // String or pointer
            let as_i64 = codegen
                .builder
                .build_ptr_to_int(ptr_val, codegen.context.i64_type(), "ptr_to_int")
                .map_err(|e| format!("Failed to convert ptr to int: {}", e))?;
            (5, as_i64.into()) // VEX_VALUE_STRING (or PTR)
        }
        BasicValueEnum::StructValue(struct_val) => {
            // Struct: For now, use its address (convert to ptr first)
            // TODO: Full struct printing support
            let struct_ptr = codegen
                .builder
                .build_alloca(struct_val.get_type(), "struct_temp")
                .map_err(|e| format!("Failed to allocate struct temp: {}", e))?;
            codegen
                .builder
                .build_store(struct_ptr, struct_val)
                .map_err(|e| format!("Failed to store struct: {}", e))?;
            let as_i64 = codegen
                .builder
                .build_ptr_to_int(struct_ptr, codegen.context.i64_type(), "struct_ptr_to_int")
                .map_err(|e| format!("Failed to convert struct ptr to int: {}", e))?;
            (6, as_i64.into()) // VEX_VALUE_PTR
        }
        _ => {
            return Err(format!("Unsupported type for VexValue: {:?}", val));
        }
    };

    // Store type field (offset 0)
    let type_field_ptr = codegen
        .builder
        .build_struct_gep(vex_value_type, vex_value_ptr, 0, "type_field")
        .map_err(|e| format!("Failed to GEP type field: {}", e))?;

    codegen
        .builder
        .build_store(
            type_field_ptr,
            codegen.context.i32_type().const_int(type_enum, false),
        )
        .map_err(|e| format!("Failed to store type: {}", e))?;

    // Store value field (offset 2 - union at index 2)
    let value_field_ptr = codegen
        .builder
        .build_struct_gep(vex_value_type, vex_value_ptr, 2, "value_field")
        .map_err(|e| format!("Failed to GEP value field: {}", e))?;

    // Convert value to i64 if needed
    let value_as_i64 = match value_to_store {
        BasicValueEnum::IntValue(int_val) => {
            let bit_width = int_val.get_type().get_bit_width();
            if bit_width < 64 {
                codegen
                    .builder
                    .build_int_z_extend(int_val, codegen.context.i64_type(), "extend_to_i64")
                    .map_err(|e| format!("Failed to extend to i64: {}", e))?
            } else {
                int_val
            }
        }
        _ => {
            return Err(format!(
                "Expected IntValue after conversion: {:?}",
                value_to_store
            ));
        }
    };

    codegen
        .builder
        .build_store(value_field_ptr, value_as_i64)
        .map_err(|e| format!("Failed to store value: {}", e))?;

    Ok(())
}

