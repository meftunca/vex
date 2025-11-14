// Type conversion runtime functions (primitive types to string)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;

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

    /// Get or declare vex_bool_to_string from runtime
    pub fn get_vex_bool_to_string(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_bool_to_string";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: char* vex_bool_to_string(bool value)
        let bool_type = self.context.bool_type();
        let str_ptr_type = self.context.ptr_type(AddressSpace::default());

        let fn_type = str_ptr_type.fn_type(&[bool_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_string_to_string from runtime
    pub fn get_vex_string_to_string(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_string_to_string";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: char* vex_string_to_string(const char* value)
        let str_ptr_type = self.context.ptr_type(AddressSpace::default());

        let fn_type = str_ptr_type.fn_type(&[str_ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }
}

// ========== BUILTIN FUNCTIONS ==========

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

    Ok(call_site.try_as_basic_value().unwrap_basic())
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

    Ok(call_site.try_as_basic_value().unwrap_basic())
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

    Ok(call_site.try_as_basic_value().unwrap_basic())
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

    Ok(call_site.try_as_basic_value().unwrap_basic())
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

    Ok(call_site.try_as_basic_value().unwrap_basic())
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

    Ok(call_site.try_as_basic_value().unwrap_basic())
}

/// Builtin: vex_bool_to_string(value: bool): *const u8
pub fn builtin_vex_bool_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "vex_bool_to_string expects 1 argument, got {}",
            args.len()
        ));
    }

    let fn_val = codegen.get_vex_bool_to_string();
    let call_site = codegen
        .builder
        .build_call(fn_val, &[args[0].into()], "bool_to_str")
        .map_err(|e| format!("Failed to call vex_bool_to_string: {}", e))?;

    Ok(call_site.try_as_basic_value().unwrap_basic())
}

/// Builtin: vex_string_to_string(value: *const u8): *const u8
/// Returns a heap-allocated copy of the input string
pub fn builtin_vex_string_to_string<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "vex_string_to_string expects 1 argument, got {}",
            args.len()
        ));
    }

    let fn_val = codegen.get_vex_string_to_string();
    let call_site = codegen
        .builder
        .build_call(fn_val, &[args[0].into()], "string_to_str")
        .map_err(|e| format!("Failed to call vex_string_to_string: {}", e))?;

    Ok(call_site.try_as_basic_value().unwrap_basic())
}
