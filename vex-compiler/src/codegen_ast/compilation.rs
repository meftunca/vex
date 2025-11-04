// Function body compilation: compile function bodies with statements

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a function with its body
    pub(crate) fn compile_function(&mut self, func: &Function) -> Result<(), String> {
        // Special handling for async functions
        if func.is_async {
            return self.compile_async_function(func);
        }

        // Determine the function name (same mangling as declare_function)
        let fn_name = if let Some(ref receiver) = func.receiver {
            let type_name = match &receiver.ty {
                Type::Named(name) => name.clone(),
                Type::Reference(inner, _) => {
                    if let Type::Named(name) = &**inner {
                        name.clone()
                    } else {
                        return Err(
                            "Receiver must be a named type or reference to named type".to_string()
                        );
                    }
                }
                _ => {
                    return Err(
                        "Receiver must be a named type or reference to named type".to_string()
                    );
                }
            };
            format!("{}_{}", type_name, func.name)
        } else {
            func.name.clone()
        };

        let fn_val = *self
            .functions
            .get(&fn_name)
            .ok_or_else(|| format!("Function {} not declared", fn_name))?;

        self.current_function = Some(fn_val);

        // Create entry block
        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        // Clear local variables for new function
        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();
        self.function_params.clear();
        self.function_param_types.clear();

        let mut param_offset = 0;

        // If there's a receiver, allocate it as the first parameter (named "self")
        if let Some(ref receiver) = func.receiver {
            let param_val = fn_val
                .get_nth_param(0)
                .ok_or_else(|| "Receiver parameter not found".to_string())?;

            let param_type = self.ast_type_to_llvm(&receiver.ty);
            // v0.9: Function receivers are always mutable (local binding)
            let alloca = self.create_entry_block_alloca("self", &receiver.ty, true)?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store receiver: {}", e))?;
            self.variables.insert("self".to_string(), alloca);
            self.variable_types.insert("self".to_string(), param_type);

            // Track struct receiver - extract type name
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

            if let Some(struct_name) = type_name {
                if self.struct_defs.contains_key(&struct_name) {
                    self.variable_struct_names
                        .insert("self".to_string(), struct_name);
                }
            }

            param_offset = 1;
        }

        // Allocate space for regular parameters and store them
        for (i, param) in func.params.iter().enumerate() {
            let param_val = fn_val
                .get_nth_param((i + param_offset) as u32)
                .ok_or_else(|| format!("Parameter {} not found", param.name))?;

            let param_type = self.ast_type_to_llvm(&param.ty);

            // Special handling for function type parameters
            // Function types are passed as pointers and don't need alloca
            if matches!(param.ty, Type::Function { .. }) {
                // Store function parameter directly as a pointer value - no alloca
                if let inkwell::values::BasicValueEnum::PointerValue(fn_ptr) = param_val {
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
                // v0.9: Function parameters are always mutable (local binding)
                let alloca = self.create_entry_block_alloca(&param.name, &param.ty, true)?;
                self.builder
                    .build_store(alloca, param_val)
                    .map_err(|e| format!("Failed to store parameter: {}", e))?;
                self.variables.insert(param.name.clone(), alloca);
                self.variable_types.insert(param.name.clone(), param_type);

                // Track struct parameters
                match &param.ty {
                    Type::Named(struct_name) => {
                        if self.struct_defs.contains_key(struct_name) {
                            self.variable_struct_names
                                .insert(param.name.clone(), struct_name.clone());
                        }
                    }
                    Type::Generic { name, type_args } => {
                        // Generic struct parameter: Pair<i32, i32>
                        // Instantiate to get mangled name: Pair_i32_i32
                        if let Ok(mangled_name) = self.instantiate_generic_struct(name, type_args) {
                            self.variable_struct_names
                                .insert(param.name.clone(), mangled_name);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Compile function body
        self.compile_block(&func.body)?;

        // Execute deferred statements before function exit
        // (explicit returns already handle this in compile_statement)
        if let Some(current_block) = self.builder.get_insert_block() {
            if current_block.get_terminator().is_none() {
                // Only execute defers if block is not already terminated
                self.execute_deferred_statements()?;
            }
        }

        // Clear deferred statements for next function
        self.clear_deferred_statements();

        // If no return statement, add default return
        if let Some(current_block) = self.builder.get_insert_block() {
            if current_block.get_terminator().is_none() {
                // Check if block is reachable (has predecessors or is entry block)
                let is_reachable = current_block.get_first_use().is_some()
                    || current_block == fn_val.get_first_basic_block().unwrap();

                if is_reachable {
                    if func.return_type.is_none() {
                        // void function
                        let zero = self.context.i32_type().const_int(0, false);
                        self.builder
                            .build_return(Some(&zero))
                            .map_err(|e| format!("Failed to build return: {}", e))?;
                    } else {
                        return Err("Non-void function must have explicit return".to_string());
                    }
                } else {
                    // Block is unreachable, add unreachable instruction
                    self.builder
                        .build_unreachable()
                        .map_err(|e| format!("Failed to build unreachable: {}", e))?;
                }
            }
        }

        Ok(())
    }

    /// Compile a trait impl method body
    pub(crate) fn compile_trait_impl_method(
        &mut self,
        trait_name: &str,
        for_type: &Type,
        method: &Function,
    ) -> Result<(), String> {
        let type_name = match for_type {
            Type::Named(name) => name,
            _ => return Err(format!("Expected named type, got: {:?}", for_type)),
        };

        // Mangle name to match declaration
        let mangled_name = format!("{}_{}_{}", type_name, trait_name, method.name);

        // For trait impl methods, we've already mangled the name and declared the function
        // So we need to compile_function WITHOUT the receiver to avoid double-mangling
        // But we DO need the receiver in the body compilation for self parameter allocation

        // Get the function we declared
        let fn_val = *self
            .functions
            .get(&mangled_name)
            .ok_or_else(|| format!("Trait impl method {} not found", mangled_name))?;

        self.current_function = Some(fn_val);

        // Create entry block
        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        // Clear local variables for new function
        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();

        let mut param_offset = 0;

        // Allocate receiver as first parameter
        if let Some(ref receiver) = method.receiver {
            let param_val = fn_val
                .get_nth_param(0)
                .ok_or("Missing receiver parameter")?;
            let receiver_ty = self.ast_type_to_llvm(&receiver.ty);

            let alloca = self
                .builder
                .build_alloca(receiver_ty, "self")
                .map_err(|e| format!("Failed to create self alloca: {}", e))?;

            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store self: {}", e))?;

            self.variables.insert("self".to_string(), alloca);
            self.variable_types.insert("self".to_string(), receiver_ty);

            // Track struct type for self
            if let Type::Reference(inner, _) = &receiver.ty {
                if let Type::Named(struct_name) = &**inner {
                    self.variable_struct_names
                        .insert("self".to_string(), struct_name.clone());
                }
            }

            param_offset = 1;
        }

        // Allocate parameters
        for (i, param) in method.params.iter().enumerate() {
            let param_val = fn_val
                .get_nth_param((i + param_offset) as u32)
                .ok_or_else(|| format!("Missing parameter {}", param.name))?;

            let param_ty = self.ast_type_to_llvm(&param.ty);
            let alloca = self
                .builder
                .build_alloca(param_ty, &param.name)
                .map_err(|e| format!("Failed to create parameter alloca: {}", e))?;

            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store parameter: {}", e))?;

            self.variables.insert(param.name.clone(), alloca);
            self.variable_types.insert(param.name.clone(), param_ty);

            // Track struct parameters (handles both Named and Generic types)
            self.track_param_struct_name(&param.name, &param.ty);
        }

        // Compile function body
        for stmt in &method.body.statements {
            self.compile_statement(stmt)?;
        }

        // Ensure function returns
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            if method.return_type.is_some() {
                return Err(format!("Function {} must return a value", mangled_name));
            } else {
                // Return default i32 value
                let ret_val = self.context.i32_type().const_int(0, false);
                self.builder
                    .build_return(Some(&ret_val))
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            }
        }

        Ok(())
    }

    /// Compile an inline struct method body
    pub(crate) fn compile_struct_method(
        &mut self,
        struct_name: &str,
        method: &Function,
    ) -> Result<(), String> {
        // Mangle name to match declaration
        let mangled_name = format!("{}_{}", struct_name, method.name);

        // Get the function we declared
        let fn_val = *self
            .functions
            .get(&mangled_name)
            .ok_or_else(|| format!("Struct method {} not found", mangled_name))?;

        self.current_function = Some(fn_val);

        // Create entry block
        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        // Clear local variables for new function
        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();

        let mut param_offset = 0;

        // Allocate receiver as first parameter
        if let Some(ref receiver) = method.receiver {
            let param_val = fn_val
                .get_nth_param(0)
                .ok_or("Missing receiver parameter")?;
            let receiver_ty = self.ast_type_to_llvm(&receiver.ty);

            let alloca = self
                .builder
                .build_alloca(receiver_ty, "self")
                .map_err(|e| format!("Failed to create receiver alloca: {}", e))?;

            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store receiver: {}", e))?;

            self.variables.insert("self".to_string(), alloca);
            self.variable_types.insert("self".to_string(), receiver_ty);

            // Track struct name for method calls on self
            if let Type::Reference(inner, _) = &receiver.ty {
                if let Type::Named(struct_name) = &**inner {
                    self.variable_struct_names
                        .insert("self".to_string(), struct_name.clone());
                }
            }

            param_offset = 1;
        }

        // Allocate parameters
        for (i, param) in method.params.iter().enumerate() {
            let param_val = fn_val
                .get_nth_param((i + param_offset) as u32)
                .ok_or_else(|| format!("Missing parameter {}", param.name))?;

            let param_ty = self.ast_type_to_llvm(&param.ty);
            let alloca = self
                .builder
                .build_alloca(param_ty, &param.name)
                .map_err(|e| format!("Failed to create parameter alloca: {}", e))?;

            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store parameter: {}", e))?;

            self.variables.insert(param.name.clone(), alloca);
            self.variable_types.insert(param.name.clone(), param_ty);

            // Track struct parameters (handles both Named and Generic types)
            self.track_param_struct_name(&param.name, &param.ty);
        }

        // Compile function body
        for stmt in &method.body.statements {
            self.compile_statement(stmt)?;
        }

        // Ensure function returns
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            if method.return_type.is_some() {
                return Err(format!("Function {} must return a value", mangled_name));
            } else {
                // Return default i32 value
                let ret_val = self.context.i32_type().const_int(0, false);
                self.builder
                    .build_return(Some(&ret_val))
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            }
        }

        Ok(())
    }

    /// Compile async function (placeholder for future async runtime)
    pub(crate) fn compile_async_function(&mut self, func: &Function) -> Result<(), String> {
        let fn_name = &func.name;

        let fn_val = *self
            .functions
            .get(fn_name)
            .ok_or_else(|| format!("Async function {} not declared", fn_name))?;

        self.current_function = Some(fn_val);

        // Create entry block
        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        // Clear local variables
        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();

        // TODO: Initialize async task state
        // For now, just compile body normally

        // Allocate parameters
        for (i, param) in func.params.iter().enumerate() {
            let param_val = fn_val
                .get_nth_param(i as u32)
                .ok_or_else(|| format!("Could not get parameter {} for function {}", i, fn_name))?;

            let param_type = self.ast_type_to_llvm(&param.ty);
            let ptr = self
                .builder
                .build_alloca(param_type, &param.name)
                .map_err(|e| format!("Failed to allocate parameter: {}", e))?;

            self.builder
                .build_store(ptr, param_val)
                .map_err(|e| format!("Failed to store parameter: {}", e))?;

            self.variables.insert(param.name.clone(), ptr);
            self.variable_types.insert(param.name.clone(), param_type);
        }

        // Compile function body
        self.compile_block(&func.body)?;

        // If no explicit return, add default return
        let current_block = self.builder.get_insert_block().unwrap();
        if current_block.get_terminator().is_none() {
            if let Some(ret_ty) = &func.return_type {
                let default_val = self.get_default_value(&self.ast_type_to_llvm(ret_ty));
                self.builder
                    .build_return(Some(&default_val))
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            } else {
                self.builder
                    .build_return(None)
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            }
        }

        Ok(())
    }
}


