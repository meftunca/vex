// Range, Slice, Array, and Channel builtin method compilation

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(super) fn compile_range_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
        inclusive: bool,
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "next" => {
                // r.next(&out) -> bool
                if args.len() != 1 {
                    return Err(
                        "Range.next() requires exactly 1 argument (output pointer)".to_string()
                    );
                }

                // Compile output pointer argument first (before borrowing range_ptr)
                let out_ptr = self.compile_expression(&args[0])?;

                // Get range pointer
                let range_ptr = self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Range variable {} not found", var_name))?
                    .clone();

                // vex_range_next expects: (ptr range, ptr out_value) -> i1
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let fn_name = if inclusive {
                    "vex_range_inclusive_next"
                } else {
                    "vex_range_next"
                };

                let next_fn = self.declare_runtime_fn(
                    fn_name,
                    &[ptr_type.into(), ptr_type.into()],
                    self.context.bool_type().into(),
                );

                let call_site = self
                    .builder
                    .build_call(next_fn, &[range_ptr.into(), out_ptr.into()], "range_next")
                    .map_err(|e| format!("Failed to call {}: {}", fn_name, e))?;

                let result = call_site.try_as_basic_value().unwrap_basic();

                Ok(Some(result))
            }
            "len" => {
                // r.len() -> i64
                if !args.is_empty() {
                    return Err("Range.len() takes no arguments".to_string());
                }

                // Declare function first
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let fn_name = if inclusive {
                    "vex_range_inclusive_len"
                } else {
                    "vex_range_len"
                };

                let len_fn = self.declare_runtime_fn(
                    fn_name,
                    &[ptr_type.into()],
                    self.context.i64_type().into(),
                );

                // Get range pointer after declaring function
                let range_ptr = self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Range variable {} not found", var_name))?
                    .clone();

                let call_site = self
                    .builder
                    .build_call(len_fn, &[range_ptr.into()], "range_len")
                    .map_err(|e| format!("Failed to call {}: {}", fn_name, e))?;

                let result = call_site.try_as_basic_value().unwrap_basic();

                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn compile_slice_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "len" => {
                if !args.is_empty() {
                    return Err("Slice.len() takes no arguments".to_string());
                }

                // Get slice struct pointer from variable
                let slice_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Slice variable {} not found", var_name))?;

                // Call vex_slice_len(&slice)
                let len_fn = self.get_vex_slice_len();

                let call_site = self
                    .builder
                    .build_call(len_fn, &[slice_ptr.into()], "slice_len")
                    .map_err(|e| format!("Failed to call vex_slice_len: {}", e))?;

                let len_val = call_site.try_as_basic_value().unwrap_basic();

                Ok(Some(len_val))
            }
            "get" => {
                if args.len() != 1 {
                    return Err("Slice.get() requires exactly 1 argument (index)".to_string());
                }

                // Get slice struct pointer
                let slice_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Slice variable {} not found", var_name))?;

                // Compile index expression
                let index = self.compile_expression(&args[0])?;

                // Call vex_slice_get(&slice, index)
                let get_fn = self.get_vex_slice_get();

                let call_site = self
                    .builder
                    .build_call(get_fn, &[slice_ptr.into(), index.into()], "slice_get")
                    .map_err(|e| format!("Failed to call vex_slice_get: {}", e))?;

                let elem_ptr = call_site.try_as_basic_value().unwrap_basic();

                // Load the element value (vex_slice_get returns void*)
                // For now, assume i32 elements (TODO: type-specific loading)
                let elem_val = self
                    .builder
                    .build_load(
                        self.context.i32_type(),
                        elem_ptr.into_pointer_value(),
                        "slice_elem",
                    )
                    .map_err(|e| format!("Failed to load slice element: {}", e))?;

                Ok(Some(elem_val))
            }
            "is_empty" => {
                if !args.is_empty() {
                    return Err("Slice.is_empty() takes no arguments".to_string());
                }

                // Get slice struct pointer
                let slice_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Slice variable {} not found", var_name))?;

                // Call vex_slice_is_empty(&slice)
                let is_empty_fn = self.get_vex_slice_is_empty();

                let call_site = self
                    .builder
                    .build_call(is_empty_fn, &[slice_ptr.into()], "slice_is_empty")
                    .map_err(|e| format!("Failed to call vex_slice_is_empty: {}", e))?;

                let result = call_site.try_as_basic_value().unwrap_basic();

                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    /// Compile array methods (len, get)

    pub(super) fn compile_array_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "len" => {
                // arr.len() - returns compile-time constant array size
                if !args.is_empty() {
                    return Err("Array.len() takes no arguments".to_string());
                }

                // Get array type from variable_types
                let array_type = self
                    .variable_types
                    .get(var_name)
                    .ok_or_else(|| format!("Array variable {} not found", var_name))?;

                if let inkwell::types::BasicTypeEnum::ArrayType(arr_ty) = array_type {
                    let len = arr_ty.len();
                    let len_val = self.context.i32_type().const_int(len as u64, false);
                    return Ok(Some(len_val.into()));
                } else {
                    return Err(format!("Variable {} is not an array", var_name));
                }
            }
            "get" => {
                // arr.get(index) - returns Option<T> with bounds checking
                if args.len() != 1 {
                    return Err("Array.get() requires exactly 1 argument (index)".to_string());
                }

                // Get array alloca pointer
                let array_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Array variable {} not found", var_name))?;

                // Get array type (clone to avoid borrow issues)
                let array_type = self
                    .variable_types
                    .get(var_name)
                    .cloned()
                    .ok_or_else(|| format!("Array variable {} type not found", var_name))?;

                let (arr_ty, elem_ty) =
                    if let inkwell::types::BasicTypeEnum::ArrayType(at) = array_type {
                        (at, at.get_element_type())
                    } else {
                        return Err(format!("Variable {} is not an array", var_name));
                    };

                let arr_len = arr_ty.len();
                let index_val = self.compile_expression(&args[0])?;

                // Bounds check
                let index_int = if let BasicValueEnum::IntValue(iv) = index_val {
                    iv
                } else {
                    return Err("Array index must be an integer".to_string());
                };

                let len_const = self.context.i32_type().const_int(arr_len as u64, false);
                let in_bounds = self
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::ULT,
                        index_int,
                        len_const,
                        "bounds_check",
                    )
                    .map_err(|e| format!("Failed to build bounds check: {}", e))?;

                // Create Option<T> return value
                // Option is enum { None, Some(T) }
                // For now, return the element directly (TODO: proper Option return)

                let current_block = self.builder.get_insert_block().unwrap();
                let function = current_block.get_parent().unwrap();

                let in_bounds_block = self.context.append_basic_block(function, "array_get_ok");
                let out_of_bounds_block =
                    self.context.append_basic_block(function, "array_get_oob");
                let merge_block = self.context.append_basic_block(function, "array_get_merge");

                self.builder
                    .build_conditional_branch(in_bounds, in_bounds_block, out_of_bounds_block)
                    .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

                // In bounds: get element
                self.builder.position_at_end(in_bounds_block);
                let zero = self.context.i32_type().const_zero();
                let elem_ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(arr_ty, array_ptr, &[zero, index_int], "elem_ptr")
                        .map_err(|e| format!("Failed to build GEP: {}", e))?
                };
                let elem_val = self
                    .builder
                    .build_load(elem_ty, elem_ptr, "elem_val")
                    .map_err(|e| format!("Failed to load element: {}", e))?;
                self.builder
                    .build_unconditional_branch(merge_block)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;

                // Out of bounds: return None (for now, return 0)
                self.builder.position_at_end(out_of_bounds_block);
                let none_val: BasicValueEnum =
                    if let inkwell::types::BasicTypeEnum::IntType(it) = elem_ty {
                        it.const_zero().into()
                    } else if let inkwell::types::BasicTypeEnum::FloatType(ft) = elem_ty {
                        ft.const_zero().into()
                    } else {
                        return Err("Unsupported array element type for get()".to_string());
                    };
                self.builder
                    .build_unconditional_branch(merge_block)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;

                // Merge: phi node
                self.builder.position_at_end(merge_block);
                let phi = self
                    .builder
                    .build_phi(elem_ty, "array_get_result")
                    .map_err(|e| format!("Failed to build phi: {}", e))?;
                phi.add_incoming(&[
                    (&elem_val, in_bounds_block),
                    (&none_val, out_of_bounds_block),
                ]);

                Ok(Some(phi.as_basic_value()))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn compile_channel_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "send" => {
                if args.len() != 1 {
                    return Err("Channel.send() requires exactly 1 argument".to_string());
                }

                // Get channel pointer from variable
                let channel_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Channel variable {} not found", var_name))?;

                // Load the channel pointer
                let channel_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let channel_ptr = self
                    .builder
                    .build_load(channel_ptr_type, channel_alloca_ptr, "channel_ptr_load")
                    .map_err(|e| format!("Failed to load channel pointer: {}", e))?
                    .into_pointer_value();

                // Compile the value to send
                let value = self.compile_expression(&args[0])?;

                // Delegate to static builtin: Channel.send(ch, value)
                let builtin_fn = self
                    .builtins
                    .get("Channel.send")
                    .ok_or_else(|| "Channel.send builtin not registered".to_string())?;

                let result = builtin_fn(self, &[channel_ptr.into(), value])?;
                return Ok(Some(result));
            }
            "recv" => {
                if !args.is_empty() {
                    return Err("Channel.recv() takes no arguments".to_string());
                }

                // Get channel pointer from variable
                let channel_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Channel variable {} not found", var_name))?;

                // Load the channel pointer
                let channel_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let channel_ptr = self
                    .builder
                    .build_load(channel_ptr_type, channel_alloca_ptr, "channel_ptr_load")
                    .map_err(|e| format!("Failed to load channel pointer: {}", e))?
                    .into_pointer_value();

                // Delegate to static builtin: Channel.recv(ch)
                let builtin_fn = self
                    .builtins
                    .get("Channel.recv")
                    .ok_or_else(|| "Channel.recv builtin not registered".to_string())?;

                let result = builtin_fn(self, &[channel_ptr.into()])?;
                return Ok(Some(result));
            }
            _ => Ok(None),
        }
    }
}
