// Method call compilation orchestrator

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

        // Check if this is a module-level function call (io.print, log.info, etc.)
        if let Expression::Ident(module_name) = receiver {
            eprintln!(
                "🔍 module_namespaces keys: {:?}",
                self.module_namespaces.keys().collect::<Vec<_>>()
            );

            // Check if this is a known module namespace OR a namespace import
            // If it's a namespace import (import * as math), we treat it as a module call
            // even if it's not explicitly in module_namespaces (which might happen with aliases)
            let is_module_namespace = self.module_namespaces.contains_key(module_name);
            let is_namespace_import = self.namespace_imports.contains_key(module_name);

            if is_module_namespace || is_namespace_import {
                eprintln!("🔍🔍🔍 NAMESPACE METHOD CALL: {}.{}()", module_name, method);
                
                // Check if method exists in module_namespaces (if available)
                // If only in namespace_imports, we assume the method exists and let the function lookup fail if not
                let method_exists =
                    if let Some(module_funcs) = self.module_namespaces.get(module_name) {
                        module_funcs.contains(&method.to_string())
                    } else {
                        true
                    };

                if method_exists {
                    eprintln!(
                        "🔍 Module call: {}.{} -> calling global function {}",
                        module_name, method, method
                    );

                    // Compile arguments first to get their types
                    let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                    let mut arg_basic_vals: Vec<BasicValueEnum> = vec![];
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        arg_vals.push(val.into());
                        arg_basic_vals.push(val);
                    }

                    // ⭐ NEW: Perform overload resolution based on argument types
                    let fn_val = if !arg_basic_vals.is_empty() {
                        // Build type suffix from arguments for overload resolution
                        let mut type_suffix = String::new();
                        for (idx, arg_val) in arg_basic_vals.iter().enumerate() {
                            let arg_type_enum = arg_val.get_type();
                            
                            // Try to infer actual argument type from source expression
                            let ast_type = if idx < args.len() {
                                if let Expression::Ident(var_name) = &args[idx] {
                                    // First check variable_ast_types (has both let vars AND function params)
                                    if let Some(ast_ty) = self.variable_ast_types.get(var_name) {
                                        eprintln!("🔍 [NAMESPACE ARG {}] var {} has AST type: {:?}", idx, var_name, ast_ty);
                                        ast_ty.clone()
                                    } else if let Some(tracked_ty) = self.variable_types.get(var_name) {
                                        // Fallback: Check if variable has tracked LLVM type
                                        eprintln!("🔍 [NAMESPACE ARG {}] var {} has tracked LLVM type: {:?}", idx, var_name, tracked_ty);
                                        match tracked_ty {
                                            inkwell::types::BasicTypeEnum::IntType(it) => {
                                                match it.get_bit_width() {
                                                    8 => Type::I8,
                                                    16 => Type::I16,
                                                    32 => Type::I32,
                                                    64 => Type::I64,
                                                    128 => Type::I128,
                                                    1 => Type::Bool,
                                                    _ => Type::I32,
                                                }
                                            }
                                            inkwell::types::BasicTypeEnum::FloatType(ft) => {
                                                if *ft == self.context.f32_type() {
                                                    Type::F32
                                                } else {
                                                    Type::F64
                                                }
                                            }
                                            _ => {
                                                // Fallback to arg_type_enum
                                                match arg_type_enum {
                                                    inkwell::types::BasicTypeEnum::IntType(it) => {
                                                        match it.get_bit_width() {
                                                            8 => Type::I8,
                                                            16 => Type::I16,
                                                            32 => Type::I32,
                                                            64 => Type::I64,
                                                            128 => Type::I128,
                                                            1 => Type::Bool,
                                                            _ => Type::I32,
                                                        }
                                                    }
                                                    inkwell::types::BasicTypeEnum::FloatType(ft) => {
                                                        if ft == self.context.f32_type() {
                                                            Type::F32
                                                        } else {
                                                            Type::F64
                                                        }
                                                    }
                                                    inkwell::types::BasicTypeEnum::PointerType(_) => Type::String,
                                                    _ => Type::I32,
                                                }
                                            }
                                        }
                                    } else {
                                        // No tracked type, infer from arg_type_enum
                                        match arg_type_enum {
                                            inkwell::types::BasicTypeEnum::IntType(it) => {
                                                match it.get_bit_width() {
                                                    8 => Type::I8,
                                                    16 => Type::I16,
                                                    32 => Type::I32,
                                                    64 => Type::I64,
                                                    128 => Type::I128,
                                                    1 => Type::Bool,
                                                    _ => Type::I32,
                                                }
                                            }
                                            inkwell::types::BasicTypeEnum::FloatType(ft) => {
                                                if ft == self.context.f32_type() {
                                                    Type::F32
                                                } else {
                                                    Type::F64
                                                }
                                            }
                                            inkwell::types::BasicTypeEnum::PointerType(_) => Type::String,
                                            _ => Type::I32,
                                        }
                                    }
                                } else {
                                    // Not an identifier, infer from LLVM type
                                    match arg_type_enum {
                                        inkwell::types::BasicTypeEnum::IntType(it) => {
                                            match it.get_bit_width() {
                                                8 => Type::I8,
                                                16 => Type::I16,
                                                32 => Type::I32,
                                                64 => Type::I64,
                                                128 => Type::I128,
                                                1 => Type::Bool,
                                                _ => Type::I32,
                                            }
                                        }
                                        inkwell::types::BasicTypeEnum::FloatType(ft) => {
                                            if ft == self.context.f32_type() {
                                                Type::F32
                                            } else {
                                                Type::F64
                                            }
                                        }
                                        inkwell::types::BasicTypeEnum::PointerType(_) => Type::String,
                                        _ => Type::I32,
                                    }
                                }
                            } else {
                                // Fallback
                                match arg_type_enum {
                                    inkwell::types::BasicTypeEnum::IntType(it) => {
                                        match it.get_bit_width() {
                                            8 => Type::I8,
                                            16 => Type::I16,
                                            32 => Type::I32,
                                            64 => Type::I64,
                                            128 => Type::I128,
                                            1 => Type::Bool,
                                            _ => Type::I32,
                                        }
                                    }
                                    inkwell::types::BasicTypeEnum::FloatType(ft) => {
                                        if ft == self.context.f32_type() {
                                            Type::F32
                                        } else {
                                            Type::F64
                                        }
                                    }
                                    inkwell::types::BasicTypeEnum::PointerType(_) => Type::String,
                                    _ => Type::I32,
                                }
                            };
                            
                            eprintln!("🔍 [NAMESPACE ARG {}] inferred type: {:?}", idx, ast_type);
                            type_suffix.push_str(&self.generate_type_suffix(&ast_type));
                        }

                        // Try exact match first: abs_i64_1
                        let param_count = arg_basic_vals.len();
                        let mangled_with_count = format!("{}{}_{}", method, type_suffix, param_count);
                        eprintln!("🔍 [NAMESPACE OVERLOAD] Trying mangled: {}", mangled_with_count);

                        if let Some(fn_val) = self.functions.get(&mangled_with_count) {
                            eprintln!("✅ Found exact match: {}", mangled_with_count);
                            *fn_val
                        } else {
                            // Try without param count
                            let mangled_name = format!("{}{}", method, type_suffix);
                            eprintln!("🔍 [NAMESPACE OVERLOAD] Trying legacy: {}", mangled_name);
                            
                            if let Some(fn_val) = self.functions.get(&mangled_name) {
                                eprintln!("✅ Found legacy match: {}", mangled_name);
                                *fn_val
                            } else {
                                // Fallback to base name
                                eprintln!("⚠️ No exact match, trying base name: {}", method);
                                *self.functions.get(method).ok_or_else(|| {
                                    format!("Module function {} not found in LLVM module", method)
                                })?
                            }
                        }
                    } else {
                        // No arguments, use base name
                        *self.functions.get(method).ok_or_else(|| {
                            format!("Module function {} not found in LLVM module", method)
                        })?
                    };

                    let call_site = self
                        .builder
                        .build_call(fn_val, &arg_vals, "modulecall")
                        .map_err(|e| format!("Failed to build module call: {}", e))?;

                    return Ok(call_site.try_as_basic_value().unwrap_basic());
                }
            }

            // Legacy: Try old-style module_function naming
            // ⚠️ CRITICAL: Don't confuse methods (Type_method) with module functions
            // Only treat as module call if it's NOT a method (has no receiver in function_defs)
            let module_func_name = format!("{}_{}", module_name, method);
            if self.functions.contains_key(&module_func_name) {
                // Check if this is actually a method (has receiver) - if so, skip module call path
                let is_method = self
                    .function_defs
                    .get(&module_func_name)
                    .map(|def| def.receiver.is_some())
                    .unwrap_or(false);

                if !is_method {
                    // It's a true module function, call it directly
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

                    return Ok(call_site.try_as_basic_value().unwrap_basic());
                }
                // If it's a method, fall through to static method resolution below
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
            eprintln!(
                "🔍 Potential type name: {} (is_type_name={})",
                potential_type_name, is_type_name
            );

            // Check if this is NOT a variable (static methods called on types, not instances)
            let is_not_variable = !self.variables.contains_key(potential_type_name);
            eprintln!("🔍 Is not variable: {}", is_not_variable);

            if is_type_name && is_not_variable {
                // This is a static method call: Type.method(args) or Vec<i32>.new()
                // Try static method first; if not found, fall through to instance method
                // resolution so that inline/instance methods are not shadowed by type name.
                eprintln!(
                    "🔍 Attempting static method call: {}.{}",
                    potential_type_name, method
                );
                match self.compile_static_method_call(potential_type_name, method, type_args, args)
                {
                    Ok(val) => {
                        eprintln!("✅ Static method call succeeded!");
                        return Ok(val);
                    }
                    Err(e) => {
                        eprintln!("❌ Static method call failed: {}", e);
                        // Return error instead of falling through
                        return Err(e);
                    }
                }
            }
        }

        // Method chaining: function_call().method()
        // Example: string_from("hello").trim()
        if let Expression::Call { func, args: call_args, .. } = receiver {
            eprintln!("🔗 Method chaining detected: {:?}().{}()", func, method);
            
            // Compile the function call first to get return value (struct by value)
            let func_return_val = self.compile_call(func, &[], call_args, None)?;
            eprintln!("🔗 Function return value compiled, now calling method: {}", method);
            
            // Infer struct type from function call
            let struct_name = if let Expression::Ident(func_name) = &**func {
                // For now, use simple heuristic: string_from -> String
                if func_name == "string_from" {
                    "String".to_string()
                } else {
                    // Try to infer from expression type
                    match self.infer_expression_type(receiver) {
                        Ok(Type::Named(name)) => name,
                        Ok(Type::Generic { name, .. }) => name,
                        _ => return Err(format!("Cannot infer struct type from function call: {}", func_name)),
                    }
                }
            } else {
                return Err("Method chaining only supported on function calls".to_string());
            };
            eprintln!("🔗 Inferred struct type: {}", struct_name);
            
            // Create temp storage for struct value (by value, not pointer!)
            let temp_ptr = self.builder
                .build_alloca(func_return_val.get_type(), "temp_chain")
                .map_err(|e| format!("Failed to create temp for chaining: {}", e))?;
            self.builder
                .build_store(temp_ptr, func_return_val)
                .map_err(|e| format!("Failed to store temp: {}", e))?;
            
            // Call method directly on temp_ptr (which holds the struct value)
            // Build method name with mangling
            let method_name = format!("{}_{}", struct_name, method);
            
            if let Some(fn_val) = self.functions.get(&method_name).cloned() {
                eprintln!("🔗 Calling method: {} on temp value", method_name);
                
                // Build args: receiver (temp_ptr) + method args
                let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![temp_ptr.into()];
                
                for arg in args {
                    let arg_val = self.compile_expression(arg)?;
                    arg_vals.push(arg_val.into());
                }
                
                let call_site = self.builder
                    .build_call(fn_val, &arg_vals, "chainedcall")
                    .map_err(|e| format!("Failed to call chained method: {}", e))?;
                
                return Ok(call_site.try_as_basic_value().unwrap_basic());
            } else {
                return Err(format!("Method {} not found for chaining on {}", method, struct_name));
            }
        }

        // Instance method call - get receiver info
        let (struct_name, receiver_val) = self.get_receiver_info(receiver)?;

        // ⭐ Phase 2: Use variable_concrete_types for receiver type (more reliable)
        let receiver_type = if let Expression::Ident(var_name) = receiver {
            // Check variable_concrete_types first (Phase 1 tracking)
            self.variable_concrete_types
                .get(var_name)
                .cloned()
                .unwrap_or_else(|| {
                    self.infer_expression_type(receiver)
                        .unwrap_or(Type::Unknown)
                })
        } else {
            self.infer_expression_type(receiver)?
        };
        eprintln!("🔍 Receiver type inferred: {:?}", receiver_type);
        let struct_type_args = self.extract_type_args_from_type(&receiver_type)?;
        eprintln!("🔍 Extracted type args: {:?}", struct_type_args);

        // ⭐ Phase 3: If type contains Unknown, try to infer from method call context
        let struct_type_args = if struct_type_args.iter().any(|t| matches!(t, Type::Unknown)) {
            if let Expression::Ident(var_name) = receiver {
                // Infer type from first method argument
                if !args.is_empty() {
                    if let Ok(first_arg_type) = self.infer_expression_type(&args[0]) {
                        eprintln!(
                            "⭐ Phase 3: Inferring Unknown from first arg: {:?}",
                            first_arg_type
                        );

                        // Update variable_concrete_types with inferred type
                        let concrete_type = Type::Generic {
                            name: struct_name.clone(),
                            type_args: vec![first_arg_type.clone()],
                        };
                        self.variable_concrete_types
                            .insert(var_name.clone(), concrete_type.clone());

                        // Extract type args from updated concrete type
                        self.extract_type_args_from_type(&concrete_type)?
                    } else {
                        struct_type_args
                    }
                } else {
                    struct_type_args
                }
            } else {
                struct_type_args
            }
        } else if struct_type_args.is_empty() {
            // ⭐ Phase 3: Empty type args - check if variable_concrete_types has updated info
            if let Expression::Ident(var_name) = receiver {
                if let Some(concrete_type) = self.variable_concrete_types.get(var_name) {
                    eprintln!(
                        "⭐ Phase 3: Re-extracting type args from variable_concrete_types: {:?}",
                        concrete_type
                    );
                    self.extract_type_args_from_type(concrete_type)?
                } else {
                    struct_type_args
                }
            } else {
                struct_type_args
            }
        } else {
            struct_type_args
        };

        // ⭐ Create mangled struct name for generic methods: Vec_i32, Container_String, etc.
        let mangled_struct_name = if struct_type_args.is_empty() {
            struct_name.clone()
        } else {
            let type_names: Vec<String> = struct_type_args
                .iter()
                .map(|ty| self.type_to_string(ty))
                .collect();
            format!("{}_{}", struct_name, type_names.join("_"))
        };

        // Resolve method name using mangled struct name
        let method_resolution_result = self.resolve_method_name(&mangled_struct_name, method, args);

        // ⭐ NEW: If method resolution fails, try generic method instantiation
        if method_resolution_result.is_err() {
            // Extract base struct name and type args from mangled name
            // Example: "Vec_i32" -> struct_name="Vec", type_args=[I32]
            let parts: Vec<&str> = struct_name.split('_').collect();
            let base_struct_name = parts[0];

            // Check if this is a generic struct with type arguments
            if let Some(struct_def) = self.struct_ast_defs.get(base_struct_name) {
                if !struct_def.type_params.is_empty() {
                    eprintln!(
                        "🔍 Attempting generic method instantiation for {}.{}",
                        struct_name, method
                    );
                    eprintln!(
                        "🔍 Final struct_type_args for instantiation: {:?}",
                        struct_type_args
                    );

                    // ⭐ Phase 2: Use type args from receiver type (already extracted above)
                    // This is more reliable than parsing mangled names
                    if struct_type_args.is_empty() {
                        eprintln!(
                            "⚠️  No type args found for generic struct {}",
                            base_struct_name
                        );
                        return Err(format!(
                            "Cannot instantiate method {}.{} without type arguments",
                            base_struct_name, method
                        ));
                    }

                    // Try to find generic method definition
                    if let Ok(method_def) = self.find_generic_method(base_struct_name, method) {
                        // Infer argument types for type parameter resolution
                        let arg_types: Vec<Type> = args
                            .iter()
                            .map(|arg| self.infer_expression_type(arg))
                            .collect::<Result<Vec<_>, _>>()?;

                        // Instantiate the generic method!
                        match self.instantiate_generic_method(
                            base_struct_name,
                            &struct_type_args,
                            method,
                            &method_def,
                            &arg_types,
                        ) {
                            Ok(fn_val) => {
                                eprintln!("✅ Generic method instantiated successfully!");

                                // Compile arguments and call the instantiated method
                                let arg_vals = self.compile_method_arguments_for_generic(
                                    base_struct_name,
                                    &struct_type_args,
                                    method,
                                    receiver,
                                    receiver_val.into(), // Convert PointerValue to BasicValueEnum
                                    args,
                                )?;

                                let call_site = self
                                    .builder
                                    .build_call(fn_val, &arg_vals, "genericmethodcall")
                                    .map_err(|e| {
                                        format!("Failed to build generic method call: {}", e)
                                    })?;

                                // Handle both value-returning and void functions
                                if let Some(val) = call_site.try_as_basic_value().basic() {
                                    return Ok(val);
                                } else {
                                    // Void function - return a dummy i32 zero
                                    return Ok(self.context.i32_type().const_int(0, false).into());
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️  Generic method instantiation failed: {}", e);
                                // Fall through to builtin fallback
                            }
                        }
                    }
                }
            }

            // If generic instantiation failed or not applicable, try builtin fallback
            // This is for stdlib types that have compiler builtin support
            let is_stdlib_with_builtin = matches!(
                base_struct_name,
                "Vec" | "Box" | "String" | "Map" | "Set" | "Channel" | "Array"
            );

            if is_stdlib_with_builtin {
                eprintln!(
                    "⚠️  Method '{}' not found, falling back to compiler builtin for {}",
                    method, base_struct_name
                );
                // Force builtin compilation by temporarily removing from struct_ast_defs
                let was_user_defined = self.struct_ast_defs.remove(base_struct_name);
                let result = self.try_compile_builtin_method(receiver, method, args)?;
                // Restore user-defined flag
                if let Some(def) = was_user_defined {
                    self.struct_ast_defs
                        .insert(base_struct_name.to_string(), def);
                }

                if let Some(builtin_result) = result {
                    return Ok(builtin_result);
                }
            }
        }

        let final_method_name = method_resolution_result?;

        // Validate method call
        self.validate_method_call(&final_method_name, method, is_mutable_call)?;

        // Compile arguments
        let arg_vals =
            self.compile_method_arguments(&final_method_name, receiver, receiver_val, args)?;

        // Get method function
        let fn_val = *self
            .functions
            .get(&final_method_name)
            .ok_or_else(|| format!("Method function {} not found", final_method_name))?;

        eprintln!(
            "📞 Calling method: {} (fn_val: {:?})",
            final_method_name,
            fn_val.get_name()
        );
        eprintln!(
            "📞 Arguments count: {}, arg_vals: {:?}",
            arg_vals.len(),
            arg_vals
        );

        // Build call
        let call_site = self
            .builder
            .build_call(fn_val, &arg_vals, "methodcall")
            .map_err(|e| format!("Failed to build method call: {}", e))?;

        // Handle both value-returning and void functions
        if let Some(val) = call_site.try_as_basic_value().basic() {
            Ok(val)
        } else {
            // Void function - return a dummy i32 zero
            Ok(self.context.i32_type().const_int(0, false).into())
        }
    }
}
