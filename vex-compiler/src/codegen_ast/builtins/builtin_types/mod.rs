// Builtin types module (split into sub-modules)
pub(crate) mod collections;
pub(crate) mod conversions;
pub(crate) mod option_result;

pub use collections::*;
pub use conversions::*;
pub use option_result::*;

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// Register all builtin types Phase 0
pub fn register_builtin_types_phase0<'ctx>(codegen: &mut ASTCodeGen<'ctx>) {
    // Pre-declare all Phase 0 builtin type functions
    // This ensures they're available during codegen

    codegen.get_vex_vec_new();
    codegen.get_vex_vec_push();
    codegen.get_vex_vec_get();
    codegen.get_vex_vec_len();
    codegen.get_vex_vec_free();

    codegen.get_vex_box_new();
    codegen.get_vex_box_get();
    codegen.get_vex_box_free();

    codegen.get_vex_option_unwrap();
    codegen.get_vex_option_is_some();

    codegen.get_vex_result_unwrap();
    codegen.get_vex_result_is_ok();

    // Phase 0.7: Numeric to string conversions
    codegen.get_vex_i32_to_string();
    codegen.get_vex_i64_to_string();
    codegen.get_vex_u32_to_string();
    codegen.get_vex_u64_to_string();
    codegen.get_vex_f32_to_string();
    codegen.get_vex_f64_to_string();
}

// ========== BUILTIN CONSTRUCTOR FUNCTIONS ==========

/// Builtin: vec_new() - Create empty Vec<T>
pub fn builtin_vec_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    let elem_size = codegen.context.i64_type().const_int(4, false);
    let vec_new_fn = codegen.get_vex_vec_new();
    let call_site = codegen
        .builder
        .build_call(vec_new_fn, &[elem_size.into()], "vec_new")
        .map_err(|e| format!("Failed to call vex_vec_new: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_vec_new returned void".to_string())
}

/// Builtin: vec_with_capacity(capacity: u64) - Create Vec<T> with pre-allocated capacity
pub fn builtin_vec_with_capacity<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("vec_with_capacity() requires exactly 1 argument (capacity)".to_string());
    }

    // TODO: Implement vex_vec_with_capacity in C runtime
    // For now, just create a normal vec (capacity parameter ignored)
    let elem_size = codegen.context.i64_type().const_int(4, false);
    let vec_new_fn = codegen.get_vex_vec_new();
    let call_site = codegen
        .builder
        .build_call(vec_new_fn, &[elem_size.into()], "vec_with_capacity")
        .map_err(|e| format!("Failed to call vex_vec_new: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_vec_new returned void".to_string())
}

/// Builtin: vec_free() - Free Vec<T>
pub fn builtin_vec_free<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("vec_free() requires exactly 1 argument".to_string());
    }

    let vec_alloca = args[0].into_pointer_value();
    let vec_opaque_type = codegen.context.opaque_struct_type("vex_vec_s");
    let vec_ptr_type = vec_opaque_type.ptr_type(AddressSpace::default());

    let vec_ptr = codegen
        .builder
        .build_load(vec_ptr_type, vec_alloca, "vec_ptr_load")
        .map_err(|e| format!("Failed to load vec pointer for free: {}", e))?;

    let vec_free_fn = codegen.get_vex_vec_free();
    codegen
        .builder
        .build_call(vec_free_fn, &[vec_ptr.into()], "vec_free")
        .map_err(|e| format!("Failed to call vex_vec_free: {}", e))?;

    Ok(codegen.context.i8_type().const_zero().into())
}

/// Builtin: box_new() - Create Box<T> with value
pub fn builtin_box_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("box_new() requires exactly 1 argument".to_string());
    }

    let value = args[0];
    let value_type = value.get_type();

    let size = match value_type {
        inkwell::types::BasicTypeEnum::IntType(it) => (it.get_bit_width() / 8) as u64,
        inkwell::types::BasicTypeEnum::FloatType(_) => 8,
        inkwell::types::BasicTypeEnum::PointerType(_) => 8,
        _ => 8,
    };

    let value_ptr = codegen
        .builder
        .build_alloca(value_type, "box_value")
        .map_err(|e| format!("Failed to allocate box value: {}", e))?;
    codegen
        .builder
        .build_store(value_ptr, value)
        .map_err(|e| format!("Failed to store box value: {}", e))?;

    let box_new_fn = codegen.get_vex_box_new();
    let size_val = codegen.context.i64_type().const_int(size, false);

    let void_ptr = codegen
        .builder
        .build_pointer_cast(
            value_ptr,
            codegen.context.i8_type().ptr_type(AddressSpace::default()),
            "value_void_ptr",
        )
        .map_err(|e| format!("Failed to cast value pointer: {}", e))?;

    let call_site = codegen
        .builder
        .build_call(box_new_fn, &[void_ptr.into(), size_val.into()], "box_new")
        .map_err(|e| format!("Failed to call vex_box_new: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_box_new returned void".to_string())
}

/// Builtin: box_free() - Free Box<T>
pub fn builtin_box_free<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("box_free() requires exactly 1 argument".to_string());
    }

    let box_alloca = args[0].into_pointer_value();

    let box_type = codegen.context.struct_type(
        &[
            codegen
                .context
                .i8_type()
                .ptr_type(AddressSpace::default())
                .into(),
            codegen.context.i64_type().into(),
        ],
        false,
    );
    let box_ptr_type = box_type.ptr_type(AddressSpace::default());

    let box_ptr = codegen
        .builder
        .build_load(box_ptr_type, box_alloca, "box_ptr_load")
        .map_err(|e| format!("Failed to load box pointer for free: {}", e))?;

    let box_free_fn = codegen.get_vex_box_free();
    codegen
        .builder
        .build_call(box_free_fn, &[box_ptr.into()], "box_free")
        .map_err(|e| format!("Failed to call vex_box_free: {}", e))?;

    Ok(codegen.context.i8_type().const_zero().into())
}

/// Builtin: string_new() - Create empty String
/// Returns vex_string_t* (heap-allocated String struct)
pub fn builtin_string_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Call vex_string_new() -> vex_string_t*
    let ptr_type = codegen.context.ptr_type(inkwell::AddressSpace::default());

    let string_new_fn = codegen.declare_runtime_fn("vex_string_new", &[], ptr_type.into());

    let result = codegen
        .builder
        .build_call(string_new_fn, &[], "string_new")
        .map_err(|e| format!("Failed to call vex_string_new: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("vex_string_new didn't return a value".to_string())
}

/// Builtin: string_from(literal) - Create String from string literal
/// Returns vex_string_t* (heap-allocated String struct)
pub fn builtin_string_from<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("string_from() requires exactly 1 argument".to_string());
    }

    // Get string literal pointer
    let str_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("string_from() requires a string literal argument".to_string()),
    };

    // Call vex_string_from_cstr(cstr) -> vex_string_t*
    let ptr_type = codegen.context.ptr_type(inkwell::AddressSpace::default());

    let string_from_fn =
        codegen.declare_runtime_fn("vex_string_from_cstr", &[ptr_type.into()], ptr_type.into());

    let result = codegen
        .builder
        .build_call(string_from_fn, &[str_ptr.into()], "string_from")
        .map_err(|e| format!("Failed to call vex_string_from_cstr: {}", e))?;

    result
        .try_as_basic_value()
        .left()
        .ok_or("vex_string_from_cstr didn't return a value".to_string())
}

/// Builtin: string_free() - Free String memory
pub fn builtin_string_free<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("string_free() requires exactly 1 argument".to_string());
    }

    let string_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("string_free() requires a String pointer".to_string()),
    };

    // Call vex_string_free(str_ptr) - void function
    let ptr_type = codegen.context.ptr_type(inkwell::AddressSpace::default());

    let void_fn_type = codegen
        .context
        .void_type()
        .fn_type(&[ptr_type.into()], false);
    let string_free_fn = codegen
        .module
        .add_function("vex_string_free", void_fn_type, None);

    codegen
        .builder
        .build_call(string_free_fn, &[string_ptr.into()], "")
        .map_err(|e| format!("Failed to call vex_string_free: {}", e))?;

    Ok(codegen.context.i8_type().const_zero().into())
}
