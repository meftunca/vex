// Slice<T> builtin functions
// Runtime: vex_slice.c

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicValueEnum, FunctionValue};
use inkwell::AddressSpace;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Get or declare vex_slice_from_vec from runtime
    pub fn get_vex_slice_from_vec(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_slice_from_vec";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: VexSlice vex_slice_from_vec(vex_vec_t *vec)
        let vec_ptr_type = self.context.ptr_type(AddressSpace::default());

        // VexSlice struct: { void*, size_t, size_t }
        let ptr_ty = self.context.ptr_type(AddressSpace::default());
        let size_ty = self.context.i64_type();
        let slice_struct = self
            .context
            .struct_type(&[ptr_ty.into(), size_ty.into(), size_ty.into()], false);

        let fn_type = slice_struct.fn_type(&[vec_ptr_type.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_slice_new from runtime
    pub fn get_vex_slice_new(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_slice_new";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: VexSlice vex_slice_new(void *data, size_t len, size_t elem_size)
        let ptr_ty = self.context.ptr_type(AddressSpace::default());
        let size_ty = self.context.i64_type();

        let slice_struct = self
            .context
            .struct_type(&[ptr_ty.into(), size_ty.into(), size_ty.into()], false);

        let fn_type = slice_struct.fn_type(&[ptr_ty.into(), size_ty.into(), size_ty.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_slice_get from runtime
    pub fn get_vex_slice_get(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_slice_get";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: void* vex_slice_get(const VexSlice *slice, size_t index)
        let ptr_ty = self.context.ptr_type(AddressSpace::default());
        let size_ty = self.context.i64_type();

        let fn_type = ptr_ty.fn_type(&[ptr_ty.into(), size_ty.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_slice_len from runtime
    pub fn get_vex_slice_len(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_slice_len";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: size_t vex_slice_len(const VexSlice *slice)
        let ptr_ty = self.context.ptr_type(AddressSpace::default());
        let size_ty = self.context.i64_type();

        let fn_type = size_ty.fn_type(&[ptr_ty.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_slice_is_empty from runtime
    pub fn get_vex_slice_is_empty(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_slice_is_empty";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: bool vex_slice_is_empty(const VexSlice *slice)
        let ptr_ty = self.context.ptr_type(AddressSpace::default());
        let bool_ty = self.context.bool_type();

        let fn_type = bool_ty.fn_type(&[ptr_ty.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }

    /// Get or declare vex_slice_subslice from runtime
    pub fn get_vex_slice_subslice(&mut self) -> FunctionValue<'ctx> {
        let fn_name = "vex_slice_subslice";

        if let Some(func) = self.module.get_function(fn_name) {
            return func;
        }

        // Declare: VexSlice vex_slice_subslice(const VexSlice *slice, size_t start, size_t end)
        let ptr_ty = self.context.ptr_type(AddressSpace::default());
        let size_ty = self.context.i64_type();

        let slice_struct = self
            .context
            .struct_type(&[ptr_ty.into(), size_ty.into(), size_ty.into()], false);

        let fn_type = slice_struct.fn_type(&[ptr_ty.into(), size_ty.into(), size_ty.into()], false);
        self.module.add_function(fn_name, fn_type, None)
    }
}

/// Builtin slice_from_vec(vec) - Create slice from Vec
pub fn builtin_slice_from_vec<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("slice_from_vec() takes exactly one argument".to_string());
    }

    let vec_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("slice_from_vec() argument must be a Vec pointer".to_string()),
    };

    let slice_from_vec_fn = codegen.get_vex_slice_from_vec();

    let result = codegen
        .builder
        .build_call(slice_from_vec_fn, &[vec_ptr.into()], "slice_from_vec_call")
        .map_err(|e| format!("Failed to call slice_from_vec: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// Builtin slice_new(data, len, elem_size) - Create slice from raw data
pub fn builtin_slice_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err("slice_new() takes exactly three arguments".to_string());
    }

    let data_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("slice_new() first argument must be a pointer".to_string()),
    };

    let len = match args[1] {
        BasicValueEnum::IntValue(i) => i,
        _ => return Err("slice_new() second argument must be an integer".to_string()),
    };

    let elem_size = match args[2] {
        BasicValueEnum::IntValue(i) => i,
        _ => return Err("slice_new() third argument must be an integer".to_string()),
    };

    let slice_new_fn = codegen.get_vex_slice_new();

    let result = codegen
        .builder
        .build_call(
            slice_new_fn,
            &[data_ptr.into(), len.into(), elem_size.into()],
            "slice_new_call",
        )
        .map_err(|e| format!("Failed to call slice_new: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// Builtin slice_get(slice, index) - Get element from slice
pub fn builtin_slice_get<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("slice_get() takes exactly two arguments".to_string());
    }

    let slice_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("slice_get() first argument must be a slice pointer".to_string()),
    };

    let index = match args[1] {
        BasicValueEnum::IntValue(i) => i,
        _ => return Err("slice_get() second argument must be an integer".to_string()),
    };

    let slice_get_fn = codegen.get_vex_slice_get();

    let result = codegen
        .builder
        .build_call(
            slice_get_fn,
            &[slice_ptr.into(), index.into()],
            "slice_get_call",
        )
        .map_err(|e| format!("Failed to call slice_get: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}

/// Builtin slice_len(slice) - Get slice length
pub fn builtin_slice_len<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("slice_len() takes exactly one argument".to_string());
    }

    let slice_ptr = match args[0] {
        BasicValueEnum::PointerValue(ptr) => ptr,
        _ => return Err("slice_len() argument must be a slice pointer".to_string()),
    };

    let slice_len_fn = codegen.get_vex_slice_len();

    let result = codegen
        .builder
        .build_call(slice_len_fn, &[slice_ptr.into()], "slice_len_call")
        .map_err(|e| format!("Failed to call slice_len: {}", e))?;

    Ok(result.try_as_basic_value().unwrap_basic())
}
