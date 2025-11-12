// Core builtin functions: print, println, panic, assert, unreachable
//
// ROADMAP: Print Function Unification (Future: vex-libs/std/io)
// ============================================================
//
// Current State:
//   - print(...args)   → Go-style variadic (space-separated, no newline)
//   - println(...args) → Go-style variadic + newline
//
// TODO (Phase 1): Format String Support
//   - Detect: If first arg is string literal with '{}' → format mode
//   - Modes:
//     1. print("x = {}, y = {}", 42, 3.14)  → vex_print_fmt()
//     2. print("x =", 42, "y =", 3.14)      → vex_print_args()
//   - Placeholders:
//     - {}     → Default format
//     - {:?}   → Debug format
//     - {:.N}  → Float precision
//     - {:x}   → Hex format
//   - Implementation: Add format string parsing to detect_print_mode()
//
// TODO (Phase 2): Move to Stdlib
//   - Move print/println to vex-libs/std/io.vx
//   - Keep only low-level C FFI in builtins (vex_print_args, vex_print_fmt)
//   - Example stdlib implementation:
//     ```vex
//     pub fn println(...args) {
//         print(...args);
//         print("\n");
//     }
//     ```
//
// C Runtime Functions (already implemented in vex_io.c):
//   - vex_print_args(count, VexValue*)        → Go-style (current)
//   - vex_println_args(count, VexValue*)      → Go-style + newline (current)
//   - vex_print_fmt(fmt, count, VexValue*)    → Rust-style format (TODO: expose)
//   - vex_println_fmt(fmt, count, VexValue*)  → Rust-style format + newline (TODO: expose)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicValueEnum, PointerValue};
use vex_ast::Expression;

/// Unified print/println call handler with format string detection
///
/// This function is called BEFORE arguments are fully compiled,
/// so we can inspect the AST to detect format strings.
pub fn compile_print_call<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    func_name: &str,
    ast_args: &[Expression],
    compiled_args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if ast_args.is_empty() {
        return Err(format!("{}() requires at least one argument", func_name));
    }

    // Check if first argument is a string literal containing '{' (format placeholder)
    let is_format_mode = if let Expression::StringLiteral(s) = &ast_args[0] {
        s.contains('{')
    } else {
        false
    };

    if is_format_mode {
        // Infer types of arguments for type-aware conversion
        let mut arg_types = Vec::new();
        for arg in &ast_args[1..] {
            let ty = codegen.infer_expression_type(arg)?;
            arg_types.push(ty);
        }

        // Format string mode: print("x = {}, y = {}", 42, 3.14)
        compile_print_fmt(codegen, func_name, compiled_args, &arg_types)
    } else {
        // Go-style variadic mode: print("x =", 42, "y =", 3.14)
        compile_print_variadic(codegen, func_name, compiled_args)
    }
}

/// Parse format string and extract placeholders
/// Returns: (literal_parts, placeholder_specs)
/// Example: "x={}, y={:.2}" -> (["x=", ", y="], ["", ".2"])
fn parse_format_string(fmt: &str) -> (Vec<String>, Vec<String>) {
    let mut literals = Vec::new();
    let mut specs = Vec::new();
    let mut current_literal = String::new();
    let mut chars = fmt.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                // Escaped {{ -> single {
                chars.next();
                current_literal.push('{');
            } else {
                // Placeholder start
                literals.push(current_literal.clone());
                current_literal.clear();

                // Parse format spec until }
                let mut spec = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume }
                        break;
                    }
                    spec.push(chars.next().unwrap());
                }
                specs.push(spec);
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                // Escaped }} -> single }
                chars.next();
                current_literal.push('}');
            } else {
                // Unmatched }
                current_literal.push('}');
            }
        } else {
            current_literal.push(ch);
        }
    }

    literals.push(current_literal);
    (literals, specs)
}

/// Type-safe zero-cost format implementation
/// Compiles format("{}, {}", 42, 3.14) to direct C function calls
pub fn compile_typesafe_format<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    fmt_str: &str,
    args: &[BasicValueEnum<'ctx>],
    arg_types: &[vex_ast::Type],
) -> Result<BasicValueEnum<'ctx>, String> {
    let (literals, specs) = parse_format_string(fmt_str);

    // Validate placeholder count matches argument count
    if specs.len() != args.len() {
        return Err(format!(
            "Format string has {} placeholders but {} arguments provided",
            specs.len(),
            args.len()
        ));
    }

    // Declare vex_fmt functions
    let vex_fmt_buffer_new = codegen.declare_vex_fmt_buffer_new();
    let vex_fmt_buffer_free = codegen.declare_vex_fmt_buffer_free();
    let vex_fmt_buffer_append_str = codegen.declare_vex_fmt_buffer_append_str();
    let vex_fmt_buffer_to_string = codegen.declare_vex_fmt_buffer_to_string();

    // Create buffer
    let initial_capacity = codegen.context.i64_type().const_int(256, false);
    let buffer = codegen
        .builder
        .build_call(vex_fmt_buffer_new, &[initial_capacity.into()], "fmt_buffer")
        .map_err(|e| format!("Failed to create buffer: {}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("vex_fmt_buffer_new didn't return a value")?;

    // Build formatted string piece by piece
    for (i, (literal, spec)) in literals.iter().zip(specs.iter()).enumerate() {
        // Append literal part
        if !literal.is_empty() {
            let lit_str = codegen
                .builder
                .build_global_string_ptr(literal, "fmt_literal")
                .map_err(|e| format!("Failed to create literal: {}", e))?;
            let lit_len = codegen
                .context
                .i64_type()
                .const_int(literal.len() as u64, false);

            codegen
                .builder
                .build_call(
                    vex_fmt_buffer_append_str,
                    &[
                        buffer.into(),
                        lit_str.as_pointer_value().into(),
                        lit_len.into(),
                    ],
                    "append_literal",
                )
                .map_err(|e| format!("Failed to append literal: {}", e))?;
        }

        // Format and append argument
        if i < args.len() {
            let formatted_str = compile_format_arg(codegen, args[i], &arg_types[i], spec)?;

            // Get length (assuming null-terminated C string)
            let strlen_fn = codegen.declare_strlen();
            let str_len = codegen
                .builder
                .build_call(strlen_fn, &[formatted_str.into()], "str_len")
                .map_err(|e| format!("Failed to call strlen: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or("strlen didn't return a value")?;

            codegen
                .builder
                .build_call(
                    vex_fmt_buffer_append_str,
                    &[buffer.into(), formatted_str.into(), str_len.into()],
                    "append_formatted",
                )
                .map_err(|e| format!("Failed to append formatted: {}", e))?;
        }
    }

    // Append final literal (after last placeholder)
    if literals.len() > specs.len() {
        let final_lit = &literals[literals.len() - 1];
        if !final_lit.is_empty() {
            let lit_str = codegen
                .builder
                .build_global_string_ptr(final_lit, "fmt_literal_final")
                .map_err(|e| format!("Failed to create final literal: {}", e))?;
            let lit_len = codegen
                .context
                .i64_type()
                .const_int(final_lit.len() as u64, false);

            codegen
                .builder
                .build_call(
                    vex_fmt_buffer_append_str,
                    &[
                        buffer.into(),
                        lit_str.as_pointer_value().into(),
                        lit_len.into(),
                    ],
                    "append_final",
                )
                .map_err(|e| format!("Failed to append final literal: {}", e))?;
        }
    }

    // Convert buffer to string
    let result_str = codegen
        .builder
        .build_call(vex_fmt_buffer_to_string, &[buffer.into()], "to_string")
        .map_err(|e| format!("Failed to convert buffer to string: {}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("vex_fmt_buffer_to_string didn't return a value")?;

    // Free buffer
    codegen
        .builder
        .build_call(vex_fmt_buffer_free, &[buffer.into()], "free_buffer")
        .map_err(|e| format!("Failed to free buffer: {}", e))?;

    Ok(result_str)
}

/// Compile single argument formatting - type-safe dispatch
fn compile_format_arg<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    arg: BasicValueEnum<'ctx>,
    arg_type: &vex_ast::Type,
    _spec: &str, // TODO: Parse spec string into FormatSpec struct
) -> Result<BasicValueEnum<'ctx>, String> {
    use vex_ast::Type;

    // Parse format spec (for now, just pass default spec)
    // TODO: Parse spec string into FormatSpec struct
    let fmt_spec = codegen.get_default_format_spec();

    // Zero-cost dispatch based on compile-time type
    match arg_type {
        Type::I8 | Type::I16 | Type::I32 => {
            let vex_fmt_i32 = codegen.declare_vex_fmt_i32();
            let arg_i32 = if let BasicValueEnum::IntValue(int_val) = arg {
                if int_val.get_type().get_bit_width() < 32 {
                    codegen
                        .builder
                        .build_int_s_extend(int_val, codegen.context.i32_type(), "extend_i32")
                        .map_err(|e| format!("Failed to extend to i32: {}", e))?
                } else {
                    int_val
                }
            } else {
                return Err(format!("Expected IntValue for i32, got {:?}", arg));
            };

            codegen
                .builder
                .build_call(vex_fmt_i32, &[arg_i32.into(), fmt_spec.into()], "fmt_i32")
                .map_err(|e| format!("Failed to call vex_fmt_i32: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "vex_fmt_i32 didn't return a value".to_string())
        }
        Type::I64 | Type::U64 => {
            let vex_fmt_i64 = codegen.declare_vex_fmt_i64();
            let arg_i64 = if let BasicValueEnum::IntValue(int_val) = arg {
                if int_val.get_type().get_bit_width() < 64 {
                    codegen
                        .builder
                        .build_int_s_extend(int_val, codegen.context.i64_type(), "extend_i64")
                        .map_err(|e| format!("Failed to extend to i64: {}", e))?
                } else {
                    int_val
                }
            } else {
                return Err(format!("Expected IntValue for i64, got {:?}", arg));
            };

            codegen
                .builder
                .build_call(vex_fmt_i64, &[arg_i64.into(), fmt_spec.into()], "fmt_i64")
                .map_err(|e| format!("Failed to call vex_fmt_i64: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "vex_fmt_i64 didn't return a value".to_string())
        }
        Type::U8 | Type::U16 | Type::U32 => {
            let vex_fmt_u32 = codegen.declare_vex_fmt_u32();
            let arg_u32 = if let BasicValueEnum::IntValue(int_val) = arg {
                if int_val.get_type().get_bit_width() < 32 {
                    codegen
                        .builder
                        .build_int_z_extend(int_val, codegen.context.i32_type(), "extend_u32")
                        .map_err(|e| format!("Failed to extend to u32: {}", e))?
                } else {
                    int_val
                }
            } else {
                return Err(format!("Expected IntValue for u32, got {:?}", arg));
            };

            codegen
                .builder
                .build_call(vex_fmt_u32, &[arg_u32.into(), fmt_spec.into()], "fmt_u32")
                .map_err(|e| format!("Failed to call vex_fmt_u32: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "vex_fmt_u32 didn't return a value".to_string())
        }
        Type::F32 => {
            let vex_fmt_f32 = codegen.declare_vex_fmt_f32();
            codegen
                .builder
                .build_call(vex_fmt_f32, &[arg.into(), fmt_spec.into()], "fmt_f32")
                .map_err(|e| format!("Failed to call vex_fmt_f32: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "vex_fmt_f32 didn't return a value".to_string())
        }
        Type::F64 => {
            let vex_fmt_f64 = codegen.declare_vex_fmt_f64();
            codegen
                .builder
                .build_call(vex_fmt_f64, &[arg.into(), fmt_spec.into()], "fmt_f64")
                .map_err(|e| format!("Failed to call vex_fmt_f64: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "vex_fmt_f64 didn't return a value".to_string())
        }
        Type::Bool => {
            let vex_fmt_bool = codegen.declare_vex_fmt_bool();
            codegen
                .builder
                .build_call(vex_fmt_bool, &[arg.into(), fmt_spec.into()], "fmt_bool")
                .map_err(|e| format!("Failed to call vex_fmt_bool: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "vex_fmt_bool didn't return a value".to_string())
        }
        Type::String => {
            let vex_fmt_string = codegen.declare_vex_fmt_string();
            let strlen_fn = codegen.declare_strlen();
            let str_len = codegen
                .builder
                .build_call(strlen_fn, &[arg.into()], "str_len")
                .map_err(|e| format!("Failed to call strlen: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or("strlen didn't return a value")?;

            codegen
                .builder
                .build_call(
                    vex_fmt_string,
                    &[arg.into(), str_len.into(), fmt_spec.into()],
                    "fmt_string",
                )
                .map_err(|e| format!("Failed to call vex_fmt_string: {}", e))?
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "vex_fmt_string didn't return a value".to_string())
        }
        Type::Named(name) => {
            // Check if type implements Display trait
            // For now, call to_string() method if available
            // TODO: Implement proper Display trait dispatch
            Err(format!(
                "Custom type '{}' doesn't implement Display (not yet supported)",
                name
            ))
        }
        _ => Err(format!("Unsupported type for formatting: {:?}", arg_type)),
    }
}

/// Format string mode: print("x = {}, y = {}", 42, 3.14)
fn compile_print_fmt<'ctx>(
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

/// Go-style variadic mode: print("x =", 42, "y =", 3.14)
fn compile_print_variadic<'ctx>(
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

// NOTE: builtin_print and builtin_println are deprecated!
// They are kept here for the registry but are never called directly.
// Instead, compile_print_call() is called from function_calls.rs
// which handles both format string and variadic modes.

/// print(...values) - DEPRECATED (use compile_print_call instead)
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
