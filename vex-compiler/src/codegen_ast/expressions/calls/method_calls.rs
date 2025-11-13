// Method call compilation (instance methods, trait methods, builtin types)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_method_call(
        &mut self,
        receiver: &Expression,
        method: &str,
        type_args: &[Type], // ⭐ NEW: Generic type arguments for static methods
        args: &[Expression],
        is_mutable_call: bool,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // ⭐ NEW: Phase 0: Check for builtin contract methods (i32.to_string(), bool.clone(), etc.)
        // This must come BEFORE builtin type methods to support contract-based dispatch
        if let Some(result) = self.try_compile_builtin_contract_method(receiver, method, args)? {
            return Ok(result);
        }

        // Phase 0.4c: Check for builtin type instance methods (vec.push, vec.len, etc.)
        // This MUST come first before struct name checking
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

        // Check if this is a static method call: Type.method() where Type is PascalCase
        // Static methods don't have a receiver instance - they're called on the type itself
        if let Expression::Ident(potential_type_name) = receiver {
            // Check if this looks like a type name (PascalCase - starts with uppercase)
            let is_type_name = potential_type_name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);

            // Check if this is NOT a variable (static methods called on types, not instances)
            let is_not_variable = !self.variables.contains_key(potential_type_name);

            if is_type_name && is_not_variable {
                // This is a static method call: Type.method(args) or Vec<i32>.new()

                // ⭐ NEW: Handle generic static methods: Vec<i32>.new()
                // Mangle function name with type arguments
                let base_method_name = if !type_args.is_empty() {
                    // Generic static method: Vec<i32>.new() → vec_i32_new
                    let mut mangled = potential_type_name.to_lowercase();
                    for ty in type_args {
                        mangled.push('_');
                        mangled.push_str(&self.type_to_string(ty));
                    }
                    mangled.push('_');
                    mangled.push_str(method);
                    mangled
                } else {
                    // Non-generic: Vec.new() → vec_new
                    format!("{}_{}", potential_type_name.to_lowercase(), method)
                };

                // Try builtin registry first (check both PascalCase and lowercase)
                let pascal_builtin_name = format!("{}.{}", potential_type_name, method);
                if let Some(builtin_fn) = self.builtins.get(&pascal_builtin_name).or_else(|| self.builtins.get(&base_method_name)) {
                    let mut arg_vals: Vec<BasicValueEnum> = vec![];
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        arg_vals.push(val);
                    }

                    return builtin_fn(self, &arg_vals);
                }

                // Check if the function exists in LLVM module
                if !self.functions.contains_key(&base_method_name) {
                    // Try PascalCase version: TypeName_method or Vec_i32_new
                    let pascal_method_name = if !type_args.is_empty() {
                        let mut mangled = potential_type_name.to_string();
                        for ty in type_args {
                            mangled.push('_');
                            mangled.push_str(&self.type_to_string(ty));
                        }
                        mangled.push('_');
                        mangled.push_str(method);
                        mangled
                    } else {
                        format!("{}_{}", potential_type_name, method)
                    };

                    if !self.functions.contains_key(&pascal_method_name) {
                        return Err(format!(
                            "Static method {}.{}() not found. Expected function: {} or {}",
                            potential_type_name, method, base_method_name, pascal_method_name
                        ));
                    }

                    // Use PascalCase version
                    let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        arg_vals.push(val.into());
                    }

                    let fn_val = *self.functions.get(&pascal_method_name).unwrap();
                    let call_site = self
                        .builder
                        .build_call(fn_val, &arg_vals, "static_method_call")
                        .map_err(|e| format!("Failed to build static method call: {}", e))?;

                    return call_site
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| "Static method returned void".to_string());
                }

                // Call lowercase version from LLVM module
                let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                for arg in args {
                    let val = self.compile_expression(arg)?;
                    arg_vals.push(val.into());
                }

                let fn_val = *self.functions.get(&base_method_name).unwrap();
                let call_site = self
                    .builder
                    .build_call(fn_val, &arg_vals, "static_method_call")
                    .map_err(|e| format!("Failed to build static method call: {}", e))?;

                return call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Static method returned void".to_string());
            }
        }

        // Get struct type from receiver (for instance method calls)
        let (struct_name, receiver_val) = if let Expression::Ident(var_name) = receiver {
            eprintln!("🔍 Receiver is identifier: {}", var_name);
            let struct_name = self
                .variable_struct_names
                .get(var_name)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "Variable {} is not a struct or module, cannot call methods",
                        var_name
                    )
                })?;

            // Get variable pointer
            let var_ptr = self
                .variables
                .get(var_name)
                .ok_or_else(|| format!("Variable {} not found", var_name))?;

            eprintln!("🔍 Receiver var_ptr: {:?}", var_ptr);
            (struct_name, *var_ptr)
        } else if let Expression::FieldAccess { object, field } = receiver {
            // Handle field access: self.counter.next()
            eprintln!(
                "🔧 Method call on field access: {}.{}.{}",
                if let Expression::Ident(n) = object.as_ref() {
                    n
                } else {
                    "?"
                },
                field,
                method
            );

            // Get the object variable
            if let Expression::Ident(var_name) = object.as_ref() {
                let object_struct_name = self
                    .variable_struct_names
                    .get(var_name)
                    .ok_or_else(|| format!("Variable {} not found or not a struct", var_name))?;

                // Get struct definition
                let struct_def = self
                    .struct_defs
                    .get(object_struct_name)
                    .ok_or_else(|| format!("Struct {} not found", object_struct_name))?
                    .clone();

                // Find field index and type
                let field_index = struct_def
                    .fields
                    .iter()
                    .position(|(name, _)| name == field)
                    .ok_or_else(|| {
                        format!("Field {} not found in struct {}", field, object_struct_name)
                    })?;

                let field_type = &struct_def.fields[field_index].1;

                // Get field struct name
                let field_struct_name = if let Type::Named(name) = field_type {
                    name.clone()
                } else {
                    return Err(format!("Field {} is not a struct type", field));
                };

                // Get object pointer
                let object_ptr = self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Variable {} not found", var_name))?;

                // Get field pointer
                let field_ptr = self
                    .builder
                    .build_struct_gep(
                        self.ast_type_to_llvm(field_type),
                        *object_ptr,
                        field_index as u32,
                        &format!("{}_field_{}", var_name, field),
                    )
                    .map_err(|e| format!("Failed to GEP field: {}", e))?;

                eprintln!("  ✅ Field GEP successful, struct: {}", field_struct_name);
                (field_struct_name, field_ptr)
            } else {
                return Err(
                    "Method calls on field access only supported when object is a variable"
                        .to_string(),
                );
            }
        } else {
            return Err(
                "Method calls only supported on variables and field access for now".to_string(),
            );
        };

        // Construct method function name: StructName_method
        // ⭐ CRITICAL: Encode operator names for LLVM compatibility
        let method_encoded = Self::encode_operator_name(method);
        
        // ⭐ NEW: For operators that can be both unary and binary, add parameter count suffix
        let param_count = args.len();
        let base_method_name = format!("{}_{}", struct_name, method_encoded);
        
        let method_func_name = if method.starts_with("op") && 
                                  (method == "op-" || method == "op+" || method == "op*") {
            format!("{}_{}", base_method_name, param_count)
        } else {
            base_method_name
        };

        // Check if method function exists (either as a struct method or trait method)
        let final_method_name = if self.functions.contains_key(&method_func_name) {
            // Found as struct method
            method_func_name
        } else {
            // Try to find trait method
            // For generic trait impls, we need to try all type arg variations
            let mut found_trait_method = None;
            
            // First, try to match against generic impl clauses in struct_ast_defs
            if let Some(struct_def) = self.struct_ast_defs.get(&struct_name) {
                // Try to match based on all implemented traits
                // For operator methods, we need to match BOTH the method name AND argument types
                // Don't break on first match - check ALL traits for the right type match
                for trait_impl in &struct_def.impl_traits {
                    if !trait_impl.type_args.is_empty() {
                        // Generic trait impl - try to match with actual argument types
                        if !args.is_empty() {
                            // Try to infer the type of first argument
                            if let Ok(arg_type) = self.infer_expression_type(&args[0]) {
                                eprintln!("🔍 Method lookup: struct={}, trait={}, method={}, arg_type={:?}, trait_type_arg={:?}", 
                                    struct_name, trait_impl.name, method, arg_type, trait_impl.type_args[0]);
                                // Check if this arg type matches trait's type arg
                                if arg_type == trait_impl.type_args[0] {
                                    let type_str = match &trait_impl.type_args[0] {
                                        Type::Named(n) => n.clone(),
                                        Type::I32 => "i32".to_string(),
                                        Type::I64 => "i64".to_string(),
                                        Type::F32 => "f32".to_string(),
                                        Type::F64 => "f64".to_string(),
                                        Type::Bool => "bool".to_string(),
                                        Type::String => "String".to_string(),
                                        _ => continue,
                                    };
                                    
                                    // Generate mangled name with type args
                                    let param_count = args.len();
                                    let trait_method_name = format!("{}_{}_{}_{}_{}", 
                                        struct_name, trait_impl.name, type_str, method_encoded, param_count);
                                    
                                    eprintln!("   🎯 Generated mangled name: {}", trait_method_name);
                                    eprintln!("   🔍 Function exists: {}", self.functions.contains_key(&trait_method_name));
                                    
                                    // Check if this function exists - if so, we found it!
                                    if self.functions.contains_key(&trait_method_name) {
                                        eprintln!("   ✅ MATCH! Using method: {}", trait_method_name);
                                        found_trait_method = Some(trait_method_name);
                                        break; // Found exact match, stop searching
                                    }
                                }
                            }
                        }
                    } else {
                        // Non-generic trait impl - use old format
                        let trait_method_name = format!("{}_{}_{}", struct_name, trait_impl.name, method_encoded);
                        if self.functions.contains_key(&trait_method_name) {
                            found_trait_method = Some(trait_method_name);
                            break;
                        }
                    }
                }
            }
            
            // Fallback: Try old format for trait_impls registry
            if found_trait_method.is_none() {
                for ((trait_name, type_name), _) in &self.trait_impls {
                    if type_name == &struct_name {
                        let trait_method_name = format!("{}_{}_{}", type_name, trait_name, method);
                        if self.functions.contains_key(&trait_method_name) {
                            found_trait_method = Some(trait_method_name);
                            break;
                        }
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
                                name: r.name.clone(),
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
                                default_value: p.default_value.clone(),
                            })
                            .collect();

                        let return_type = trait_method
                            .return_type
                            .as_ref()
                            .map(|t| Self::replace_self_type(t, &type_name));

                        let receiver = trait_method.receiver.as_ref().map(|r| Receiver {
                            name: r.name.clone(),
                            is_mutable: r.is_mutable,
                            ty: Self::replace_self_type(&r.ty, &type_name),
                        });

                        let params: Vec<Param> = trait_method
                            .params
                            .iter()
                            .map(|p| Param {
                                name: p.name.clone(),
                                ty: Self::replace_self_type(&p.ty, &type_name),
                                default_value: p.default_value.clone(),
                            })
                            .collect();

                        let return_type = trait_method
                            .return_type
                            .as_ref()
                            .map(|t| Self::replace_self_type(t, &type_name));

                        // Convert TraitMethod to Function for compilation
                        let func = vex_ast::Function {
                            is_async: false,
                            is_gpu: false,
                            is_mutable: trait_method.is_mutable, // ⭐ NEW: Copy mutability from trait
                            is_operator: trait_method.is_operator, // ⭐ NEW: Copy operator flag from trait
                            name: method.to_string(),
                            type_params: vec![],
                            where_clause: vec![],
                            receiver,
                            params,
                            return_type,
                            body: trait_method.body.clone().unwrap(), // Safe because we checked is_some()
                            is_variadic: false,
                            variadic_type: None,
                        }; // Declare and compile the default method for this specific type
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
                    // Method not found - check if this is a global function call
                    // This handles cases where parser incorrectly converted function calls to method calls
                    // in method bodies (e.g., log2(msg) parsed as self.log2(msg))
                    if self.functions.contains_key(method)
                        || self.function_defs.contains_key(method)
                    {
                        // This is a global function, not a method!
                        // Convert method call to regular function call
                        eprintln!(
                            "⚠️  Method '{}' not found on struct '{}', trying as global function",
                            method, struct_name
                        );

                        // Compile as regular function call (without receiver)
                        return self.compile_call(
                            &Expression::Ident(method.to_string()),
                            &[],
                            args,
                        );
                    }
                    return Err(format!(
                        "Method '{}' not found for struct '{}' (neither as struct method, trait method, nor default trait method)",
                        method, struct_name
                    ));
                }
            }
        };

        // ⭐ Contract-based mutability validation
        // Check if the method is declared as mutable and if it's an external method
        let (method_is_mutable, is_external_method) = self
            .function_defs
            .get(&final_method_name)
            .map(|func| {
                let is_external = func
                    .receiver
                    .as_ref()
                    .map_or(false, |r| matches!(r.ty, Type::Reference(_, _)));
                (func.is_mutable, is_external)
            })
            .unwrap_or((false, false));

        // CONTRACT RULES:
        // 1. External methods (Golang-style with &Type! receiver): NO '!' at call site
        // 2. Inline methods: '!' is OPTIONAL (compiler validates mutability at compile time)

        if is_external_method {
            // External method: '!' suffix is FORBIDDEN
            if is_mutable_call {
                return Err(format!(
                    "External method '{}' cannot use '!' suffix at call site (Golang-style methods don't use '!')",
                    method
                ));
            }
            // For external methods, ignore mutability check (receiver handles it)
        } else {
            // Inline method: '!' suffix is OPTIONAL but recommended for clarity
            // We validate mutability at compile time regardless of '!' presence
            if method_is_mutable && !is_mutable_call {
                eprintln!(
                    "ℹ️  Inline mutable method '{}' called without '!' suffix (allowed but not recommended)",
                    method
                );
                // Don't error, just warn
            }

            if !method_is_mutable && is_mutable_call {
                return Err(format!(
                    "Method '{}' is immutable, cannot use '!' suffix at call site",
                    method
                ));
            }
        }

        // Receiver value is already retrieved above (receiver_val)
        // No need to compile_expression again

        eprintln!(
            "🔧 Method call: {}.{}(), receiver_val is pointer, receiver_val={:?}",
            if let Expression::Ident(name) = receiver {
                name.as_str()
            } else if let Expression::FieldAccess { field, .. } = receiver {
                field.as_str()
            } else {
                "<expr>"
            },
            method,
            receiver_val
        );

        // Compile other arguments
        let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![receiver_val.into()];
        eprintln!("📝 Receiver arg added to arg_vals, receiver_val={:?}", receiver_val);
        
        for (arg_idx, arg) in args.iter().enumerate() {
            eprintln!("📝 Compiling arg {}: {:?}", arg_idx, arg);
            let val = self.compile_expression(arg)?;
            eprintln!("📝 Arg {} compiled: {:?}", arg_idx, val);

            // ⚠️ NEW: Struct arguments are now passed BY VALUE
            // If we have a pointer (from variable), we need to LOAD the value
            // If we already have a struct value (from function return), use it directly
            if let Some(func_def) = self.function_defs.get(&final_method_name) {
                // Account for receiver as first param
                if arg_idx < func_def.params.len() {
                    let param_ty = &func_def.params[arg_idx].ty;
                    let is_struct_param = match param_ty {
                        Type::Named(type_name) => self.struct_defs.contains_key(type_name),
                        _ => false,
                    };

                    if is_struct_param {
                        eprintln!(
                            "🔧 Method arg {}: is_struct=true, val.is_pointer={}, val.is_struct={}",
                            arg_idx,
                            val.is_pointer_value(),
                            val.is_struct_value()
                        );

                        if val.is_pointer_value() {
                            // We have a POINTER but need a VALUE - load it
                            eprintln!("   ⚠️ Loading struct value from pointer for method arg");
                            let ptr_val = val.into_pointer_value();
                            let struct_llvm_type = self.ast_type_to_llvm(param_ty);
                            let loaded_val = self
                                .builder
                                .build_load(struct_llvm_type, ptr_val, "arg_load")
                                .map_err(|e| format!("Failed to load struct arg: {}", e))?;
                            arg_vals.push(loaded_val.into());
                            continue;
                        }
                        // else: already a struct value, fall through
                    }
                }
            }

            arg_vals.push(val.into());
        }

        // Get method function
        let fn_val = *self
            .functions
            .get(&final_method_name)
            .ok_or_else(|| format!("Method function {} not found", final_method_name))?;

        eprintln!("📞 Calling method: {} (fn_val: {:?})", final_method_name, fn_val.get_name());
        eprintln!("📞 Arguments count: {}, arg_vals: {:?}", arg_vals.len(), arg_vals);

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
}
