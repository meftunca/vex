// src/codegen/methods.rs
use super::*;
use inkwell::types::BasicTypeEnum;

impl<'ctx> ASTCodeGen<'ctx> {
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
                    if !trait_impl.type_args.is_empty() {
                        // ⭐ CRITICAL: Check if this trait defines this operator method!
                        // Get trait definition and check if it has this method
                        let trait_has_method = if let Some(trait_def) = self.trait_defs.get(&trait_impl.name) {
                            trait_def.methods.iter().any(|m| m.name == method.name)
                        } else {
                            false // Trait not found, assume no
                        };
                        
                        if !trait_has_method {
                            // This trait doesn't define this method, skip it
                            continue;
                        }
                        
                        // Check if first param type matches trait's type arg
                        if first_param_ty == &trait_impl.type_args[0] {
                            // Mangle: StructName_TraitName_TypeArg_method_paramCount
                            let type_str = match &trait_impl.type_args[0] {
                                Type::Named(n) => n.clone(),
                                Type::I32 => "i32".to_string(),
                                Type::I64 => "i64".to_string(),
                                Type::F32 => "f32".to_string(),
                                Type::F64 => "f64".to_string(),
                                Type::Bool => "bool".to_string(),
                                Type::String => "String".to_string(),
                                _ => "unknown".to_string(),
                            };
                            found_match = Some(format!("{}_{}_{}_{}", struct_name, trait_impl.name, type_str, method_name_encoded));
                            break;
                        }
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
        } else if method.name.starts_with("op") && 
                  (method.name == "op-" || method.name == "op+" || method.name == "op*") {
            format!("{}_{}", base_name, param_count)
        } else {
            base_name
        };

        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();

        // Add receiver parameter (explicit or implicit)
        if let Some(ref receiver) = method.receiver {
            // Explicit receiver: fn (self: &T) method()
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        } else {
            // Implicit receiver: fn method() - auto-generate &StructName! parameter (mutable by default)
            // This allows both read and write access to struct fields
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            param_types.push(ptr_type.into());
        }

        for param in &method.params {
            let param_llvm_ty = self.ast_type_to_llvm(&param.ty);
            eprintln!("  📝 Method param: {} (AST type: {:?}, LLVM type: {:?})", param.name, param.ty, param_llvm_ty);
            param_types.push(param_llvm_ty.into());
        }

        let ret_type = if let Some(ref ty) = method.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            inkwell::types::BasicTypeEnum::IntType(self.context.i32_type())
        };

        eprintln!("🔧 Declaring method: {} with {} params (including receiver)", mangled_name, param_types.len());

        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_types, false),
        };

        let fn_val = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name.clone(), fn_val);

        let mut mangled_method = method.clone();
        mangled_method.name = mangled_name.clone();
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
        eprintln!("🔍 compile_struct_method: method.params.len()={}, param_count={}", method.params.len(), param_count);
        let base_name = format!("{}_{}", struct_name, method_name_encoded);
        
        let mangled_name = if method.name.starts_with("op") && !method.params.is_empty() {
            if let Some(struct_def) = self.struct_ast_defs.get(struct_name) {
                let first_param_ty = &method.params[0].ty;
                
                let mut found_match = None;
                for trait_impl in &struct_def.impl_traits {
                    if !trait_impl.type_args.is_empty() {
                        // ⭐ CRITICAL: Check if this trait defines this operator method!
                        let trait_has_method = if let Some(trait_def) = self.trait_defs.get(&trait_impl.name) {
                            trait_def.methods.iter().any(|m| m.name == method.name)
                        } else {
                            false
                        };
                        
                        if !trait_has_method {
                            continue;
                        }
                        
                        if first_param_ty == &trait_impl.type_args[0] {
                            let type_str = match &trait_impl.type_args[0] {
                                Type::Named(n) => n.clone(),
                                Type::I32 => "i32".to_string(),
                                Type::I64 => "i64".to_string(),
                                Type::F32 => "f32".to_string(),
                                Type::F64 => "f64".to_string(),
                                Type::Bool => "bool".to_string(),
                                Type::String => "String".to_string(),
                                _ => "unknown".to_string(),
                            };
                            found_match = Some(format!("{}_{}_{}_{}", struct_name, trait_impl.name, type_str, method_name_encoded));
                            break;
                        }
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
        } else if method.name.starts_with("op") && 
                  (method.name == "op-" || method.name == "op+" || method.name == "op*") {
            format!("{}_{}", base_name, param_count)
        } else {
            base_name
        };
        
        eprintln!("🔧 compile_struct_method: struct={}, method={}, mangled_name={}", struct_name, method.name, mangled_name);
        
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

        eprintln!("   🔍 method.receiver = {:?}", method.receiver);

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
            let receiver_llvm_ty = match &receiver.ty {
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

            let struct_name_opt = match &receiver.ty {
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

            eprintln!(
                "🔧 compile_struct_method: struct={}, receiver.ty={:?}, struct_name_opt={:?}",
                struct_name, receiver.ty, struct_name_opt
            );
            eprintln!(
                "   receiver_llvm_ty={:?}, self_ptr={:?}",
                receiver_llvm_ty, self_ptr
            );

            if let Some(name) = struct_name_opt {
                eprintln!("   ✅ Tracking 'self' as struct: {}", name);
                self.variable_struct_names.insert("self".to_string(), name);
            } else {
                eprintln!("   ❌ No struct name extracted from receiver type!");
            }

            param_offset = 1;
        } else {
            // Implicit receiver: fn method() - auto-create immutable reference receiver
            eprintln!(
                "📝 Simplified method syntax: auto-generating receiver &{} for method {}",
                struct_name, method.name
            );

            let param_val = fn_val
                .get_nth_param(0)
                .ok_or("Missing implicit receiver parameter")?;

            // Create pointer type for receiver (it's already a pointer parameter)
            let receiver_ty = self.context.ptr_type(inkwell::AddressSpace::default());

            // Store receiver DIRECTLY (no alloca needed, it's already a pointer)
            let self_ptr = param_val.into_pointer_value();
            eprintln!("   📍 self_ptr = {:?}", self_ptr);
            self.variables.insert("self".to_string(), self_ptr);
            self.variable_types
                .insert("self".to_string(), receiver_ty.into());
            self.variable_struct_names
                .insert("self".to_string(), struct_name.to_string());

            eprintln!("   ✅ Implicit receiver tracked as struct: {}", struct_name);

            param_offset = 1;
        }

        eprintln!("   Method has {} parameters", method.params.len());
        for (i, param) in method.params.iter().enumerate() {
            eprintln!(
                "   Processing param {}: {} (type: {:?})",
                i, param.name, param.ty
            );
            let param_val = fn_val
                .get_nth_param((i + param_offset) as u32)
                .ok_or_else(|| format!("Missing parameter {}", param.name))?;
            
            eprintln!("      param_val: {:?}", param_val);

            // ⚠️ CRITICAL: Struct parameters are now passed BY VALUE (as StructValue)
            // We need to allocate storage and store the value
            let is_struct_param = match &param.ty {
                Type::Named(type_name) => self.struct_defs.contains_key(type_name),
                _ => false,
            };

            eprintln!(
                "      is_struct={}, is_struct_value={}",
                is_struct_param,
                param_val.is_struct_value()
            );

            if is_struct_param && param_val.is_struct_value() {
                // Struct parameter passed by value - allocate storage and store it
                eprintln!("      → Allocating storage for struct value");
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
            } else {
                // Non-struct parameter - allocate and store as usual
                eprintln!("      → Standard alloca+store");
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
            }

            self.track_param_struct_name(&param.name, &param.ty);
        }

        // Compile method body
        eprintln!("📋 compile_struct_method: About to compile body with {} statements", method.body.statements.len());
        let mut last_expr_value: Option<BasicValueEnum> = None;
        
        for (i, stmt) in method.body.statements.iter().enumerate() {
            eprintln!("   📝 Statement {}: {:?}", i, std::mem::discriminant(stmt));
            let is_last = i == method.body.statements.len() - 1;
            
            // If last statement is expression, save its value for potential implicit return
            if is_last && matches!(stmt, Statement::Expression(_)) && method.return_type.is_some() {
                if let Statement::Expression(expr) = stmt {
                    let val = self.compile_expression(expr)?;
                    last_expr_value = Some(val);
                    continue; // Don't compile as statement
                }
            }
            
            self.compile_statement(stmt)?;
        }
        
        // If we have a last expression value and block is not terminated, use implicit return
        if let Some(return_val) = last_expr_value {
            let is_terminated = if let Some(bb) = self.builder.get_insert_block() {
                bb.get_terminator().is_some()
            } else {
                false
            };
            
            if !is_terminated {
                eprintln!("🔄 Implicit return from last expression in function body");
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
            if method.return_type.is_some() {
                return Err(format!("Function {} must return a value", mangled_name));
            } else {
                let ret_val = self.context.i32_type().const_int(0, false);
                self.builder
                    .build_return(Some(&ret_val))
                    .map_err(|e| format!("Failed to build return: {}", e))?;
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
