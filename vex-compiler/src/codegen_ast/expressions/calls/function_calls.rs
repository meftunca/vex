// Function call compilation

use crate::codegen_ast::ASTCodeGen;
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

            // Special case: print/println with format string detection
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
