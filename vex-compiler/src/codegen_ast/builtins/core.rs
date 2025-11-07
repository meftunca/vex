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

    // Check if first argument is a string literal containing '{}'
    let is_format_mode = if let Expression::StringLiteral(s) = &ast_args[0] {
        s.contains("{}")
    } else {
        false
    };

    if is_format_mode {
        // Format string mode: print("x = {}, y = {}", 42, 3.14)
        compile_print_fmt(codegen, func_name, compiled_args)
    } else {
        // Go-style variadic mode: print("x =", 42, "y =", 3.14)
        compile_print_variadic(codegen, func_name, compiled_args)
    }
}

/// Format string mode: print!("x = {}, y = {}", 42, 3.14)
fn compile_print_fmt<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    func_name: &str,
    args: &[BasicValueEnum<'ctx>],
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

    // VexValue struct layout
    let vex_value_type = codegen.context.struct_type(
        &[
            codegen.context.i32_type().into(), // type (i32)
            codegen.context.i32_type().into(), // padding
            codegen.context.i64_type().into(), // union (largest: i64/f64/ptr)
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

        convert_to_vex_value(codegen, val, elem_ptr)?;
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

/// Convert LLVM BasicValueEnum to VexValue struct and store at pointer
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
