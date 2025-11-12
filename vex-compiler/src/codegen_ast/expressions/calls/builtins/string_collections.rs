// String, Map, and Set builtin method compilation

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(super) fn compile_string_method(
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

    pub(super) fn compile_map_method(
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

    pub(super) fn compile_set_method(
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

}
