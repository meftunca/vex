// Builtin type method compilation (Vec, Box, etc.)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Try to compile builtin type methods (Vec.push, Vec.len, Box.get, etc.)
    /// Returns Some(value) if the method was handled, None if not a builtin method
    pub(super) fn try_compile_builtin_method(
        &mut self,
        receiver: &Expression,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        // Get receiver variable to check its type
        let var_name = match receiver {
            Expression::Ident(name) => name.clone(),
            _ => return Ok(None), // Not a simple identifier, skip
        };

        // Check if this is a builtin type (Vec, Box)
        let struct_name = self.variable_struct_names.get(&var_name).cloned();

        if let Some(type_name) = struct_name {
            // Handle builtin type methods
            match type_name.as_str() {
                "Vec" => return self.compile_vec_method(&var_name, method, args),
                "Box" => return self.compile_box_method(&var_name, method, args),
                _ => return Ok(None), // Not a builtin type
            }
        }

        Ok(None)
    }

    fn compile_vec_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "push" => {
                if args.len() != 1 {
                    return Err("Vec.push() requires exactly 1 argument".to_string());
                }

                // Get alloca pointer for Vec variable
                let vec_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

                // Load the actual vex_vec_t* pointer from alloca
                let vec_opaque_type = self.context.opaque_struct_type("vex_vec_s");
                let vec_ptr_type = vec_opaque_type.ptr_type(inkwell::AddressSpace::default());
                let vec_ptr = self
                    .builder
                    .build_load(vec_ptr_type, vec_alloca_ptr, "vec_ptr_load")
                    .map_err(|e| format!("Failed to load vec pointer: {}", e))?;

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
                        self.context
                            .i8_type()
                            .ptr_type(inkwell::AddressSpace::default()),
                        "value_void_ptr",
                    )
                    .map_err(|e| format!("Failed to cast value pointer: {}", e))?;

                self.builder
                    .build_call(push_fn, &[vec_ptr.into(), void_ptr.into()], "vec_push")
                    .map_err(|e| format!("Failed to call vex_vec_push: {}", e))?;

                Ok(Some(self.context.i8_type().const_zero().into()))
            }
            "len" => {
                if !args.is_empty() {
                    return Err("Vec.len() takes no arguments".to_string());
                }

                // Get alloca pointer for Vec variable
                let vec_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

                // Load the actual vex_vec_t* pointer from alloca
                let vec_opaque_type = self.context.opaque_struct_type("vex_vec_s");
                let vec_ptr_type = vec_opaque_type.ptr_type(inkwell::AddressSpace::default());
                let vec_ptr = self
                    .builder
                    .build_load(vec_ptr_type, vec_alloca_ptr, "vec_ptr_load")
                    .map_err(|e| format!("Failed to load vec pointer: {}", e))?;

                let len_fn = self.get_vex_vec_len();

                let call_site = self
                    .builder
                    .build_call(len_fn, &[vec_ptr.into()], "vec_len")
                    .map_err(|e| format!("Failed to call vex_vec_len: {}", e))?;

                let len_val = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_vec_len returned void".to_string())?;

                Ok(Some(len_val))
            }
            "get" => {
                if args.len() != 1 {
                    return Err("Vec.get() requires exactly 1 argument (index)".to_string());
                }

                // Get alloca pointer for Vec variable
                let vec_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

                // Load the actual vex_vec_t* pointer from alloca
                let vec_opaque_type = self.context.opaque_struct_type("vex_vec_s");
                let vec_ptr_type = vec_opaque_type.ptr_type(inkwell::AddressSpace::default());
                let vec_ptr = self
                    .builder
                    .build_load(vec_ptr_type, vec_alloca_ptr, "vec_ptr_load")
                    .map_err(|e| format!("Failed to load vec pointer: {}", e))?;

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
                    .left()
                    .ok_or_else(|| "vex_vec_get returned void".to_string())?
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
            _ => Ok(None),
        }
    }

    fn compile_box_method(
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

                let ptr_value = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_box_get returned void".to_string())?;

                Ok(Some(ptr_value))
            }
            _ => Ok(None),
        }
    }
}
