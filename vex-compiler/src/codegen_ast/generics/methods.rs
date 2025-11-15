// generics/methods.rs
// Generic method instantiation for struct/enum methods
// This enables monomorphization of generic methods like Vec<T>::push

use super::super::*;
use inkwell::types::BasicTypeEnum;
use inkwell::values::FunctionValue;
use std::collections::HashMap;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Instantiate a generic method for a specific type
    ///
    /// Example: Vec<T>::push(value: T) with T=i32 becomes Vec_i32_push(value: i32)
    ///
    /// # Arguments
    /// * `struct_name` - Base struct name (e.g., "Vec")
    /// * `struct_type_args` - Struct's type arguments (e.g., [I32])
    /// * `method_name` - Method name (e.g., "push")
    /// * `method_def` - Generic method AST definition
    /// * `arg_types` - Argument types for additional type inference
    ///
    /// # Returns
    /// LLVM function value for the instantiated method
    pub(crate) fn instantiate_generic_method(
        &mut self,
        struct_name: &str,
        struct_type_args: &[Type],
        method_name: &str,
        method_def: &Function,
        _arg_types: &[Type],
    ) -> Result<FunctionValue<'ctx>, String> {
        eprintln!(
            "🔧 instantiate_generic_method: {}::<{}>::{}",
            struct_name,
            struct_type_args
                .iter()
                .map(|t| self.type_to_string(t))
                .collect::<Vec<_>>()
                .join(", "),
            method_name
        );

        // Build type names for mangling
        let struct_type_names: Vec<String> = struct_type_args
            .iter()
            .map(|t| self.type_to_string(t))
            .collect();

        // Build mangled name for this specific instantiation
        // Format: StructName_TypeArgs_methodname
        // Example: Vec_i32_push, HashMap_String_i32_insert
        let mangled_name = if struct_type_args.is_empty() {
            format!("{}_{}", struct_name, method_name)
        } else {
            format!(
                "{}_{}_{}",
                struct_name,
                struct_type_names.join("_"),
                method_name
            )
        };

        eprintln!("  → Mangled name: {}", mangled_name);

        // Check cache - already instantiated?
        if let Some(fn_val) = self.functions.get(&mangled_name) {
            eprintln!("  ✅ Found in cache!");
            return Ok(*fn_val);
        }

        // Build type substitution map
        // Map generic type parameters to concrete types
        let mut type_subst = HashMap::new();

        // First: Substitute struct-level type parameters
        // For Vec<T>, if we have Vec<i32>, map T → i32
        if let Some(struct_def) = self.struct_ast_defs.get(struct_name) {
            eprintln!(
                "  📚 Found struct definition with {} type params",
                struct_def.type_params.len()
            );
            eprintln!("  📥 Provided type args: {:?}", struct_type_args);

            if struct_type_args.is_empty() && !struct_def.type_params.is_empty() {
                eprintln!("  ⚠️  WARNING: No type arguments provided for generic struct!");
                eprintln!("  📋 Struct type params: {:?}", struct_def.type_params);

                // Try defaulting to i32 as fallback
                for type_param in &struct_def.type_params {
                    eprintln!(
                        "  ⚠️  Unknown type name '{}', defaulting to i32",
                        type_param.name
                    );
                    type_subst.insert(type_param.name.clone(), Type::I32);
                }
            } else {
                for (i, type_param) in struct_def.type_params.iter().enumerate() {
                    if let Some(concrete_type) = struct_type_args.get(i) {
                        type_subst.insert(type_param.name.clone(), concrete_type.clone());
                        eprintln!(
                            "  📝 Type param {} → {}",
                            type_param.name,
                            self.type_to_string(concrete_type)
                        );
                    } else {
                        eprintln!(
                            "  ⚠️  Missing type argument for parameter '{}' in struct '{}'",
                            type_param.name, struct_name
                        );
                    }
                }
            }
        } else {
            eprintln!(
                "  ⚠️  Could not find struct definition for '{}'",
                struct_name
            );
        }

        // Second: Substitute method-level type parameters if any
        // For generic methods like map<U>(f: fn(T) -> U)
        if !method_def.type_params.is_empty() {
            // Try to infer method type params from argument types
            // This is more complex and might need explicit type args in the future
            eprintln!(
                "  ⚠️  Method has {} type parameters - inference not fully implemented yet",
                method_def.type_params.len()
            );
        }

        // Substitute all type parameters in the method definition
        let concrete_method = self.substitute_types_in_function(method_def, &type_subst)?;

        eprintln!("  🔨 Declaring function: {}", mangled_name);
        eprintln!(
            "  📋 Concrete method has {} params",
            concrete_method.params.len()
        );
        eprintln!(
            "  📋 Concrete method receiver: {:?}",
            concrete_method.receiver
        );
        eprintln!(
            "  ⭐ Concrete method return type: {:?}",
            concrete_method.return_type
        );
        for (i, param) in concrete_method.params.iter().enumerate() {
            eprintln!("    Param {}: {} : {:?}", i, param.name, param.ty);
        }

        // Declare the function in LLVM
        let fn_val = self.declare_function(&concrete_method)?;

        // Store mangled name mapping
        self.functions.insert(mangled_name.clone(), fn_val);

        // ⚠️ CRITICAL: Register concrete method in function_defs for argument type checking
        self.function_defs.insert(mangled_name.clone(), concrete_method.clone());
        eprintln!("  ✅ Registered function_def: {}", mangled_name);

        // Check trait bounds before compilation
        if let Some(ref mut checker) = self.trait_bounds_checker {
            // Validate struct type args against struct's trait bounds
            if let Some(struct_def) = self.struct_ast_defs.get(struct_name) {
                if !struct_def.type_params.is_empty() {
                    checker.check_struct_bounds(struct_def, struct_type_args)?;
                    eprintln!(
                        "  ✅ Trait bounds validated for {}::<{}>",
                        struct_name,
                        struct_type_names.join(", ")
                    );
                }
            }

            // Validate method's own trait bounds
            if !method_def.type_params.is_empty() {
                // For now, skip method-level bounds
                // TODO: Implement method-level type param inference
            }
        }

        eprintln!("  🏗️  Compiling method body...");

        // Preserve current codegen context so method compilation doesn't corrupt caller state
        let saved_current_function = self.current_function;
        let saved_insert_block = self.builder.get_insert_block();
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_variable_types = std::mem::take(&mut self.variable_types);
        let saved_variable_ast_types = std::mem::take(&mut self.variable_ast_types);
        let saved_variable_struct_names = std::mem::take(&mut self.variable_struct_names);
        let saved_variable_enum_names = std::mem::take(&mut self.variable_enum_names);
        let saved_tuple_variable_types = std::mem::take(&mut self.tuple_variable_types);
        let saved_function_params = std::mem::take(&mut self.function_params);
        let saved_function_param_types = std::mem::take(&mut self.function_param_types);
        let saved_scope_stack = std::mem::take(&mut self.scope_stack);
        let saved_loop_context_stack = std::mem::take(&mut self.loop_context_stack);
        let saved_deferred_statements = std::mem::take(&mut self.deferred_statements);
        let saved_closure_envs = std::mem::take(&mut self.closure_envs);
        let saved_closure_variables = std::mem::take(&mut self.closure_variables);
        let saved_last_tuple_type = self.last_compiled_tuple_type.take();
        let saved_method_mutability = self.current_method_is_mutable;

        // Compile the method (declare_function already done above)
        // We need to compile the body separately
        self.current_function = Some(fn_val);

        // Create entry block
        let entry_block = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry_block);

        // Set up receiver parameter (if exists)
        let mut param_offset = 0;
        if let Some(ref receiver) = concrete_method.receiver {
            eprintln!(
                "  🔧 Setting up receiver: {} (type: {:?})",
                receiver.name, receiver.ty
            );

            if let Some(llvm_param) = fn_val.get_nth_param(0) {
                let param_type = self.ast_type_to_llvm(&receiver.ty);

                let receiver_var = match &receiver.ty {
                    Type::Reference(_, _) => {
                        eprintln!(
                            "  📌 Receiver '{}' is reference type, using pointer directly",
                            receiver.name
                        );
                        if let BasicValueEnum::PointerValue(ptr) = llvm_param {
                            ptr
                        } else {
                            return Err(format!(
                                "Receiver parameter '{}' expected pointer value",
                                receiver.name
                            ));
                        }
                    }
                    _ => {
                        eprintln!(
                            "  📌 Receiver '{}' is value type, allocating and storing",
                            receiver.name
                        );
                        let alloca =
                            self.create_entry_block_alloca(&receiver.name, &receiver.ty, true)?;
                        self.builder
                            .build_store(alloca, llvm_param)
                            .map_err(|e| format!("Failed to store receiver: {}", e))?;
                        alloca
                    }
                };

                self.variables.insert(receiver.name.clone(), receiver_var);
                self.variable_types
                    .insert(receiver.name.clone(), param_type);
                self.variable_ast_types
                    .insert(receiver.name.clone(), receiver.ty.clone());

                let extract_struct_name = |ty: &Type| -> Option<String> {
                    match ty {
                        Type::Named(name) => Some(name.clone()),
                        Type::Reference(inner, _) => match &**inner {
                            Type::Named(name) => Some(name.clone()),
                            Type::Vec(_) | Type::Box(_) | Type::Option(_) | Type::Result(_, _) => {
                                Some(self.type_to_string(inner))
                            }
                            Type::Generic { .. } => Some(self.type_to_string(inner)),
                            _ => None,
                        },
                        Type::Vec(_) | Type::Box(_) | Type::Option(_) | Type::Result(_, _) => {
                            Some(self.type_to_string(ty))
                        }
                        Type::Generic { .. } => Some(self.type_to_string(ty)),
                        _ => None,
                    }
                };

                if let Some(struct_name) = extract_struct_name(&receiver.ty) {
                    if receiver.name == "self" {
                        eprintln!("  📌 Registering 'self' as struct: {}", struct_name);
                        self.variable_struct_names
                            .insert(receiver.name.clone(), struct_name.clone());

                        if !self.struct_defs.contains_key(&struct_name) {
                            eprintln!(
                                "  🔨 Struct {} not in struct_defs, creating...",
                                struct_name
                            );

                            let parts: Vec<&str> = struct_name.split('_').collect();
                            let base_name = parts[0];

                            if let Some(ast_def) = self.struct_ast_defs.get(base_name) {
                                eprintln!(
                                    "  📚 Found AST definition for base struct: {}",
                                    base_name
                                );

                                let mut concrete_fields = Vec::new();
                                for field in &ast_def.fields {
                                    let substituted_type =
                                        self.substitute_type(&field.ty, &type_subst);
                                    eprintln!(
                                        "    🔄 Field {} : {:?} → {:?}",
                                        field.name, field.ty, substituted_type
                                    );
                                    concrete_fields.push((field.name.clone(), substituted_type));
                                }

                                use crate::codegen_ast::StructDef;
                                let struct_def = StructDef {
                                    fields: concrete_fields,
                                };
                                self.struct_defs.insert(struct_name.clone(), struct_def);
                                eprintln!("  ✅ Registered concrete struct: {}", struct_name);
                            } else {
                                eprintln!(
                                    "  ❌ Could not find AST definition for base struct: {}",
                                    base_name
                                );
                            }
                        } else {
                            eprintln!("  ✓ Struct {} already in struct_defs", struct_name);
                        }
                    }
                } else {
                    eprintln!(
                        "  ⚠️  Unable to extract struct name for receiver: {:?}",
                        receiver.ty
                    );
                }

                param_offset = 1; // Regular params start at index 1
            }
        }

        // Set up regular parameters
        for (i, param) in concrete_method.params.iter().enumerate() {
            eprintln!(
                "  🔧 Setting up parameter {}: {} (type: {:?})",
                i, param.name, param.ty
            );

            let param_idx = crate::safe_param_index(i, param_offset)
                .map_err(|e| format!("Parameter index overflow for {}: {}", param.name, e))?;
            if let Some(llvm_param) = fn_val.get_nth_param(param_idx) {
                let param_type = self.ast_type_to_llvm(&param.ty);
                let alloca = self
                    .builder
                    .build_alloca(param_type, &param.name)
                    .map_err(|e| format!("Failed to allocate param: {}", e))?;
                self.builder
                    .build_store(alloca, llvm_param)
                    .map_err(|e| format!("Failed to store param: {}", e))?;
                self.variables.insert(param.name.clone(), alloca);
                self.variable_types.insert(param.name.clone(), param_type);
                self.variable_ast_types
                    .insert(param.name.clone(), param.ty.clone());
            }
        }

        // Compile function body
        self.push_scope();
        let compile_result = self.compile_block(&concrete_method.body);

        if compile_result.is_ok() {
            if let Some(current_block) = self.builder.get_insert_block() {
                if current_block.get_terminator().is_none() {
                    self.pop_scope().map_err(|e| {
                        format!("Failed to pop scope for method {}: {}", mangled_name, e)
                    })?;
                    self.execute_deferred_statements()?;
                }
            }

            self.clear_deferred_statements();

            if let Some(current_block) = self.builder.get_insert_block() {
                if current_block.get_terminator().is_none() {
                    // ✅ FIX: Void methods should always get implicit return, not unreachable
                    if concrete_method.return_type.is_none() {
                        // Void method - add implicit return
                        self.builder
                            .build_return(None)
                            .map_err(|e| format!("Failed to build void return: {}", e))?;
                    } else {
                        // Non-void method - check if entry block
                        let is_entry_block = fn_val
                            .get_first_basic_block()
                            .map(|entry| entry == current_block)
                            .unwrap_or(false);

                        if is_entry_block {
                            let ret_type = self
                                .ast_type_to_llvm(concrete_method.return_type.as_ref().unwrap());
                            match ret_type {
                                BasicTypeEnum::IntType(int_ty) => {
                                    let zero = int_ty.const_int(0, false);
                                    self.builder
                                        .build_return(Some(&zero))
                                        .map_err(|e| format!("Failed to build return: {}", e))?;
                                }
                                _ => {
                                    return Err(
                                        "Non-void generic method must have explicit return"
                                            .to_string(),
                                    );
                                }
                            }
                        } else {
                            // Non-entry block without terminator in non-void method = unreachable
                            self.builder
                                .build_unreachable()
                                .map_err(|e| format!("Failed to build unreachable: {}", e))?;
                        }
                    }
                }
            }
        } // Restore previous context before propagating errors
        self.current_function = saved_current_function;
        self.variables = saved_variables;
        self.variable_types = saved_variable_types;
        self.variable_ast_types = saved_variable_ast_types;
        self.variable_struct_names = saved_variable_struct_names;
        self.variable_enum_names = saved_variable_enum_names;
        self.tuple_variable_types = saved_tuple_variable_types;
        self.function_params = saved_function_params;
        self.function_param_types = saved_function_param_types;
        self.scope_stack = saved_scope_stack;
        self.loop_context_stack = saved_loop_context_stack;
        self.deferred_statements = saved_deferred_statements;
        self.closure_envs = saved_closure_envs;
        self.closure_variables = saved_closure_variables;
        self.last_compiled_tuple_type = saved_last_tuple_type;
        self.current_method_is_mutable = saved_method_mutability;

        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        // Propagate compilation errors after restoring state
        compile_result?;

        eprintln!("  ✅ Generic method instantiated successfully!");

        Ok(fn_val)
    }

    /// Find a generic method definition from struct AST
    pub(crate) fn find_generic_method(
        &self,
        struct_name: &str,
        method_name: &str,
    ) -> Result<Function, String> {
        eprintln!("  🔍 find_generic_method: {}.{}", struct_name, method_name);

        // Check struct AST defs for external methods
        if let Some(struct_def) = self.struct_ast_defs.get(struct_name) {
            eprintln!(
                "      Found struct: {} with {} type params",
                struct_name,
                struct_def.type_params.len()
            );

            // Look for external method (Go-style: fn (self: &Type) method())
            // These are stored in function_defs with mangled names
            // Pattern: StructName_methodname OR StructName_methodname_typename_paramcount

            // Try exact base name first (no type suffix)
            let base_name = format!("{}_{}", struct_name, method_name);
            eprintln!("      Looking for base name: {}", base_name);

            if let Some(method_def) = self.function_defs.get(&base_name) {
                eprintln!("  ✅ Found external method with base name: {}", base_name);
                return Ok(method_def.clone());
            }

            // Try finding with type suffix for overloaded methods
            // Pattern: Vec_push_i32_1, Vec_push_String_1, etc.
            for (func_name, func_def) in &self.function_defs {
                // Match: starts with "Vec_push" but may have "_i32_1" suffix
                if func_name.starts_with(&base_name) {
                    // Ensure it's exactly the method (not a longer name like "Vec_push_back")
                    if func_name == &base_name
                        || func_name.as_bytes().get(base_name.len()) == Some(&b'_')
                    {
                        eprintln!("  ✅ Found external method with type suffix: {}", func_name);
                        return Ok(func_def.clone());
                    }
                }
            }

            // Look for inline method (inside struct definition)
            for method in &struct_def.methods {
                if method.name == method_name {
                    eprintln!("  ✅ Found inline method: {}.{}", struct_name, method_name);
                    return Ok(method.clone());
                }
            }

            // Debug: Show what's available
            eprintln!("  ⚠️  Available function_defs with struct name:");
            for key in self.function_defs.keys() {
                if key.contains(struct_name) {
                    eprintln!("      - {}", key);
                }
            }
        } else {
            eprintln!(
                "  ⚠️  Struct '{}' not found in struct_ast_defs",
                struct_name
            );
        }

        Err(format!(
            "Generic method '{}' not found for struct '{}'",
            method_name, struct_name
        ))
    }

    /// Check if a method requires generic instantiation
    pub(crate) fn method_needs_instantiation(
        &self,
        method_def: &Function,
        struct_name: &str,
    ) -> bool {
        // Check if struct is generic
        let struct_is_generic = self
            .struct_ast_defs
            .get(struct_name)
            .map(|s| !s.type_params.is_empty())
            .unwrap_or(false);

        // Check if method itself is generic
        let method_is_generic = !method_def.type_params.is_empty();

        struct_is_generic || method_is_generic
    }

    /// Parse type arguments from mangled name parts
    /// Example: ["i32", "String"] -> [Type::I32, Type::Named("String")]
    pub(crate) fn parse_type_args_from_mangled_name(
        &self,
        parts: &[&str],
    ) -> Result<Vec<Type>, String> {
        let mut type_args = Vec::new();

        for part in parts {
            let ty = match *part {
                // Primitive types
                "i8" => Type::I8,
                "i16" => Type::I16,
                "i32" => Type::I32,
                "i64" => Type::I64,
                "u8" => Type::U8,
                "u16" => Type::U16,
                "u32" => Type::U32,
                "u64" => Type::U64,
                "f32" => Type::F32,
                "f64" => Type::F64,
                "bool" => Type::Bool,
                "string" => Type::String,
                // TODO: Add more complex types (Generic, Function, etc.)

                // Named types (structs, enums, etc.)
                other => Type::Named(other.to_string()),
            };
            type_args.push(ty);
        }

        if type_args.is_empty() {
            return Err("No type arguments could be parsed from mangled name".to_string());
        }

        Ok(type_args)
    }

    /// Infer struct type arguments from variable context
    /// This is a fallback when mangled name doesn't contain type info
    pub(crate) fn infer_struct_type_args(&self, _struct_name: &str) -> Result<Vec<Type>, String> {
        // Placeholder: Would need to look up variable types in scope
        // For now, return empty (will fail if type params are required)
        Ok(Vec::new())
    }

    /// Extract type arguments from a Type
    /// Examples:
    /// - Vec<i32> -> [i32]
    /// - HashMap<String, i32> -> [String, i32]
    /// - &Vec<i32> -> [i32] (unwrap reference first)
    /// - Vec (non-generic usage) -> []
    pub fn extract_type_args_from_type(&self, ty: &Type) -> Result<Vec<Type>, String> {
        match ty {
            Type::Reference(inner, _) => self.extract_type_args_from_type(inner),
            Type::Vec(elem_ty) => Ok(vec![(**elem_ty).clone()]),
            Type::Box(elem_ty) => Ok(vec![(**elem_ty).clone()]),
            Type::Option(elem_ty) => Ok(vec![(**elem_ty).clone()]),
            Type::Result(ok_ty, err_ty) => Ok(vec![(**ok_ty).clone(), (**err_ty).clone()]),
            Type::Generic { type_args, .. } => Ok(type_args.clone()),
            _ => Ok(Vec::new()), // Non-generic type
        }
    }

    /// Compile method arguments for generic method call
    /// This is similar to compile_method_arguments but handles generic receivers
    pub(crate) fn compile_method_arguments_for_generic(
        &mut self,
        struct_name: &str,
        type_args: &[Type],
        method_name: &str,
        _receiver: &Expression,
        receiver_val: BasicValueEnum<'ctx>,
        args: &[Expression],
    ) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, String> {
        use inkwell::values::BasicMetadataValueEnum;

        let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];

        // Build mangled method name to look up function definition
        let type_names: Vec<String> = type_args
            .iter()
            .map(|t| self.type_to_string(t))
            .collect();
        
        let mangled_name = if type_args.is_empty() {
            format!("{}_{}", struct_name, method_name)
        } else {
            format!(
                "{}_{}_{}",
                struct_name,
                type_names.join("_"),
                method_name
            )
        };

        // First argument is always the receiver (self)
        arg_vals.push(receiver_val.into());

        // Compile remaining arguments with type casting
        for (arg_idx, arg) in args.iter().enumerate() {
            // ⭐ NEW: For integer/float literals, compile with expected type if available
            let mut val = if let Some(func_def) = self.function_defs.get(&mangled_name) {
                if arg_idx < func_def.params.len() {
                    let expected_ty = &func_def.params[arg_idx].ty;
                    eprintln!("  🔍 Arg {}: expected_ty = {:?}", arg_idx, expected_ty);
                    
                    // For integer literals, compile directly to expected type
                    if matches!(arg, Expression::IntLiteral(_)) {
                        match expected_ty {
                            Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::I128 |
                            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 => {
                                let target_llvm = self.ast_type_to_llvm(expected_ty);
                                if let BasicTypeEnum::IntType(target_int_type) = target_llvm {
                                    if let Expression::IntLiteral(value) = arg {
                                        eprintln!("  ✅ Compiling IntLiteral({}) as {:?}", value, expected_ty);
                                        target_int_type.const_int(*value as u64, false).into()
                                    } else {
                                        self.compile_expression(arg)?
                                    }
                                } else {
                                    self.compile_expression(arg)?
                                }
                            }
                            _ => self.compile_expression(arg)?
                        }
                    }
                    // For float literals, compile directly to expected type
                    else if matches!(arg, Expression::FloatLiteral(_)) {
                        match expected_ty {
                            Type::F16 | Type::F32 | Type::F64 => {
                                let target_llvm = self.ast_type_to_llvm(expected_ty);
                                if let BasicTypeEnum::FloatType(target_float_type) = target_llvm {
                                    if let Expression::FloatLiteral(value) = arg {
                                        target_float_type.const_float(*value).into()
                                    } else {
                                        self.compile_expression(arg)?
                                    }
                                } else {
                                    self.compile_expression(arg)?
                                }
                            }
                            _ => self.compile_expression(arg)?
                        }
                    } else {
                        self.compile_expression(arg)?
                    }
                } else {
                    self.compile_expression(arg)?
                }
            } else {
                self.compile_expression(arg)?
            };
            
            // ⚠️ CRITICAL: Cast argument types to match function signature (for non-literals)
            if let Some(func_def) = self.function_defs.get(&mangled_name) {
                if arg_idx < func_def.params.len() {
                    let expected_ty = &func_def.params[arg_idx].ty;
                    let source_ty = self.infer_expression_type(arg)?;
                    
                    // Integer width casting
                    if self.is_integer_type(&source_ty) && self.is_integer_type(expected_ty) {
                        if let BasicValueEnum::IntValue(int_val) = val {
                            let target_llvm = self.ast_type_to_llvm(expected_ty);
                            if let BasicTypeEnum::IntType(target_int_type) = target_llvm {
                                let src_bits = int_val.get_type().get_bit_width();
                                let dst_bits = target_int_type.get_bit_width();
                                
                                if src_bits != dst_bits {
                                    let source_is_unsigned = self.is_unsigned_integer_type(&source_ty);
                                    
                                    val = if src_bits < dst_bits {
                                        if source_is_unsigned {
                                            self.builder
                                                .build_int_z_extend(int_val, target_int_type, "generic_arg_zext")
                                                .map_err(|e| format!("Failed to zero-extend: {}", e))?
                                                .into()
                                        } else {
                                            self.builder
                                                .build_int_s_extend(int_val, target_int_type, "generic_arg_sext")
                                                .map_err(|e| format!("Failed to sign-extend: {}", e))?
                                                .into()
                                        }
                                    } else {
                                        self.builder
                                            .build_int_truncate(int_val, target_int_type, "generic_arg_trunc")
                                            .map_err(|e| format!("Failed to truncate: {}", e))?
                                            .into()
                                    };
                                }
                            }
                        }
                    }
                }
            }
            
            arg_vals.push(val.into());
        }

        Ok(arg_vals)
    }
}
