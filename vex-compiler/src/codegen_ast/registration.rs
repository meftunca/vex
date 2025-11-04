// Type registration: register structs, enums, traits, type aliases

use super::ASTCodeGen;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Register a type alias
    pub(crate) fn register_type_alias(&mut self, type_alias: &TypeAlias) -> Result<(), String> {
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
    pub(crate) fn register_struct(&mut self, struct_def: &Struct) -> Result<(), String> {
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

        // Register inline trait implementations
        // This allows default trait methods to be found
        for trait_name in &struct_def.impl_traits {
            let key = (trait_name.clone(), struct_def.name.clone());
            // Convert inline methods to Function format
            let methods: Vec<Function> = struct_def.methods.clone();
            self.trait_impls.insert(key, methods);
        }

        Ok(())
    }

    /// Register an enum definition
    pub(crate) fn register_enum(&mut self, enum_def: &Enum) -> Result<(), String> {
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
    pub(crate) fn register_trait(&mut self, trait_def: &Trait) -> Result<(), String> {
        // Store trait definition for type checking and method resolution
        self.trait_defs
            .insert(trait_def.name.clone(), trait_def.clone());
        Ok(())
    }

    /// Register a trait implementation
    pub(crate) fn register_trait_impl(&mut self, trait_impl: &TraitImpl) -> Result<(), String> {
        // Extract type name from for_type
        let type_name = match &trait_impl.for_type {
            Type::Named(name) => name.clone(),
            _ => {
                return Err(format!(
                    "Trait implementations currently only support named types, got: {:?}",
                    trait_impl.for_type
                ));
            }
        };

        // Store trait impl methods: (TraitName, TypeName) -> Vec<Function>
        let key = (trait_impl.trait_name.clone(), type_name.clone());
        self.trait_impls.insert(key, trait_impl.methods.clone());

        // Declare trait impl methods (with mangling)
        use super::declaration;
        for method in &trait_impl.methods {
            declaration::ASTCodeGen::declare_trait_impl_method(
                self,
                &trait_impl.trait_name,
                &trait_impl.for_type,
                method,
            )?;
        }
        
        Ok(())
    }
}

