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
                format!("{}_{}", type_name, func.name)
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

            // CRITICAL FIX: For external (Golang-style) methods with reference receivers,
            // the parameter is already a pointer, so we DON'T need alloca+store
            let is_reference_receiver = matches!(receiver.ty, Type::Reference(_, _));

            if is_reference_receiver {
                // External method: fn (self: &Type!) - receiver is already a pointer
                // Use it directly, no alloca needed
                let receiver_llvm_ty = match &receiver.ty {
                    Type::Reference(inner, _) => {
                        // For &Type, the LLVM parameter is already a pointer
                        self.ast_type_to_llvm(inner)
                    }
                    _ => unreachable!(),
                };

                let self_ptr = param_val.into_pointer_value();
                self.variables.insert("self".to_string(), self_ptr);
                self.variable_types
                    .insert("self".to_string(), receiver_llvm_ty);

                eprintln!("📌 External method receiver: using pointer directly (no alloca)");
            } else {
                // Inline method or non-reference receiver: allocate and store
                let param_type = self.ast_type_to_llvm(&receiver.ty);
                let alloca = self.create_entry_block_alloca("self", &receiver.ty, true)?;
                self.builder
                    .build_store(alloca, param_val)
                    .map_err(|e| format!("Failed to store receiver: {}", e))?;
                self.variables.insert("self".to_string(), alloca);
                self.variable_types.insert("self".to_string(), param_type);

                eprintln!("📌 Inline method receiver: allocated and stored");
            }

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
                    eprintln!("   ✅ Tracking 'self' as struct: {}", struct_name);
                    self.variable_struct_names
                        .insert("self".to_string(), struct_name);
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
                // ⚠️ CRITICAL: Struct parameters are passed as POINTERS (not values)
                // So if the parameter type is a struct, param_val is already a pointer.
                // We should NOT allocate+store, just use the pointer directly!
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

                if is_struct_param && param_val.is_pointer_value() {
                    // Struct parameter - use the pointer directly, don't allocate
                    eprintln!("   → Using pointer directly (no alloca)");
                    let ptr_val = param_val.into_pointer_value();
                    self.variables.insert(param.name.clone(), ptr_val);
                    self.variable_types.insert(param.name.clone(), param_type);
                } else {
                    // Non-struct parameter - allocate and store as usual
                    eprintln!("   → Allocating and storing");
                    let alloca = self.create_entry_block_alloca(&param.name, &param.ty, true)?;
                    self.builder
                        .build_store(alloca, param_val)
                        .map_err(|e| format!("Failed to store parameter: {}", e))?;
                    self.variables.insert(param.name.clone(), alloca);
                    self.variable_types.insert(param.name.clone(), param_type);
                }

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
        self.compile_block(&func.body)?;

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
