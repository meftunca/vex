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

        // EARLY CHECK: Type constructor with explicit type args (e.g., Vec<i32>())
        if !type_args.is_empty() {
            if let Expression::Ident(func_name) = func_expr {
                let has_user_ctor = self.functions.contains_key(func_name)
                    || self.function_defs.contains_key(func_name);

                if !has_user_ctor {
                    // Try builtin type constructor: Type.new
                    let builtin_name = format!("{}.new", func_name);
                    eprintln!("  🔍 Checking builtin: {}", builtin_name);
                    if let Some(builtin_fn) = self.builtins.get(&builtin_name) {
                        eprintln!("  ✅ Using builtin constructor: {}", builtin_name);
                        // Compile arguments first
                        let mut arg_basic_vals: Vec<BasicValueEnum> = Vec::new();
                        for arg in args.iter() {
                            let val = self.compile_expression(arg)?;
                            arg_basic_vals.push(val);
                        }
                        return builtin_fn(self, &arg_basic_vals);
                    } else {
                        eprintln!("  ⚠️  Builtin not found: {}", builtin_name);
                    }
                }
            }
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
            // ⭐ CRITICAL: Check parameter type BEFORE compiling expression
            let param_type_opt = if let Some(func_def) = &func_def_opt {
                if i < func_def.params.len() {
                    Some(&func_def.params[i].ty)
                } else {
                    None
                }
            } else {
                None
            };

            // ⭐ CRITICAL: Struct parameters expect BY VALUE, not pointer
            // Check if we need to load struct value before compiling
            let param_expects_struct_by_value = if let Some(param_ty) = param_type_opt {
                match param_ty {
                    Type::Named(type_name) => self.struct_defs.contains_key(type_name),
                    Type::Generic { name, .. } => {
                        self.struct_defs.contains_key(name) || name.contains('_')
                    }
                    _ => false,
                }
            } else {
                false
            };

            let is_struct_variable = if let Expression::Ident(name) = arg {
                self.variable_struct_names.contains_key(name)
            } else {
                false
            };

            // ⭐ For struct variables, load struct value for BY VALUE parameter
            let mut val = if is_struct_variable && param_expects_struct_by_value {
                if let Expression::Ident(name) = arg {
                    if let Some(struct_ptr) = self.variables.get(name) {
                        // Load struct value for BY VALUE parameter
                        let param_llvm_ty = if let Some(param_ty) = param_type_opt {
                            self.ast_type_to_llvm(param_ty)
                        } else {
                            return Err("Cannot determine struct type".to_string());
                        };

                        eprintln!("🔄 Loading struct {} by value for parameter", name);
                        self.builder
                            .build_load(param_llvm_ty, *struct_ptr, "struct_by_value_load")
                            .map_err(|e| {
                                format!("Failed to load struct for by-value param: {}", e)
                            })?
                    } else {
                        return Err(format!("Struct variable {} not found", name));
                    }
                } else {
                    self.compile_expression(arg)?
                }
            } else {
                self.compile_expression(arg)?
            };

            // ⭐ CRITICAL FIX: Cast integer arguments to match parameter type width
            // This prevents "i32 100 passed to i64 parameter" LLVM errors
            if let Some(param_ty) = param_type_opt {
                if let BasicValueEnum::IntValue(int_val) = val {
                    let target_llvm_type = self.ast_type_to_llvm(param_ty);
                    if let inkwell::types::BasicTypeEnum::IntType(target_int_type) = target_llvm_type {
                        let current_width = int_val.get_type().get_bit_width();
                        let target_width = target_int_type.get_bit_width();
                        
                        if current_width != target_width {
                            eprintln!(
                                "🔄 Casting argument {} from i{} to i{} to match parameter type",
                                i, current_width, target_width
                            );
                            
                            val = if current_width < target_width {
                                // Extend (sign or zero based on target type)
                                let is_unsigned = matches!(
                                    param_ty,
                                    Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128
                                );
                                if is_unsigned {
                                    self.builder
                                        .build_int_z_extend(int_val, target_int_type, "arg_zext")
                                        .map_err(|e| format!("Failed to zero-extend argument: {}", e))?
                                        .into()
                                } else {
                                    self.builder
                                        .build_int_s_extend(int_val, target_int_type, "arg_sext")
                                        .map_err(|e| format!("Failed to sign-extend argument: {}", e))?
                                        .into()
                                }
                            } else {
                                // Truncate
                                self.builder
                                    .build_int_truncate(int_val, target_int_type, "arg_trunc")
                                    .map_err(|e| format!("Failed to truncate argument: {}", e))?
                                    .into()
                            };
                        }
                    }
                }
            }

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

            // ⭐ REMOVED: Duplicate struct loading logic - already handled above
            // The early check at lines 116-161 already loads struct values when needed
            // Just pass the value as-is
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

                        return Ok(call_site.try_as_basic_value().unwrap_basic());
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

            // Special case: sizeof<T>() - compile-time size calculation
            if func_name == "sizeof" && !type_args.is_empty() {
                let ty = &type_args[0];
                let llvm_type = self.ast_type_to_llvm(ty);
                let size = match llvm_type {
                    inkwell::types::BasicTypeEnum::IntType(it) => it.get_bit_width() / 8,
                    inkwell::types::BasicTypeEnum::FloatType(ft) => {
                        if ft == self.context.f32_type() {
                            4
                        } else {
                            8
                        }
                    }
                    inkwell::types::BasicTypeEnum::PointerType(_) => 8,
                    inkwell::types::BasicTypeEnum::StructType(st) => {
                        // Sum field sizes (simplified - no padding)
                        let mut total = 0u32;
                        for i in 0..st.count_fields() {
                            if let Some(field_ty) = st.get_field_type_at_index(i) {
                                total += match field_ty {
                                    inkwell::types::BasicTypeEnum::IntType(it) => {
                                        it.get_bit_width() / 8
                                    }
                                    inkwell::types::BasicTypeEnum::FloatType(_) => 8,
                                    inkwell::types::BasicTypeEnum::PointerType(_) => 8,
                                    _ => 8,
                                };
                            }
                        }
                        total
                    }
                    inkwell::types::BasicTypeEnum::ArrayType(at) => {
                        // Get element type and array length
                        let elem_ty = at.get_element_type();
                        let array_len = at.len();

                        // Calculate element size
                        let elem_size = match elem_ty.try_into() {
                            Ok(inkwell::types::BasicTypeEnum::IntType(it)) => {
                                it.get_bit_width() / 8
                            }
                            Ok(inkwell::types::BasicTypeEnum::FloatType(_)) => 8,
                            Ok(inkwell::types::BasicTypeEnum::PointerType(_)) => 8,
                            _ => 8,
                        };
                        elem_size * array_len
                    }
                    _ => return Err(format!("Cannot determine size of type: {:?}", ty)),
                };
                let size_val = self.context.i64_type().const_int(size as u64, false);
                return Ok(size_val.into());
            }

            // Special case: print() and println() with format string detection
            if func_name == "print" || func_name == "println" {
                return self.compile_print_call(func_name, args, &arg_basic_vals);
            }

            // Check if this is a builtin function (only if no user-defined version exists)
            let has_user_defined = self.functions.contains_key(func_name)
                || self.function_defs.contains_key(func_name);
            if !has_user_defined {
                if let Some(builtin_fn) = self.builtins.get(func_name) {
                    return builtin_fn(self, &arg_basic_vals);
                }
            }

            // Check if this is a type constructor (e.g., Vec<i32>())
            if !type_args.is_empty() {
                let has_user_ctor = self.functions.contains_key(func_name)
                    || self.function_defs.contains_key(func_name);
                if !has_user_ctor {
                    // Try builtin type constructor: Type.new
                    let builtin_name = format!("{}.new", func_name);
                    if let Some(builtin_fn) = self.builtins.get(&builtin_name) {
                        eprintln!("  ✅ Using builtin constructor: {}", builtin_name);
                        return builtin_fn(self, &arg_basic_vals);
                    }
                }
            }

            // ⭐ Get or instantiate function
            // After instantiation, update func_def_opt to point to instantiated version
            let fn_val_opt = if self.variables.contains_key(func_name) {
                // This is a variable - will be handled in complex expression path
                None
            } else if self.function_params.contains_key(func_name) {
                // Function pointer parameter - will be handled in complex expression path
                None
            } else {
                // Check if this is a global function that needs instantiation
                if let Some(fn_val) = self.functions.get(func_name) {
                    Some(*fn_val)
                } else if let Some(func_def) = self.function_defs.get(func_name).cloned() {
                    // Generic function - instantiate it
                    if !func_def.type_params.is_empty() {
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

                        // Instantiate
                        let fn_val =
                            self.instantiate_generic_function(&func_def, &final_type_args)?;

                        // ⭐ CRITICAL: Now RE-COMPILE ARGUMENTS with instantiated function's parameter types
                        // Clear previously compiled arguments
                        arg_vals.clear();
                        arg_basic_vals.clear();

                        // Get instantiated function definition
                        let type_names: Vec<String> = final_type_args
                            .iter()
                            .map(|t| self.type_to_string(t))
                            .collect();
                        let mangled_name = format!("{}_{}", func_name, type_names.join("_"));
                        let inst_func_def = self.function_defs.get(&mangled_name).cloned();

                        // Recompile arguments with correct types
                        if let Some(inst_def) = &inst_func_def {
                            for (i, arg) in final_args.iter().enumerate() {
                                let param_type_opt = if i < inst_def.params.len() {
                                    Some(&inst_def.params[i].ty)
                                } else {
                                    None
                                };

                                let param_expects_struct_by_value =
                                    if let Some(param_ty) = param_type_opt {
                                        match param_ty {
                                            Type::Named(type_name) => {
                                                self.struct_defs.contains_key(type_name)
                                            }
                                            Type::Generic { name, .. } => {
                                                self.struct_defs.contains_key(name)
                                                    || name.contains('_')
                                            }
                                            _ => false,
                                        }
                                    } else {
                                        false
                                    };

                                let is_struct_variable = if let Expression::Ident(name) = arg {
                                    self.variable_struct_names.contains_key(name)
                                } else {
                                    false
                                };

                                let mut val = if is_struct_variable && param_expects_struct_by_value {
                                    if let Expression::Ident(name) = arg {
                                        if let Some(struct_ptr) = self.variables.get(name) {
                                            let param_llvm_ty =
                                                self.ast_type_to_llvm(param_type_opt.unwrap());
                                            self.builder
                                                .build_load(param_llvm_ty, *struct_ptr, "struct_by_value_load")
                                                .map_err(|e| format!("Failed to load struct for by-value param: {}", e))?
                                        } else {
                                            return Err(format!(
                                                "Struct variable {} not found",
                                                name
                                            ));
                                        }
                                    } else {
                                        self.compile_expression(arg)?
                                    }
                                } else {
                                    self.compile_expression(arg)?
                                };

                                // ⭐ CRITICAL FIX: Cast integer arguments to match parameter type (generic functions too)
                                if let Some(param_ty) = param_type_opt {
                                    if let BasicValueEnum::IntValue(int_val) = val {
                                        let target_llvm_type = self.ast_type_to_llvm(param_ty);
                                        if let inkwell::types::BasicTypeEnum::IntType(target_int_type) = target_llvm_type {
                                            let current_width = int_val.get_type().get_bit_width();
                                            let target_width = target_int_type.get_bit_width();
                                            
                                            if current_width != target_width {
                                                eprintln!(
                                                    "🔄 [Generic] Casting argument {} from i{} to i{} to match parameter type",
                                                    i, current_width, target_width
                                                );
                                                
                                                val = if current_width < target_width {
                                                    let is_unsigned = matches!(
                                                        param_ty,
                                                        Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128
                                                    );
                                                    if is_unsigned {
                                                        self.builder
                                                            .build_int_z_extend(int_val, target_int_type, "arg_zext")
                                                            .map_err(|e| format!("Failed to zero-extend argument: {}", e))?
                                                            .into()
                                                    } else {
                                                        self.builder
                                                            .build_int_s_extend(int_val, target_int_type, "arg_sext")
                                                            .map_err(|e| format!("Failed to sign-extend argument: {}", e))?
                                                            .into()
                                                    }
                                                } else {
                                                    self.builder
                                                        .build_int_truncate(int_val, target_int_type, "arg_trunc")
                                                        .map_err(|e| format!("Failed to truncate argument: {}", e))?
                                                        .into()
                                                };
                                            }
                                        }
                                    }
                                }

                                arg_vals.push(val.into());
                                arg_basic_vals.push(val);
                            }
                        }

                        Some(fn_val)
                    } else {
                        None
                    }
                } else {
                    return Err(format!("Function {} not found", func_name));
                }
            };

            // If we have a function value, we can proceed with the call now
            if let Some(fn_val) = fn_val_opt {
                // ⭐ CRITICAL FIX: Cast arguments to match LLVM function signature
                // This is needed for external C functions (malloc, free, etc.) that don't have AST definitions
                let fn_type = fn_val.get_type();
                let param_types = fn_type.get_param_types();
                
                // Cast arguments if needed
                for (i, arg_val_meta) in arg_vals.iter_mut().enumerate() {
                    if i < param_types.len() {
                        let param_type = param_types[i];
                        let arg_val = match arg_val_meta {
                            inkwell::values::BasicMetadataValueEnum::IntValue(iv) => BasicValueEnum::IntValue(*iv),
                            inkwell::values::BasicMetadataValueEnum::FloatValue(fv) => BasicValueEnum::FloatValue(*fv),
                            inkwell::values::BasicMetadataValueEnum::PointerValue(pv) => BasicValueEnum::PointerValue(*pv),
                            inkwell::values::BasicMetadataValueEnum::ArrayValue(av) => BasicValueEnum::ArrayValue(*av),
                            inkwell::values::BasicMetadataValueEnum::StructValue(sv) => BasicValueEnum::StructValue(*sv),
                            inkwell::values::BasicMetadataValueEnum::VectorValue(vv) => BasicValueEnum::VectorValue(*vv),
                            _ => continue,
                        };
                        
                        // Cast integers if width mismatch
                        if let (BasicValueEnum::IntValue(int_val), inkwell::types::BasicMetadataTypeEnum::IntType(target_int_type)) = (arg_val, param_type) {
                            let current_width = int_val.get_type().get_bit_width();
                            let target_width = target_int_type.get_bit_width();
                            
                            if current_width != target_width {
                                eprintln!(
                                    "🔄 [External] Casting argument {} from i{} to i{} for function signature",
                                    i, current_width, target_width
                                );
                                
                                let casted = if current_width < target_width {
                                    // For external C functions, always zero-extend (size_t is unsigned)
                                    self.builder
                                        .build_int_z_extend(int_val, target_int_type, "ext_arg_cast")
                                        .map_err(|e| format!("Failed to cast external function argument: {}", e))?
                                        .into()
                                } else {
                                    self.builder
                                        .build_int_truncate(int_val, target_int_type, "ext_arg_trunc")
                                        .map_err(|e| format!("Failed to truncate external function argument: {}", e))?
                                        .into()
                                };
                                *arg_val_meta = casted;
                            }
                        }
                    }
                }
                
                // Build call
                let call_site = self
                    .builder
                    .build_call(fn_val, &arg_vals, "call")
                    .map_err(|e| format!("Failed to build call: {}", e))?;

                // Handle both value-returning and void functions
                if let Some(val) = call_site.try_as_basic_value().basic() {
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
                    // Check if environment pointer is null (pure closure without captures)
                    if env_ptr.is_null() {
                        eprintln!(
                            "🎯 Found closure variable '{}' without environment (pure function)",
                            name
                        );
                        (false, None)
                    } else {
                        eprintln!("🎯 Found closure variable '{}' with environment", name);
                        (true, Some(*env_ptr))
                    }
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
                    // Try to retrieve stored closure type first
                    if let Some((param_types, return_type)) = self.closure_types.get(name) {
                        eprintln!("✨ Using stored closure type for {}", name);

                        // Build function type from stored signature
                        let mut llvm_param_types: Vec<inkwell::types::BasicMetadataTypeEnum> =
                            vec![];
                        if has_environment {
                            llvm_param_types.push(
                                self.context
                                    .ptr_type(inkwell::AddressSpace::default())
                                    .into(),
                            );
                        }
                        for param_ty in param_types {
                            llvm_param_types.push(self.ast_type_to_llvm(param_ty).into());
                        }

                        let ret_basic_type = self.ast_type_to_llvm(return_type);
                        // Match on the BasicTypeEnum to extract the concrete type
                        match ret_basic_type {
                            inkwell::types::BasicTypeEnum::IntType(it) => {
                                it.fn_type(&llvm_param_types, false)
                            }
                            inkwell::types::BasicTypeEnum::FloatType(ft) => {
                                ft.fn_type(&llvm_param_types, false)
                            }
                            inkwell::types::BasicTypeEnum::PointerType(pt) => {
                                pt.fn_type(&llvm_param_types, false)
                            }
                            inkwell::types::BasicTypeEnum::StructType(st) => {
                                st.fn_type(&llvm_param_types, false)
                            }
                            inkwell::types::BasicTypeEnum::ArrayType(at) => {
                                at.fn_type(&llvm_param_types, false)
                            }
                            inkwell::types::BasicTypeEnum::VectorType(vt) => {
                                vt.fn_type(&llvm_param_types, false)
                            }
                            inkwell::types::BasicTypeEnum::ScalableVectorType(svt) => {
                                svt.fn_type(&llvm_param_types, false)
                            }
                        }
                    } else {
                        // Fallback: infer from call arguments
                        eprintln!(
                            "⚠️  Inferring closure type from call arguments for {}",
                            name
                        );

                        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![];
                        if has_environment {
                            param_types.push(
                                self.context
                                    .ptr_type(inkwell::AddressSpace::default())
                                    .into(),
                            );
                        }
                        for arg_val in &arg_basic_vals {
                            param_types.push(arg_val.get_type().into());
                        }

                        // Assume i32 return type as last resort
                        let ret_type = self.context.i32_type();
                        ret_type.fn_type(&param_types, false)
                    }
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
            eprintln!(
                "🔧 Indirect call: fn_type={:?}, args_count={}, has_env={}",
                fn_type,
                final_args.len(),
                has_environment
            );
            let call_site = self
                .builder
                .build_indirect_call(fn_type, fn_ptr, &final_args, "indirect_call")
                .map_err(|e| format!("Failed to build indirect call: {}", e))?;

            return Ok(call_site.try_as_basic_value().unwrap_basic());
        } else {
            return Err("Function expression did not evaluate to a function pointer".to_string());
        }
    }
}
