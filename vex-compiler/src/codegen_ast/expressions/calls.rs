// Function and method calls

use super::super::ASTCodeGen;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile function call
    pub(crate) fn compile_call(
        &mut self,
        func_expr: &Expression,
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile arguments first
        let mut arg_vals: Vec<BasicMetadataValueEnum> = Vec::new();
        let mut arg_basic_vals: Vec<BasicValueEnum> = Vec::new();
        for arg in args {
            let val = self.compile_expression(arg)?;

            // If argument is a struct, we need to pass it by pointer (alloca)
            // Check if this is a struct variable
            let is_struct = if let Expression::Ident(name) = arg {
                self.variable_struct_names.contains_key(name)
            } else {
                false
            };

            if is_struct {
                // Argument is a struct stored in a variable
                // Pass the pointer (alloca) instead of loading the value
                if let Expression::Ident(name) = arg {
                    if let Some(struct_ptr) = self.variables.get(name) {
                        arg_vals.push((*struct_ptr).into());
                        arg_basic_vals.push((*struct_ptr).into());
                        continue;
                    }
                }
            }

            arg_vals.push(val.into());
            arg_basic_vals.push(val);
        }

        // Check if this is an enum constructor call: EnumName.Variant(data)
        if let Expression::FieldAccess { object, field } = func_expr {
            if let Expression::Ident(enum_name) = object.as_ref() {
                // Check if this is a registered enum
                if self.enum_ast_defs.contains_key(enum_name) {
                    // This is an enum constructor call
                    let constructor_name = format!("{}_{}", enum_name, field);

                    if let Some(fn_val) = self.functions.get(&constructor_name) {
                        let call_site = self
                            .builder
                            .build_call(*fn_val, &arg_vals, "enum_ctor")
                            .map_err(|e| format!("Failed to build enum constructor call: {}", e))?;

                        return call_site
                            .try_as_basic_value()
                            .left()
                            .ok_or_else(|| "Enum constructor returned void".to_string());
                    } else {
                        return Err(format!("Enum constructor {} not found", constructor_name));
                    }
                }
            }
        }

        // Check if this is a direct function identifier or an expression
        if let Expression::Ident(func_name) = func_expr {
            // Direct function call by name

            // Check if this is a builtin function
            if let Some(builtin_fn) = self.builtins.get(func_name) {
                return builtin_fn(self, &arg_basic_vals);
            }

            // Check if this is a local variable (could be a closure stored in a variable)
            if self.variables.contains_key(func_name) {
                // This is a variable - fall through to complex expression handling
                // It will be loaded and might be a closure with environment
            } else if self.function_params.contains_key(func_name) {
                // This is a function pointer parameter - fall through to complex expression handling
                // (it will be looked up via compile_expression -> Expression::Ident)
            } else {
                // Check if this is a global function that needs instantiation
                let fn_val = if let Some(fn_val) = self.functions.get(func_name) {
                    *fn_val
                } else if let Some(func_def) = self.function_defs.get(func_name).cloned() {
                    // Check if it's a generic function
                    if !func_def.type_params.is_empty() {
                        // Infer type arguments from arguments
                        // For now, simple approach: use argument types
                        let type_args = self.infer_type_args_from_call(&func_def, args)?;

                        // Instantiate generic function
                        self.instantiate_generic_function(&func_def, &type_args)?
                    } else {
                        return Err(format!("Function {} not declared", func_name));
                    }
                } else {
                    return Err(format!("Function {} not found", func_name));
                };

                // Build call
                let call_site = self
                    .builder
                    .build_call(fn_val, &arg_vals, "call")
                    .map_err(|e| format!("Failed to build call: {}", e))?;

                return call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Function call returned void".to_string());
            }
        }

        // Complex function expression or function parameter
        // Complex function expression - compile it to get function pointer
        let func_val = self.compile_expression(func_expr)?;

        // It should be a pointer value (function pointer)
        if let BasicValueEnum::PointerValue(fn_ptr) = func_val {
            // Check if this is a closure with captured environment
            // First check if the variable name is a tracked closure
            let (has_environment, env_ptr_opt) = if let Expression::Ident(name) = func_expr {
                if let Some((_, env_ptr)) = self.closure_variables.get(name) {
                    eprintln!("🎯 Found closure variable '{}' with environment", name);
                    (true, Some(*env_ptr))
                } else if let Some(env_ptr) = self.closure_envs.get(&fn_ptr) {
                    (true, Some(*env_ptr))
                } else {
                    (false, None)
                }
            } else {
                // Try direct lookup
                if let Some(env_ptr) = self.closure_envs.get(&fn_ptr) {
                    (true, Some(*env_ptr))
                } else {
                    (false, None)
                }
            };

            // Get function type - we need to infer it from the closure or look it up
            let fn_type = if let Expression::Ident(name) = func_expr {
                // Try to get type from function_param_types (for function parameters)
                if let Some(param_type) = self.function_param_types.get(name) {
                    if let Type::Function {
                        params,
                        return_type,
                    } = param_type
                    {
                        // Extract LLVM function type from AST type
                        self.ast_function_type_to_llvm(params, return_type)?
                    } else {
                        return Err(format!("Parameter {} is not a function type", name));
                    }
                } else {
                    // This might be a closure stored in a variable
                    // For now, assume it takes the same args as provided and returns i32
                    // TODO: Store closure types when creating them
                    eprintln!("⚠️  Inferring closure type from call arguments");

                    // Build function type from arguments
                    let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![];
                    if has_environment {
                        // Add environment pointer as first parameter
                        param_types.push(
                            self.context
                                .ptr_type(inkwell::AddressSpace::default())
                                .into(),
                        );
                    }
                    for arg_val in &arg_basic_vals {
                        param_types.push(arg_val.get_type().into());
                    }

                    // Assume i32 return type for now (should be stored with closure)
                    let ret_type = self.context.i32_type();
                    ret_type.fn_type(&param_types, false)
                }
            } else {
                return Err("Complex function expressions not yet supported".to_string());
            };

            // If this closure has an environment, prepend it to arguments
            let final_args = if has_environment {
                let env_ptr = env_ptr_opt.ok_or("Closure environment pointer missing")?;
                eprintln!(
                    "🎯 Calling closure with captured environment: {:?}",
                    env_ptr
                );

                // Prepend environment pointer to arguments
                let mut closure_args = vec![env_ptr.into()];
                closure_args.extend(arg_vals);
                closure_args
            } else {
                arg_vals
            };

            // Build indirect call using the function pointer
            let call_site = self
                .builder
                .build_indirect_call(fn_type, fn_ptr, &final_args, "indirect_call")
                .map_err(|e| format!("Failed to build indirect call: {}", e))?;

            return call_site
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "Function call returned void".to_string());
        } else {
            return Err("Function expression did not evaluate to a function pointer".to_string());
        }
    }

    /// Compile method call: obj.method(args)
    /// Methods are compiled as functions with the receiver as the first argument
    pub(crate) fn compile_method_call(
        &mut self,
        receiver: &Expression,
        method: &str,
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Phase 0.4c: Check for builtin type instance methods (vec.push, vec.len, etc.)
        if let Some(result) = self.try_compile_builtin_method(receiver, method, args)? {
            return Ok(result);
        }

        // Check if this is a module-level function call (io.print, log.info, etc.)
        if let Expression::Ident(module_name) = receiver {
            // Check if this is a known module namespace
            if let Some(module_funcs) = self.module_namespaces.get(module_name) {
                // This is a module namespace, check if the method exists
                if module_funcs.contains(&method.to_string()) {
                    // Found! Call the function directly
                    let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        arg_vals.push(val.into());
                    }

                    let fn_val = *self.functions.get(method).ok_or_else(|| {
                        format!("Module function {} not found in LLVM module", method)
                    })?;

                    let call_site = self
                        .builder
                        .build_call(fn_val, &arg_vals, "modulecall")
                        .map_err(|e| format!("Failed to build module call: {}", e))?;

                    return call_site
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| "Module function returned void".to_string());
                }
            }

            // Legacy: Try old-style module_function naming
            let module_func_name = format!("{}_{}", module_name, method);
            if self.functions.contains_key(&module_func_name) {
                let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                for arg in args {
                    let val = self.compile_expression(arg)?;
                    arg_vals.push(val.into());
                }

                let fn_val = *self.functions.get(&module_func_name).unwrap();
                let call_site = self
                    .builder
                    .build_call(fn_val, &arg_vals, "modulecall")
                    .map_err(|e| format!("Failed to build module call: {}", e))?;

                return call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Module function returned void".to_string());
            }
        }

        // Get struct type from receiver (for actual method calls)
        let struct_name = if let Expression::Ident(var_name) = receiver {
            self.variable_struct_names
                .get(var_name)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "Variable {} is not a struct or module, cannot call methods",
                        var_name
                    )
                })?
        } else {
            return Err("Method calls only supported on variables for now".to_string());
        };

        // Construct method function name: StructName_method
        let method_func_name = format!("{}_{}", struct_name, method);

        // Check if method function exists (either as a struct method or trait method)
        let final_method_name = if self.functions.contains_key(&method_func_name) {
            // Found as struct method
            method_func_name
        } else {
            // Try to find trait method: TypeName_TraitName_methodName
            // Search all trait impls for this type
            let mut found_trait_method = None;
            for ((trait_name, type_name), _) in &self.trait_impls {
                if type_name == &struct_name {
                    let trait_method_name = format!("{}_{}_{}", type_name, trait_name, method);
                    if self.functions.contains_key(&trait_method_name) {
                        found_trait_method = Some(trait_method_name);
                        break;
                    }
                }
            }

            if let Some(trait_method) = found_trait_method {
                trait_method
            } else {
                // Try to find default trait method
                // Check all traits implemented by this type
                // First, collect trait information to avoid borrow checker issues
                let mut default_method_info: Option<(String, String, vex_ast::TraitMethod)> = None;

                for ((trait_name, type_name), _) in &self.trait_impls {
                    if type_name == &struct_name {
                        // Check if the trait has a default method with this name
                        if let Some(trait_def) = self.trait_defs.get(trait_name) {
                            for trait_method in &trait_def.methods {
                                if trait_method.name == method && trait_method.body.is_some() {
                                    // Found default method! Save info for compilation
                                    default_method_info = Some((
                                        trait_name.clone(),
                                        type_name.clone(),
                                        trait_method.clone(),
                                    ));
                                    break;
                                }
                            }
                        }
                        if default_method_info.is_some() {
                            break;
                        }
                    }
                }

                // Now compile if found
                if let Some((trait_name, type_name, trait_method)) = default_method_info {
                    let default_method_name = format!("{}_{}_{}", type_name, trait_name, method);

                    // Check if already compiled
                    if !self.functions.contains_key(&default_method_name) {
                        // Save current function context (variables, types, current_function, builder position)
                        let saved_variables = self.variables.clone();
                        let saved_variable_types = self.variable_types.clone();
                        let saved_variable_struct_names = self.variable_struct_names.clone();
                        let saved_current_function = self.current_function;

                        // Replace Self with concrete type in receiver and params
                        let concrete_type = vex_ast::Type::Named(type_name.clone());

                        let receiver = if let Some(ref r) = trait_method.receiver {
                            Some(vex_ast::Receiver {
                                is_mutable: r.is_mutable,
                                ty: Self::replace_self_type(&r.ty, &type_name),
                            })
                        } else {
                            None
                        };

                        let params: Vec<_> = trait_method
                            .params
                            .iter()
                            .map(|p| vex_ast::Param {
                                name: p.name.clone(),
                                ty: Self::replace_self_type(&p.ty, &type_name),
                            })
                            .collect();

                        let return_type = trait_method
                            .return_type
                            .as_ref()
                            .map(|t| Self::replace_self_type(t, &type_name));

                        // Convert TraitMethod to Function for compilation
                        let func = vex_ast::Function {
                            attributes: vec![],
                            is_async: false,
                            is_gpu: false,
                            name: method.to_string(),
                            type_params: vec![],
                            receiver,
                            params,
                            return_type,
                            body: trait_method.body.clone().unwrap(), // Safe because we checked is_some()
                        };

                        // Declare and compile the default method for this specific type
                        self.declare_trait_impl_method(&trait_name, &concrete_type, &func)?;
                        self.compile_trait_impl_method(&trait_name, &concrete_type, &func)?;

                        // Restore function context
                        self.variables = saved_variables;
                        self.variable_types = saved_variable_types;
                        self.variable_struct_names = saved_variable_struct_names;
                        self.current_function = saved_current_function;

                        // Restore builder position if we have a current function
                        if let Some(func) = self.current_function {
                            if let Some(bb) = func.get_last_basic_block() {
                                self.builder.position_at_end(bb);
                            }
                        }
                    }

                    default_method_name
                } else {
                    return Err(format!(
                        "Method '{}' not found for struct '{}' (neither as struct method, trait method, nor default trait method)",
                        method, struct_name
                    ));
                }
            }
        };

        // Compile receiver (this will be the first argument)
        let receiver_val = self.compile_expression(receiver)?;

        // Compile other arguments
        let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![receiver_val.into()];
        for arg in args {
            let val = self.compile_expression(arg)?;
            arg_vals.push(val.into());
        }

        // Get method function
        let fn_val = *self
            .functions
            .get(&final_method_name)
            .ok_or_else(|| format!("Method function {} not found", final_method_name))?;

        // Build call
        let call_site = self
            .builder
            .build_call(fn_val, &arg_vals, "methodcall")
            .map_err(|e| format!("Failed to build method call: {}", e))?;

        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Method call returned void".to_string())
    }

    /// Replace Self type with concrete type name (for default trait methods)
    fn replace_self_type(ty: &Type, concrete_type: &str) -> Type {
        match ty {
            Type::Named(name) if name == "Self" => Type::Named(concrete_type.to_string()),
            Type::Reference(inner, is_mut) => Type::Reference(
                Box::new(Self::replace_self_type(inner, concrete_type)),
                *is_mut,
            ),
            Type::Generic { name, type_args } => {
                let new_name = if name == "Self" {
                    concrete_type.to_string()
                } else {
                    name.clone()
                };
                Type::Generic {
                    name: new_name,
                    type_args: type_args
                        .iter()
                        .map(|t| Self::replace_self_type(t, concrete_type))
                        .collect(),
                }
            }
            Type::Array(inner, size) => Type::Array(
                Box::new(Self::replace_self_type(inner, concrete_type)),
                *size,
            ),
            Type::Slice(inner, is_mut) => Type::Slice(
                Box::new(Self::replace_self_type(inner, concrete_type)),
                *is_mut,
            ),
            Type::Union(types) => Type::Union(
                types
                    .iter()
                    .map(|t| Self::replace_self_type(t, concrete_type))
                    .collect(),
            ),
            Type::Intersection(types) => Type::Intersection(
                types
                    .iter()
                    .map(|t| Self::replace_self_type(t, concrete_type))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }

    /// Try to compile builtin type instance methods (Phase 0.4c)
    /// Returns Some(value) if handled, None if not a builtin method
    fn try_compile_builtin_method(
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

        // Phase 0.4b: Check if this is a builtin type (Vec, Box)
        let struct_name = self.variable_struct_names.get(&var_name).cloned();

        if let Some(type_name) = struct_name {
            // Handle builtin type methods
            match type_name.as_str() {
                "Vec" => return self.compile_vec_method(&var_name, method, args),
                "Box" => return self.compile_box_method(&var_name, method, args),
                _ => return Ok(None), // Not a builtin type
            }
        }

        // Fallback: Try LLVM type detection (backward compatibility)
        let receiver_type = self
            .variable_types
            .get(&var_name)
            .ok_or_else(|| format!("Variable {} not found", var_name))?
            .clone();

        if let inkwell::types::BasicTypeEnum::StructType(st) = receiver_type {
            // Check if this looks like a Vec: { ptr, i64, i64, i64 }
            if st.count_fields() == 4 {
                match method {
                    "push" => {
                        // vec.push(value) -> vex_vec_push(&vec, &value)
                        if args.len() != 1 {
                            return Err("Vec.push() requires exactly 1 argument".to_string());
                        }

                        // Get vec pointer (clone to avoid borrow issues)
                        let vec_ptr = *self
                            .variables
                            .get(&var_name)
                            .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

                        // Compile argument value
                        let value = self.compile_expression(&args[0])?;

                        // Allocate value on stack to get pointer
                        let value_ptr =
                            self.builder
                                .build_alloca(value.get_type(), "push_value")
                                .map_err(|e| format!("Failed to allocate push value: {}", e))?;
                        self.builder
                            .build_store(value_ptr, value)
                            .map_err(|e| format!("Failed to store push value: {}", e))?;

                        // Get vex_vec_push function
                        let push_fn = self.get_vex_vec_push();

                        // Call vex_vec_push(vec_ptr, value_ptr)
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

                        // push returns void, return unit value (i8 zero)
                        let unit = self.context.i8_type().const_zero();
                        return Ok(Some(unit.into()));
                    }
                    "len" => {
                        // vec.len() -> vex_vec_len(&vec)
                        if !args.is_empty() {
                            return Err("Vec.len() takes no arguments".to_string());
                        }

                        // Get vec pointer (clone to avoid borrow issues)
                        let vec_ptr = *self
                            .variables
                            .get(&var_name)
                            .ok_or_else(|| format!("Vec variable {} not found", var_name))?;

                        // Get vex_vec_len function
                        let len_fn = self.get_vex_vec_len();

                        // Call vex_vec_len(vec_ptr)
                        let call_site = self
                            .builder
                            .build_call(len_fn, &[vec_ptr.into()], "vec_len")
                            .map_err(|e| format!("Failed to call vex_vec_len: {}", e))?;

                        let len_value = call_site
                            .try_as_basic_value()
                            .left()
                            .ok_or_else(|| "vex_vec_len returned void".to_string())?;

                        // Convert i64 to i32 for now (most common return type)
                        let len_i32 = self
                            .builder
                            .build_int_truncate(
                                len_value.into_int_value(),
                                self.context.i32_type(),
                                "len_i32",
                            )
                            .map_err(|e| format!("Failed to truncate len: {}", e))?;

                        return Ok(Some(len_i32.into()));
                    }
                    _ => return Ok(None), // Not a Vec method
                }
            }

            // Check if this looks like a Box: { ptr, i64 }
            if st.count_fields() == 2 {
                match method {
                    "get" => {
                        // box.get() -> vex_box_get(&box)
                        if !args.is_empty() {
                            return Err("Box.get() takes no arguments".to_string());
                        }

                        // Get box pointer (clone to avoid borrow issues)
                        let box_ptr = *self
                            .variables
                            .get(&var_name)
                            .ok_or_else(|| format!("Box variable {} not found", var_name))?;

                        // Get vex_box_get function
                        let get_fn = self.get_vex_box_get();

                        // Call vex_box_get(box_ptr)
                        let call_site = self
                            .builder
                            .build_call(get_fn, &[box_ptr.into()], "box_get")
                            .map_err(|e| format!("Failed to call vex_box_get: {}", e))?;

                        let ptr_value = call_site
                            .try_as_basic_value()
                            .left()
                            .ok_or_else(|| "vex_box_get returned void".to_string())?;

                        return Ok(Some(ptr_value));
                    }
                    _ => return Ok(None), // Not a Box method
                }
            }
        }

        Ok(None) // Not a builtin type or method
    }

    /// Compile Vec instance methods (Phase 0.4b)
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

                let len_value = call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "vex_vec_len returned void".to_string())?;

                let len_i32 = self
                    .builder
                    .build_int_truncate(
                        len_value.into_int_value(),
                        self.context.i32_type(),
                        "len_i32",
                    )
                    .map_err(|e| format!("Failed to truncate len: {}", e))?;

                Ok(Some(len_i32.into()))
            }
            _ => Ok(None),
        }
    }

    /// Compile Box instance methods (Phase 0.4b)
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
