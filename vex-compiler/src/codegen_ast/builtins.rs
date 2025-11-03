// Builtin functions registry for Vex compiler
// Core builtins that are always available without imports

use super::ASTCodeGen;
use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;
use std::collections::HashMap;

/// Builtin function generator type
pub type BuiltinGenerator<'ctx> =
    fn(&mut ASTCodeGen<'ctx>, &[BasicValueEnum<'ctx>]) -> Result<BasicValueEnum<'ctx>, String>;

/// Registry of all builtin functions
pub struct BuiltinRegistry<'ctx> {
    functions: HashMap<&'static str, BuiltinGenerator<'ctx>>,
}

impl<'ctx> BuiltinRegistry<'ctx> {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // Register core builtins
        registry.register("print", builtin_print);
        registry.register("println", builtin_println);
        registry.register("panic", builtin_panic);
        registry.register("assert", builtin_assert);
        registry.register("alloc", builtin_alloc);
        registry.register("free", builtin_free);
        registry.register("realloc", builtin_realloc);
        registry.register("sizeof", builtin_sizeof);
        registry.register("alignof", builtin_alignof);
        registry.register("unreachable", builtin_unreachable);

        // Register LLVM intrinsics - bit manipulation
        registry.register("ctlz", builtin_ctlz);
        registry.register("cttz", builtin_cttz);
        registry.register("ctpop", builtin_ctpop);
        registry.register("bswap", builtin_bswap);
        registry.register("bitreverse", builtin_bitreverse);

        // Register LLVM intrinsics - overflow checking
        registry.register("sadd_overflow", builtin_sadd_overflow);
        registry.register("ssub_overflow", builtin_ssub_overflow);
        registry.register("smul_overflow", builtin_smul_overflow);

        // Register LLVM intrinsics - compiler hints
        registry.register("assume", builtin_assume);
        registry.register("likely", builtin_likely);
        registry.register("unlikely", builtin_unlikely);
        registry.register("prefetch", builtin_prefetch);

        registry
    }

    fn register(&mut self, name: &'static str, generator: BuiltinGenerator<'ctx>) {
        self.functions.insert(name, generator);
    }

    pub fn get(&self, name: &str) -> Option<BuiltinGenerator<'ctx>> {
        self.functions.get(name).copied()
    }

    #[allow(dead_code)]
    pub fn is_builtin(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

// ============================================================================
// BUILTIN IMPLEMENTATIONS
// ============================================================================

/// print(value) - Output without newline
fn builtin_print<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("print() takes exactly one argument".to_string());
    }

    let val = args[0];

    // Determine format string based on type (NO newline)
    match val {
        BasicValueEnum::IntValue(_) => {
            codegen.build_printf("%d", &[val])?;
        }
        BasicValueEnum::FloatValue(_) => {
            codegen.build_printf("%f", &[val])?;
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
fn builtin_println<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("println() takes exactly one argument".to_string());
    }

    let val = args[0];

    // Determine format string based on type (WITH newline)
    match val {
        BasicValueEnum::IntValue(_) => {
            codegen.build_printf("%d\n", &[val])?;
        }
        BasicValueEnum::FloatValue(_) => {
            codegen.build_printf("%f\n", &[val])?;
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
fn builtin_panic<'ctx>(
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
fn builtin_assert<'ctx>(
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

/// alloc(size) - Allocate memory
fn builtin_alloc<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("alloc() takes exactly one argument (size)".to_string());
    }

    let size = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("alloc() size must be an integer".to_string()),
    };

    // Cast size to i64 (size_t)
    let size_i64 = codegen
        .builder
        .build_int_z_extend(size, codegen.context.i64_type(), "size_cast")
        .map_err(|e| format!("Failed to cast size to i64: {}", e))?;

    // Declare vex_malloc from runtime
    let vex_malloc = codegen.declare_vex_malloc();

    // Call vex_malloc(size)
    let result = codegen
        .builder
        .build_call(vex_malloc, &[size_i64.into()], "alloc_call")
        .map_err(|e| format!("Failed to call vex_malloc: {}", e))?;

    let ptr = result
        .try_as_basic_value()
        .left()
        .ok_or("vex_malloc didn't return a value")?;

    Ok(ptr)
}

/// free(ptr) - Free memory
fn builtin_free<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("free() takes exactly one argument (pointer)".to_string());
    }

    let ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr_val) => ptr_val,
        _ => return Err("free() argument must be a pointer".to_string()),
    };

    // Declare vex_free from runtime
    let vex_free = codegen.declare_vex_free();

    // Call vex_free(ptr)
    codegen
        .builder
        .build_call(vex_free, &[ptr.into()], "free_call")
        .map_err(|e| format!("Failed to call vex_free: {}", e))?;

    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// realloc(ptr, new_size) - Reallocate memory
fn builtin_realloc<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("realloc() takes exactly two arguments (ptr, new_size)".to_string());
    }

    let ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr_val) => ptr_val,
        _ => return Err("realloc() first argument must be a pointer".to_string()),
    };

    let new_size = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("realloc() second argument must be an integer".to_string()),
    };

    // Cast new_size to i64 (size_t)
    let new_size_i64 = codegen
        .builder
        .build_int_z_extend(new_size, codegen.context.i64_type(), "new_size_cast")
        .map_err(|e| format!("Failed to cast new_size to i64: {}", e))?;

    // Declare vex_realloc from runtime
    let vex_realloc = codegen.declare_vex_realloc();

    // Call vex_realloc(ptr, new_size)
    let result = codegen
        .builder
        .build_call(
            vex_realloc,
            &[ptr.into(), new_size_i64.into()],
            "realloc_call",
        )
        .map_err(|e| format!("Failed to call vex_realloc: {}", e))?;

    let new_ptr = result
        .try_as_basic_value()
        .left()
        .ok_or("vex_realloc didn't return a value")?;

    Ok(new_ptr)
}

/// sizeof<T>() - Get size of type in bytes
fn builtin_sizeof<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // sizeof is typically handled at compile-time with type information
    // For now, if called with a value, return the size of its type
    if args.is_empty() {
        return Err("sizeof() requires a value to determine type size".to_string());
    }

    let value = args[0];
    let size = match value {
        BasicValueEnum::IntValue(int_val) => {
            let int_type = int_val.get_type();
            int_type.get_bit_width() / 8
        }
        BasicValueEnum::FloatValue(float_val) => match float_val.get_type() {
            ty if ty == codegen.context.f32_type() => 4,
            ty if ty == codegen.context.f64_type() => 8,
            _ => return Err("Unknown float type".to_string()),
        },
        BasicValueEnum::PointerValue(_) => 8, // 64-bit pointer
        BasicValueEnum::StructValue(struct_val) => {
            // For structs, sum up field sizes (simplified, no padding calculation)
            let struct_type = struct_val.get_type();
            let field_count = struct_type.count_fields();
            let mut total_size = 0u32;

            for i in 0..field_count {
                if let Some(field_type) = struct_type.get_field_type_at_index(i) {
                    let field_size = match field_type {
                        inkwell::types::BasicTypeEnum::IntType(int_type) => {
                            int_type.get_bit_width() / 8
                        }
                        inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                            if float_type == codegen.context.f32_type() {
                                4
                            } else {
                                8
                            }
                        }
                        inkwell::types::BasicTypeEnum::PointerType(_) => 8,
                        _ => 8, // Default to pointer size
                    };
                    total_size += field_size;
                }
            }
            total_size
        }
        BasicValueEnum::ArrayValue(arr_val) => {
            let arr_type = arr_val.get_type();
            let elem_type = arr_type.get_element_type();
            let len = arr_type.len();
            // Simplified: assume elem size based on type
            match elem_type {
                inkwell::types::BasicTypeEnum::IntType(int_type) => {
                    (int_type.get_bit_width() / 8) * len
                }
                inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                    if float_type == codegen.context.f32_type() {
                        4 * len
                    } else {
                        8 * len
                    }
                }
                _ => return Err("Cannot determine array element size".to_string()),
            }
        }
        _ => return Err("sizeof() cannot determine size of this type".to_string()),
    };

    Ok(codegen
        .context
        .i64_type()
        .const_int(size as u64, false)
        .into())
}

/// alignof<T>() - Get alignment of type in bytes
fn builtin_alignof<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Err("alignof() requires a value to determine type alignment".to_string());
    }

    let value = args[0];
    let alignment = match value {
        BasicValueEnum::IntValue(int_val) => {
            let int_type = int_val.get_type();
            let bit_width = int_type.get_bit_width();
            match bit_width {
                8 => 1,
                16 => 2,
                32 => 4,
                64 => 8,
                128 => 16,
                _ => (bit_width / 8).max(1),
            }
        }
        BasicValueEnum::FloatValue(float_val) => match float_val.get_type() {
            ty if ty == codegen.context.f32_type() => 4,
            ty if ty == codegen.context.f64_type() => 8,
            _ => return Err("Unknown float type".to_string()),
        },
        BasicValueEnum::PointerValue(_) => 8, // 64-bit pointer alignment
        BasicValueEnum::StructValue(struct_val) => {
            // For structs, alignment is the max alignment of all fields
            let struct_type = struct_val.get_type();
            let field_count = struct_type.count_fields();
            let mut max_align = 1u32;

            for i in 0..field_count {
                if let Some(field_type) = struct_type.get_field_type_at_index(i) {
                    let field_align = match field_type {
                        inkwell::types::BasicTypeEnum::IntType(int_type) => {
                            let bit_width = int_type.get_bit_width();
                            match bit_width {
                                8 => 1,
                                16 => 2,
                                32 => 4,
                                64 => 8,
                                128 => 16,
                                _ => (bit_width / 8).max(1),
                            }
                        }
                        inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                            if float_type == codegen.context.f32_type() {
                                4
                            } else {
                                8
                            }
                        }
                        inkwell::types::BasicTypeEnum::PointerType(_) => 8,
                        _ => 8, // Default to pointer alignment
                    };
                    max_align = max_align.max(field_align);
                }
            }
            max_align
        }
        BasicValueEnum::ArrayValue(arr_val) => {
            // Array alignment is same as element alignment
            let arr_type = arr_val.get_type();
            let elem_type = arr_type.get_element_type();
            match elem_type {
                inkwell::types::BasicTypeEnum::IntType(int_type) => {
                    let bit_width = int_type.get_bit_width();
                    match bit_width {
                        8 => 1,
                        16 => 2,
                        32 => 4,
                        64 => 8,
                        128 => 16,
                        _ => (bit_width / 8).max(1),
                    }
                }
                inkwell::types::BasicTypeEnum::FloatType(float_type) => {
                    if float_type == codegen.context.f32_type() {
                        4
                    } else {
                        8
                    }
                }
                _ => return Err("Cannot determine array element alignment".to_string()),
            }
        }
        _ => return Err("alignof() cannot determine alignment of this type".to_string()),
    };

    Ok(codegen
        .context
        .i64_type()
        .const_int(alignment as u64, false)
        .into())
}

/// unreachable() - Mark code as unreachable (optimization hint + runtime trap)
fn builtin_unreachable<'ctx>(
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

// ============================================================================
// LLVM INTRINSICS - BIT MANIPULATION
// ============================================================================

/// ctlz(x) - Count leading zeros
fn builtin_ctlz<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("ctlz() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("ctlz() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.ctlz.i{}", int_type.get_bit_width());

    // Declare LLVM intrinsic - takes (value, is_zero_undef)
    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), codegen.context.bool_type().into()],
        int_type.into(),
    );

    // Call with is_zero_undef=false (return bit_width for zero input)
    let is_zero_undef = codegen.context.bool_type().const_int(0, false);
    let result = codegen
        .builder
        .build_call(
            intrinsic,
            &[value.into(), is_zero_undef.into()],
            "ctlz_call",
        )
        .map_err(|e| format!("Failed to call ctlz: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("ctlz didn't return a value".to_string())
}

/// cttz(x) - Count trailing zeros
fn builtin_cttz<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("cttz() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("cttz() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.cttz.i{}", int_type.get_bit_width());

    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), codegen.context.bool_type().into()],
        int_type.into(),
    );

    let is_zero_undef = codegen.context.bool_type().const_int(0, false);
    let result = codegen
        .builder
        .build_call(
            intrinsic,
            &[value.into(), is_zero_undef.into()],
            "cttz_call",
        )
        .map_err(|e| format!("Failed to call cttz: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("cttz didn't return a value".to_string())
}

/// ctpop(x) - Count population (number of 1 bits)
fn builtin_ctpop<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("ctpop() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("ctpop() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.ctpop.i{}", int_type.get_bit_width());

    let intrinsic =
        codegen.declare_llvm_intrinsic(&intrinsic_name, &[int_type.into()], int_type.into());

    let result = codegen
        .builder
        .build_call(intrinsic, &[value.into()], "ctpop_call")
        .map_err(|e| format!("Failed to call ctpop: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("ctpop didn't return a value".to_string())
}

/// bswap(x) - Byte swap (reverse byte order)
fn builtin_bswap<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("bswap() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("bswap() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.bswap.i{}", int_type.get_bit_width());

    let intrinsic =
        codegen.declare_llvm_intrinsic(&intrinsic_name, &[int_type.into()], int_type.into());

    let result = codegen
        .builder
        .build_call(intrinsic, &[value.into()], "bswap_call")
        .map_err(|e| format!("Failed to call bswap: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("bswap didn't return a value".to_string())
}

/// bitreverse(x) - Reverse all bits
fn builtin_bitreverse<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("bitreverse() takes exactly one argument".to_string());
    }

    let value = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("bitreverse() argument must be an integer".to_string()),
    };

    let int_type = value.get_type();
    let intrinsic_name = format!("llvm.bitreverse.i{}", int_type.get_bit_width());

    let intrinsic =
        codegen.declare_llvm_intrinsic(&intrinsic_name, &[int_type.into()], int_type.into());

    let result = codegen
        .builder
        .build_call(intrinsic, &[value.into()], "bitreverse_call")
        .map_err(|e| format!("Failed to call bitreverse: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("bitreverse didn't return a value".to_string())
}

// ============================================================================
// LLVM INTRINSICS - OVERFLOW CHECKING
// ============================================================================

/// sadd_overflow(a, b) - Signed add with overflow check
fn builtin_sadd_overflow<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("sadd_overflow() takes exactly two arguments".to_string());
    }

    let a = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("sadd_overflow() first argument must be an integer".to_string()),
    };

    let b = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("sadd_overflow() second argument must be an integer".to_string()),
    };

    let int_type = a.get_type();
    let intrinsic_name = format!("llvm.sadd.with.overflow.i{}", int_type.get_bit_width());

    // Result type is {i32, i1} struct
    let result_struct_type = codegen.context.struct_type(
        &[int_type.into(), codegen.context.bool_type().into()],
        false,
    );

    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), int_type.into()],
        result_struct_type.into(),
    );

    let result = codegen
        .builder
        .build_call(intrinsic, &[a.into(), b.into()], "sadd_overflow_call")
        .map_err(|e| format!("Failed to call sadd_overflow: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("sadd_overflow didn't return a value".to_string())
}

/// ssub_overflow(a, b) - Signed subtract with overflow check
fn builtin_ssub_overflow<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("ssub_overflow() takes exactly two arguments".to_string());
    }

    let a = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("ssub_overflow() first argument must be an integer".to_string()),
    };

    let b = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("ssub_overflow() second argument must be an integer".to_string()),
    };

    let int_type = a.get_type();
    let intrinsic_name = format!("llvm.ssub.with.overflow.i{}", int_type.get_bit_width());

    let result_struct_type = codegen.context.struct_type(
        &[int_type.into(), codegen.context.bool_type().into()],
        false,
    );

    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), int_type.into()],
        result_struct_type.into(),
    );

    let result = codegen
        .builder
        .build_call(intrinsic, &[a.into(), b.into()], "ssub_overflow_call")
        .map_err(|e| format!("Failed to call ssub_overflow: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("ssub_overflow didn't return a value".to_string())
}

/// smul_overflow(a, b) - Signed multiply with overflow check
fn builtin_smul_overflow<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("smul_overflow() takes exactly two arguments".to_string());
    }

    let a = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("smul_overflow() first argument must be an integer".to_string()),
    };

    let b = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("smul_overflow() second argument must be an integer".to_string()),
    };

    let int_type = a.get_type();
    let intrinsic_name = format!("llvm.smul.with.overflow.i{}", int_type.get_bit_width());

    let result_struct_type = codegen.context.struct_type(
        &[int_type.into(), codegen.context.bool_type().into()],
        false,
    );

    let intrinsic = codegen.declare_llvm_intrinsic(
        &intrinsic_name,
        &[int_type.into(), int_type.into()],
        result_struct_type.into(),
    );

    let result = codegen
        .builder
        .build_call(intrinsic, &[a.into(), b.into()], "smul_overflow_call")
        .map_err(|e| format!("Failed to call smul_overflow: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("smul_overflow didn't return a value".to_string())
}

// ============================================================================
// LLVM INTRINSICS - COMPILER HINTS
// ============================================================================

/// assume(condition) - Optimization hint that condition is true
fn builtin_assume<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("assume() takes exactly one argument".to_string());
    }

    let condition = args[0];
    let cond_bool = match condition {
        BasicValueEnum::IntValue(int_val) => {
            let int_type = int_val.get_type();
            // Convert to i1 if not already
            if int_type.get_bit_width() == 1 {
                // Already i1 (bool)
                int_val
            } else {
                // Convert to i1 by comparing with 0
                codegen
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        int_val,
                        int_type.const_int(0, false),
                        "cond_bool",
                    )
                    .map_err(|e| format!("Failed to convert condition to bool: {}", e))?
            }
        }
        _ => return Err("assume() condition must be a boolean or integer".to_string()),
    };

    // Declare llvm.assume
    let intrinsic =
        codegen.declare_llvm_intrinsic_void("llvm.assume", &[codegen.context.bool_type().into()]);

    codegen
        .builder
        .build_call(intrinsic, &[cond_bool.into()], "assume_call")
        .map_err(|e| format!("Failed to call assume: {}", e))?;

    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// likely(x) - Hint that condition is likely true
fn builtin_likely<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("likely() takes exactly one argument".to_string());
    }

    let condition = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("likely() argument must be an integer or boolean".to_string()),
    };

    // Use llvm.expect.i32(val, 1) for "likely true"
    let int_type = condition.get_type();
    let intrinsic = codegen.declare_llvm_intrinsic(
        &format!("llvm.expect.i{}", int_type.get_bit_width()),
        &[int_type.into(), int_type.into()],
        int_type.into(),
    );

    let expected = int_type.const_int(1, false);
    let result = codegen
        .builder
        .build_call(
            intrinsic,
            &[condition.into(), expected.into()],
            "likely_call",
        )
        .map_err(|e| format!("Failed to call likely: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("likely didn't return a value".to_string())
}

/// unlikely(x) - Hint that condition is likely false
fn builtin_unlikely<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("unlikely() takes exactly one argument".to_string());
    }

    let condition = match args[0] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("unlikely() argument must be an integer or boolean".to_string()),
    };

    // Use llvm.expect.i32(val, 0) for "likely false"
    let int_type = condition.get_type();
    let intrinsic = codegen.declare_llvm_intrinsic(
        &format!("llvm.expect.i{}", int_type.get_bit_width()),
        &[int_type.into(), int_type.into()],
        int_type.into(),
    );

    let expected = int_type.const_int(0, false);
    let result = codegen
        .builder
        .build_call(
            intrinsic,
            &[condition.into(), expected.into()],
            "unlikely_call",
        )
        .map_err(|e| format!("Failed to call unlikely: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("unlikely didn't return a value".to_string())
}

/// prefetch(addr, rw, locality, cache_type) - Memory prefetch hint
fn builtin_prefetch<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 4 {
        return Err(
            "prefetch() takes exactly 4 arguments (addr, rw, locality, cache_type)".to_string(),
        );
    }

    let addr = match args[0] {
        BasicValueEnum::PointerValue(ptr_val) => ptr_val,
        _ => return Err("prefetch() first argument must be a pointer".to_string()),
    };

    let rw = match args[1] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("prefetch() rw must be an integer".to_string()),
    };

    let locality = match args[2] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("prefetch() locality must be an integer".to_string()),
    };

    let cache_type = match args[3] {
        BasicValueEnum::IntValue(int_val) => int_val,
        _ => return Err("prefetch() cache_type must be an integer".to_string()),
    };

    // Declare llvm.prefetch
    let i8_ptr_type = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let intrinsic = codegen.declare_llvm_intrinsic_void(
        "llvm.prefetch.p0",
        &[
            i8_ptr_type.into(),
            codegen.context.i32_type().into(),
            codegen.context.i32_type().into(),
            codegen.context.i32_type().into(),
        ],
    );

    codegen
        .builder
        .build_call(
            intrinsic,
            &[addr.into(), rw.into(), locality.into(), cache_type.into()],
            "prefetch_call",
        )
        .map_err(|e| format!("Failed to call prefetch: {}", e))?;

    Ok(codegen.context.i32_type().const_int(0, false).into())
}

// ============================================================================
// HELPER DECLARATIONS
// ============================================================================

impl<'ctx> ASTCodeGen<'ctx> {
    /// Declare vex_malloc from runtime
    pub(crate) fn declare_vex_malloc(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("vex_malloc") {
            return func;
        }

        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let size_type = self.context.i64_type(); // size_t

        let fn_type = i8_ptr_type.fn_type(&[size_type.into()], false);
        self.module.add_function("vex_malloc", fn_type, None)
    }

    /// Declare vex_free from runtime
    pub(crate) fn declare_vex_free(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("vex_free") {
            return func;
        }

        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();

        let fn_type = void_type.fn_type(&[i8_ptr_type.into()], false);
        self.module.add_function("vex_free", fn_type, None)
    }

    /// Declare vex_realloc from runtime
    pub(crate) fn declare_vex_realloc(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("vex_realloc") {
            return func;
        }

        let i8_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let size_type = self.context.i64_type(); // size_t

        let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), size_type.into()], false);
        self.module.add_function("vex_realloc", fn_type, None)
    }

    /// Declare abort() from libc
    pub(crate) fn declare_abort(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("abort") {
            return func;
        }

        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[], false);
        self.module.add_function("abort", fn_type, None)
    }

    /// Declare LLVM intrinsic function with basic return type
    pub(crate) fn declare_llvm_intrinsic(
        &mut self,
        name: &str,
        param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
        return_type: inkwell::types::BasicMetadataTypeEnum<'ctx>,
    ) -> FunctionValue<'ctx> {
        // Check if already declared
        if let Some(func) = self.module.get_function(name) {
            return func;
        }

        // Create function type
        use inkwell::types::BasicMetadataTypeEnum;

        let fn_type = match return_type {
            BasicMetadataTypeEnum::IntType(int_type) => int_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::FloatType(float_type) => float_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::PointerType(ptr_type) => ptr_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::StructType(struct_type) => {
                struct_type.fn_type(param_types, false)
            }
            BasicMetadataTypeEnum::ArrayType(arr_type) => arr_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::VectorType(vec_type) => vec_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::MetadataType(_) => {
                // Metadata type - use i8 as placeholder
                self.context.i8_type().fn_type(param_types, false)
            }
        };

        self.module.add_function(name, fn_type, None)
    }

    /// Declare LLVM intrinsic function with void return type
    pub(crate) fn declare_llvm_intrinsic_void(
        &mut self,
        name: &str,
        param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
    ) -> FunctionValue<'ctx> {
        // Check if already declared
        if let Some(func) = self.module.get_function(name) {
            return func;
        }

        let fn_type = self.context.void_type().fn_type(param_types, false);
        self.module.add_function(name, fn_type, None)
    }
}
