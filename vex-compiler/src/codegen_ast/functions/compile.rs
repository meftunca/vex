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

        if func.is_async {
            return self.compile_async_function(func);
        }

        let fn_name = if let Some(ref receiver) = func.receiver {
            let type_name = match &receiver.ty {
                Type::Named(name) => name.clone(),
                Type::Generic { name, .. } => name.clone(), // Generic types like Container<T>
                Type::Reference(inner, _) => match &**inner {
                    Type::Named(name) => name.clone(),
                    Type::Generic { name, .. } => name.clone(),
                    _ => {
                        return Err(
                            "Receiver must be a named type or reference to named type".to_string()
                        );
                    }
                },
                _ => {
                    return Err(
                        "Receiver must be a named type or reference to named type".to_string()
                    )
                }
            };
            
            // Check if name is already mangled (imported methods)
            if func.name.starts_with(&format!("{}_", type_name)) {
                func.name.clone()
            } else {
                // ⭐ Type-based method overloading: Include first parameter type in mangling
                let param_count = func.params.len();
                let base_name = format!("{}_{}", type_name, func.name);
                
                // Add type suffix for overloading (same logic as program.rs registration)
                let name = if !func.params.is_empty() {
                    let first_param_type = &func.params[0].ty;
                    let type_suffix = self.generate_type_suffix(first_param_type);
                    
                    // Add type suffix for operators
                    if func.name.starts_with("op") && !type_suffix.is_empty() {
                        format!("{}{}_{}", base_name, type_suffix, param_count)
                    } else if !type_suffix.is_empty() {
                        // For non-operators, add suffix only if not empty
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

        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();
        self.function_params.clear();
        self.function_param_types.clear();

        let mut param_offset = 0;

        if let Some(ref receiver) = func.receiver {
            let param_val = fn_val
                .get_nth_param(0)
                .ok_or_else(|| "Receiver parameter not found".to_string())?;

            // CRITICAL: External methods use Golang-style `fn (p: Point)` (pass by value)
            // Inline methods use `fn op+(self)` (also pass by value in new convention)
            // Both should ALWAYS allocate and store for consistent access
            let param_type = self.ast_type_to_llvm(&receiver.ty);
            let alloca = self.create_entry_block_alloca(&receiver.name, &receiver.ty, true)?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store receiver: {}", e))?;
            self.variables.insert(receiver.name.clone(), alloca);
            self.variable_types.insert(receiver.name.clone(), param_type);

            eprintln!("📌 Receiver '{}': allocated and stored", receiver.name);

            let type_name = match &receiver.ty {
                Type::Named(name) => Some(name.clone()),
                Type::Reference(inner, _) => {
                    if let Type::Named(name) = &**inner {
                        Some(name.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            eprintln!("📌 Receiver type_name extracted: {:?}", type_name);

            if let Some(struct_name) = type_name {
                if self.struct_defs.contains_key(&struct_name)
                    || self.struct_ast_defs.contains_key(&struct_name)
                {
                    eprintln!("   ✅ Tracking '{}' as struct: {}", receiver.name, struct_name);
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
            let param_val = fn_val
                .get_nth_param((i + param_offset) as u32)
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

                // ALWAYS allocate and store - regardless of struct or not
                // This ensures consistent behavior between external and inline methods
                eprintln!("   → Allocating and storing");
                let alloca = self.create_entry_block_alloca(&param.name, &param.ty, true)?;
                self.builder
                    .build_store(alloca, param_val)
                    .map_err(|e| format!("Failed to store parameter: {}", e))?;
                self.variables.insert(param.name.clone(), alloca);
                self.variable_types.insert(param.name.clone(), param_type);

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
        eprintln!("📋 About to compile function body with {} statements", func.body.statements.len());
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
                // Simple approach: if it's not the entry block, it's unreachable
                let is_entry_block = current_block == fn_val.get_first_basic_block().unwrap();

                if is_entry_block {
                    if func.return_type.is_none() {
                        let zero = self.context.i32_type().const_int(0, false);
                        self.builder
                            .build_return(Some(&zero))
                            .map_err(|e| format!("Failed to build return: {}", e))?;
                    } else {
                        return Err("Non-void function must have explicit return".to_string());
                    }
                } else {
                    // Non-entry block without terminator = unreachable
                    self.builder
                        .build_unreachable()
                        .map_err(|e| format!("Failed to build unreachable: {}", e))?;
                }
            }
        }

        Ok(())
    }
}
