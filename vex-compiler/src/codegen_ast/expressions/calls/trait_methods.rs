// Trait method resolution and compilation

use crate::{debug_log, debug_println};
use crate::codegen_ast::ASTCodeGen;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Resolve method name for instance method calls
    pub(crate) fn resolve_method_name(
        &mut self,
        struct_name: &str,
        method: &str,
        args: &[Expression],
    ) -> Result<String, String> {
        // Construct method function name: StructName_method
        // ⭐ IMPORTANT: For external methods (registered in program.rs), we use ENCODED operator names (opadd)
        // Both external and inline methods now use encoded names for consistency

        // ⭐ NEW: Type-based method overloading for external methods
        // Format: StructName_methodname_typename_paramcount
        let param_count = args.len();

        // CRITICAL: Encode operator names for LLVM compatibility
        let method_encoded = if method.starts_with("op") {
            match method {
                "op+" => "opadd",
                "op-" => "opsub",
                "op*" => "opmul",
                "op/" => "opdiv",
                "op%" => "opmod",
                "op**" => "oppow",
                "op==" => "opeq",
                "op!=" => "opne",
                "op<" => "oplt",
                "op<=" => "ople",
                "op>" => "opgt",
                "op>=" => "opge",
                "op&" => "opbitand",
                "op|" => "opbitor",
                "op^" => "opbitxor",
                "op<<" => "opshl",
                "op>>" => "opshr",
                "op!" => "opnot",
                "op~" => "opbitnot",
                "op++" => "opinc",
                "op--" => "opdec",
                "op[]" => "opindex",
                "op[]=" => "opindexset",
                _ => method,
            }
        } else {
            method
        };
        let base_method_name = format!("{}_{}", struct_name, method_encoded);

        // Try to get first argument type for type-based lookup
        let first_arg_type_suffix = if !args.is_empty() {
            eprintln!(
                "🔍 Method call {}.{}() with {} args",
                struct_name,
                method,
                args.len()
            );
            debug_println!("🔍 First arg: {:?}", args[0]);
            if let Ok(arg_type) = self.infer_expression_type(&args[0]) {
                debug_println!("🔍 First arg type inferred: {:?}", arg_type);
                // For method lookup, use the FULL type including references
                // This matches how registration works in program.rs
                let suffix = self.generate_type_suffix(&arg_type);
                debug_println!("🔍 Type suffix generated: {}", suffix);
                suffix
            } else {
                debug_println!("🔍 Failed to infer first arg type");
                String::new()
            }
        } else {
            debug_println!("🔍 Method call {}.{}() with no args", struct_name, method);
            String::new()
        };

        // Try external method with type suffix first (for overloading support)
        let external_method_typed = if !first_arg_type_suffix.is_empty() {
            // Both operators and regular methods support type-based overloading
            format!(
                "{}{}_{}",
                base_method_name, first_arg_type_suffix, param_count
            )
        } else {
            String::new()
        };

        // Fallback: external method without type suffix (for backward compatibility)
        let external_method_name = if method.starts_with("op")
            && (method == "op-" || method == "op+" || method == "op*")
        {
            format!("{}_{}", base_method_name, param_count)
        } else {
            base_method_name.clone()
        };

        // For inline methods: receiver + args (param_count = args.len() + 1)
        // Note: base_method_name already uses encoded operator name
        // ⚠️ CRITICAL: Include type suffix for inline methods to support overloading
        let encoded_param_count = args.len() + 1; // For inline methods, param count includes receiver
        let inline_method_name = if method.starts_with("op")
            && (method == "op-" || method == "op+" || method == "op*")
        {
            // For operators with type suffix (overloaded methods), use it
            if !first_arg_type_suffix.is_empty() {
                format!(
                    "{}{}_{}",
                    base_method_name, first_arg_type_suffix, encoded_param_count
                )
            } else {
                format!("{}_{}", base_method_name, encoded_param_count)
            }
        } else {
            base_method_name.clone()
        };

        // Check all naming schemes: inline typed > inline untyped > external typed > external untyped
        // ⚠️ CRITICAL: Prioritize inline methods to avoid collision with external methods
        debug_println!("🔍 Checking method names:");
        eprintln!(
            "   inline_method_name: {} (exists: {})",
            inline_method_name,
            self.functions.contains_key(&inline_method_name)
        );
        eprintln!(
            "   external_method_typed: {} (exists: {})",
            external_method_typed,
            self.functions.contains_key(&external_method_typed)
        );
        eprintln!(
            "   external_method_name: {} (exists: {})",
            external_method_name,
            self.functions.contains_key(&external_method_name)
        );

        // DEBUG: List all functions starting with the struct name
        let debug_funcs: Vec<_> = self
            .functions
            .keys()
            .filter(|k| k.starts_with(&format!("{}_{}", struct_name, method_encoded)))
            .collect();
        eprintln!(
            "   🔍 Available functions for {}.{}: {:?}",
            struct_name, method, debug_funcs
        );

        // Priority order: inline typed > inline untyped > external typed > external untyped
        // This ensures binary operators don't collide with unary operators
        let method_func_name = if self.functions.contains_key(&inline_method_name) {
            inline_method_name
        } else if !external_method_typed.is_empty()
            && self.functions.contains_key(&external_method_typed)
        {
            external_method_typed
        } else if self.functions.contains_key(&external_method_name) {
            external_method_name
        } else {
            // Final fallback: try to find ANY function that starts with struct_method pattern
            // This handles cases where type suffix doesn't match exactly
            let pattern = format!("{}_{}", struct_name, method_encoded);
            if let Some(found) = self.functions.keys().find(|k| k.starts_with(&pattern)) {
                eprintln!("   ✅ Found via pattern match: {}", found);
                found.clone()
            } else {
                // Default to inline naming for error messages (most common case)
                inline_method_name
            }
        };

        // Check if method function exists (either as a struct method or trait method)
        eprintln!(
            "🔍 Looking for method: {} (exists: {})",
            method_func_name,
            self.functions.contains_key(&method_func_name)
        );
        if self.functions.contains_key(&method_func_name) {
            // Found as struct method (including external methods)
            eprintln!("✅ Found method in functions: {}", method_func_name);
            Ok(method_func_name)
        } else {
            // Try to find trait method (method_encoded is already the encoded name from earlier)
            self.find_trait_method(struct_name, method, args, method_encoded)
        }
    }

    /// Find trait method implementation
    fn find_trait_method(
        &mut self,
        struct_name: &str,
        method: &str,
        args: &[Expression],
        method_encoded: &str,
    ) -> Result<String, String> {
        let mut found_trait_method = None;

        // First, try to match against generic impl clauses in struct_ast_defs
        if let Some(struct_def) = self.struct_ast_defs.get(struct_name) {
            // Try to match based on all implemented traits
            // For operator methods, we need to match BOTH the method name AND argument types
            // Don't break on first match - check ALL traits for the right type match
            for trait_impl in &struct_def.impl_traits {
                if !trait_impl.type_args.is_empty() {
                    // Generic trait impl - try to match with actual argument types
                    if !args.is_empty() {
                        // Try to infer the type of first argument
                        if let Ok(arg_type) = self.infer_expression_type(&args[0]) {
                            debug_println!("🔍 Method lookup: struct={}, trait={}, method={}, arg_type={:?}, trait_type_arg={:?}",
                                struct_name, trait_impl.name, method, arg_type, trait_impl.type_args[0]);
                            // Check if this arg type matches trait's type arg
                            if arg_type == trait_impl.type_args[0] {
                                let type_str = match &trait_impl.type_args[0] {
                                    Type::Named(n) => n.clone(),
                                    Type::I32 => "i32".to_string(),
                                    Type::I64 => "i64".to_string(),
                                    Type::F32 => "f32".to_string(),
                                    Type::F64 => "f64".to_string(),
                                    Type::Bool => "bool".to_string(),
                                    Type::String => "String".to_string(),
                                    _ => continue,
                                };

                                // Generate mangled name with type args
                                // For inline methods, param_count includes implicit receiver (+1)
                                let param_count = args.len() + 1;
                                let trait_method_name = format!(
                                    "{}_{}_{}_{}_{}",
                                    struct_name,
                                    trait_impl.name,
                                    type_str,
                                    method_encoded,
                                    param_count
                                );

                                eprintln!("   🎯 Generated mangled name: {}", trait_method_name);
                                eprintln!(
                                    "   🔍 Function exists: {}",
                                    self.functions.contains_key(&trait_method_name)
                                );

                                // Check if this function exists - if so, we found it!
                                if self.functions.contains_key(&trait_method_name) {
                                    eprintln!("   ✅ MATCH! Using method: {}", trait_method_name);
                                    found_trait_method = Some(trait_method_name);
                                    break; // Found exact match, stop searching
                                }
                            }
                        }
                    }
                } else {
                    // Non-generic trait impl - use old format
                    let trait_method_name =
                        format!("{}_{}_{}", struct_name, trait_impl.name, method_encoded);
                    if self.functions.contains_key(&trait_method_name) {
                        found_trait_method = Some(trait_method_name);
                        break;
                    }
                }
            }
        }

        // Fallback: Try old format for trait_impls registry
        if found_trait_method.is_none() {
            for ((trait_name, type_name), _) in &self.trait_impls {
                if type_name == struct_name {
                    let trait_method_name = format!("{}_{}_{}", type_name, trait_name, method);
                    if self.functions.contains_key(&trait_method_name) {
                        found_trait_method = Some(trait_method_name);
                        break;
                    }
                }
            }
        }

        if let Some(trait_method) = found_trait_method {
            Ok(trait_method)
        } else {
            // Try to find default trait method
            self.find_and_compile_default_trait_method(struct_name, method)
        }
    }

    /// Find and compile default trait method if available
    fn find_and_compile_default_trait_method(
        &mut self,
        struct_name: &str,
        method: &str,
    ) -> Result<String, String> {
        // Check all traits implemented by this type
        // First, collect trait information to avoid borrow checker issues
        let mut default_method_info: Option<(String, String, vex_ast::TraitMethod)> = None;

        for ((trait_name, type_name), _) in &self.trait_impls {
            if type_name == struct_name {
                // Check if the trait has a default method with this name
                if let Some(trait_def) = self.trait_defs.get(trait_name) {
                    for trait_method in &trait_def.methods {
                        if trait_method.name == method && trait_method.body.is_some() {
                            // Found default method! Save info for compilation
                            default_method_info =
                                Some((trait_name.clone(), type_name.clone(), trait_method.clone()));
                            break;
                        }
                    }
                }
                if default_method_info.is_some() {
                    break;
                }
            }
        }

        // Now compile if found
        if let Some((trait_name, type_name, trait_method)) = default_method_info {
            let default_method_name = format!("{}_{}_{}", type_name, trait_name, method);

            // Check if already compiled
            if !self.functions.contains_key(&default_method_name) {
                self.compile_default_trait_method(&trait_name, &type_name, &trait_method)?;
            }

            Ok(default_method_name)
        } else {
            // Method not found - check if this is a global function call
            // This handles cases where parser incorrectly converted function calls to method calls
            // in method bodies (e.g., log2(msg) parsed as self.log2(msg))

            if self.functions.contains_key(method) || self.function_defs.contains_key(method) {
                // This is a global function, not a method!
                Err(format!(
                    "Method '{}' not found on struct '{}' (but global function exists - parser bug?)",
                    method, struct_name
                ))
            } else {
                Err(format!(
                    "Method '{}' not found for struct '{}' (neither as struct method, trait method, nor default trait method)",
                    method, struct_name
                ))
            }
        }
    }

    /// Compile a default trait method for a specific concrete type
    fn compile_default_trait_method(
        &mut self,
        trait_name: &str,
        concrete_type_name: &str,
        trait_method: &vex_ast::TraitMethod,
    ) -> Result<(), String> {
        // Save current function context (variables, types, current_function, builder position)
        let saved_variables = self.variables.clone();
        let saved_variable_types = self.variable_types.clone();
        let saved_variable_struct_names = self.variable_struct_names.clone();
        let saved_current_function = self.current_function;

        let concrete_type = vex_ast::Type::Named(concrete_type_name.to_string());

        let receiver = trait_method.receiver.as_ref().map(|r| Receiver {
            name: r.name.clone(),
            is_mutable: r.is_mutable,
            ty: Self::replace_self_type(&r.ty, concrete_type_name),
        });

        let params: Vec<Param> = trait_method
            .params
            .iter()
            .map(|p| Param {
                name: p.name.clone(),
                ty: Self::replace_self_type(&p.ty, concrete_type_name),
                default_value: p.default_value.clone(),
            })
            .collect();

        let return_type = trait_method
            .return_type
            .as_ref()
            .map(|t| Self::replace_self_type(t, concrete_type_name));

        // Convert TraitMethod to Function for compilation
        let func = vex_ast::Function {
            is_async: false,
            is_gpu: false,
            is_mutable: trait_method.is_mutable, // ⭐ NEW: Copy mutability from trait
            is_operator: trait_method.is_operator, // ⭐ NEW: Copy operator flag from trait
            is_static: false,
            static_type: None,
            receiver,
            name: trait_method.name.clone(),
            type_params: vec![],
            const_params: vec![],
            where_clause: vec![],
            params,
            return_type,
            body: trait_method.body.clone().unwrap(), // Safe because we checked is_some()
            is_variadic: false,
            variadic_type: None,
        };

        // Declare and compile the default method for this specific type
        self.declare_trait_impl_method(trait_name, &concrete_type, &func)?;
        self.compile_trait_impl_method(trait_name, &concrete_type, &func)?;

        // Restore function context
        self.variables = saved_variables;
        self.variable_types = saved_variable_types;
        self.variable_struct_names = saved_variable_struct_names;
        self.current_function = saved_current_function;

        // Restore builder position if we have a current function
        if let Some(func) = self.current_function {
            if let Some(bb) = func.get_last_basic_block() {
                self.builder.position_at_end(bb);
            }
        }

        Ok(())
    }

    /// Replace Self type with concrete type name (for default trait methods)
    fn replace_self_type(ty: &Type, concrete_type: &str) -> Type {
        match ty {
            Type::Named(name) if name == "Self" => Type::Named(concrete_type.to_string()),
            Type::Reference(inner, is_mut) => Type::Reference(
                Box::new(Self::replace_self_type(inner, concrete_type)),
                *is_mut,
            ),
            Type::Generic { name, type_args } => {
                let new_name = if name == "Self" {
                    concrete_type.to_string()
                } else {
                    name.clone()
                };
                Type::Generic {
                    name: new_name,
                    type_args: type_args
                        .iter()
                        .map(|t| Self::replace_self_type(t, concrete_type))
                        .collect(),
                }
            }
            Type::Array(inner, size) => Type::Array(
                Box::new(Self::replace_self_type(inner, concrete_type)),
                *size,
            ),
            Type::Slice(inner, is_mut) => Type::Slice(
                Box::new(Self::replace_self_type(inner, concrete_type)),
                *is_mut,
            ),
            Type::Union(types) => Type::Union(
                types
                    .iter()
                    .map(|t| Self::replace_self_type(t, concrete_type))
                    .collect(),
            ),
            Type::Intersection(types) => Type::Intersection(
                types
                    .iter()
                    .map(|t| Self::replace_self_type(t, concrete_type))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }
}
