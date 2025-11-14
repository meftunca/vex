// generics/methods.rs
// Generic method instantiation for struct/enum methods
// This enables monomorphization of generic methods like Vec<T>::push

use super::super::*;
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
        for (i, param) in concrete_method.params.iter().enumerate() {
            eprintln!("    Param {}: {} : {:?}", i, param.name, param.ty);
        }

        // Declare the function in LLVM
        let fn_val = self.declare_function(&concrete_method)?;

        // Store mangled name mapping
        self.functions.insert(mangled_name.clone(), fn_val);

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

        // Compile the method (declare_function already done above)
        // We need to compile the body separately
        let prev_func = self.current_function;
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
                let alloca = self
                    .builder
                    .build_alloca(param_type, &receiver.name)
                    .map_err(|e| format!("Failed to allocate receiver: {}", e))?;
                self.builder
                    .build_store(alloca, llvm_param)
                    .map_err(|e| format!("Failed to store receiver: {}", e))?;
                self.variables.insert(receiver.name.clone(), alloca);
                self.variable_types
                    .insert(receiver.name.clone(), param_type);
                self.variable_ast_types
                    .insert(receiver.name.clone(), receiver.ty.clone());

                // Register struct name for receiver parameter
                // This is critical for field access like self._ptr
                if receiver.name == "self" {
                    eprintln!("  🔍 Found 'self' receiver with type: {:?}", receiver.ty);

                    // Extract struct name from receiver type
                    // Reference(Vec(I32)) → "Vec_i32"
                    if let Type::Reference(inner_type, _) = &receiver.ty {
                        let struct_name = self.type_to_string(inner_type);
                        eprintln!("  📌 Registering 'self' as struct: {}", struct_name);
                        self.variable_struct_names
                            .insert(receiver.name.clone(), struct_name.clone());

                        // Also register the actual concrete struct in struct_defs if not present
                        // This allows field access to work correctly
                        if !self.struct_defs.contains_key(&struct_name) {
                            eprintln!(
                                "  🔨 Struct {} not in struct_defs, creating...",
                                struct_name
                            );

                            // Get base struct name (e.g., "Vec" from "Vec_i32")
                            let parts: Vec<&str> = struct_name.split('_').collect();
                            let base_name = parts[0];

                            if let Some(ast_def) = self.struct_ast_defs.get(base_name) {
                                eprintln!(
                                    "  📚 Found AST definition for base struct: {}",
                                    base_name
                                );

                                // Build concrete struct definition with substituted types
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

                                // Register concrete struct
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
                    } else {
                        eprintln!("  ⚠️  'self' receiver is not a reference type!");
                    }
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

            if let Some(llvm_param) = fn_val.get_nth_param((i + param_offset) as u32) {
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
        let _result = self.compile_block(&concrete_method.body)?;

        // Restore previous function
        self.current_function = prev_func;

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
    pub(crate) fn extract_type_args_from_type(&self, ty: &Type) -> Result<Vec<Type>, String> {
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
        _struct_name: &str,
        _type_args: &[Type],
        _method_name: &str,
        _receiver: &Expression,
        receiver_val: BasicValueEnum<'ctx>,
        args: &[Expression],
    ) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, String> {
        use inkwell::values::BasicMetadataValueEnum;

        let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];

        // First argument is always the receiver (self)
        arg_vals.push(receiver_val.into());

        // Compile remaining arguments
        for arg in args {
            let val = self.compile_expression(arg)?;
            arg_vals.push(val.into());
        }

        Ok(arg_vals)
    }
}
