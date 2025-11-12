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
            // ⚠️ CRITICAL: Check if this is a user-defined struct vs builtin type
            // User-defined structs take precedence over builtin types with same name
            let is_user_defined_struct = self.struct_defs.contains_key(&type_name);
            
            // Extract base type name (Vec_i32 -> Vec, Box_string -> Box)
            let base_type = type_name.split('_').next().unwrap_or(&type_name);
            
            // Handle builtin type methods ONLY if not shadowed by user struct
            if !is_user_defined_struct {
                match base_type {
                    "Vec" => return self.compile_vec_method(&var_name, method, args),
                    "Box" => return self.compile_box_method(&var_name, method, args),
                    "String" => return self.compile_string_method(&var_name, method, args),
                    "Map" => return self.compile_map_method(&var_name, method, args),
                    "Set" => return self.compile_set_method(&var_name, method, args),
                    "Range" => return self.compile_range_method(&var_name, method, args, false),
                    "RangeInclusive" => {
                        return self.compile_range_method(&var_name, method, args, true)
                    }
                    "Slice" => return self.compile_slice_method(&var_name, method, args),
                    "Channel" => return self.compile_channel_method(&var_name, method, args),
                    _ => return Ok(None), // Not a builtin type
                }
            }
        }

        // Check if this is an array type (arrays don't have struct names)
        if let Some(var_type) = self.variable_types.get(&var_name) {
            if let inkwell::types::BasicTypeEnum::ArrayType(_) = var_type {
                return self.compile_array_method(&var_name, method, args);
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

                // Get Vec pointer (already a pointer, no need to load)
                let vec_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

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
                    return Err("Vec.len() takes no arguments".to_string());
                }

                // Get Vec pointer (already a pointer, no need to load)
                let vec_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

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

    fn compile_string_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "len" => {
                // String.len() -> size_t (byte length)
                if !args.is_empty() {
                    return Err("String.len() takes no arguments".to_string());
                }

                // Get vex_string_t* from variable
                let string_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("String variable {} not found", var_name))?;

                // Load vex_string_t*
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let string_ptr = self
                    .builder
                    .build_load(ptr_type, string_alloca_ptr, "string_ptr_load")
                    .map_err(|e| format!("Failed to load string pointer: {}", e))?;

                // Call vex_string_len(vex_string_t*) -> size_t
                let vex_string_len_fn = self.declare_runtime_fn(
                    "vex_string_len",
                    &[ptr_type.into()],
                    self.context.i64_type().into(),
                );

                let result = self
                    .builder
                    .build_call(vex_string_len_fn, &[string_ptr.into()], "string_len")
                    .map_err(|e| format!("Failed to call vex_string_len: {}", e))?;

                result
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_string_len returned void".to_string())
                    .map(Some)
            }
            "is_empty" => {
                // String.is_empty() -> bool
                if !args.is_empty() {
                    return Err("String.is_empty() takes no arguments".to_string());
                }

                // Get vex_string_t*
                let string_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("String variable {} not found", var_name))?;

                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let string_ptr = self
                    .builder
                    .build_load(ptr_type, string_alloca_ptr, "string_ptr_load")
                    .map_err(|e| format!("Failed to load string pointer: {}", e))?;

                // Call vex_string_is_empty(vex_string_t*) -> bool
                let vex_string_is_empty_fn = self.declare_runtime_fn(
                    "vex_string_is_empty",
                    &[ptr_type.into()],
                    self.context.bool_type().into(),
                );

                let result = self
                    .builder
                    .build_call(
                        vex_string_is_empty_fn,
                        &[string_ptr.into()],
                        "string_is_empty",
                    )
                    .map_err(|e| format!("Failed to call vex_string_is_empty: {}", e))?;

                result
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_string_is_empty returned void".to_string())
                    .map(Some)
            }
            "char_count" => {
                // String.char_count() -> size_t (UTF-8 character count)
                if !args.is_empty() {
                    return Err("String.char_count() takes no arguments".to_string());
                }

                let string_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("String variable {} not found", var_name))?;

                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let string_ptr = self
                    .builder
                    .build_load(ptr_type, string_alloca_ptr, "string_ptr_load")
                    .map_err(|e| format!("Failed to load string pointer: {}", e))?;

                // Call vex_string_char_count(vex_string_t*) -> size_t
                let vex_string_char_count_fn = self.declare_runtime_fn(
                    "vex_string_char_count",
                    &[ptr_type.into()],
                    self.context.i64_type().into(),
                );

                let result = self
                    .builder
                    .build_call(
                        vex_string_char_count_fn,
                        &[string_ptr.into()],
                        "string_char_count",
                    )
                    .map_err(|e| format!("Failed to call vex_string_char_count: {}", e))?;

                result
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_string_char_count returned void".to_string())
                    .map(Some)
            }
            "push_str" => {
                // String.push_str(s: &str) -> void
                if args.len() != 1 {
                    return Err("String.push_str() takes exactly 1 argument".to_string());
                }

                let string_alloca_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("String variable {} not found", var_name))?;

                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let string_ptr = self
                    .builder
                    .build_load(ptr_type, string_alloca_ptr, "string_ptr_load")
                    .map_err(|e| format!("Failed to load string pointer: {}", e))?;

                // Compile argument (string literal)
                let arg_val = self.compile_expression(&args[0])?;
                let arg_ptr = match arg_val {
                    BasicValueEnum::PointerValue(ptr) => ptr,
                    _ => return Err("String.push_str() requires a string argument".to_string()),
                };

                // Call vex_string_push_str(vex_string_t*, const char*)
                let void_fn_type = self
                    .context
                    .void_type()
                    .fn_type(&[ptr_type.into(), ptr_type.into()], false);
                let vex_string_push_str_fn =
                    self.module
                        .add_function("vex_string_push_str", void_fn_type, None);

                self.builder
                    .build_call(
                        vex_string_push_str_fn,
                        &[string_ptr.into(), arg_ptr.into()],
                        "",
                    )
                    .map_err(|e| format!("Failed to call vex_string_push_str: {}", e))?;

                Ok(Some(self.context.i8_type().const_zero().into()))
            }
            _ => Ok(None),
        }
    }

    fn compile_map_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "insert" => {
                if args.len() != 2 {
                    return Err("Map.insert() requires 2 arguments (key, value)".to_string());
                }

                // Get Map pointer from variable
                let map_ptr_alloca = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Map variable {} not found", var_name))?;

                // Load Map pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let map_ptr = self
                    .builder
                    .build_load(ptr_type, map_ptr_alloca, "map_ptr_load")
                    .map_err(|e| format!("Failed to load map pointer: {}", e))?;

                // Compile key and value
                let key = self.compile_expression(&args[0])?;
                let value = self.compile_expression(&args[1])?;

                // map_insert expects: (ptr map, ptr key, ptr value)
                let insert_fn = self.declare_runtime_fn(
                    "vex_map_insert",
                    &[ptr_type.into(), ptr_type.into(), ptr_type.into()],
                    self.context.bool_type().into(),
                );

                self.builder
                    .build_call(
                        insert_fn,
                        &[map_ptr.into(), key.into(), value.into()],
                        "map_insert",
                    )
                    .map_err(|e| format!("Failed to call vex_map_insert: {}", e))?;

                Ok(Some(self.context.i8_type().const_zero().into()))
            }
            "get" => {
                if args.len() != 1 {
                    return Err("Map.get() requires 1 argument (key)".to_string());
                }

                // Get Map pointer from variable
                let map_ptr_alloca = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Map variable {} not found", var_name))?;

                // Load Map pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let map_ptr = self
                    .builder
                    .build_load(ptr_type, map_ptr_alloca, "map_ptr_load")
                    .map_err(|e| format!("Failed to load map pointer: {}", e))?;

                // Compile key
                let key = self.compile_expression(&args[0])?;

                // map_get expects: (ptr map, ptr key) -> ptr
                let get_fn = self.declare_runtime_fn(
                    "vex_map_get",
                    &[ptr_type.into(), ptr_type.into()],
                    ptr_type.into(),
                );

                let call_site = self
                    .builder
                    .build_call(get_fn, &[map_ptr.into(), key.into()], "map_get")
                    .map_err(|e| format!("Failed to call vex_map_get: {}", e))?;

                let value = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_map_get returned void".to_string())?;

                Ok(Some(value))
            }
            "len" => {
                if !args.is_empty() {
                    return Err("Map.len() takes no arguments".to_string());
                }

                // Get Map pointer from variable
                let map_ptr_alloca = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Map variable {} not found", var_name))?;

                // Load Map pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let map_ptr = self
                    .builder
                    .build_load(ptr_type, map_ptr_alloca, "map_ptr_load")
                    .map_err(|e| format!("Failed to load map pointer: {}", e))?;

                // map_len expects: (ptr map) -> i64
                let len_fn = self.declare_runtime_fn(
                    "vex_map_len",
                    &[ptr_type.into()],
                    self.context.i64_type().into(),
                );

                let call_site = self
                    .builder
                    .build_call(len_fn, &[map_ptr.into()], "map_len")
                    .map_err(|e| format!("Failed to call vex_map_len: {}", e))?;

                let len_val = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_map_len returned void".to_string())?;

                Ok(Some(len_val))
            }
            _ => Ok(None),
        }
    }

    fn compile_set_method(
        &mut self,
        var_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        match method {
            "insert" => {
                if args.len() != 1 {
                    return Err("Set.insert() requires 1 argument (value)".to_string());
                }

                // Get Set pointer from variable
                let set_ptr_alloca = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Set variable {} not found", var_name))?;

                // Load Set pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let set_ptr = self
                    .builder
                    .build_load(ptr_type, set_ptr_alloca, "set_ptr_load")
                    .map_err(|e| format!("Failed to load set pointer: {}", e))?;

                // Compile value and allocate on stack
                let value = self.compile_expression(&args[0])?;
                let value_ptr = self
                    .builder
                    .build_alloca(value.get_type(), "set_value")
                    .map_err(|e| format!("Failed to allocate set value: {}", e))?;
                self.builder
                    .build_store(value_ptr, value)
                    .map_err(|e| format!("Failed to store set value: {}", e))?;

                // set_insert expects: (ptr set, ptr value) -> bool
                let insert_fn = self.declare_runtime_fn(
                    "vex_set_insert",
                    &[ptr_type.into(), ptr_type.into()],
                    self.context.bool_type().into(),
                );

                self.builder
                    .build_call(insert_fn, &[set_ptr.into(), value_ptr.into()], "set_insert")
                    .map_err(|e| format!("Failed to call vex_set_insert: {}", e))?;

                Ok(Some(self.context.i8_type().const_zero().into()))
            }
            "contains" => {
                if args.len() != 1 {
                    return Err("Set.contains() requires 1 argument (value)".to_string());
                }

                // Get Set pointer from variable
                let set_ptr_alloca = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Set variable {} not found", var_name))?;

                // Load Set pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let set_ptr = self
                    .builder
                    .build_load(ptr_type, set_ptr_alloca, "set_ptr_load")
                    .map_err(|e| format!("Failed to load set pointer: {}", e))?;

                // Compile value and allocate on stack
                let value = self.compile_expression(&args[0])?;
                let value_ptr = self
                    .builder
                    .build_alloca(value.get_type(), "set_value")
                    .map_err(|e| format!("Failed to allocate set value: {}", e))?;
                self.builder
                    .build_store(value_ptr, value)
                    .map_err(|e| format!("Failed to store set value: {}", e))?;

                // set_contains expects: (ptr set, ptr value) -> bool
                let contains_fn = self.declare_runtime_fn(
                    "vex_set_contains",
                    &[ptr_type.into(), ptr_type.into()],
                    self.context.bool_type().into(),
                );

                let call_site = self
                    .builder
                    .build_call(
                        contains_fn,
                        &[set_ptr.into(), value_ptr.into()],
                        "set_contains",
                    )
                    .map_err(|e| format!("Failed to call vex_set_contains: {}", e))?;

                let result = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_set_contains returned void".to_string())?;

                Ok(Some(result))
            }
            "remove" => {
                if args.len() != 1 {
                    return Err("Set.remove() requires 1 argument (value)".to_string());
                }

                // Get Set pointer from variable
                let set_ptr_alloca = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Set variable {} not found", var_name))?;

                // Load Set pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let set_ptr = self
                    .builder
                    .build_load(ptr_type, set_ptr_alloca, "set_ptr_load")
                    .map_err(|e| format!("Failed to load set pointer: {}", e))?;

                // Compile value and allocate on stack
                let value = self.compile_expression(&args[0])?;
                let value_ptr = self
                    .builder
                    .build_alloca(value.get_type(), "set_value")
                    .map_err(|e| format!("Failed to allocate set value: {}", e))?;
                self.builder
                    .build_store(value_ptr, value)
                    .map_err(|e| format!("Failed to store set value: {}", e))?;

                // set_remove expects: (ptr set, ptr value) -> bool
                let remove_fn = self.declare_runtime_fn(
                    "vex_set_remove",
                    &[ptr_type.into(), ptr_type.into()],
                    self.context.bool_type().into(),
                );

                let call_site = self
                    .builder
                    .build_call(remove_fn, &[set_ptr.into(), value_ptr.into()], "set_remove")
                    .map_err(|e| format!("Failed to call vex_set_remove: {}", e))?;

                let result = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_set_remove returned void".to_string())?;

                Ok(Some(result))
            }
            "len" => {
                if !args.is_empty() {
                    return Err("Set.len() takes no arguments".to_string());
                }

                // Get Set pointer from variable
                let set_ptr_alloca = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Set variable {} not found", var_name))?;

                // Load Set pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let set_ptr = self
                    .builder
                    .build_load(ptr_type, set_ptr_alloca, "set_ptr_load")
                    .map_err(|e| format!("Failed to load set pointer: {}", e))?;

                // set_len expects: (ptr set) -> i64
                let len_fn = self.declare_runtime_fn(
                    "vex_set_len",
                    &[ptr_type.into()],
                    self.context.i64_type().into(),
                );

                let call_site = self
                    .builder
                    .build_call(len_fn, &[set_ptr.into()], "set_len")
                    .map_err(|e| format!("Failed to call vex_set_len: {}", e))?;

                let len_val = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_set_len returned void".to_string())?;

                Ok(Some(len_val))
            }
            "clear" => {
                if !args.is_empty() {
                    return Err("Set.clear() takes no arguments".to_string());
                }

                // Get Set pointer from variable
                let set_ptr_alloca = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Set variable {} not found", var_name))?;

                // Load Set pointer
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let set_ptr = self
                    .builder
                    .build_load(ptr_type, set_ptr_alloca, "set_ptr_load")
                    .map_err(|e| format!("Failed to load set pointer: {}", e))?;

                // set_clear expects: (ptr set) -> void
                let void_fn = self.module.add_function(
                    "vex_set_clear",
                    self.context.void_type().fn_type(&[ptr_type.into()], false),
                    None,
                );

                self.builder
                    .build_call(void_fn, &[set_ptr.into()], "")
                    .map_err(|e| format!("Failed to call vex_set_clear: {}", e))?;

                Ok(Some(self.context.i8_type().const_zero().into()))
            }
            _ => Ok(None),
        }
    }

    fn compile_range_method(
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

                let result = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| format!("{} returned void", fn_name))?;

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

                let result = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| format!("{} returned void", fn_name))?;

                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    fn compile_slice_method(
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

                let len_val = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_slice_len returned void".to_string())?;

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

                let elem_ptr = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_slice_get returned void".to_string())?;

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

                let result = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_slice_is_empty returned void".to_string())?;

                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    /// Compile array methods (len, get)
    fn compile_array_method(
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

    fn compile_channel_method(
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

                // Load the channel pointer (opaque pointer to VexChannel)
                let channel_opaque_type = self.context.opaque_struct_type("struct.VexChannel");
                let channel_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let channel_ptr = self
                    .builder
                    .build_load(channel_ptr_type, channel_alloca_ptr, "channel_ptr_load")
                    .map_err(|e| format!("Failed to load channel pointer: {}", e))?
                    .into_pointer_value();

                // Compile the value to send
                let value = self.compile_expression(&args[0])?;

                // Allocate heap space for value (channel stores pointer, not copy!)
                // Calculate size in bytes
                let value_type = value.get_type();
                let size_in_bytes = match value_type {
                    inkwell::types::BasicTypeEnum::IntType(it) => it.get_bit_width() / 8,
                    inkwell::types::BasicTypeEnum::FloatType(ft) => {
                        match ft.get_context().f64_type() == ft {
                            true => 8,
                            false => 4,
                        }
                    }
                    _ => return Err(format!("Unsupported channel value type: {:?}", value_type)),
                };

                // Declare malloc: void* malloc(size_t size)
                let malloc_fn = if let Some(func) = self.module.get_function("malloc") {
                    func
                } else {
                    let i64_type = self.context.i64_type();
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let fn_type = ptr_type.fn_type(&[i64_type.into()], false);
                    self.module.add_function(
                        "malloc",
                        fn_type,
                        Some(inkwell::module::Linkage::External),
                    )
                };

                let size_value = self
                    .context
                    .i64_type()
                    .const_int(size_in_bytes as u64, false);
                let heap_ptr = self
                    .builder
                    .build_call(malloc_fn, &[size_value.into()], "heap_alloc")
                    .map_err(|e| format!("Failed to call malloc: {}", e))?
                    .try_as_basic_value()
                    .left()
                    .ok_or("malloc returned void")?
                    .into_pointer_value();

                // Store value to heap
                self.builder
                    .build_store(heap_ptr, value)
                    .map_err(|e| format!("Failed to store value to heap: {}", e))?;

                // Already a void pointer
                let void_ptr = heap_ptr;

                // Call vex_channel_send
                let send_fn = self.get_or_declare_vex_channel_send();
                self.builder
                    .build_call(
                        send_fn,
                        &[channel_ptr.into(), void_ptr.into()],
                        "channel_send",
                    )
                    .map_err(|e| format!("Failed to call vex_channel_send: {}", e))?;

                // Return void (unit type)
                Ok(Some(self.context.i8_type().const_zero().into()))
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

                // Get element type from variable type (Channel<T>)
                // For now, assume i64 as element type (TODO: extract from Channel<T> type)
                let elem_type = self.context.i64_type();

                // Allocate space for the out parameter (void**)
                let data_out_ptr = self
                    .builder
                    .build_alloca(channel_ptr_type, "data_out")
                    .map_err(|e| format!("Failed to allocate data_out: {}", e))?;

                // Call vex_channel_recv(chan, &data_out)
                let recv_fn = self.get_or_declare_vex_channel_recv();
                let status = self
                    .builder
                    .build_call(
                        recv_fn,
                        &[channel_ptr.into(), data_out_ptr.into()],
                        "channel_recv",
                    )
                    .map_err(|e| format!("Failed to call vex_channel_recv: {}", e))?
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_channel_recv returned void".to_string())?
                    .into_int_value();

                // Load the received data pointer from data_out
                let received_ptr = self
                    .builder
                    .build_load(channel_ptr_type, data_out_ptr, "received_ptr_load")
                    .map_err(|e| format!("Failed to load received pointer: {}", e))?
                    .into_pointer_value();

                // Load the actual value from heap
                let value = self
                    .builder
                    .build_load(elem_type, received_ptr, "recv_value")
                    .map_err(|e| format!("Failed to load recv value: {}", e))?;

                // Free the heap-allocated memory (sender malloc'd it)
                let free_fn = if let Some(func) = self.module.get_function("free") {
                    func
                } else {
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let void_type = self.context.void_type();
                    let fn_type = void_type.fn_type(&[ptr_type.into()], false);
                    self.module.add_function(
                        "free",
                        fn_type,
                        Some(inkwell::module::Linkage::External),
                    )
                };

                self.builder
                    .build_call(free_fn, &[received_ptr.into()], "free_recv_value")
                    .map_err(|e| format!("Failed to call free: {}", e))?;

                Ok(Some(value))
            }
            _ => Ok(None),
        }
    }
}
