// Print format string parsing and type-safe formatting

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::Expression;

use super::print_execution::{compile_print_fmt, compile_print_variadic};

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

