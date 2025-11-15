// Vec and Box builtin method compilation

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(super) fn compile_vec_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "push" => {
                if args.len() != 1 {
                    self.emit_argument_count_mismatch("Vec.push", 1, args.len(), vex_diagnostics::Span::unknown());
                    return Err("Vec.push() requires exactly 1 argument".to_string());
                }

                // Get Vec pointer (already a pointer, no need to load)
                let vec_ptr = match self.variables.get(var_name) {
                    Some(v) => *v,
                    None => {
                        self.emit_undefined_variable(var_name, vex_diagnostics::Span::unknown());
                        return Err(format!("Vec variable {} not found", var_name));
                    }
                };

                let value = self.compile_expression(&args[0])?;

                let value_ptr = self
                    .builder
                    .build_alloca(value.get_type(), "push_value")
                    .map_err(|e| format!("Failed to allocate push value: {}", e))?;
                self.builder
                    .build_store(value_ptr, value)
                    .map_err(|e| format!("Failed to store push value: {}", e))?;

                let push_fn = self.get_vex_vec_push();

                let void_ptr = self
                    .builder
                    .build_pointer_cast(
                        value_ptr,
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                        "value_void_ptr",
                    )
                    .map_err(|e| format!("Failed to cast value pointer: {}", e))?;

                self.builder
                    .build_call(push_fn, &[vec_ptr.into(), void_ptr.into()], "vec_push")
                    .map_err(|e| format!("Failed to call vex_vec_push: {}", e))?;

                // Vec.push() returns void - return unit value (i8 0)
                Ok(Some(self.context.i8_type().const_zero().into()))
            }
            "len" => {
                if !args.is_empty() {
                    self.emit_argument_count_mismatch("Vec.len", 0, args.len(), vex_diagnostics::Span::unknown());
                    return Err("Vec.len() takes no arguments".to_string());
                }

                // Get Vec pointer (already a pointer, no need to load)
                let vec_ptr = match self.variables.get(var_name) {
                    Some(v) => *v,
                    None => {
                        self.emit_undefined_variable(var_name, vex_diagnostics::Span::unknown());
                        return Err(format!("Vec variable {} not found", var_name));
                    }
                };

                let len_fn = self.get_vex_vec_len();

                let call_site = self
                    .builder
                    .build_call(len_fn, &[vec_ptr.into()], "vec_len")
                    .map_err(|e| format!("Failed to call vex_vec_len: {}", e))?;

                let len_val = call_site.try_as_basic_value().unwrap_basic();

                Ok(Some(len_val))
            }
            "get" => {
                if args.len() != 1 {
                    self.emit_argument_count_mismatch("Vec.get", 1, args.len(), vex_diagnostics::Span::unknown());
                    return Err("Vec.get() requires exactly 1 argument (index)".to_string());
                }

                // Get Vec pointer (already a pointer, no need to load)
                let vec_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

                // Compile index expression
                let index = self.compile_expression(&args[0])?;

                // Cast index to i64 (vex_vec_get expects size_t = i64)
                let index_i64 = if index.get_type().is_int_type() {
                    let index_int = index.into_int_value();
                    if index_int.get_type().get_bit_width() < 64 {
                        self.builder
                            .build_int_z_extend(index_int, self.context.i64_type(), "index_i64")
                            .map_err(|e| format!("Failed to extend index to i64: {}", e))?
                            .into()
                    } else {
                        index
                    }
                } else {
                    return Err("Vec.get() index must be an integer".to_string());
                };

                // Call vex_vec_get
                let get_fn = self.get_vex_vec_get();
                let call_site = self
                    .builder
                    .build_call(get_fn, &[vec_ptr.into(), index_i64.into()], "vec_get")
                    .map_err(|e| format!("Failed to call vex_vec_get: {}", e))?;

                // vex_vec_get returns void* - cast back to i32*
                let elem_ptr = call_site
                    .try_as_basic_value()
                    .unwrap_basic()
                    .into_pointer_value();

                // Cast void* to i32* and load
                let i32_ptr_type = self
                    .context
                    .i32_type()
                    .ptr_type(inkwell::AddressSpace::default());
                let typed_elem_ptr = self
                    .builder
                    .build_pointer_cast(elem_ptr, i32_ptr_type, "typed_elem_ptr")
                    .map_err(|e| format!("Failed to cast element pointer: {}", e))?;

                let elem_val = self
                    .builder
                    .build_load(self.context.i32_type(), typed_elem_ptr, "elem_val")
                    .map_err(|e| format!("Failed to load element value: {}", e))?;

                Ok(Some(elem_val))
            }
            "as_slice" => {
                // Vec.as_slice() -> Slice<T>
                // Returns VexSlice { data: void*, len: usize, elem_size: usize }
                // Using LLVM sret attribute for struct return
                if !args.is_empty() {
                    return Err("Vec.as_slice() takes no arguments".to_string());
                }

                // Get Vec pointer
                let vec_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

                let vec_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let vec_ptr = self
                    .builder
                    .build_load(vec_ptr_type, vec_alloca_ptr, "vec_ptr_load")
                    .map_err(|e| format!("Failed to load vec pointer: {}", e))?;

                // Create VexSlice struct type
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let slice_struct_type = self.context.struct_type(
                    &[
                        ptr_type.into(),                // data
                        self.context.i64_type().into(), // len
                        self.context.i64_type().into(), // elem_size
                    ],
                    false,
                );

                // Allocate space for return value on stack
                let slice_alloca = self
                    .builder
                    .build_alloca(slice_struct_type, "slice_ret")
                    .map_err(|e| format!("Failed to alloca slice: {}", e))?;

                // Declare function with sret: void vex_slice_from_vec(VexSlice* sret, vex_vec_t* vec)
                let void_fn_type = self
                    .context
                    .void_type()
                    .fn_type(&[ptr_type.into(), ptr_type.into()], false);
                let slice_fn = self
                    .module
                    .add_function("vex_slice_from_vec", void_fn_type, None);

                // Add sret attribute to first parameter (return slot)
                let sret_attr = self.context.create_type_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("sret"),
                    slice_struct_type.into(),
                );
                slice_fn.add_attribute(inkwell::attributes::AttributeLoc::Param(0), sret_attr);

                // Call: vex_slice_from_vec(&slice_alloca, vec_ptr)
                self.builder
                    .build_call(slice_fn, &[slice_alloca.into(), vec_ptr.into()], "")
                    .map_err(|e| format!("Failed to call vex_slice_from_vec: {}", e))?;

                // Load result from stack
                let slice_val = self
                    .builder
                    .build_load(slice_struct_type, slice_alloca, "slice_load")
                    .map_err(|e| format!("Failed to load slice: {}", e))?;

                Ok(Some(slice_val))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn compile_box_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "get" => {
                if !args.is_empty() {
                    return Err("Box.get() takes no arguments".to_string());
                }

                // Get alloca pointer for Box variable
                let box_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Box variable {} not found", var_name))?;

                // Load the actual vex_box_t* pointer from alloca
                let box_type = self.context.struct_type(
                    &[
                        self.context
                            .i8_type()
                            .ptr_type(inkwell::AddressSpace::default())
                            .into(),
                        self.context.i64_type().into(),
                    ],
                    false,
                );
                let box_ptr_type = box_type.ptr_type(inkwell::AddressSpace::default());
                let box_ptr = self
                    .builder
                    .build_load(box_ptr_type, box_alloca_ptr, "box_ptr_load")
                    .map_err(|e| format!("Failed to load box pointer: {}", e))?;

                let get_fn = self.get_vex_box_get();

                let call_site = self
                    .builder
                    .build_call(get_fn, &[box_ptr.into()], "box_get")
                    .map_err(|e| format!("Failed to call vex_box_get: {}", e))?;

                let ptr_value = call_site.try_as_basic_value().unwrap_basic();

                Ok(Some(ptr_value))
            }
            _ => Ok(None),
        }
    }
}
