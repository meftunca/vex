// Vec and Box runtime function declarations

use crate::codegen_ast::ASTCodeGen;
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
