// Builtin Types - Phase 0: Core Types (Vec, Option, Result, Box)
// External C runtime functions compiled to LLVM IR

use super::ASTCodeGen;
use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Get or declare vex_vec_new from runtime
    pub fn get_vex_vec_new(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_vec_new";

        // Check if already declared
        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: vex_vec_t* vex_vec_new(size_t elem_size)
        let size_t = self.context.i64_type();
        let vec_type = self.context.opaque_struct_type("vex_vec_s");
        let vec_ptr_type = vec_type.ptr_type(AddressSpace::default());

        let fn_type = vec_ptr_type.fn_type(&[size_t.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_vec_push from runtime
    pub fn get_vex_vec_push(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_vec_push";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: void vex_vec_push(vex_vec_t *vec, const void *elem)
        let void_type = self.context.void_type();
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let vec_ptr_type = self
            .context
            .opaque_struct_type("vex_vec_s")
            .ptr_type(AddressSpace::default());

        let fn_type = void_type.fn_type(&[vec_ptr_type.into(), ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_vec_get from runtime
    pub fn get_vex_vec_get(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_vec_get";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: void *vex_vec_get(vex_vec_t *vec, size_t index)
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let vec_ptr_type = self
            .context
            .opaque_struct_type("vex_vec_s")
            .ptr_type(AddressSpace::default());
        let size_t = self.context.i64_type();

        let fn_type = ptr_type.fn_type(&[vec_ptr_type.into(), size_t.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_vec_len from runtime
    pub fn get_vex_vec_len(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_vec_len";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: size_t vex_vec_len(vex_vec_t *vec)
        let size_t = self.context.i64_type();
        let vec_ptr_type = self
            .context
            .opaque_struct_type("vex_vec_s")
            .ptr_type(AddressSpace::default());

        let fn_type = size_t.fn_type(&[vec_ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_vec_free from runtime
    pub fn get_vex_vec_free(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_vec_free";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: void vex_vec_free(vex_vec_t *vec)
        let void_type = self.context.void_type();
        let vec_ptr_type = self
            .context
            .opaque_struct_type("vex_vec_s")
            .ptr_type(AddressSpace::default());

        let fn_type = void_type.fn_type(&[vec_ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_box_new from runtime
    pub fn get_vex_box_new(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_box_new";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: vex_box_t* vex_box_new(const void *value, size_t size)
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let size_t = self.context.i64_type();
        let box_type = self
            .context
            .struct_type(&[ptr_type.into(), size_t.into()], false);
        let box_ptr_type = box_type.ptr_type(AddressSpace::default());

        let fn_type = box_ptr_type.fn_type(&[ptr_type.into(), size_t.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_box_get from runtime
    pub fn get_vex_box_get(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_box_get";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: void *vex_box_get(vex_box_t *box)
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let box_ptr_type = self
            .context
            .struct_type(&[ptr_type.into(), self.context.i64_type().into()], false)
            .ptr_type(AddressSpace::default());

        let fn_type = ptr_type.fn_type(&[box_ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_box_free from runtime
    pub fn get_vex_box_free(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_box_free";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: void vex_box_free(vex_box_t *box)
        let void_type = self.context.void_type();
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let box_type = self
            .context
            .struct_type(&[ptr_type.into(), self.context.i64_type().into()], false);
        let box_ptr_type = box_type.ptr_type(AddressSpace::default());

        let fn_type = void_type.fn_type(&[box_ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_option_unwrap from runtime
    pub fn get_vex_option_unwrap(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_option_unwrap";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: void *vex_option_unwrap(void *opt_ptr, size_t type_size, const char *file, int line)
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let size_t = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let fn_type = ptr_type.fn_type(
            &[
                ptr_type.into(),
                size_t.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_option_is_some from runtime
    pub fn get_vex_option_is_some(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_option_is_some";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: bool vex_option_is_some(void *opt_ptr)
        let bool_type = self.context.bool_type();
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = bool_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_result_unwrap from runtime
    pub fn get_vex_result_unwrap(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_result_unwrap";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: void *vex_result_unwrap(void *result_ptr, size_t type_size, const char *file, int line)
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let size_t = self.context.i64_type();
        let i32_type = self.context.i32_type();

        let fn_type = ptr_type.fn_type(
            &[
                ptr_type.into(),
                size_t.into(),
                ptr_type.into(),
                i32_type.into(),
            ],
            false,
        );
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_result_is_ok from runtime
    pub fn get_vex_result_is_ok(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_result_is_ok";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: bool vex_result_is_ok(void *result_ptr)
        let bool_type = self.context.bool_type();
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = bool_type.fn_type(&[ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }
}

/// Builtin function: Vec operations are handled by the above declarations
/// These are called during codegen when compiling Vec<T> types
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

// ========== PHASE 0.4b: BUILTIN CONSTRUCTOR FUNCTIONS ==========

/// Builtin: vec_new() - Create empty Vec<T>
/// Usage: let v = vec_new::<i32>();
pub fn builtin_vec_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Get element size from type context (default to 4 bytes for now)
    // TODO: Get actual type from generic parameter
    let elem_size = codegen.context.i64_type().const_int(4, false);

    // Call vex_vec_new(elem_size)
    // Call vex_vec_new(elem_size) - returns vex_vec_t* pointer
    let vec_new_fn = codegen.get_vex_vec_new();
    let call_site = codegen
        .builder
        .build_call(vec_new_fn, &[elem_size.into()], "vec_new")
        .map_err(|e| format!("Failed to call vex_vec_new: {}", e))?;

    // Return pointer directly
    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_vec_new returned void".to_string())
}

/// Builtin: vec_free() - Free Vec<T>
/// Usage: vec_free(v);  // v is Vec (pointer), pass by value
pub fn builtin_vec_free<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("vec_free() requires exactly 1 argument".to_string());
    }

    // arg[0] is alloca pointer holding vex_vec_t* - need to load it
    let vec_alloca = args[0].into_pointer_value();

    let vec_opaque_type = codegen.context.opaque_struct_type("vex_vec_s");
    let vec_ptr_type = vec_opaque_type.ptr_type(inkwell::AddressSpace::default());

    let vec_ptr = codegen
        .builder
        .build_load(vec_ptr_type, vec_alloca, "vec_ptr_load")
        .map_err(|e| format!("Failed to load vec pointer for free: {}", e))?;

    // Call vex_vec_free(vec_ptr)
    let vec_free_fn = codegen.get_vex_vec_free();
    codegen
        .builder
        .build_call(vec_free_fn, &[vec_ptr.into()], "vec_free")
        .map_err(|e| format!("Failed to call vex_vec_free: {}", e))?;

    // Return unit (i8 zero)
    Ok(codegen.context.i8_type().const_zero().into())
}

/// Builtin: box_new() - Create Box<T> with value
/// Usage: let b = box_new(42);
pub fn builtin_box_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("box_new() requires exactly 1 argument".to_string());
    }

    let value = args[0];
    let value_type = value.get_type();

    // Get size of value type
    let size = match value_type {
        inkwell::types::BasicTypeEnum::IntType(it) => (it.get_bit_width() / 8) as u64,
        inkwell::types::BasicTypeEnum::FloatType(_) => 8, // Assume f64
        inkwell::types::BasicTypeEnum::PointerType(_) => 8,
        _ => 8, // Default
    };

    // Allocate value on stack to get pointer
    let value_ptr = codegen
        .builder
        .build_alloca(value_type, "box_value")
        .map_err(|e| format!("Failed to allocate box value: {}", e))?;
    codegen
        .builder
        .build_store(value_ptr, value)
        .map_err(|e| format!("Failed to store box value: {}", e))?;

    // Call vex_box_new(value_ptr, size)
    let box_new_fn = codegen.get_vex_box_new();
    let size_val = codegen.context.i64_type().const_int(size, false);

    // Cast value_ptr to i8*
    let void_ptr = codegen
        .builder
        .build_pointer_cast(
            value_ptr,
            codegen.context.i8_type().ptr_type(AddressSpace::default()),
            "value_void_ptr",
        )
        .map_err(|e| format!("Failed to cast value pointer: {}", e))?;

    // Call vex_box_new(value_ptr, size) - returns vex_box_t* pointer
    let call_site = codegen
        .builder
        .build_call(box_new_fn, &[void_ptr.into(), size_val.into()], "box_new")
        .map_err(|e| format!("Failed to call vex_box_new: {}", e))?;

    // Return pointer directly
    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_box_new returned void".to_string())
}

/// Builtin: box_free() - Free Box<T>
/// Usage: box_free(b);  // b is Box (pointer), pass by value
pub fn builtin_box_free<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("box_free() requires exactly 1 argument".to_string());
    }

    // arg[0] is alloca pointer holding vex_box_t* - need to load it
    let box_alloca = args[0].into_pointer_value();

    let box_type = codegen.context.struct_type(
        &[
            codegen
                .context
                .i8_type()
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            codegen.context.i64_type().into(),
        ],
        false,
    );
    let box_ptr_type = box_type.ptr_type(inkwell::AddressSpace::default());

    let box_ptr = codegen
        .builder
        .build_load(box_ptr_type, box_alloca, "box_ptr_load")
        .map_err(|e| format!("Failed to load box pointer for free: {}", e))?;

    // Call vex_box_free(box_ptr)
    let box_free_fn = codegen.get_vex_box_free();
    codegen
        .builder
        .build_call(box_free_fn, &[box_ptr.into()], "box_free")
        .map_err(|e| format!("Failed to call vex_box_free: {}", e))?;

    // Return unit (i8 zero)
    Ok(codegen.context.i8_type().const_zero().into())
}

impl<'ctx> ASTCodeGen<'ctx> {
    /// Get or declare vex_i32_to_string from runtime
    pub fn get_vex_i32_to_string(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_i32_to_string";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: char* vex_i32_to_string(int32_t value)
        let i32_type = self.context.i32_type();
        let str_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = str_ptr_type.fn_type(&[i32_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_i64_to_string from runtime
    pub fn get_vex_i64_to_string(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_i64_to_string";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: char* vex_i64_to_string(int64_t value)
        let i64_type = self.context.i64_type();
        let str_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = str_ptr_type.fn_type(&[i64_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_u32_to_string from runtime
    pub fn get_vex_u32_to_string(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_u32_to_string";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: char* vex_u32_to_string(uint32_t value)
        let u32_type = self.context.i32_type(); // u32 same as i32 in LLVM
        let str_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = str_ptr_type.fn_type(&[u32_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_u64_to_string from runtime
    pub fn get_vex_u64_to_string(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_u64_to_string";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: char* vex_u64_to_string(uint64_t value)
        let u64_type = self.context.i64_type();
        let str_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = str_ptr_type.fn_type(&[u64_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_f32_to_string from runtime
    pub fn get_vex_f32_to_string(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_f32_to_string";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: char* vex_f32_to_string(float value)
        let f32_type = self.context.f32_type();
        let str_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = str_ptr_type.fn_type(&[f32_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_f64_to_string from runtime
    pub fn get_vex_f64_to_string(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_f64_to_string";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: char* vex_f64_to_string(double value)
        let f64_type = self.context.f64_type();
        let str_ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());

        let fn_type = str_ptr_type.fn_type(&[f64_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }
}

// ========== PHASE 0.7: NUMERIC TO STRING BUILTINS ==========

/// Builtin: vex_i32_to_string(value: i32): *const u8
/// Converts i32 to heap-allocated string
pub fn builtin_vex_i32_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "vex_i32_to_string expects 1 argument, got {}",
            args.len()
        ));
    }

    let fn_val = codegen.get_vex_i32_to_string();
    let call_site = codegen
        .builder
        .build_call(fn_val, &[args[0].into()], "i32_to_str")
        .map_err(|e| format!("Failed to call vex_i32_to_string: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_i32_to_string returned void".to_string())
}

/// Builtin: vex_i64_to_string(value: i64): *const u8
pub fn builtin_vex_i64_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "vex_i64_to_string expects 1 argument, got {}",
            args.len()
        ));
    }

    let fn_val = codegen.get_vex_i64_to_string();
    let call_site = codegen
        .builder
        .build_call(fn_val, &[args[0].into()], "i64_to_str")
        .map_err(|e| format!("Failed to call vex_i64_to_string: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_i64_to_string returned void".to_string())
}

/// Builtin: vex_u32_to_string(value: u32): *const u8
pub fn builtin_vex_u32_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "vex_u32_to_string expects 1 argument, got {}",
            args.len()
        ));
    }

    let fn_val = codegen.get_vex_u32_to_string();
    let call_site = codegen
        .builder
        .build_call(fn_val, &[args[0].into()], "u32_to_str")
        .map_err(|e| format!("Failed to call vex_u32_to_string: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_u32_to_string returned void".to_string())
}

/// Builtin: vex_u64_to_string(value: u64): *const u8
pub fn builtin_vex_u64_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "vex_u64_to_string expects 1 argument, got {}",
            args.len()
        ));
    }

    let fn_val = codegen.get_vex_u64_to_string();
    let call_site = codegen
        .builder
        .build_call(fn_val, &[args[0].into()], "u64_to_str")
        .map_err(|e| format!("Failed to call vex_u64_to_string: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_u64_to_string returned void".to_string())
}

/// Builtin: vex_f32_to_string(value: f32): *const u8
pub fn builtin_vex_f32_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "vex_f32_to_string expects 1 argument, got {}",
            args.len()
        ));
    }

    let fn_val = codegen.get_vex_f32_to_string();
    let call_site = codegen
        .builder
        .build_call(fn_val, &[args[0].into()], "f32_to_str")
        .map_err(|e| format!("Failed to call vex_f32_to_string: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_f32_to_string returned void".to_string())
}

/// Builtin: vex_f64_to_string(value: f64): *const u8
pub fn builtin_vex_f64_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "vex_f64_to_string expects 1 argument, got {}",
            args.len()
        ));
    }

    let fn_val = codegen.get_vex_f64_to_string();
    let call_site = codegen
        .builder
        .build_call(fn_val, &[args[0].into()], "f64_to_str")
        .map_err(|e| format!("Failed to call vex_f64_to_string: {}", e))?;

    call_site
        .try_as_basic_value()
        .left()
        .ok_or_else(|| "vex_f64_to_string returned void".to_string())
}

// ========== PHASE 0.8: OPTION<T> CONSTRUCTORS ==========

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

// ========== PHASE 0.8: RESULT<T,E> CONSTRUCTORS ==========

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
