// src/codegen/program.rs
use super::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        eprintln!(
            "📋 compile_program: {} total items in AST",
            program.items.len()
        );

        // Check if any async functions exist in the program
        let has_async = program
            .items
            .iter()
            .any(|item| matches!(item, Item::Function(f) if f.is_async));

        if has_async {
            eprintln!("🔄 Async functions detected - runtime will be initialized in main");
        }

        // Initialize trait bounds checker
        use crate::trait_bounds_checker::TraitBoundsChecker;
        let mut trait_checker = TraitBoundsChecker::new();
        trait_checker.initialize(program);
        self.trait_bounds_checker = Some(trait_checker);
        eprintln!("✅ Trait bounds checker initialized");

        // First pass: register types, constants, and function signatures
        for item in &program.items {
            if let Item::TypeAlias(type_alias) = item {
                self.register_type_alias(type_alias)?;
            } else if let Item::Struct(struct_def) = item {
                self.register_struct(struct_def)?;
            } else if let Item::Enum(enum_def) = item {
                self.register_enum(enum_def)?;
            } else if let Item::Policy(policy) = item {
                self.register_policy(policy)?;
            } else if let Item::Trait(trait_def) = item {
                // Check for policy collision before registering trait
                self.check_trait_policy_collision(&trait_def.name)?;
                self.register_trait(trait_def)?;
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
        for item in &program.items {
            if let Item::Struct(struct_def) = item {
                self.apply_policies_to_struct(struct_def)?;
            }
        }

        // Check for circular dependencies in struct definitions
        self.check_circular_struct_dependencies(&program)?;

        // Second pass: store and declare non-generic functions
        for item in &program.items {
            if let Item::Function(func) = item {
                self.function_defs.insert(func.name.clone(), func.clone());

                // Skip generic functions for now - they'll be instantiated on-demand
                if func.type_params.is_empty() {
                    self.declare_function(func)?;
                }
            }
        }

        // Declare inline struct methods (new trait system v1.3)
        for item in &program.items {
            if let Item::Struct(struct_def) = item {
                if struct_def.type_params.is_empty() {
                    for method in &struct_def.methods {
                        self.declare_struct_method(&struct_def.name, method)?;
                    }
                }
            }
        }

        // Generate enum constructor functions for non-generic enums
        for item in &program.items {
            if let Item::Enum(enum_def) = item {
                if enum_def.type_params.is_empty() {
                    self.generate_enum_constructors(enum_def)?;
                }
            }
        }

        // Third pass: compile trait impl method bodies
        // MUST come before function bodies, as functions may call trait methods
        for item in &program.items {
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
        for item in &program.items {
            if let Item::Struct(struct_def) = item {
                eprintln!(
                    "   Struct: {} (generic: {}, methods: {})",
                    struct_def.name,
                    !struct_def.type_params.is_empty(),
                    struct_def.methods.len()
                );
                if struct_def.type_params.is_empty() {
                    for method in &struct_def.methods {
                        eprintln!("      - Compiling method: {}", method.name);
                        self.compile_struct_method(&struct_def.name, method)?;
                    }
                }
            }
        }

        // Fifth pass: compile non-generic function bodies
        // Comes AFTER struct methods, so generic instantiation can use them
        for item in &program.items {
            if let Item::Function(func) = item {
                // Skip generic functions - they'll be compiled when instantiated
                if func.type_params.is_empty() {
                    self.compile_function(func)?;
                }
            }
        }

        Ok(())
    }
}
