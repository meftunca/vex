// src/codegen/methods.rs
use super::*;
use inkwell::types::BasicTypeEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Replace `Self` types with concrete struct name in local context
    pub(crate) fn replace_self_in_type(ty: &vex_ast::Type, concrete_type: &str) -> vex_ast::Type {
        match ty {
            vex_ast::Type::Named(name) if name == "Self" => {
                vex_ast::Type::Named(concrete_type.to_string())
            }
            vex_ast::Type::SelfType => {
                vex_ast::Type::Named(concrete_type.to_string())
            }
            vex_ast::Type::Reference(inner, is_mut) => vex_ast::Type::Reference(
                Box::new(Self::replace_self_in_type(inner, concrete_type)),
                *is_mut,
            ),
            vex_ast::Type::Generic { name, type_args } => {
                let new_name = if name == "Self" {
                    concrete_type.to_string()
                } else {
                    name.clone()
                };
                vex_ast::Type::Generic {
                    name: new_name,
                    type_args: type_args
                        .iter()
                        .map(|t| Self::replace_self_in_type(t, concrete_type))
                        .collect(),
                }
            }
            vex_ast::Type::Array(inner, size) => vex_ast::Type::Array(
                Box::new(Self::replace_self_in_type(inner, concrete_type)),
                *size,
            ),
            vex_ast::Type::Slice(inner, is_mut) => vex_ast::Type::Slice(
                Box::new(Self::replace_self_in_type(inner, concrete_type)),
                *is_mut,
            ),
            vex_ast::Type::Union(types) => vex_ast::Type::Union(
                types
                    .iter()
                    .map(|t| Self::replace_self_in_type(t, concrete_type))
                    .collect(),
            ),
            vex_ast::Type::Intersection(types) => vex_ast::Type::Intersection(
                types
                    .iter()
                    .map(|t| Self::replace_self_in_type(t, concrete_type))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }
    /// Encode operator names for LLVM compatibility
    /// LLVM doesn't allow special characters like +, -, *, / in function names
    pub(crate) fn encode_operator_name(name: &str) -> String {
        match name {
            "op+" => "opadd".to_string(),
            "op-" => "opsub".to_string(),
            "op*" => "opmul".to_string(),
            "op/" => "opdiv".to_string(),
            "op%" => "opmod".to_string(),
            "op**" => "oppow".to_string(),
            "op==" => "opeq".to_string(),
            "op!=" => "opne".to_string(),
            "op<" => "oplt".to_string(),
            "op<=" => "ople".to_string(),
            "op>" => "opgt".to_string(),
            "op>=" => "opge".to_string(),
            "op&" => "opbitand".to_string(),
            "op|" => "opbitor".to_string(),
            "op^" => "opbitxor".to_string(),
            "op<<" => "opshl".to_string(),
            "op>>" => "opshr".to_string(),
            "op!" => "opnot".to_string(),
            "op~" => "opbitnot".to_string(),
            "op++" => "opinc".to_string(),
            "op--" => "opdec".to_string(),
            "op[]" => "opindex".to_string(),
            "op[]=" => "opindexset".to_string(),
            _ => name.to_string(),
        }
    }

    pub(crate) fn declare_struct_method(
        &mut self,
        struct_name: &str,
        method: &Function,
    ) -> Result<(), String> {
        // ⭐ CRITICAL: Encode operator symbols for LLVM compatibility
        // LLVM doesn't allow +, *, -, etc. in function names
        let method_name_encoded = Self::encode_operator_name(&method.name);

        // ⭐ NEW: Enhanced mangling for generic trait implementations
        // For inline methods, param_count includes implicit receiver
        let param_count = method.params.len() + 1; // +1 for implicit receiver
        let base_name = format!("{}_{}", struct_name, method_name_encoded);

        // Try to find trait + type args for this method by matching parameter types
        let mangled_name = if method.name.starts_with("op") && !method.params.is_empty() {
            if let Some(struct_def) = self.struct_ast_defs.get(struct_name) {
                // For operator methods, check the first parameter type against trait impl type args
                let first_param_ty = &method.params[0].ty;

                // Find matching trait impl
                let mut found_match = None;
                for trait_impl in &struct_def.impl_traits {
                    // ⭐ CRITICAL: Check if this trait defines this operator method!
                    // Get trait definition and check if it has this method
                    let trait_has_method =
                        if let Some(trait_def) = self.trait_defs.get(&trait_impl.name) {
                            trait_def.methods.iter().any(|m| m.name == method.name)
                        } else {
                            false // Trait not found, assume no
                        };

                    if !trait_has_method {
                        // This trait doesn't define this method, skip it
                        continue;
                    }

                    // Handle both explicit type args (Add<i32>) and default (Add uses Self)
                    let matches = if !trait_impl.type_args.is_empty() {
                        // Explicit type arg - check if first param matches
                        first_param_ty == &trait_impl.type_args[0]
                    } else {
                        // Default type arg (Self) - always generate suffix for overload resolution
                        true
                    };

                    if matches {
                        // Mangle: StructName_method_TypeArg_paramCount
                        let type_suffix = self.generate_type_suffix(first_param_ty);
                        found_match = Some(format!(
                            "{}{}",
                            base_name, type_suffix
                        ));
                        break;
                    }
                }

                if let Some(name) = found_match {
                    format!("{}_{}", name, param_count)
                } else if method.name == "op-" || method.name == "op+" || method.name == "op*" {
                    format!("{}_{}", base_name, param_count)
                } else {
                    base_name
                }
            } else if method.name == "op-" || method.name == "op+" || method.name == "op*" {
                format!("{}_{}", base_name, param_count)
            } else {
                base_name
            }
        } else if method.name.starts_with("op")
            && (method.name == "op-" || method.name == "op+" || method.name == "op*")
        {
            format!("{}_{}", base_name, param_count)
        } else {
            base_name
        };

        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();

        // Add receiver parameter (explicit or implicit)
        if let Some(ref receiver) = method.receiver {
            // Explicit receiver: fn (self: &T) method()
            // Replace `Self` with the concrete struct name for LLVM type generation
            let receiver_ty_concrete = Self::replace_self_in_type(&receiver.ty, struct_name);
            param_types.push(self.ast_type_to_llvm(&receiver_ty_concrete).into());
        } else {
            // Implicit receiver: fn method() - auto-generate &StructName! parameter (mutable by default)
            // This allows both read and write access to struct fields
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            param_types.push(ptr_type.into());
        }

        for param in &method.params {
            let param_concrete_ty = Self::replace_self_in_type(&param.ty, struct_name);
            let param_llvm_ty = self.ast_type_to_llvm(&param_concrete_ty);

            param_types.push(param_llvm_ty.into());
        }

        // Determine return type: Nil → void, None → i32, Some(ty) → convert ty
        let fn_type = if let Some(ref ty) = method.return_type {
            // Substitute Self -> Named(struct_name) for inline methods
            let concrete_ret = Self::replace_self_in_type(ty, struct_name);

            if matches!(concrete_ret, Type::Nil) {
                // Nil return type → void
                self.context.void_type().fn_type(&param_types, false)
            } else {
                // Regular return type
                let ret_type = self.ast_type_to_llvm(&concrete_ret);
                match ret_type {
                    BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_types, false),
                    _ => {
                        return Err(format!(
                            "Unsupported return type for method {}",
                            method.name
                        ))
                    }
                }
            }
        } else {
            // No return type → void (not i32 anymore)
            self.context.void_type().fn_type(&param_types, false)
        };

        let fn_val = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name.clone(), fn_val);

        let mut mangled_method = method.clone();
        mangled_method.name = mangled_name.clone();
        // Also update AST method signature in the mangled copy: replace `Self` with actual struct name
        if let Some(ref mut receiver) = mangled_method.receiver {
            receiver.ty = Self::replace_self_in_type(&receiver.ty, struct_name);
        }

        for param in &mut mangled_method.params {
            param.ty = Self::replace_self_in_type(&param.ty, struct_name);
        }

        if let Some(ref mut rt) = mangled_method.return_type {
            *rt = Self::replace_self_in_type(rt, struct_name);
        }
        self.function_defs.insert(mangled_name, mangled_method);
        Ok(())
    }

    pub(crate) fn compile_struct_method(
        &mut self,
        struct_name: &str,
        method: &Function,
    ) -> Result<(), String> {
        // ⭐ CRITICAL: Encode operator symbols for LLVM compatibility
        let method_name_encoded = Self::encode_operator_name(&method.name);

        // ⭐ NEW: Enhanced mangling for generic trait implementations (same as declare_struct_method)
        // For inline methods, param_count includes implicit receiver
        let param_count = method.params.len() + 1; // +1 for implicit receiver
        let base_name = format!("{}_{}", struct_name, method_name_encoded);

        let mangled_name = if method.name.starts_with("op") && !method.params.is_empty() {
            if let Some(struct_def) = self.struct_ast_defs.get(struct_name) {
                let first_param_ty = &method.params[0].ty;

                let mut found_match = None;
                for trait_impl in &struct_def.impl_traits {
                    // ⭐ CRITICAL: Check if this trait defines this operator method!
                    let trait_has_method =
                        if let Some(trait_def) = self.trait_defs.get(&trait_impl.name) {
                            trait_def.methods.iter().any(|m| m.name == method.name)
                        } else {
                            false
                        };

                    if !trait_has_method {
                        continue;
                    }

                    // Handle both explicit type args (Add<i32>) and default (Add uses Self)
                    let matches = if !trait_impl.type_args.is_empty() {
                        // Explicit type arg - check if first param matches
                        first_param_ty == &trait_impl.type_args[0]
                    } else {
                        // Default type arg (Self) - always generate suffix for overload resolution
                        true
                    };

                    if matches {
                        let type_suffix = self.generate_type_suffix(first_param_ty);
                        eprintln!("  🔍 compile_struct_method found match, type_suffix={}", type_suffix);
                        found_match = Some(format!(
                            "{}{}",
                            base_name, type_suffix
                        ));
                        break;
                    }
                }

                if let Some(name) = found_match {
                    format!("{}_{}", name, param_count)
                } else if method.name == "op-" || method.name == "op+" || method.name == "op*" {
                    format!("{}_{}", base_name, param_count)
                } else {
                    base_name
                }
            } else if method.name == "op-" || method.name == "op+" || method.name == "op*" {
                format!("{}_{}", base_name, param_count)
            } else {
                base_name
            }
        } else if method.name.starts_with("op")
            && (method.name == "op-" || method.name == "op+" || method.name == "op*")
        {
            format!("{}_{}", base_name, param_count)
        } else {
            base_name
        };

        let fn_val = *self
            .functions
            .get(&mangled_name)
            .ok_or_else(|| format!("Struct method {} not found", mangled_name))?;

        self.current_function = Some(fn_val);

        // ⭐ NEW: Set method mutability context
        self.current_method_is_mutable = method.is_mutable;

        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();
        self.variable_ast_types.clear();

        let param_offset;

        // Handle both explicit receiver (golang-style) and implicit receiver (simplified syntax)
        if let Some(ref receiver) = method.receiver {
            // Explicit receiver: fn (self: &T) method()
            // Receiver is already a POINTER (&T or &T!), so we DON'T need alloca
            let param_val = fn_val
                .get_nth_param(0)
                .ok_or("Missing receiver parameter")?;

            // CRITICAL FIX: For reference types, we need to get the inner type for LLVM
            // because the parameter is already a pointer (C calling convention)
            let receiver_concrete_ty = Self::replace_self_in_type(&receiver.ty, struct_name);
            let receiver_llvm_ty = match &receiver_concrete_ty {
                Type::Reference(inner, _) => {
                    // For &Type, the LLVM parameter is already a pointer
                    // so we need the inner type's LLVM representation
                    self.ast_type_to_llvm(inner)
                }
                other => {
                    // For non-reference types, use as-is
                    self.ast_type_to_llvm(other)
                }
            };

            // Store receiver DIRECTLY (it's already a pointer, no need for alloca+store)
            let self_ptr = param_val.into_pointer_value();
            self.variables.insert("self".to_string(), self_ptr);
            self.variable_types
                .insert("self".to_string(), receiver_llvm_ty);

            // ⭐ CRITICAL: Store AST type for type inference
            self.variable_ast_types
                .insert("self".to_string(), receiver_concrete_ty.clone());

            let struct_name_opt = match &receiver_concrete_ty {
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

            if let Some(name) = struct_name_opt {
                self.variable_struct_names.insert("self".to_string(), name);
            } else {
                eprintln!("   ❌ No struct name extracted from receiver type!");
            }

            param_offset = 1;
        } else {
            // Implicit receiver: fn method() - auto-create immutable reference receiver

            let param_val = fn_val
                .get_nth_param(0)
                .ok_or("Missing implicit receiver parameter")?;

            // Create pointer type for receiver (it's already a pointer parameter)
            let receiver_ty = self.context.ptr_type(inkwell::AddressSpace::default());

            // Store receiver DIRECTLY (no alloca needed, it's already a pointer)
            let self_ptr = param_val.into_pointer_value();
            self.variables.insert("self".to_string(), self_ptr);
            self.variable_types
                .insert("self".to_string(), receiver_ty.into());
            self.variable_struct_names
                .insert("self".to_string(), struct_name.to_string());

            // ⭐ CRITICAL: Store AST type for type inference (implicit receiver)
            let receiver_ast_type =
                Type::Reference(Box::new(Type::Named(struct_name.to_string())), false);
            self.variable_ast_types
                .insert("self".to_string(), receiver_ast_type);

            param_offset = 1;
        }

        for (i, param) in method.params.iter().enumerate() {
            let param_val = fn_val
                .get_nth_param((i + param_offset) as u32)
                .ok_or_else(|| format!("Missing parameter {}", param.name))?;

            // ⚠️ CRITICAL: Struct parameters are now passed BY VALUE (as StructValue)
            // We need to allocate storage and store the value
            let param_concrete_ty = Self::replace_self_in_type(&param.ty, struct_name);
            let is_struct_param = match &param_concrete_ty {
                Type::Named(type_name) => self.struct_defs.contains_key(type_name),
                _ => false,
            };

            if is_struct_param && param_val.is_struct_value() {
                // Struct parameter passed by value - allocate storage and store it
                let struct_val = param_val.into_struct_value();
                let alloca = self
                    .builder
                    .build_alloca(struct_val.get_type(), &param.name)
                    .map_err(|e| format!("Failed to create struct param alloca: {}", e))?;

                self.builder
                    .build_store(alloca, struct_val)
                    .map_err(|e| format!("Failed to store struct param: {}", e))?;

                self.variables.insert(param.name.clone(), alloca);
                self.variable_types
                    .insert(param.name.clone(), struct_val.get_type().into());

                // ⭐ CRITICAL: Store AST type for type inference
                self.variable_ast_types
                    .insert(param.name.clone(), param_concrete_ty.clone());
            } else {
                // Non-struct parameter - allocate and store as usual
                let param_ty = self.ast_type_to_llvm(&param_concrete_ty);
                let alloca = self
                    .builder
                    .build_alloca(param_ty, &param.name)
                    .map_err(|e| format!("Failed to create parameter alloca: {}", e))?;

                self.builder
                    .build_store(alloca, param_val)
                    .map_err(|e| format!("Failed to store parameter: {}", e))?;

                self.variables.insert(param.name.clone(), alloca);
                self.variable_types.insert(param.name.clone(), param_ty);

                // ⭐ CRITICAL: Store AST type for type inference
                self.variable_ast_types
                    .insert(param.name.clone(), param_concrete_ty.clone());
            }

            self.track_param_struct_name(&param.name, &param_concrete_ty);
        }

        // Compile method body
        let mut last_expr_value: Option<BasicValueEnum> = None;

        for (i, stmt) in method.body.statements.iter().enumerate() {
            let is_last = i == method.body.statements.len() - 1;

            // If last statement is expression and method has non-void return, save for implicit return
            if is_last && matches!(stmt, Statement::Expression(_)) && method.return_type.is_some() {
                if let Statement::Expression(expr) = stmt {
                    // Check if return type is void/nil
                    let is_void_return = matches!(method.return_type.as_ref(), Some(Type::Nil));
                    
                    if is_void_return {
                        // Void/nil function: compile expression as statement (for side effects)
                        self.compile_statement(stmt)?;
                    } else {
                        // Non-void function: save expression value for implicit return
                        let val = self.compile_expression(expr)?;
                        last_expr_value = Some(val);
                        continue; // Don't compile as statement
                    }
                }
            } else {
                self.compile_statement(stmt)?;
            }
        }

        // If we have a last expression value and block is not terminated, use implicit return
        if let Some(return_val) = last_expr_value {
            let is_terminated = if let Some(bb) = self.builder.get_insert_block() {
                bb.get_terminator().is_some()
            } else {
                false
            };

            if !is_terminated {
                self.builder
                    .build_return(Some(&return_val))
                    .map_err(|e| format!("Failed to build implicit return: {}", e))?;
            }
        }

        // Check if function needs explicit return
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            // Determine if void/nil return
            let is_void_or_nil = method.return_type.is_none()
                || matches!(method.return_type.as_ref(), Some(Type::Nil));

            eprintln!("📍 Method {} terminator check: return_type={:?}, is_void_or_nil={}", 
                mangled_name, method.return_type, is_void_or_nil);

            if is_void_or_nil {
                // Void/nil function - add implicit void return
                eprintln!("   → Adding void return");
                self.builder
                    .build_return(None)
                    .map_err(|e| format!("Failed to build void return: {}", e))?;
            } else if method.return_type.is_some() {
                return Err(format!("Function {} must return a value", mangled_name));
            }
        }

        // ⭐ NEW: Clear method mutability context
        self.current_method_is_mutable = false;

        Ok(())
    }

    pub(crate) fn track_param_struct_name(&mut self, param_name: &str, param_ty: &Type) {
        match param_ty {
            Type::Named(struct_name) => {
                if self.struct_defs.contains_key(struct_name) {
                    self.variable_struct_names
                        .insert(param_name.to_string(), struct_name.clone());
                }
            }
            Type::Generic { name, type_args } => {
                if let Ok(mangled_name) = self.instantiate_generic_struct(name, type_args) {
                    self.variable_struct_names
                        .insert(param_name.to_string(), mangled_name);
                }
            }
            _ => {}
        }
    }
}
