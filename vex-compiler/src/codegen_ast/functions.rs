// Function and program code generation

use super::ASTCodeGen;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::FunctionValue;
use std::collections::HashMap;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub fn compile_program(&mut self, program: &Program) -> Result<(), String> {
        // First pass: register type aliases, structs, enums, traits, and extern blocks
        for item in &program.items {
            if let Item::TypeAlias(type_alias) = item {
                self.register_type_alias(type_alias)?;
            } else if let Item::Struct(struct_def) = item {
                self.register_struct(struct_def)?;
            } else if let Item::Enum(enum_def) = item {
                self.register_enum(enum_def)?;
            } else if let Item::Trait(trait_def) = item {
                self.register_trait(trait_def)?;
            } else if let Item::TraitImpl(trait_impl) = item {
                self.register_trait_impl(trait_impl)?;
            } else if let Item::ExternBlock(extern_block) = item {
                self.compile_extern_block(extern_block)?;
            }
        }

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

        // Third pass: compile non-generic function bodies
        for item in &program.items {
            if let Item::Function(func) = item {
                // Skip generic functions - they'll be compiled when instantiated
                if func.type_params.is_empty() {
                    self.compile_function(func)?;
                }
            }
        }

        // Fourth pass: compile trait impl method bodies
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

        // Fifth pass: compile inline struct method bodies (new trait system v1.3)
        for item in &program.items {
            if let Item::Struct(struct_def) = item {
                if struct_def.type_params.is_empty() {
                    for method in &struct_def.methods {
                        self.compile_struct_method(&struct_def.name, method)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Register a type alias
    fn register_type_alias(&mut self, type_alias: &TypeAlias) -> Result<(), String> {
        // For now, only support non-generic type aliases
        // Generic type aliases would need monomorphization like generic structs
        if !type_alias.type_params.is_empty() {
            // Just ignore generic type aliases for now
            return Ok(());
        }

        // Resolve the type (in case it references other aliases)
        let resolved_type = self.resolve_type(&type_alias.ty);

        self.type_aliases
            .insert(type_alias.name.clone(), resolved_type);

        Ok(())
    }

    /// Register a struct definition in the struct registry
    fn register_struct(&mut self, struct_def: &Struct) -> Result<(), String> {
        use super::StructDef;

        // Store AST definition (for generic structs)
        self.struct_ast_defs
            .insert(struct_def.name.clone(), struct_def.clone());

        // Skip generic structs - they'll be instantiated on-demand
        if !struct_def.type_params.is_empty() {
            return Ok(());
        }

        // Register non-generic struct
        let fields: Vec<(String, Type)> = struct_def
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.ty.clone()))
            .collect();

        self.struct_defs
            .insert(struct_def.name.clone(), StructDef { fields });

        Ok(())
    }

    /// Register an enum definition
    fn register_enum(&mut self, enum_def: &Enum) -> Result<(), String> {
        // Store AST definition (for generic enums)
        self.enum_ast_defs
            .insert(enum_def.name.clone(), enum_def.clone());

        // Skip generic enums - they'll be instantiated on-demand like generic structs
        if !enum_def.type_params.is_empty() {
            return Ok(());
        }

        // For non-generic enums, we'll generate constructor functions later
        // Enums are represented as tagged unions in LLVM
        // For now, just store the definition

        Ok(())
    }

    /// Register a trait definition
    fn register_trait(&mut self, trait_def: &Trait) -> Result<(), String> {
        // Store trait definition for type checking and method resolution
        self.trait_defs
            .insert(trait_def.name.clone(), trait_def.clone());
        Ok(())
    }

    /// Register a trait implementation
    fn register_trait_impl(&mut self, trait_impl: &TraitImpl) -> Result<(), String> {
        // Extract type name from for_type
        let type_name = match &trait_impl.for_type {
            Type::Named(name) => name.clone(),
            _ => {
                return Err(format!(
                    "Trait implementations currently only support named types, got: {:?}",
                    trait_impl.for_type
                ))
            }
        };

        // Store trait impl methods: (TraitName, TypeName) -> Vec<Function>
        let key = (trait_impl.trait_name.clone(), type_name.clone());
        self.trait_impls.insert(key, trait_impl.methods.clone());

        // Declare trait impl methods (with mangling)
        for method in &trait_impl.methods {
            self.declare_trait_impl_method(&trait_impl.trait_name, &trait_impl.for_type, method)?;
        }

        Ok(())
    }

    /// Declare a trait impl method with proper name mangling
    fn declare_trait_impl_method(
        &mut self,
        trait_name: &str,
        for_type: &Type,
        method: &Function,
    ) -> Result<(), String> {
        let type_name = match for_type {
            Type::Named(name) => name,
            _ => return Err(format!("Expected named type, got: {:?}", for_type)),
        };

        // Mangle name: TypeName_TraitName_methodName
        // Example: Point_Printable_print
        let mangled_name = format!("{}_{}_{}", type_name, trait_name, method.name);

        // Build parameter types (receiver becomes first parameter)
        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();

        if let Some(ref receiver) = method.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }

        for param in &method.params {
            param_types.push(self.ast_type_to_llvm(&param.ty).into());
        }

        // Build return type
        let ret_type = if let Some(ref ty) = method.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            inkwell::types::BasicTypeEnum::IntType(self.context.i32_type())
        };

        // Create function type and declare function
        use inkwell::types::BasicTypeEnum;
        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
        };

        let fn_val = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name.clone(), fn_val);

        // Store function def for later compilation
        let mut mangled_method = method.clone();
        mangled_method.name = mangled_name.clone();
        self.function_defs.insert(mangled_name, mangled_method);

        Ok(())
    }

    /// Compile a trait impl method body
    fn compile_trait_impl_method(
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

    /// Declare an inline struct method (new trait system v1.3)
    /// Inline methods are declared directly inside struct body: struct Foo { ... fn bar() {...} }
    fn declare_struct_method(
        &mut self,
        struct_name: &str,
        method: &Function,
    ) -> Result<(), String> {
        // Mangle name: StructName_methodName
        // Example: FileLogger_log
        let mangled_name = format!("{}_{}", struct_name, method.name);

        // Build parameter types (receiver becomes first parameter)
        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();

        if let Some(ref receiver) = method.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }

        for param in &method.params {
            param_types.push(self.ast_type_to_llvm(&param.ty).into());
        }

        // Build return type
        let ret_type = if let Some(ref ty) = method.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            inkwell::types::BasicTypeEnum::IntType(self.context.i32_type())
        };

        // Create function type and declare function
        use inkwell::types::BasicTypeEnum;
        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
        };

        let fn_val = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name.clone(), fn_val);

        // Store function def for later compilation
        let mut mangled_method = method.clone();
        mangled_method.name = mangled_name.clone();
        self.function_defs.insert(mangled_name, mangled_method);

        Ok(())
    }

    /// Compile an inline struct method body
    fn compile_struct_method(
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

    /// Generate constructor functions for enum variants
    /// For C-style enums: Color::Red -> Color_Red() returns i32 (tag value)
    /// For data-carrying enums: Option::Some(T) -> Option_Some(value: T) returns struct
    fn generate_enum_constructors(&mut self, enum_def: &Enum) -> Result<(), String> {
        // For now, generate simple tag-returning functions for C-style enums
        // Data-carrying enums will need more complex struct-based representation

        for (tag_index, variant) in enum_def.variants.iter().enumerate() {
            let constructor_name = format!("{}_{}", enum_def.name, variant.name);

            if variant.data.is_none() {
                // Unit variant: just return tag value (i32)
                let i32_type = self.context.i32_type();
                let fn_type = i32_type.fn_type(&[], false);
                let function = self.module.add_function(&constructor_name, fn_type, None);

                // Create function body
                let entry = self.context.append_basic_block(function, "entry");
                self.builder.position_at_end(entry);

                // Return tag value
                let tag_value = i32_type.const_int(tag_index as u64, false);
                self.builder
                    .build_return(Some(&tag_value))
                    .map_err(|e| format!("Failed to build return: {}", e))?;

                // Store function for later use
                self.functions.insert(constructor_name, function);
            }
            // TODO: Handle data-carrying variants (Option::Some(T))
        }

        Ok(())
    }

    /// Declare a function signature (without body)
    pub(crate) fn declare_function(
        &mut self,
        func: &Function,
    ) -> Result<FunctionValue<'ctx>, String> {
        // Determine the function name (with mangling for methods)
        let fn_name = if let Some(ref receiver) = func.receiver {
            // Extract type name from receiver
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
                    )
                }
            };

            // Mangle method name: TypeName_methodName
            format!("{}_{}", type_name, func.name)
        } else {
            func.name.clone()
        };

        // Build parameter types (receiver becomes first parameter if present)
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();

        if let Some(ref receiver) = func.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }

        for param in &func.params {
            param_types.push(self.ast_type_to_llvm(&param.ty).into());
        }

        // Build return type
        let ret_type = if let Some(ref ty) = func.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            BasicTypeEnum::IntType(self.context.i32_type()) // Default to i32
        };

        // Create function type
        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            _ => {
                return Err(format!(
                    "Unsupported return type for function {}",
                    func.name
                ))
            }
        };

        // Add function to module (use mangled name)
        let fn_val = self.module.add_function(&fn_name, fn_type, None);
        self.functions.insert(fn_name.clone(), fn_val);

        Ok(fn_val)
    }

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
                    )
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
            // v0.9: Function parameters are always mutable (local binding)
            let alloca = self.create_entry_block_alloca(&param.name, &param.ty, true)?;
            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store parameter: {}", e))?;
            self.variables.insert(param.name.clone(), alloca);
            self.variable_types.insert(param.name.clone(), param_type);

            // Track struct parameters
            if let Type::Named(struct_name) = &param.ty {
                if self.struct_defs.contains_key(struct_name) {
                    self.variable_struct_names
                        .insert(param.name.clone(), struct_name.clone());
                }
            }
        }

        // Compile function body
        self.compile_block(&func.body)?;

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

    /// Instantiate a generic function with concrete types
    /// Creates a monomorphized version: identity<i32> -> identity_i32
    pub(crate) fn instantiate_generic_function(
        &mut self,
        func_def: &Function,
        type_args: &[Type],
    ) -> Result<FunctionValue<'ctx>, String> {
        // Generate mangled name: identity_i32, pair_i32_f64
        let type_names: Vec<String> = type_args.iter().map(|t| self.type_to_string(t)).collect();
        let mangled_name = format!("{}_{}", func_def.name, type_names.join("_"));

        // Check if already instantiated
        if let Some(fn_val) = self.functions.get(&mangled_name) {
            return Ok(*fn_val);
        }

        // Create type substitution map: T -> i32, U -> f64
        let mut type_subst = HashMap::new();
        for (i, type_param) in func_def.type_params.iter().enumerate() {
            if let Some(concrete_type) = type_args.get(i) {
                type_subst.insert(type_param.clone(), concrete_type.clone());
            }
        }

        // Substitute types in function signature
        let subst_func = self.substitute_types_in_function(func_def, &type_subst)?;

        // Save current compilation state INCLUDING builder position
        let saved_current_function = self.current_function;
        let saved_insert_block = self.builder.get_insert_block();
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_variable_types = std::mem::take(&mut self.variable_types);
        let saved_variable_struct_names = std::mem::take(&mut self.variable_struct_names);

        // Declare and compile the specialized function
        let fn_val = self.declare_function(&subst_func)?;
        self.functions.insert(mangled_name.clone(), fn_val);

        // Compile body
        self.compile_function(&subst_func)?;

        // Restore compilation state INCLUDING builder position
        self.current_function = saved_current_function;
        self.variables = saved_variables;
        self.variable_types = saved_variable_types;
        self.variable_struct_names = saved_variable_struct_names;

        // Restore builder position to where we were
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        Ok(fn_val)
    }

    /// Instantiate a generic struct with concrete types
    /// Creates a monomorphized version: Box<i32> -> Box_i32
    pub(crate) fn instantiate_generic_struct(
        &mut self,
        struct_name: &str,
        type_args: &[Type],
    ) -> Result<String, String> {
        use super::StructDef;

        // Check if already instantiated (memoization)
        let type_arg_strings: Vec<String> =
            type_args.iter().map(|t| self.type_to_string(t)).collect();
        let cache_key = (struct_name.to_string(), type_arg_strings.clone());

        if let Some(mangled_name) = self.generic_instantiations.get(&cache_key) {
            return Ok(mangled_name.clone());
        }

        // Get the generic struct definition
        let struct_ast = self
            .struct_ast_defs
            .get(struct_name)
            .cloned()
            .ok_or_else(|| format!("Generic struct '{}' not found", struct_name))?;

        // Check type parameter count
        if struct_ast.type_params.len() != type_args.len() {
            return Err(format!(
                "Struct '{}' expects {} type parameters, got {}",
                struct_name,
                struct_ast.type_params.len(),
                type_args.len()
            ));
        }

        // Create type substitution map: T -> i32, U -> f64
        let mut type_subst = HashMap::new();
        for (param, arg) in struct_ast.type_params.iter().zip(type_args.iter()) {
            type_subst.insert(param.clone(), arg.clone());
        }

        // Generate mangled name: Box<i32> -> Box_i32
        let mangled_name = format!("{}_{}", struct_name, type_arg_strings.join("_"));

        // Substitute types in struct fields
        let specialized_fields: Vec<(String, Type)> = struct_ast
            .fields
            .iter()
            .map(|f| {
                let substituted_ty = self.substitute_type(&f.ty, &type_subst);
                (f.name.clone(), substituted_ty)
            })
            .collect();

        // Register the specialized struct
        self.struct_defs.insert(
            mangled_name.clone(),
            StructDef {
                fields: specialized_fields,
            },
        );

        // Cache the instantiation
        self.generic_instantiations
            .insert(cache_key, mangled_name.clone());

        Ok(mangled_name)
    }

    /// Infer type arguments from function call arguments
    /// Simple version: just infer from argument types
    pub(crate) fn infer_type_args_from_call(
        &mut self,
        _func_def: &Function,
        args: &[Expression],
    ) -> Result<Vec<Type>, String> {
        // Infer type from each argument - matches order of type params
        let mut type_args = Vec::new();

        for arg in args {
            let arg_type = self.infer_expression_type(arg)?;
            type_args.push(arg_type);
        }

        Ok(type_args)
    }

    /// Substitute type parameters in a function
    fn substitute_types_in_function(
        &self,
        func: &Function,
        type_subst: &HashMap<String, Type>,
    ) -> Result<Function, String> {
        let mut new_func = func.clone();

        // Clear type parameters (no longer generic)
        new_func.type_params.clear();

        // Substitute in parameter types
        for param in &mut new_func.params {
            param.ty = self.substitute_type(&param.ty, type_subst);
        }

        // Substitute in return type
        if let Some(ret_ty) = &new_func.return_type {
            new_func.return_type = Some(self.substitute_type(ret_ty, type_subst));
        }

        // Update function name with mangled name
        let type_names: Vec<String> = type_subst
            .values()
            .map(|t| self.type_to_string(t))
            .collect();
        new_func.name = format!("{}_{}", func.name, type_names.join("_"));

        Ok(new_func)
    }

    /// Compile async function (simplified - no state machine yet)
    /// For now, async functions execute synchronously but register with runtime
    fn compile_async_function(&mut self, func: &Function) -> Result<(), String> {
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
