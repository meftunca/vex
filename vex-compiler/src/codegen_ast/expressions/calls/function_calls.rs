// Function call compilation

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile function call (with default parameter support)
    pub(crate) fn compile_call(
        &mut self,
        func_expr: &Expression,
        type_args: &[Type],
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        eprintln!("🔵 compile_call: type_args.len()={}", type_args.len());
        if !type_args.is_empty() {
            eprintln!("  Type args: {:?}", type_args);
        }

        // Get function definition to check for default parameters
        let func_def_opt = if let Expression::Ident(func_name) = func_expr {
            self.function_defs.get(func_name).cloned()
        } else {
            None
        };

        // Build final argument list (filling in defaults if needed)
        let mut final_args = args.to_vec();
        
        if let Some(func_def) = &func_def_opt {
            let provided_count = args.len();
            let expected_count = func_def.params.len();
            
            // Check if function is variadic
            if func_def.is_variadic {
                // Variadic function: allow more args than params
                if provided_count < expected_count {
                    // Fill in missing arguments with defaults
                    for i in provided_count..expected_count {
                        if let Some(default_expr) = &func_def.params[i].default_value {
                            final_args.push((**default_expr).clone());
                        } else {
                            return Err(format!(
                                "Missing argument {} for function (no default value)",
                                func_def.params[i].name
                            ));
                        }
                    }
                }
                // For variadic, extra args are OK
            } else {
                // Non-variadic: strict arg count check
                if provided_count < expected_count {
                    // Fill in missing arguments with defaults
                    for i in provided_count..expected_count {
                        if let Some(default_expr) = &func_def.params[i].default_value {
                            final_args.push((**default_expr).clone());
                        } else {
                            return Err(format!(
                                "Missing argument {} for function (no default value)",
                                func_def.params[i].name
                            ));
                        }
                    }
                } else if provided_count > expected_count {
                    return Err(format!(
                        "Too many arguments: expected {}, got {}",
                        expected_count, provided_count
                    ));
                }
            }
        }

        // Compile arguments
        let mut arg_vals: Vec<BasicMetadataValueEnum> = Vec::new();
        let mut arg_basic_vals: Vec<BasicValueEnum> = Vec::new();
        
        for (i, arg) in final_args.iter().enumerate() {
            let val = self.compile_expression(arg)?;

            // Check if parameter expects 'any' type
            let expects_any = if let Some(func_def) = &func_def_opt {
                if i < func_def.params.len() {
                    matches!(func_def.params[i].ty, Type::Any)
                } else if func_def.is_variadic {
                    // Variadic parameter type
                    matches!(func_def.variadic_type, Some(Type::Any))
                } else {
                    false
                }
            } else {
                false
            };

            // If parameter expects 'any', box the value (allocate on stack and pass pointer)
            if expects_any {
                let val_type = val.get_type();
                let alloca = self
                    .builder
                    .build_alloca(val_type, "any_box")
                    .map_err(|e| format!("Failed to allocate any box: {}", e))?;
                
                self.builder
                    .build_store(alloca, val)
                    .map_err(|e| format!("Failed to store to any box: {}", e))?;
                
                // Pass pointer as any
                arg_vals.push(alloca.into());
                arg_basic_vals.push(alloca.into());
                continue;
            }

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

            // Special case: format() - type-safe zero-cost formatting
            if func_name == "format" {
                if args.is_empty() {
                    return Err("format() requires at least a format string".to_string());
                }

                // First argument must be a string literal
                if let Expression::StringLiteral(fmt_str) = &args[0] {
                    // Infer types of remaining arguments
                    let mut arg_types = Vec::new();
                    for arg in &args[1..] {
                        let ty = self.infer_expression_type(arg)?;
                        arg_types.push(ty);
                    }

                    // Call type-safe format compiler
                    return crate::codegen_ast::builtins::compile_typesafe_format(
                        self,
                        fmt_str,
                        &arg_basic_vals[1..],
                        &arg_types,
                    );
                } else {
                    return Err("format() first argument must be a string literal".to_string());
                }
            }

            // Special case: print() and println() with format string detection
            if func_name == "print" || func_name == "println" {
                return self.compile_print_call(func_name, args, &arg_basic_vals);
            }

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
                        // Use explicit type arguments if provided, otherwise infer from call arguments
                        let final_type_args = if !type_args.is_empty() {
                            eprintln!(
                                "  ✅ Using explicit type args for {}: {:?}",
                                func_name, type_args
                            );
                            type_args.to_vec()
                        } else {
                            eprintln!("  ⚠️  Inferring type args for {} from arguments", func_name);
                            self.infer_type_args_from_call(&func_def, args)?
                        };

                        eprintln!(
                            "  🔧 Final type args for {}: {:?}",
                            func_name, final_type_args
                        );

                        // Instantiate generic function
                        self.instantiate_generic_function(&func_def, &final_type_args)?
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

                // Handle both value-returning and void functions
                if let Some(val) = call_site.try_as_basic_value().left() {
                    return Ok(val);
                } else {
                    // Void function - return a dummy i32 zero
                    // This is OK for expression statements (result ignored)
                    return Ok(self.context.i32_type().const_int(0, false).into());
                }
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
}
