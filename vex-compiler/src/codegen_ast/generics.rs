// src/codegen/generics.rs
use super::*;
use std::collections::HashMap;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn instantiate_generic_function(
        &mut self,
        func_def: &Function,
        type_args: &[Type],
    ) -> Result<inkwell::values::FunctionValue<'ctx>, String> {
        let type_names: Vec<String> = type_args.iter().map(|t| self.type_to_string(t)).collect();
        let mangled_name = format!("{}_{}", func_def.name, type_names.join("_"));

        if let Some(fn_val) = self.functions.get(&mangled_name) {
            return Ok(*fn_val);
        }

        // ⭐ NEW: Check trait bounds before instantiation
        if let Some(ref mut checker) = self.trait_bounds_checker {
            checker.check_function_bounds(func_def, type_args)?;
            eprintln!(
                "✅ Trait bounds validated for {}::<{}>",
                func_def.name,
                type_names.join(", ")
            );
        }

        let mut type_subst = HashMap::new();
        for (i, type_param) in func_def.type_params.iter().enumerate() {
            if let Some(concrete_type) = type_args.get(i) {
                type_subst.insert(type_param.name.clone(), concrete_type.clone());
            }
        }

        let subst_func = self.substitute_types_in_function(func_def, &type_subst)?;

        let saved_current_function = self.current_function;
        let saved_insert_block = self.builder.get_insert_block();
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_variable_types = std::mem::take(&mut self.variable_types);
        let saved_variable_struct_names = std::mem::take(&mut self.variable_struct_names);

        // ⭐ NEW: Store instantiated function in function_defs for type inference
        self.function_defs
            .insert(mangled_name.clone(), subst_func.clone());

        let fn_val = self.declare_function(&subst_func)?;
        self.functions.insert(mangled_name.clone(), fn_val);

        self.compile_function(&subst_func)?;

        self.current_function = saved_current_function;
        self.variables = saved_variables;
        self.variable_types = saved_variable_types;
        self.variable_struct_names = saved_variable_struct_names;

        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        Ok(fn_val)
    }

    pub(crate) fn instantiate_generic_struct(
        &mut self,
        struct_name: &str,
        type_args: &[Type],
    ) -> Result<String, String> {
        // uses MAX_GENERIC_DEPTH from super
        for type_arg in type_args {
            let depth = self.get_generic_depth(type_arg);
            if depth > super::MAX_GENERIC_DEPTH {
                return Err(format!(
                    "Generic type nesting too deep (depth {}, max {}): {}",
                    depth,
                    super::MAX_GENERIC_DEPTH,
                    self.type_to_string(type_arg)
                ));
            }
        }

        let type_arg_strings: Vec<String> =
            type_args.iter().map(|t| self.type_to_string(t)).collect();
        let cache_key = (struct_name.to_string(), type_arg_strings.clone());

        if let Some(mangled_name) = self.generic_instantiations.get(&cache_key) {
            return Ok(mangled_name.clone());
        }

        let struct_ast = self
            .struct_ast_defs
            .get(struct_name)
            .cloned()
            .ok_or_else(|| format!("Generic struct '{}' not found", struct_name))?;

        if struct_ast.type_params.len() != type_args.len() {
            return Err(format!(
                "Struct '{}' expects {} type parameters, got {}",
                struct_name,
                struct_ast.type_params.len(),
                type_args.len()
            ));
        }

        // ⭐ NEW: Check trait bounds before instantiation
        if let Some(ref mut checker) = self.trait_bounds_checker {
            checker.check_struct_bounds(&struct_ast, type_args)?;
            eprintln!(
                "✅ Trait bounds validated for {}<{}>",
                struct_name,
                type_arg_strings.join(", ")
            );
        }

        let mut type_subst = HashMap::new();
        for (param, arg) in struct_ast.type_params.iter().zip(type_args.iter()) {
            type_subst.insert(param.name.clone(), arg.clone());
        }

        let mangled_name = format!("{}_{}", struct_name, type_arg_strings.join("_"));

        let specialized_fields: Vec<(String, Type)> = struct_ast
            .fields
            .iter()
            .map(|f| {
                let substituted_ty = self.substitute_type(&f.ty, &type_subst);
                (f.name.clone(), substituted_ty)
            })
            .collect();

        // StructDef type lives in super (as in the original file's usage)
        use super::StructDef;
        self.struct_defs.insert(
            mangled_name.clone(),
            StructDef {
                fields: specialized_fields,
            },
        );

        // ⭐ NEW: Instantiate all methods for this struct
        self.instantiate_struct_methods(
            struct_name,
            &struct_ast.type_params,
            type_args,
            &mangled_name,
        )?;

        self.generic_instantiations
            .insert(cache_key, mangled_name.clone());

        Ok(mangled_name)
    }

    pub(crate) fn infer_type_args_from_call(
        &mut self,
        func_def: &Function,
        args: &[Expression],
    ) -> Result<Vec<Type>, String> {
        // For functions with multiple type parameters of the same type,
        // we need to infer unique type parameters, not all argument types
        // Example: fn max<T>(a: T, b: T): T has 1 type param T, not 2

        if func_def.type_params.is_empty() {
            return Ok(Vec::new());
        }

        // Infer first type parameter from first argument
        if args.is_empty() {
            return Err(format!(
                "Cannot infer type arguments for '{}': no arguments provided",
                func_def.name
            ));
        }

        let first_arg_type = self.infer_expression_type(&args[0])?;

        // For now, simple strategy: assume all type params are the same type as first arg
        // This works for max<T>(a: T, b: T), identity<T>(x: T), etc.
        // TODO: More sophisticated type inference for multi-param generics
        let mut type_args = Vec::new();
        for _ in 0..func_def.type_params.len() {
            type_args.push(first_arg_type.clone());
        }

        Ok(type_args)
    }

    /// Instantiate all methods of a generic struct with concrete type arguments
    /// This is called when a generic struct is instantiated (e.g., HashMap<str, i32>)
    fn instantiate_struct_methods(
        &mut self,
        struct_name: &str,
        struct_type_params: &[TypeParam],
        type_args: &[Type],
        mangled_struct_name: &str,
    ) -> Result<(), String> {
        // Build type substitution map: K -> str, V -> i32, etc.
        let mut type_subst = HashMap::new();
        for (param, arg) in struct_type_params.iter().zip(type_args.iter()) {
            type_subst.insert(param.name.clone(), arg.clone());
        }

        eprintln!(
            "🔧 Instantiating methods for struct {} -> {}",
            struct_name, mangled_struct_name
        );
        eprintln!("   Type substitution: {:?}", type_subst);

        // Debug: List all function_defs
        eprintln!(
            "   All registered functions ({} total):",
            self.function_defs.len()
        );
        for (name, _) in self.function_defs.iter() {
            eprintln!("      - {}", name);
        }

        // Find all methods for this struct
        // Methods are stored as regular functions with receiver parameter
        let method_names: Vec<String> = self
            .function_defs
            .keys()
            .filter(|name| {
                // Generic struct methods: HashMap_insert, HashMap_get, etc.
                name.starts_with(&format!("{}_", struct_name))
                    && !name.contains("_str_") // Not already instantiated
                    && !name.contains("_i32_")
                    && !name.contains("_i64_")
            })
            .cloned()
            .collect();

        eprintln!("   Found {} methods to instantiate", method_names.len());

        for method_name in method_names {
            let func_def = self.function_defs.get(&method_name).cloned();
            if let Some(func) = func_def {
                eprintln!("   → Method: {}", method_name);

                // Check if this method has a receiver parameter
                // Either has func.receiver field OR first param is named "self"
                let has_receiver = func.receiver.is_some()
                    || func.params.first().map_or(false, |p| p.name == "self");

                if !has_receiver {
                    eprintln!("      ⚠️  Skipping - not a method (no receiver)");
                    continue;
                }

                // Instantiate the method
                let specialized_func = self.substitute_types_in_method(
                    &func,
                    &type_subst,
                    struct_name,
                    mangled_struct_name,
                )?;

                eprintln!(
                    "      ✅ Instantiated: {} -> {}",
                    method_name, specialized_func.name
                );

                // Register the instantiated method in function_defs (AST)
                self.function_defs
                    .insert(specialized_func.name.clone(), specialized_func.clone());

                // ⭐ FIX: Declare and compile the method NOW (not later)
                // Save current context
                let saved_current_function = self.current_function;
                let saved_insert_block = self.builder.get_insert_block();
                let saved_variables = std::mem::take(&mut self.variables);
                let saved_variable_types = std::mem::take(&mut self.variable_types);
                let saved_variable_struct_names = std::mem::take(&mut self.variable_struct_names);

                // Declare the method
                match self.declare_function(&specialized_func) {
                    Ok(fn_val) => {
                        self.functions.insert(specialized_func.name.clone(), fn_val);
                        eprintln!("      → Declared LLVM function");

                        // Compile the method body
                        match self.compile_function(&specialized_func) {
                            Ok(_) => {
                                eprintln!("      ✅ Compiled successfully");
                            }
                            Err(e) => {
                                eprintln!("      ⚠️  Compilation failed: {}", e);
                                eprintln!("         Continuing with next method...");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("      ⚠️  Declaration failed: {}", e);
                        eprintln!("         Continuing with next method...");
                    }
                }

                // Restore context
                self.current_function = saved_current_function;
                self.variables = saved_variables;
                self.variable_types = saved_variable_types;
                self.variable_struct_names = saved_variable_struct_names;

                if let Some(block) = saved_insert_block {
                    self.builder.position_at_end(block);
                }
            }
        }

        Ok(())
    }

    /// Substitute types in a method, including receiver type
    fn substitute_types_in_method(
        &self,
        func: &Function,
        type_subst: &HashMap<String, Type>,
        struct_name: &str,
        mangled_struct_name: &str,
    ) -> Result<Function, String> {
        let mut new_func = func.clone();

        // Clear type parameters (they're now concrete)
        new_func.type_params.clear();

        // ⭐ FIX: Update receiver field with mangled struct name
        if let Some(ref mut receiver) = new_func.receiver {
            receiver.ty = self.substitute_type(&receiver.ty, type_subst);

            // Ensure receiver references the mangled struct
            if let Type::Reference(inner, is_mut) = &receiver.ty {
                match inner.as_ref() {
                    Type::Named(_) | Type::Generic { .. } => {
                        receiver.ty = Type::Reference(
                            Box::new(Type::Named(mangled_struct_name.to_string())),
                            *is_mut,
                        );
                    }
                    _ => {}
                }
            }
        }

        // Substitute parameter types
        for param in &mut new_func.params {
            param.ty = self.substitute_type(&param.ty, type_subst);

            // Also update receiver type if it references the struct
            if let Type::Reference(inner, is_mut) = &param.ty {
                // After substitution, if inner is Named OR still Generic, update to mangled struct
                match inner.as_ref() {
                    Type::Named(_) | Type::Generic { .. } => {
                        // Update struct name to mangled version
                        param.ty = Type::Reference(
                            Box::new(Type::Named(mangled_struct_name.to_string())),
                            *is_mut,
                        );
                    }
                    _ => {}
                }
            } else if let Type::Named(_name) = &param.ty {
                // Direct struct parameter
                param.ty = Type::Named(mangled_struct_name.to_string());
            }
        }

        // Substitute return type
        if let Some(ret_ty) = &new_func.return_type {
            new_func.return_type = Some(self.substitute_type(ret_ty, type_subst));
        }

        // Build mangled method name: HashMap_insert -> HashMap_str_i32_insert
        // Extract method name by removing struct prefix
        let struct_prefix = format!("{}_", struct_name);
        let method_suffix = if func.name.starts_with(&struct_prefix) {
            &func.name[struct_prefix.len()..]
        } else {
            // Fallback: use original name if prefix doesn't match
            &func.name
        };

        new_func.name = format!("{}_{}", mangled_struct_name, method_suffix);

        Ok(new_func)
    }

    fn substitute_types_in_function(
        &self,
        func: &Function,
        type_subst: &HashMap<String, Type>,
    ) -> Result<Function, String> {
        let mut new_func = func.clone();
        new_func.type_params.clear();
        for param in &mut new_func.params {
            param.ty = self.substitute_type(&param.ty, type_subst);
        }
        if let Some(ret_ty) = &new_func.return_type {
            new_func.return_type = Some(self.substitute_type(ret_ty, type_subst));
        }

        // ⭐ NEW: Substitute types in function body
        new_func.body = self.substitute_types_in_block(&new_func.body, type_subst);

        let type_names: Vec<String> = type_subst
            .values()
            .map(|t| self.type_to_string(t))
            .collect();
        new_func.name = format!("{}_{}", func.name, type_names.join("_"));
        Ok(new_func)
    }

    fn substitute_types_in_block(
        &self,
        block: &Block,
        type_subst: &HashMap<String, Type>,
    ) -> Block {
        Block {
            statements: block
                .statements
                .iter()
                .map(|stmt| self.substitute_types_in_statement(stmt, type_subst))
                .collect(),
        }
    }

    fn substitute_types_in_statement(
        &self,
        stmt: &Statement,
        type_subst: &HashMap<String, Type>,
    ) -> Statement {
        match stmt {
            Statement::Return(Some(expr)) => {
                Statement::Return(Some(self.substitute_types_in_expression(expr, type_subst)))
            }
            // TODO: Handle other statement types as needed
            _ => stmt.clone(),
        }
    }

    fn substitute_types_in_expression(
        &self,
        expr: &Expression,
        type_subst: &HashMap<String, Type>,
    ) -> Expression {
        match expr {
            Expression::StructLiteral {
                name,
                type_args,
                fields,
            } => {
                let new_type_args = type_args
                    .iter()
                    .map(|ty| self.substitute_type(ty, type_subst))
                    .collect();

                let new_fields = fields
                    .iter()
                    .map(|(field_name, field_expr)| {
                        (
                            field_name.clone(),
                            self.substitute_types_in_expression(field_expr, type_subst),
                        )
                    })
                    .collect();

                Expression::StructLiteral {
                    name: name.clone(),
                    type_args: new_type_args,
                    fields: new_fields,
                }
            }
            // TODO: Handle other expression types as needed
            _ => expr.clone(),
        }
    }
}
