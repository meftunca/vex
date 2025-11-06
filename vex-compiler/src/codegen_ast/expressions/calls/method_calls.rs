// Method call compilation (instance methods, trait methods, builtin types)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
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
}
