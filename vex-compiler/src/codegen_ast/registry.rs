// src/codegen/registry.rs
use super::*;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn register_type_alias(&mut self, type_alias: &TypeAlias) -> Result<(), String> {
        if !type_alias.type_params.is_empty() {
            return Ok(());
        }
        let resolved_type = self.resolve_type(&type_alias.ty);
        self.type_aliases.insert(type_alias.name.clone(), resolved_type);
        Ok(())
    }

    pub(crate) fn register_struct(&mut self, struct_def: &Struct) -> Result<(), String> {
        use super::StructDef;

        self.struct_ast_defs.insert(struct_def.name.clone(), struct_def.clone());

        if !struct_def.type_params.is_empty() {
            return Ok(());
        }

        let fields: Vec<(String, Type)> = struct_def
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.ty.clone()))
            .collect();

        self.struct_defs.insert(struct_def.name.clone(), StructDef { fields });

        for trait_name in &struct_def.impl_traits {
            let key = (trait_name.clone(), struct_def.name.clone());
            let methods: Vec<Function> = struct_def.methods.clone();
            self.trait_impls.insert(key, methods);
        }

        Ok(())
    }

    pub(crate) fn register_enum(&mut self, enum_def: &Enum) -> Result<(), String> {
        self.enum_ast_defs.insert(enum_def.name.clone(), enum_def.clone());
        if !enum_def.type_params.is_empty() { return Ok(()); }
        Ok(())
    }

    pub(crate) fn register_trait(&mut self, trait_def: &Trait) -> Result<(), String> {
        self.trait_defs.insert(trait_def.name.clone(), trait_def.clone());
        Ok(())
    }

    pub(crate) fn register_trait_impl(&mut self, trait_impl: &TraitImpl) -> Result<(), String> {
        let type_name = match &trait_impl.for_type {
            Type::Named(name) => name.clone(),
            _ => {
                return Err(format!(
                    "Trait implementations currently only support named types, got: {:?}",
                    trait_impl.for_type
                ));
            }
        };

        let key = (trait_impl.trait_name.clone(), type_name.clone());
        self.trait_impls.insert(key, trait_impl.methods.clone());

        for method in &trait_impl.methods {
            self.declare_trait_impl_method(&trait_impl.trait_name, &trait_impl.for_type, method)?;
        }
        Ok(())
    }

    pub(crate) fn declare_trait_impl_method(
        &mut self,
        trait_name: &str,
        for_type: &Type,
        method: &Function,
    ) -> Result<(), String> {
        let type_name = match for_type {
            Type::Named(name) => name,
            _ => return Err(format!("Expected named type, got: {:?}", for_type)),
        };

        let mangled_name = format!("{}_{}_{}", type_name, trait_name, method.name);

        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();
        if let Some(ref receiver) = method.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }
        for param in &method.params {
            param_types.push(self.ast_type_to_llvm(&param.ty).into());
        }

        let ret_type = if let Some(ref ty) = method.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            inkwell::types::BasicTypeEnum::IntType(self.context.i32_type())
        };

        use inkwell::types::BasicTypeEnum;
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
}
