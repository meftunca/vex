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

                    return Ok(call_site.try_as_basic_value().unwrap_basic());
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

                return Ok(call_site.try_as_basic_value().unwrap_basic());
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
                return self.compile_static_method_call(
                    potential_type_name,
                    method,
                    type_args,
                    args,
                );
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

        // Resolve method name
        let method_resolution_result = self.resolve_method_name(&struct_name, method, args);

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
                "Vec" | "Box" | "String" | "Map" | "Set" | "Channel"
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
