// generics/structs.rs
// Generic struct instantiation

use super::super::*;
use std::collections::HashMap;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn instantiate_generic_struct(
        &mut self,
        struct_name: &str,
        type_args: &[Type],
    ) -> Result<String, String> {
        // uses MAX_GENERIC_DEPTH from super
        for type_arg in type_args {
            let depth = self.get_generic_depth(type_arg);
            if depth > super::super::MAX_GENERIC_DEPTH {
                return Err(format!(
                    "Generic type nesting too deep (depth {}, max {}): {}",
                    depth,
                    super::super::MAX_GENERIC_DEPTH,
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

        // Allow fewer type args if remaining params have defaults
        if type_args.len() > struct_ast.type_params.len() {
            return Err(format!(
                "Struct '{}' expects at most {} type parameters, got {}",
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

        // Build type substitution map, using defaults for missing type args
        let mut type_subst = HashMap::new();
        for (i, param) in struct_ast.type_params.iter().enumerate() {
            let concrete_type = if let Some(provided_type) = type_args.get(i) {
                provided_type.clone()
            } else if let Some(ref default_type) = param.default_type {
                // Use default type (Self will be substituted later if needed)
                default_type.clone()
            } else {
                return Err(format!(
                    "Missing type argument for parameter '{}' in struct '{}'",
                    param.name, struct_name
                ));
            };
            type_subst.insert(param.name.clone(), concrete_type);
        }

        // Rebuild mangled name with all type args (including defaults)
        let all_type_args: Vec<Type> = struct_ast
            .type_params
            .iter()
            .map(|p| {
                type_subst
                    .get(&p.name)
                    .cloned()
                    .expect("type_subst should have all params")
            })
            .collect();
        let all_type_arg_strings: Vec<String> =
            all_type_args.iter().map(|t| self.type_to_string(t)).collect();

        // Include const params in mangled name if present
        let mangled_name = if struct_ast.const_params.is_empty() {
            format!("{}_{}", struct_name, all_type_arg_strings.join("_"))
        } else {
            // For now, const params are not specialized - they're just in signature
            // Future: Add const values to mangled name when instantiated with concrete values
            format!("{}_{}", struct_name, all_type_arg_strings.join("_"))
        };

        let specialized_fields: Vec<(String, Type)> = struct_ast
            .fields
            .iter()
            .map(|f| {
                let substituted_ty = self.substitute_type(&f.ty, &type_subst);
                (f.name.clone(), substituted_ty)
            })
            .collect();

        // StructDef type lives in super (as in the original file's usage)
        use super::super::StructDef;
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
}