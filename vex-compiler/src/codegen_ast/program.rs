// src/codegen/program.rs
use super::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Extract struct name from receiver type
    /// &HashMap<K, V> → Some("HashMap")
    /// HashMap<K, V> → Some("HashMap")
    /// &SomeStruct → Some("SomeStruct")
    fn extract_struct_name_from_receiver(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Reference(inner, _) => self.extract_struct_name_from_receiver(inner),
            Type::Named(name) => Some(name.clone()),
            Type::Generic { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    /// Resolve imports and merge all items from imported modules into main program
    /// This ensures generic methods from imported modules are available
    fn resolve_and_merge_imports(&self, program: &mut Program) -> Result<(), String> {
        use crate::module_resolver::ModuleResolver;

        let mut resolver = ModuleResolver::new("stdlib");

        eprintln!("🔄 Resolving {} imports...", program.imports.len());
        eprintln!("   📄 Source file: {}", self.source_file);

        // Collect all items from imported modules
        let mut imported_items = Vec::new();

        for import in &program.imports {
            eprintln!("   → Import: {}", import.module);

            // Load the module - pass source file for relative path resolution
            let imported_module = resolver.load_module(&import.module, Some(&self.source_file))?;

            eprintln!("      Items in module: {}", imported_module.items.len());

            // Add all items from imported module
            // (In a real implementation, we'd respect the import { ... } selectors)
            for item in &imported_module.items {
                match item {
                    Item::Function(func) => {
                        eprintln!(
                            "         ✅ Function: {} (receiver: {})",
                            func.name,
                            func.receiver.is_some()
                        );

                        // If this is a method (has receiver), mangle its name NOW
                        // This prevents double-mangling in compile_program
                        if let Some(receiver) = &func.receiver {
                            if let Some(struct_name) =
                                self.extract_struct_name_from_receiver(&receiver.ty)
                            {
                                let mut mangled_func = func.clone();
                                let mangled_name = format!("{}_{}", struct_name, func.name);
                                mangled_func.name = mangled_name.clone();
                                eprintln!("            → Mangled to: {}", mangled_name);
                                imported_items.push(Item::Function(mangled_func));
                            } else {
                                imported_items.push(item.clone());
                            }
                        } else {
                            imported_items.push(item.clone());
                        }
                    }
                    Item::Struct(s) => {
                        eprintln!("         ✅ Struct: {}", s.name);
                        imported_items.push(item.clone());
                    }
                    Item::Enum(e) => {
                        eprintln!("         ✅ Enum: {}", e.name);
                        imported_items.push(item.clone());
                    }
                    Item::Contract(c) => {
                        eprintln!("         ✅ Contract: {}", c.name);
                        imported_items.push(item.clone());
                    }
                    _ => {
                        // Skip other items for now
                    }
                }
            }
        }

        eprintln!(
            "   → Merging {} imported items into program",
            imported_items.len()
        );

        // Merge imported items into program (before original items to avoid shadowing)
        let original_items = program.items.clone();
        program.items = imported_items;
        program.items.extend(original_items);

        Ok(())
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        eprintln!(
            "📋 compile_program: {} total items in AST (before import resolution)",
            program.items.len()
        );

        // ⭐ NEW: Resolve and merge imported modules
        let mut merged_program = program.clone();
        self.resolve_and_merge_imports(&mut merged_program)?;

        eprintln!(
            "📋 After import resolution: {} total items",
            merged_program.items.len()
        );

        // Check if any async functions exist in the program
        let has_async = merged_program
            .items
            .iter()
            .any(|item| matches!(item, Item::Function(f) if f.is_async));

        if has_async {
            eprintln!("🔄 Async functions detected - runtime will be initialized in main");
        }

        // Initialize trait bounds checker
        use crate::trait_bounds_checker::TraitBoundsChecker;
        let mut trait_checker = TraitBoundsChecker::new();
        trait_checker.initialize(&merged_program);
        
        // Validate const generic parameters in functions and structs
        for item in &merged_program.items {
            match item {
                Item::Function(func) => {
                    if let Err(e) = trait_checker.validate_const_params(&func.const_params) {
                        return Err(format!("Function '{}': {}", func.name, e));
                    }
                }
                Item::Struct(struct_def) => {
                    if let Err(e) = trait_checker.validate_const_params(&struct_def.const_params) {
                        return Err(format!("Struct '{}': {}", struct_def.name, e));
                    }
                }
                _ => {}
            }
        }
        
        self.trait_bounds_checker = Some(trait_checker);
        eprintln!("✅ Trait bounds checker initialized");

        // First pass: register types, constants, and function signatures
        for item in &merged_program.items {
            if let Item::TypeAlias(type_alias) = item {
                self.register_type_alias(type_alias)?;
            } else if let Item::Struct(struct_def) = item {
                self.register_struct(struct_def)?;
            } else if let Item::Enum(enum_def) = item {
                self.register_enum(enum_def)?;
            } else if let Item::Policy(policy) = item {
                self.register_policy(policy)?;
            } else if let Item::Contract(contract_def) = item {
                // Check for policy collision before registering contract
                self.check_trait_policy_collision(&contract_def.name)?;
                self.register_trait(contract_def)?;
            } else if let Item::TraitImpl(trait_impl) = item {
                self.register_trait_impl(trait_impl)?;
            } else if let Item::ExternBlock(extern_block) = item {
                self.compile_extern_block(extern_block)?;
            } else if let Item::Const(const_decl) = item {
                // Register constants as global variables
                eprintln!("📋 Found const item: {}", const_decl.name);
                self.compile_const(const_decl)?;
            }
        }

        // Apply policies to structs (after all policies registered)
        eprintln!("📋 Applying policies to structs...");
        for item in &merged_program.items {
            if let Item::Struct(struct_def) = item {
                self.apply_policies_to_struct(struct_def)?;
            }
        }

        // Check for circular dependencies in struct definitions
        self.check_circular_struct_dependencies(&merged_program)?;

        // Second pass: store and declare non-generic functions
        for item in &merged_program.items {
            if let Item::Function(func) = item {
                // Debug: Print function info
                if func.receiver.is_some() {
                    eprintln!("🔍 Function with receiver: {}", func.name);
                    eprintln!("   Receiver: {:?}", func.receiver);
                }

                // Check if this is a method (has receiver parameter)
                let (func_name, is_method) = if let Some(receiver) = &func.receiver {
                    // This is a method - extract struct name from receiver type and mangle the name
                    let struct_name = self.extract_struct_name_from_receiver(&receiver.ty);
                    if let Some(sn) = struct_name {
                        // Check if name is already mangled (from imported modules)
                        let mangled_name = if func.name.starts_with(&format!("{}_", sn)) {
                            eprintln!("📌 Method already mangled: {}", func.name);
                            func.name.clone()
                        } else {
                            // ⭐ NEW: For operator overloading, include parameter count to distinguish
                            // unary vs binary operators (e.g., op-(self) vs op-(self, other))
                            let param_count = func.params.len();
                            let base_name = format!("{}_{}", sn, func.name);
                            
                            // Only add parameter count suffix for operators that can be both unary and binary
                            let name = if func.name.starts_with("op") && 
                                       (func.name == "op-" || func.name == "op+" || func.name == "op*") {
                                format!("{}_{}", base_name, param_count)
                            } else {
                                base_name
                            };
                            
                            eprintln!(
                                "📌 Registering method: {} (receiver type: {}, generic: {}, params: {}) as {}",
                                func.name,
                                sn,
                                !func.type_params.is_empty(),
                                param_count,
                                name
                            );
                            name
                        };

                        // Store with mangled name (but keep original func structure)
                        let mut method_func = func.clone();
                        method_func.name = mangled_name.clone();
                        self.function_defs.insert(mangled_name.clone(), method_func);

                        (mangled_name, true)
                    } else {
                        // Couldn't extract struct name, use original name
                        self.function_defs.insert(func.name.clone(), func.clone());
                        (func.name.clone(), false)
                    }
                } else {
                    // Regular function
                    self.function_defs.insert(func.name.clone(), func.clone());
                    (func.name.clone(), false)
                };

                // Skip generic functions for now - they'll be instantiated on-demand
                // But we still register them above for later instantiation
                if func.type_params.is_empty() {
                    // For methods with receivers, we already mangled the name
                    if is_method {
                        let method_func = self.function_defs.get(&func_name).cloned();
                        if let Some(method_func) = method_func {
                            self.declare_function(&method_func)?;
                        }
                    } else {
                        self.declare_function(func)?;
                    }
                }
            }
        }

        // Declare inline struct methods (new trait system v1.3)
        for item in &merged_program.items {
            if let Item::Struct(struct_def) = item {
                if struct_def.type_params.is_empty() {
                    for method in &struct_def.methods {
                        self.declare_struct_method(&struct_def.name, method)?;
                    }
                }
            }
        }

        // Generate enum constructor functions for non-generic enums
        for item in &merged_program.items {
            if let Item::Enum(enum_def) = item {
                if enum_def.type_params.is_empty() {
                    self.generate_enum_constructors(enum_def)?;
                }
            }
        }

        // Third pass: compile trait impl method bodies
        // MUST come before function bodies, as functions may call trait methods
        for item in &merged_program.items {
            if let Item::TraitImpl(trait_impl) = item {
                for method in &trait_impl.methods {
                    self.compile_trait_impl_method(
                        &trait_impl.trait_name,
                        &trait_impl.for_type,
                        method,
                    )?;
                }
            }
        }

        // Fourth pass: compile inline struct method bodies (new trait system v1.3)
        // MUST come before function bodies, as generic instantiation may need these methods
        eprintln!("📋 Fourth pass: compiling inline struct method bodies...");
        for item in &merged_program.items {
            if let Item::Struct(struct_def) = item {
                eprintln!(
                    "   Struct: {} (generic: {}, methods: {})",
                    struct_def.name,
                    !struct_def.type_params.is_empty(),
                    struct_def.methods.len()
                );
                if struct_def.type_params.is_empty() {
                    for method in &struct_def.methods {
                        self.compile_struct_method(&struct_def.name, method)?;
                    }
                }
            }
        }

        // Fifth pass: compile non-generic function bodies
        // Comes AFTER struct methods, so generic instantiation can use them
        for item in &merged_program.items {
            if let Item::Function(func) = item {
                // Skip generic functions - they'll be compiled when instantiated
                // Also skip methods with generic receivers (they'll be instantiated with their struct)
                let is_generic_method = if let Some(receiver) = &func.receiver {
                    match &receiver.ty {
                        Type::Generic { .. } => true,
                        Type::Reference(inner, _) => matches!(&**inner, Type::Generic { .. }),
                        _ => false,
                    }
                } else {
                    false
                };

                if func.type_params.is_empty() && !is_generic_method {
                    self.compile_function(func)?;
                }
            }
        }

        Ok(())
    }
}
