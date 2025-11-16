// src/codegen/program.rs
use super::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Extract struct name from receiver type
    /// &HashMap<K, V> → Some("HashMap")
    /// HashMap<K, V> → Some("HashMap")
    /// &SomeStruct → Some("SomeStruct")
    /// &Vec<T> → Some("Vec")
    /// &Box<T> → Some("Box")
    pub(crate) fn extract_struct_name_from_receiver(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Reference(inner, _) => self.extract_struct_name_from_receiver(inner),
            Type::Named(name) => Some(name.clone()),
            Type::Generic { name, .. } => Some(name.clone()),
            Type::Vec(_) => Some("Vec".to_string()),
            Type::Box(_) => Some("Box".to_string()),
            Type::Option(_) => Some("Option".to_string()),
            Type::Result(_, _) => Some("Result".to_string()),
            _ => None,
        }
    }

    /// Resolve imports and merge all items from imported modules into main program
    /// This ensures generic methods from imported modules are available
    fn resolve_and_merge_imports(&self, program: &mut Program) -> Result<(), String> {
        use crate::module_resolver::ModuleResolver;

        // Two-tier module resolution:
        // 1. vex-libs/std - Standard library packages (import "conv", "http", etc.)
        // 2. stdlib - Prelude (auto-injected core types: Vec, Box, Option, Result)
        let mut std_resolver = ModuleResolver::new("vex-libs/std");
        let mut prelude_resolver = ModuleResolver::new("stdlib");

        // Collect all items from imported modules
        let mut imported_items = Vec::new();

        for import in &program.imports {
            // Try standard library first (vex-libs/std), then prelude (stdlib)
            let imported_module =
                match std_resolver.load_module(&import.module, Some(&self.source_file)) {
                    Ok(module) => {
                        eprintln!("   ✅ Loaded from vex-libs/std: {}", import.module);
                        module
                    }
                    Err(_) => {
                        eprintln!(
                            "   ⏭️  Not in vex-libs/std, trying stdlib (prelude): {}",
                            import.module
                        );
                        prelude_resolver.load_module(&import.module, Some(&self.source_file))?
                    }
                };

            // Add all items from imported module
            // (In a real implementation, we'd respect the import { ... } selectors)
            for item in &imported_module.items {
                match item {
                    Item::Function(func) => {
                        // If this is a method (has receiver), mangle its name NOW
                        // This prevents double-mangling in compile_program
                        if let Some(receiver) = &func.receiver {
                            if let Some(struct_name) =
                                self.extract_struct_name_from_receiver(&receiver.ty)
                            {
                                let mut mangled_func = func.clone();
                                let mangled_name = format!("{}_{}", struct_name, func.name);
                                mangled_func.name = mangled_name.clone();
                                imported_items.push(Item::Function(mangled_func));
                            } else {
                                imported_items.push(item.clone());
                            }
                        } else {
                            imported_items.push(item.clone());
                        }
                    }
                    Item::Struct(s) => {
                        imported_items.push(item.clone());
                    }
                    Item::Enum(e) => {
                        imported_items.push(item.clone());
                    }
                    Item::Contract(c) => {
                        imported_items.push(item.clone());
                    }
                    Item::ExternBlock(_) => {
                        // ✅ FIX: Include extern blocks from imported modules
                        // This ensures extern "C" functions are available in importing module
                        imported_items.push(item.clone());
                    }
                    _ => {
                        // Skip other items for now
                    }
                }
            }
        }

        // Merge imported items into program (before original items to avoid shadowing)
        let mut original_items = Vec::new();
        std::mem::swap(&mut program.items, &mut original_items);
        program.items = imported_items;
        program.items.extend(original_items);

        Ok(())
    }

    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        // ⭐ NOTE: Prelude now injected at CLI level (vex-cli/src/main.rs)
        // Layer 1 prelude is embedded in compiler binary and prepended to user code
        let mut merged_program = program.clone();

        // ⭐ NEW: Resolve and merge imported modules
        self.resolve_and_merge_imports(&mut merged_program)?;

        // Check if any async functions exist in the program
        let has_async = merged_program
            .items
            .iter()
            .any(|item| matches!(item, Item::Function(f) if f.is_async));

        if has_async {
            eprintln!("🔄 Async functions detected - runtime will be initialized in main");
        }

        // ⭐ ASYNC: Track if we need runtime initialization
        let needs_runtime_init = has_async;

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
                self.compile_const(const_decl)?;
            }
        }

        // Apply policies to structs (after all policies registered)
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
                            func.name.clone()
                        } else {
                            // ⭐ Method name mangling with type-based overloading support
                            // CRITICAL: Encode operator symbols (op[], op[]=) for LLVM
                            let encoded_method_name = Self::encode_operator_name(&func.name);
                            let param_count = func.params.len();
                            let base_name = format!("{}_{}", sn, encoded_method_name);

                            // ⭐ For method overloading: Include first parameter type in mangling
                            // This allows overloading like: add(i32), add(f64), op+(i32), op+(f64)
                            let name = if !func.params.is_empty() {
                                let first_param_type = &func.params[0].ty;
                                let type_suffix = self.generate_type_suffix(first_param_type);

                                // Add type suffix for all external methods (operators and regular methods)
                                // Format: StructName_methodname_typename_paramcount
                                if !type_suffix.is_empty() {
                                    format!("{}{}_{}", base_name, type_suffix, param_count)
                                } else {
                                    format!("{}_{}", base_name, param_count)
                                }
                            } else {
                                // No parameters (e.g., getter methods)
                                base_name
                            };

                            name
                        };

                        // Store with mangled name (but keep original func structure)
                        let mut method_func = func.clone();
                        method_func.name = mangled_name.clone();

                        // Replace `Self` in the stored AST for this method so subsequent
                        // declaration uses concrete types during codegen
                        if let Some(receiver) = &method_func.receiver {
                            // We know `sn` is Some in this branch
                            method_func.receiver = Some(vex_ast::Receiver {
                                name: receiver.name.clone(),
                                is_mutable: receiver.is_mutable,
                                ty: Self::replace_self_in_type(&receiver.ty, &sn),
                            });
                        }
                        for param in &mut method_func.params {
                            param.ty = Self::replace_self_in_type(&param.ty, &sn);
                        }
                        if let Some(rt) = &mut method_func.return_type {
                            *rt = Self::replace_self_in_type(rt, &sn);
                        }

                        // ⭐ CRITICAL FIX: Store BOTH the fully mangled name AND the base name
                        // Fully mangled: Vec_push_i32_1 (for direct lookups)
                        // Base name: Vec_push (for generic instantiation)
                        self.function_defs
                            .insert(mangled_name.clone(), method_func.clone());

                        // Also store with base name (no type suffix) for generic lookup
                        let base_lookup_name =
                            format!("{}_{}", sn, Self::encode_operator_name(&func.name));
                        if base_lookup_name != mangled_name {
                            self.function_defs.insert(base_lookup_name, method_func);
                        }

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
                // Also skip if this is a method on a generic struct
                let is_on_generic_struct = if let Some(ref receiver) = func.receiver {
                    // Extract struct name from receiver type
                    let struct_name = match &receiver.ty {
                        Type::Named(name) => Some(name.as_str()),
                        Type::Generic { name, .. } => Some(name.as_str()),
                        Type::Reference(inner, _) => match &**inner {
                            Type::Named(name) => Some(name.as_str()),
                            Type::Generic { name, .. } => Some(name.as_str()),
                            Type::Vec(_) => Some("Vec"),
                            Type::Box(_) => Some("Box"),
                            Type::Option(_) => Some("Option"),
                            Type::Result(_, _) => Some("Result"),
                            _ => None,
                        },
                        Type::Vec(_) => Some("Vec"),
                        Type::Box(_) => Some("Box"),
                        Type::Option(_) => Some("Option"),
                        Type::Result(_, _) => Some("Result"),
                        _ => None,
                    };

                    // Check if the struct is generic
                    struct_name
                        .and_then(|name| self.struct_ast_defs.get(name))
                        .map(|s| !s.type_params.is_empty())
                        .unwrap_or(false)
                } else {
                    false
                };

                if func.type_params.is_empty() && !is_on_generic_struct {
                    // For methods with receivers, we already mangled the name
                    if is_method {
                        let method_func = self.function_defs.get(&func_name).cloned();
                        if let Some(method_func) = method_func {
                            self.declare_function(&method_func)?;
                        }
                    } else {
                        self.declare_function(func)?;
                    }
                } else if is_on_generic_struct {
                    eprintln!(
                        "⏭️  Skipping method {} on generic struct (will be instantiated on-demand)",
                        func_name
                    );
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

        // ⭐ NEW: Auto-generate op!= from op== for Eq trait implementations
        self.auto_generate_eq_methods(&merged_program)?;

        // Fourth pass: compile inline struct method bodies (new trait system v1.3)
        // MUST come before function bodies, as generic instantiation may need these methods
        for item in &merged_program.items {
            if let Item::Struct(struct_def) = item {
                if struct_def.type_params.is_empty() {
                    for method in &struct_def.methods {
                        self.compile_struct_method(&struct_def.name, method)?;
                    }
                }
            }
        }

        // Fifth pass: compile non-generic function bodies
        // Comes AFTER struct methods, so generic instantiation can use them

        // ⭐ ASYNC: Declare global runtime variable if needed
        if needs_runtime_init {
            eprintln!("🔄 Declaring global async runtime variable");

            // Create global runtime variable: Runtime* __vex_global_runtime = NULL;
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let global_runtime = self.module.add_global(
                ptr_type,
                Some(inkwell::AddressSpace::default()),
                "__vex_global_runtime",
            );
            global_runtime.set_initializer(&ptr_type.const_null());

            // Store the global variable for later use
            self.global_runtime = Some(global_runtime.as_pointer_value());
            eprintln!("✅ Global runtime variable declared");
        }

        for item in &merged_program.items {
            if let Item::Function(func) = item {
                // Skip generic functions - they'll be compiled when instantiated
                // Also skip methods with generic receivers (they'll be instantiated with their struct)
                let is_on_generic_struct_compile = if let Some(ref receiver) = func.receiver {
                    // Extract struct name from receiver type
                    let struct_name = match &receiver.ty {
                        Type::Named(name) => Some(name.as_str()),
                        Type::Generic { name, .. } => Some(name.as_str()),
                        Type::Reference(inner, _) => match &**inner {
                            Type::Named(name) => Some(name.as_str()),
                            Type::Generic { name, .. } => Some(name.as_str()),
                            Type::Vec(_) => Some("Vec"),
                            Type::Box(_) => Some("Box"),
                            Type::Option(_) => Some("Option"),
                            Type::Result(_, _) => Some("Result"),
                            _ => None,
                        },
                        Type::Vec(_) => Some("Vec"),
                        Type::Box(_) => Some("Box"),
                        Type::Option(_) => Some("Option"),
                        Type::Result(_, _) => Some("Result"),
                        _ => None,
                    };

                    // Check if the struct is generic
                    struct_name
                        .and_then(|name| self.struct_ast_defs.get(name))
                        .map(|s| !s.type_params.is_empty())
                        .unwrap_or(false)
                } else {
                    false
                };

                if func.type_params.is_empty() && !is_on_generic_struct_compile {
                    eprintln!("🔨 Compiling function: {}", func.name);
                    match self.compile_function(func) {
                        Ok(_) => eprintln!("✅ Successfully compiled: {}", func.name),
                        Err(e) => {
                            eprintln!("❌ Failed to compile {}: {}", func.name, e);
                            return Err(e);
                        }
                    }
                } else if is_on_generic_struct_compile {
                    eprintln!(
                        "⏭️  Skipping compilation of method {} on generic struct",
                        func.name
                    );
                }
            }
        }

        eprintln!("✅ All functions compiled successfully");

        // ⭐ ASYNC: Verify LLVM module for correctness
        eprintln!("🔍 Verifying LLVM module...");
        if let Err(e) = self.module.verify() {
            eprintln!("❌ LLVM module verification failed:");
            eprintln!("{}", e.to_string());
            eprintln!("\n📝 Dumping LLVM IR for inspection:");
            eprintln!("{}", self.module.print_to_string().to_string());
            return Err(format!("Invalid LLVM IR generated: {}", e));
        }
        eprintln!("✅ LLVM module verification passed");

        Ok(())
    }

    /// Auto-generate op!= from op== for Eq trait implementations
    /// If a type implements op== but not op!=, we generate: op!=(rhs) = !op==(rhs)
    fn auto_generate_eq_methods(&mut self, program: &Program) -> Result<(), String> {
        use inkwell::values::BasicMetadataValueEnum;

        // Collect all types that implement Eq and have op== but not op!=
        let mut types_needing_ne = Vec::new();

        for item in &program.items {
            match item {
                Item::Struct(struct_def)
                    if struct_def.impl_traits.iter().any(|t| t.name == "Eq") =>
                {
                    let has_eq = struct_def.methods.iter().any(|m| m.name == "op==");
                    let has_ne = struct_def.methods.iter().any(|m| m.name == "op!=");

                    if has_eq && !has_ne {
                        types_needing_ne.push((struct_def.name.clone(), false));
                        // false = inline method
                    }
                }
                Item::Function(func) if func.receiver.is_some() && func.name == "op==" => {
                    // External method: fn (self: Point) op==
                    if let Some(receiver) = &func.receiver {
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

                        if let Some(type_name) = type_name {
                            // Check if op!= exists for this type
                            let has_ne = program.items.iter().any(|i| {
                                if let Item::Function(f) = i {
                                    if f.name == "op!=" && f.receiver.is_some() {
                                        if let Some(r) = &f.receiver {
                                            match &r.ty {
                                                Type::Named(n) => n == &type_name,
                                                Type::Reference(inner, _) => {
                                                    if let Type::Named(n) = &**inner {
                                                        n == &type_name
                                                    } else {
                                                        false
                                                    }
                                                }
                                                _ => false,
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            });

                            if !has_ne && !types_needing_ne.iter().any(|(t, _)| t == &type_name) {
                                types_needing_ne.push((type_name, true)); // true = external method
                            }
                        }
                    }
                }
                Item::TraitImpl(trait_impl) if trait_impl.trait_name == "Eq" => {
                    let has_eq = trait_impl.methods.iter().any(|m| m.name == "op==");
                    let has_ne = trait_impl.methods.iter().any(|m| m.name == "op!=");

                    if has_eq && !has_ne {
                        if let Type::Named(type_name) = &trait_impl.for_type {
                            types_needing_ne.push((type_name.clone(), false));
                        }
                    }
                }
                _ => {}
            }
        }

        // Generate op!= for each type
        for (type_name, is_external) in types_needing_ne {
            eprintln!(
                "🔧 Auto-generating op!= from op== for type: {} (external: {})",
                type_name, is_external
            );

            // For inline methods, use param_count=2, for external use param_count=1
            let eq_method_name = if is_external {
                // External: Point_opeq_Point_1 (has type suffix)
                // Try to find the actual method name
                let base_name = format!("{}_opeq", type_name);

                // Search for method with type suffix
                let mut found_name = None;
                for (name, _) in &self.functions {
                    if name.starts_with(&base_name) {
                        found_name = Some(name.clone());
                        break;
                    }
                }

                found_name.unwrap_or(base_name)
            } else {
                // Inline: Point_opeq_2
                format!("{}_opeq_2", type_name)
            };

            let eq_fn = self.module.get_function(&eq_method_name).ok_or_else(|| {
                format!(
                    "op== method '{}' not found for auto-generation",
                    eq_method_name
                )
            })?;

            // Create op!= function with same signature
            let ne_method_name = if is_external {
                eq_method_name.replace("_opeq", "_opne")
            } else {
                format!("{}_opne_2", type_name)
            };

            let fn_type = eq_fn.get_type();
            let ne_fn = self.module.add_function(&ne_method_name, fn_type, None);

            // ⚠️ Copy parameter attributes from op== to ensure signature compatibility
            for i in 0..eq_fn.count_params() {
                let eq_param = eq_fn
                    .get_nth_param(i)
                    .ok_or_else(|| format!("Missing parameter {} in op== function", i))?;
                let ne_param = ne_fn
                    .get_nth_param(i)
                    .ok_or_else(|| format!("Missing parameter {} in op!= function", i))?;

                // Copy struct return attribute if present
                for attr_idx in [
                    inkwell::attributes::AttributeLoc::Param(i),
                    inkwell::attributes::AttributeLoc::Return,
                ] {
                    for attr_kind in eq_fn.attributes(attr_idx) {
                        ne_fn.add_attribute(attr_idx, attr_kind);
                    }
                }

                // Preserve parameter types exactly
                ne_param.set_name(eq_param.get_name().to_str().unwrap_or("param"));
            }

            // Build function body: return !op==(rhs)
            let entry = self.context.append_basic_block(ne_fn, "entry");
            let builder = self.context.create_builder();
            builder.position_at_end(entry);

            // Get parameters (same as op==)
            let params: Vec<BasicMetadataValueEnum> =
                ne_fn.get_params().iter().map(|p| (*p).into()).collect();

            // Call op==(self, rhs)
            let eq_result = builder
                .build_call(eq_fn, &params, "eq_result")
                .map_err(|e| format!("Failed to call op== in auto-generated op!=: {}", e))?;

            let eq_val = eq_result.try_as_basic_value().unwrap_basic();

            // Negate the result: !eq_val
            let ne_val = builder
                .build_not(eq_val.into_int_value(), "ne_result")
                .map_err(|e| format!("Failed to negate op== result: {}", e))?;

            // Return !op==(rhs)
            builder
                .build_return(Some(&ne_val))
                .map_err(|e| format!("Failed to build return in auto-generated op!=: {}", e))?;

            // Register in functions map
            self.functions.insert(ne_method_name.clone(), ne_fn);

            // ⚠️ CRITICAL: Register function_def so argument handling knows parameter types
            // Clone the op== function definition and change the name to op!=
            if let Some(eq_func_def) = program.items.iter().find_map(|item| {
                if let Item::Function(f) = item {
                    if f.receiver.is_some() && f.name == "op==" {
                        if let Some(receiver) = &f.receiver {
                            let recv_type_name = match &receiver.ty {
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
                            if recv_type_name.as_ref() == Some(&type_name) {
                                return Some(f);
                            }
                        }
                    }
                }
                None
            }) {
                // Create a copy with op!= name
                let mut ne_func_def = eq_func_def.clone();
                ne_func_def.name = "op!=".to_string();

                // Register under mangled name
                self.function_defs
                    .insert(ne_method_name.clone(), ne_func_def);
            }

            eprintln!(
                "✅ Auto-generated: {} (from {})",
                ne_method_name, eq_method_name
            );
        }

        Ok(())
    }
}
