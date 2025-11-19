// Static method call compilation (Type.method() calls)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile static method calls: Type.method(args) or Type<i32>.method(args)
    pub(crate) fn compile_static_method_call(
        &mut self,
        type_name: &str,
        method: &str,
        type_args: &[Type],
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Handle generic static methods: HashMap.new<K, V>()
        // Need to instantiate generic method with concrete types
        
        eprintln!("🔍 compile_static_method_call: {}.{}() with type_args: {:?}",
            type_name, method, type_args);

        // For generic static methods, look up the generic function definition
        // Expected in function_defs: HashMap_K_V_new
        let generic_lookup_key = if !type_args.is_empty() {
            // Build generic key with type parameter NAMES (not concrete types)
            // Need to find the function definition first to get type param names
            let base_key = format!("{}_{}", type_name, method);
            
            eprintln!("🔍 Looking for generic static method in function_defs");
            eprintln!("🔍 function_defs keys: {:?}", self.function_defs.keys().filter(|k| k.contains(type_name)).collect::<Vec<_>>());
            
            // Try to find a matching generic static method in function_defs
            let mut found_def = None;
            for (key, def) in &self.function_defs {
                eprintln!("  Checking key: {} (is_static={}, type_params={}, starts_with={})", 
                    key, def.is_static, def.type_params.len(),
                    key.starts_with(&format!("{}_", type_name)));
                
                let method_suffix = format!("_{}", method);
                let method_mid = format!("_{}_", method);

                // Check if key matches: Type_..._Method or Type_..._Method_Params
                if key.starts_with(&format!("{}_", type_name)) && 
                   (key.ends_with(&method_suffix) || key.contains(&method_mid)) &&
                   def.is_static &&
                   !def.type_params.is_empty() &&
                   def.params.len() == args.len() {
                    found_def = Some((key.clone(), def.clone()));
                    break;
                }
            }

            if let Some((key, generic_def)) = found_def {
                eprintln!("🔍 Found generic static method: {}", key);
                eprintln!("🔍 Type params: {:?}", generic_def.type_params);
                
                // Instantiate the generic function with concrete types
                // Similar to how we handle generic functions
                let mangled_name = if !type_args.is_empty() {
                    let mut mangled = type_name.to_string();
                    for ty in type_args {
                        mangled.push('_');
                        mangled.push_str(&self.type_to_string(ty));
                    }
                    mangled.push('_');
                    mangled.push_str(method);
                    mangled
                } else {
                    format!("{}_{}", type_name, method)
                };

                eprintln!("🔍 Instantiating as: {}", mangled_name);

                // Check if already instantiated
                if let Some(fn_val) = self.functions.get(&mangled_name).copied() {
                    eprintln!("✅ Found instantiated function: {}", mangled_name);
                    let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        arg_vals.push(val.into());
                    }

                    let call_site = self
                        .builder
                        .build_call(fn_val, &arg_vals, "static_method_call")
                        .map_err(|e| format!("Failed to build static method call: {}", e))?;

                    return Ok(call_site.try_as_basic_value().unwrap_basic());
                }

                // Instantiate generic static method
                let instantiated_fn = self.instantiate_generic_static_method(
                    type_name,
                    method,
                    type_args,
                    &generic_def,
                )?;

                // Call the instantiated function
                let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                for arg in args {
                    let val = self.compile_expression(arg)?;
                    arg_vals.push(val.into());
                }

                let call_site = self
                    .builder
                    .build_call(instantiated_fn, &arg_vals, "static_method_call")
                    .map_err(|e| format!("Failed to build static method call: {}", e))?;

                return Ok(call_site.try_as_basic_value().unwrap_basic());
            }

            base_key
        } else {
            format!("{}_{}", type_name, method)
        };

        // Fallback: Try old non-generic path
        // Handle generic static methods: Vec<i32>.new()
        // Mangle function name with type arguments
        let base_method_name = if !type_args.is_empty() {
            // Generic static method: Vec<i32>.new() → vec_i32_new
            let mut mangled = type_name.to_lowercase();
            for ty in type_args {
                mangled.push('_');
                mangled.push_str(&self.type_to_string(ty));
            }
            mangled.push('_');
            mangled.push_str(method);
            mangled
        } else {
            // Non-generic: Vec.new() → vec_new
            format!("{}_{}", type_name.to_lowercase(), method)
        };

        // Determine PascalCase variant once so we can reuse it below
        let pascal_method_name = if !type_args.is_empty() {
            let mut mangled = type_name.to_string();
            for ty in type_args {
                mangled.push('_');
                mangled.push_str(&self.type_to_string(ty));
            }
            mangled.push('_');
            mangled.push_str(method);
            mangled
        } else {
            format!("{}_{}", type_name, method)
        };

        // Prefer user-defined/static stdlib implementations before compiler builtins
        if let Some(fn_val) = self.functions.get(&base_method_name).copied() {
            eprintln!(
                "🔍 Static lookup (lowercase): {} -> found function? {}",
                base_method_name, true
            );
            // If the function is actually an inline/instance method (has a receiver),
            // don't treat it as a static method. This allows calling instance methods
            // on variables while still supporting Type.method() for true static functions.
            if let Some(func_def) = self.function_defs.get(&base_method_name) {
                if func_def.receiver.is_some() {
                    // It's an instance method - skip static resolution
                } else {
                    let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        arg_vals.push(val.into());
                    }

                    let call_site = self
                        .builder
                        .build_call(fn_val, &arg_vals, "static_method_call")
                        .map_err(|e| format!("Failed to build static method call: {}", e))?;

                    return Ok(call_site.try_as_basic_value().unwrap_basic());
                }
            } else {
                let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                for arg in args {
                    let val = self.compile_expression(arg)?;
                    arg_vals.push(val.into());
                }

                let call_site = self
                    .builder
                    .build_call(fn_val, &arg_vals, "static_method_call")
                    .map_err(|e| format!("Failed to build static method call: {}", e))?;

                return Ok(call_site.try_as_basic_value().unwrap_basic());
            }
        } else if let Some(fn_val) = self.functions.get(&pascal_method_name).copied() {
            eprintln!(
                "🔍 Static lookup (pascal): {} -> found function? {}",
                pascal_method_name, true
            );
            if let Some(func_def) = self.function_defs.get(&pascal_method_name) {
                if func_def.receiver.is_some() {
                    // It's an instance method. Allow a special-case where the
                    // method can be invoked as a constructor via Type.method():
                    // If the method returns `Self` (or the same named type), build a
                    // temporary receiver alloca and pass it so the call matches the
                    // function signature. Otherwise, skip this instance method.
                    if let Some(ret_ty) = &func_def.return_type {
                        let is_constructor = matches!(ret_ty, vex_ast::Type::SelfType)
                            || matches!(ret_ty, vex_ast::Type::Named(name) if name == type_name);

                        if is_constructor {
                            if let Some(receiver_param) = &func_def.receiver {
                                // Build a zero-sized or typed receiver on stack as pointer
                                let receiver_llvm_ty = self.ast_type_to_llvm(&receiver_param.ty);
                                let receiver_ptr = self
                                    .builder
                                    .build_alloca(receiver_llvm_ty, "static_self")
                                    .map_err(|e| {
                                        format!(
                                            "Failed to allocate receiver for static call: {}",
                                            e
                                        )
                                    })?;

                                let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                                // Pass receiver first, then other args
                                arg_vals.push(receiver_ptr.into());
                                for arg in args {
                                    let val = self.compile_expression(arg)?;
                                    arg_vals.push(val.into());
                                }

                                let call_site = self
                                    .builder
                                    .build_call(fn_val, &arg_vals, "static_method_call")
                                    .map_err(|e| {
                                        format!("Failed to build static method call: {}", e)
                                    })?;

                                // ⭐ CRITICAL FIX: Handle struct return values properly
                                // If function returns a struct by value, the call returns a struct value
                                // We need to allocate space and store it, then return the pointer
                                if let Some(return_val) = call_site.try_as_basic_value().basic() {
                                    if return_val.is_struct_value() {
                                        // Allocate space for the returned struct
                                        let struct_ty = return_val.get_type();
                                        let result_ptr = self
                                            .builder
                                            .build_alloca(struct_ty, "constructor_result")
                                            .map_err(|e| {
                                                format!(
                                                    "Failed to allocate constructor result: {}",
                                                    e
                                                )
                                            })?;

                                        // Store the returned struct value
                                        self.builder.build_store(result_ptr, return_val).map_err(
                                            |e| {
                                                format!("Failed to store constructor result: {}", e)
                                            },
                                        )?;

                                        // Return the pointer to the struct
                                        return Ok(result_ptr.into());
                                    } else {
                                        return Ok(return_val);
                                    }
                                } else {
                                    return Err(
                                        "Constructor method must return a value".to_string()
                                    );
                                }
                            }
                        }
                    }
                    // Not a constructor - skip instance method match
                } else {
                    let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        arg_vals.push(val.into());
                    }

                    let call_site = self
                        .builder
                        .build_call(fn_val, &arg_vals, "static_method_call")
                        .map_err(|e| format!("Failed to build static method call: {}", e))?;

                    return Ok(call_site.try_as_basic_value().unwrap_basic());
                }
            } else {
                let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                for arg in args {
                    let val = self.compile_expression(arg)?;
                    arg_vals.push(val.into());
                }

                let call_site = self
                    .builder
                    .build_call(fn_val, &arg_vals, "static_method_call")
                    .map_err(|e| format!("Failed to build static method call: {}", e))?;

                return Ok(call_site.try_as_basic_value().unwrap_basic());
            }
        }

        // No user-defined version found; try compiler builtin as fallback
        let pascal_builtin_name = format!("{}.{}", type_name, method);
        if let Some(builtin_fn) = self
            .builtins
            .get(&pascal_builtin_name)
            .or_else(|| self.builtins.get(&base_method_name))
        {
            let mut arg_vals: Vec<BasicValueEnum> = vec![];
            for arg in args {
                let val = self.compile_expression(arg)?;
                arg_vals.push(val);
            }

            return builtin_fn(self, &arg_vals);
        }

        let is_stdlib_type = matches!(
            type_name,
            "Vec" | "Box" | "String" | "Map" | "Set" | "Channel"
        );

        if is_stdlib_type && method == "new" {
            eprintln!(
                "⚠️  Static method {}.{}() not found, no compiler builtin registered",
                type_name, method
            );
        }

        Err(format!(
            "Static method {}.{}() not found. Expected function: {} or {}",
            type_name, method, base_method_name, pascal_method_name
        ))
    }
}
