// src/codegen/functions/compile.rs
use super::super::*;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_function(&mut self, func: &Function) -> Result<(), String> {
        eprintln!(
            "🔨 compile_function: {} (receiver: {})",
            func.name,
            func.receiver.is_some()
        );
        eprintln!("   Body has {} statements", func.body.statements.len());
        if !func.body.statements.is_empty() {
            eprintln!("   First stmt: {:?}", func.body.statements[0]);
        }

        // Initialize async runtime if this is main() and we have a runtime handle
        if func.name == "main" && self.global_runtime.is_some() {
            eprintln!("🔄 main() function with async runtime - runtime already initialized");
        }

        if func.is_async {
            let previous_return_type = self.current_function_return_type.clone();
            self.current_function_return_type = func.return_type.clone();
            let result = self.compile_async_function(func);
            self.current_function_return_type = previous_return_type;
            return result;
        }

        let previous_return_type = self.current_function_return_type.clone();
        self.current_function_return_type = func.return_type.clone();

        let fn_name = if let Some(ref receiver) = func.receiver {
            let type_name = match &receiver.ty {
                Type::Named(name) => name.clone(),
                Type::Generic { name, .. } => name.clone(),
                Type::Reference(inner, _) => match &**inner {
                    Type::Named(name) => name.clone(),
                    Type::Generic { name, .. } => name.clone(),
                    Type::Vec(_) => "Vec".to_string(),
                    Type::Box(_) => "Box".to_string(),
                    Type::Option(_) => "Option".to_string(),
                    Type::Result(_, _) => "Result".to_string(),
                    _ => {
                        eprintln!("⚠️  Unsupported receiver type in compile: {:?}", inner);
                        return Err(format!(
                            "Receiver must be a named type or reference to named type, got {:?}",
                            inner
                        ));
                    }
                },
                Type::Vec(_) => "Vec".to_string(),
                Type::Box(_) => "Box".to_string(),
                Type::Option(_) => "Option".to_string(),
                Type::Result(_, _) => "Result".to_string(),
                _ => {
                    eprintln!(
                        "⚠️  Unsupported receiver type in compile: {:?}",
                        receiver.ty
                    );
                    return Err(format!(
                        "Receiver must be a named type or reference to named type, got {:?}",
                        receiver.ty
                    ));
                }
            };

            // Check if name is already mangled (imported methods)
            if func.name.starts_with(&format!("{}_", type_name)) {
                func.name.clone()
            } else {
                // CRITICAL: Encode operator symbols (op[], op[]=) for LLVM
                let encoded_method_name = Self::encode_operator_name(&func.name);
                let param_count = func.params.len();
                let base_name = format!("{}_{}", type_name, encoded_method_name);

                // Add type suffix for overloading (same logic as program.rs)
                let name = if !func.params.is_empty() {
                    let first_param_type = &func.params[0].ty;
                    let type_suffix = self.generate_type_suffix(first_param_type);

                    if func.name.starts_with("op") && !type_suffix.is_empty() {
                        format!("{}{}_{}", base_name, type_suffix, param_count)
                    } else if !type_suffix.is_empty() {
                        format!("{}{}_{}", base_name, type_suffix, param_count)
                    } else {
                        format!("{}_{}", base_name, param_count)
                    }
                } else {
                    base_name
                };
                name
            }
        } else {
            func.name.clone()
        };

        let fn_val = *self
            .functions
            .get(&fn_name)
            .ok_or_else(|| format!("Function {} not declared", fn_name))?;
        self.current_function = Some(fn_val);

        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        // ⭐ SPECIAL: main() runtime initialization
        if func.name == "main" {
            // Call __vex_runtime_init(argc, argv) first
            let argc_param = fn_val
                .get_nth_param(0)
                .ok_or("main() missing argc parameter")?
                .into_int_value();
            let argv_param = fn_val
                .get_nth_param(1)
                .ok_or("main() missing argv parameter")?
                .into_pointer_value();
            
            // Declare __vex_runtime_init
            let void_type = self.context.void_type();
            let i32_type = self.context.i32_type();
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let init_type = void_type.fn_type(&[i32_type.into(), ptr_type.into()], false);
            let init_fn = self.module.add_function("__vex_runtime_init", init_type, None);
            
            // Call it
            self.builder
                .build_call(init_fn, &[argc_param.into(), argv_param.into()], "")
                .map_err(|e| format!("Failed to call __vex_runtime_init: {}", e))?;
        }

        // ⭐ ASYNC: Initialize runtime at the start of main if needed
        if func.name == "main" && self.global_runtime.is_some() {
            eprintln!("🔄 Initializing runtime at start of main()");

            // Runtime* rt = runtime_create(4);
            let runtime_create = self.get_or_declare_runtime_create();
            let num_workers = self.context.i32_type().const_int(4, false);
            let runtime_call = self
                .builder
                .build_call(runtime_create, &[num_workers.into()], "runtime")
                .map_err(|e| format!("Failed to call runtime_create: {}", e))?;

            let runtime_ptr = runtime_call
                .try_as_basic_value()
                .unwrap_basic()
                .into_pointer_value();

            // Store in global: __vex_global_runtime = rt;
            let global_runtime = self
                .global_runtime
                .ok_or("global_runtime not initialized")?;
            self.builder
                .build_store(global_runtime, runtime_ptr)
                .map_err(|e| format!("Failed to store runtime: {}", e))?;

            eprintln!("✅ Runtime initialized and stored in global");
        }

        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();
        self.function_params.clear();
        self.function_param_types.clear();

        // ⭐ SPECIAL: Skip argc/argv parameters in main()
        let mut param_offset = if func.name == "main" { 2 } else { 0 };

        if let Some(ref receiver) = func.receiver {
            let param_val = fn_val
                .get_nth_param(0)
                .ok_or_else(|| "Receiver parameter not found".to_string())?;

            let param_type = self.ast_type_to_llvm(&receiver.ty);

            // CRITICAL: Handle reference receivers differently
            // Reference receivers (&Vec<T>, &Point, etc.) are already pointers - use directly
            // Value receivers (Vec<T>, Point, etc.) need allocation
            let receiver_var = match &receiver.ty {
                Type::Reference(_, _) => {
                    // Reference receiver: param_val is already a pointer, use it directly
                    eprintln!(
                        "📌 Receiver '{}': reference type, using pointer directly",
                        receiver.name
                    );
                    param_val.into_pointer_value()
                }
                _ => {
                    // Value receiver: allocate and store (Golang-style)
                    eprintln!(
                        "📌 Receiver '{}': value type, allocating and storing",
                        receiver.name
                    );
                    let alloca =
                        self.create_entry_block_alloca(&receiver.name, &receiver.ty, true)?;
                    self.builder
                        .build_store(alloca, param_val)
                        .map_err(|e| format!("Failed to store receiver: {}", e))?;
                    alloca
                }
            };

            self.variables.insert(receiver.name.clone(), receiver_var);
            self.variable_types
                .insert(receiver.name.clone(), param_type);

            // ⭐ CRITICAL: Store AST type for type inference
            self.variable_ast_types
                .insert(receiver.name.clone(), receiver.ty.clone());

            // Extract struct type name from receiver type
            // Handle: Type::Named, Type::Reference(Type::Named | Type::Vec | Type::Box | ...)
            let type_name = match &receiver.ty {
                Type::Named(name) => Some(name.clone()),
                Type::Reference(inner, _) => {
                    // Handle references to any struct type (Named, Vec, Box, etc.)
                    match &**inner {
                        Type::Named(name) => Some(name.clone()),
                        Type::Vec(_) | Type::Box(_) | Type::Option(_) | Type::Result(_, _) => {
                            // Use type_to_string to get mangled name: Vec<i32> -> Vec_i32
                            Some(self.type_to_string(inner))
                        }
                        Type::Generic { .. } => {
                            // Generic types get mangled: HashMap<K, V> -> HashMap_K_V
                            Some(self.type_to_string(inner))
                        }
                        _ => None,
                    }
                }
                Type::Vec(_) | Type::Box(_) | Type::Option(_) | Type::Result(_, _) => {
                    Some(self.type_to_string(&receiver.ty))
                }
                Type::Generic { .. } => Some(self.type_to_string(&receiver.ty)),
                _ => None,
            };

            eprintln!("📌 Receiver type_name extracted: {:?}", type_name);

            if let Some(struct_name) = type_name {
                if self.struct_defs.contains_key(&struct_name)
                    || self.struct_ast_defs.contains_key(&struct_name)
                {
                    eprintln!(
                        "   ✅ Tracking '{}' as struct: {}",
                        receiver.name, struct_name
                    );
                    self.variable_struct_names
                        .insert(receiver.name.clone(), struct_name);
                } else {
                    eprintln!("   ❌ Struct {} not found in defs", struct_name);
                }
            } else {
                eprintln!("   ❌ No type name extracted from receiver");
            }

            param_offset = 1;
        }

        for (i, param) in func.params.iter().enumerate() {
            let param_idx = crate::safe_param_index(i, param_offset)
                .map_err(|e| format!("Parameter index overflow for {}: {}", param.name, e))?;
            let param_val = fn_val
                .get_nth_param(param_idx)
                .ok_or_else(|| format!("Parameter {} not found", param.name))?;
            let param_type = self.ast_type_to_llvm(&param.ty);

            if matches!(param.ty, Type::Function { .. }) {
                if let BasicValueEnum::PointerValue(fn_ptr) = param_val {
                    self.function_params.insert(param.name.clone(), fn_ptr);
                    self.function_param_types
                        .insert(param.name.clone(), param.ty.clone());
                } else {
                    return Err(format!(
                        "Function parameter {} is not a pointer",
                        param.name
                    ));
                }
            } else {
                // ⚠️ CRITICAL: For external methods, struct parameters are passed BY VALUE
                // For all parameters, we ALWAYS allocate and store to maintain consistent access patterns
                let is_struct_param = match &param.ty {
                    Type::Named(type_name) => self.struct_defs.contains_key(type_name),
                    _ => false,
                };

                eprintln!(
                    "📌 Parameter '{}': type={:?}, is_struct={}, is_pointer={}",
                    param.name,
                    param.ty,
                    is_struct_param,
                    param_val.is_pointer_value()
                );

                // ⭐ CRITICAL: Struct parameters are passed BY VALUE in function signature
                // But we still allocate+store for consistent variable access
                // This mirrors the behavior of non-struct parameters
                eprintln!("   → Allocating and storing");
                let alloca = self.create_entry_block_alloca(&param.name, &param.ty, true)?;
                self.builder
                    .build_store(alloca, param_val)
                    .map_err(|e| format!("Failed to store parameter: {}", e))?;
                self.variables.insert(param.name.clone(), alloca);
                self.variable_types.insert(param.name.clone(), param_type);

                // ⭐ CRITICAL: Store AST type for type inference (print, format, etc.)
                self.variable_ast_types
                    .insert(param.name.clone(), param.ty.clone());

                let extract_struct_name = |ty: &Type| -> Option<String> {
                    match ty {
                        Type::Named(name) => Some(name.clone()),
                        Type::Reference(inner, _) => {
                            if let Type::Named(name) = &**inner {
                                Some(name.clone())
                            } else {
                                None
                            }
                        }
                        Type::Generic { name, .. } => Some(name.clone()),
                        _ => None,
                    }
                };

                if let Some(struct_name) = extract_struct_name(&param.ty) {
                    if self.struct_defs.contains_key(&struct_name)
                        || self.struct_ast_defs.contains_key(&struct_name)
                    {
                        self.variable_struct_names
                            .insert(param.name.clone(), struct_name.clone());
                    }
                }

                match &param.ty {
                    Type::Generic { name, type_args } => {
                        if let Ok(mangled_name) = self.instantiate_generic_struct(name, type_args) {
                            self.variable_struct_names
                                .insert(param.name.clone(), mangled_name);
                        }
                    }
                    _ => {}
                }
            }
        }

        self.push_scope();
        eprintln!(
            "📋 About to compile function body with {} statements",
            func.body.statements.len()
        );
        self.compile_block(&func.body)?;
        eprintln!("📋 Finished compiling function body");

        if let Some(current_block) = self.builder.get_insert_block() {
            if current_block.get_terminator().is_none() {
                self.pop_scope()?;
                self.execute_deferred_statements()?;
            }
        }

        self.clear_deferred_statements();

        if let Some(current_block) = self.builder.get_insert_block() {
            if current_block.get_terminator().is_none() {
                // ⭐ ASYNC: If this is main() with runtime, add runtime_run() and runtime_destroy()
                if func.name == "main" && self.global_runtime.is_some() {
                    eprintln!("🔄 Adding runtime_run() and runtime_destroy() to main()");

                    let runtime_ptr = self
                        .global_runtime
                        .ok_or("global_runtime not initialized")?;

                    // void runtime_run(Runtime* runtime);
                    let runtime_run = self.get_or_declare_runtime_run();
                    self.builder
                        .build_call(runtime_run, &[runtime_ptr.into()], "run_runtime")
                        .map_err(|e| format!("Failed to call runtime_run: {}", e))?;

                    // void runtime_destroy(Runtime* runtime);
                    let runtime_destroy = self.get_or_declare_runtime_destroy();
                    self.builder
                        .build_call(runtime_destroy, &[runtime_ptr.into()], "destroy_runtime")
                        .map_err(|e| format!("Failed to call runtime_destroy: {}", e))?;
                }

                // ✅ FIX: Void functions should always get implicit return, not unreachable
                let is_void_function = func.return_type.is_none()
                    || matches!(func.return_type.as_ref(), Some(Type::Nil));

                eprintln!("📍 Function {} implicit terminator: is_void={}, current_block={:?}", func.name, is_void_function, current_block.get_name().to_str());

                if is_void_function {
                    // Void/nil function - add implicit return
                    eprintln!("   → Adding void return");
                    self.builder
                        .build_return(None)
                        .map_err(|e| format!("Failed to build void return: {}", e))?;
                } else {
                    // Non-void function - check if entry block
                    let is_entry_block = current_block
                        == fn_val
                            .get_first_basic_block()
                            .ok_or("Function missing entry block")?;

                    if is_entry_block {
                        // Get the actual return type to generate correct default value
                        let ret_type = self.ast_type_to_llvm(
                            func.return_type
                                .as_ref()
                                .ok_or("Function missing return type")?,
                        );
                        match ret_type {
                            BasicTypeEnum::IntType(int_ty) => {
                                let zero = int_ty.const_int(0, false);
                                self.builder
                                    .build_return(Some(&zero))
                                    .map_err(|e| format!("Failed to build return: {}", e))?;
                            }
                            _ => {
                                return Err(
                                    "Non-void function must have explicit return".to_string()
                                );
                            }
                        }
                    } else {
                        // Non-entry block without terminator in non-void function = unreachable
                        self.builder
                            .build_unreachable()
                            .map_err(|e| format!("Failed to build unreachable: {}", e))?;
                    }
                }
            }
        }

        let result = Ok(());
        self.current_function_return_type = previous_return_type;
        result
    }
}
