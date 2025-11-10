// associated_types.rs
// Associated types resolution and substitution

use super::ASTCodeGen;
use vex_ast::Type;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Resolve an associated type (Self.Item) to its concrete type
    ///
    /// Examples:
    /// - In Counter impl Iterator: Self.Item → i32
    /// - In VecIterator: Self.Item → T (generic parameter)
    pub(crate) fn resolve_associated_type(
        &self,
        self_type: &Type,
        assoc_name: &str,
    ) -> Result<Type, String> {
        // 1. Get the concrete type name from Self or explicit type
        let type_name = match self_type {
            Type::SelfType => {
                // In impl context, Self refers to the current struct
                // Try to infer from current context
                // For now, we'll need to track this in method compilation
                return Err(format!(
                    "Cannot resolve Self.{} - Self type not in context",
                    assoc_name
                ));
            }
            Type::Named(name) => name.clone(),
            Type::Generic { name, .. } => name.clone(),
            _ => {
                return Err(format!(
                    "Invalid base type for associated type: {:?}",
                    self_type
                ))
            }
        };

        // 2. Look up the associated type binding
        let key = (type_name.clone(), assoc_name.to_string());
        if let Some(bound_type) = self.associated_type_bindings.get(&key) {
            eprintln!(
                "✅ Resolved {}.{} → {:?}",
                type_name, assoc_name, bound_type
            );
            return Ok(bound_type.clone());
        }

        // 3. Check if it's defined in the struct's AST definition
        if let Some(struct_def) = self.struct_ast_defs.get(&type_name) {
            for (name, bound_type) in &struct_def.associated_type_bindings {
                if name == assoc_name {
                    eprintln!(
                        "✅ Resolved {}.{} → {:?} (from struct def)",
                        type_name, name, bound_type
                    );
                    return Ok(bound_type.clone());
                }
            }
        }

        Err(format!(
            "Associated type {}.{} not found (type not implementing trait with this associated type)",
            type_name, assoc_name
        ))
    }

    /// Register associated type bindings from a struct's trait implementation
    /// Called when registering a struct that implements traits with associated types
    pub(crate) fn register_associated_type_bindings(
        &mut self,
        type_name: &str,
        bindings: &[(String, Type)],
    ) {
        for (assoc_name, concrete_type) in bindings {
            let key = (type_name.to_string(), assoc_name.clone());
            eprintln!(
                "📝 Registering associated type: {}.{} = {:?}",
                type_name, assoc_name, concrete_type
            );
            self.associated_type_bindings
                .insert(key, concrete_type.clone());
        }
    }

    /// Substitute all associated types in a type with their concrete bindings
    /// Used when compiling trait methods with concrete implementations
    pub(crate) fn substitute_associated_types(
        &self,
        ty: &Type,
        type_name: &str,
    ) -> Result<Type, String> {
        match ty {
            Type::AssociatedType {
                self_type,
                name: assoc_name,
            } => {
                // Check if self_type is Self
                if matches!(self_type.as_ref(), Type::SelfType) {
                    // Resolve Self.Item using type_name context
                    let key = (type_name.to_string(), assoc_name.clone());
                    if let Some(bound_type) = self.associated_type_bindings.get(&key) {
                        return Ok(bound_type.clone());
                    }
                }

                // Otherwise resolve normally
                self.resolve_associated_type(self_type, assoc_name)
            }

            // Recursively substitute in generic types
            Type::Generic { name, type_args } => {
                let mut new_args = Vec::new();
                for arg in type_args {
                    new_args.push(self.substitute_associated_types(arg, type_name)?);
                }
                Ok(Type::Generic {
                    name: name.clone(),
                    type_args: new_args,
                })
            }

            Type::Option(inner) => {
                let new_inner = self.substitute_associated_types(inner, type_name)?;
                Ok(Type::Option(Box::new(new_inner)))
            }

            Type::Result(ok, err) => {
                let new_ok = self.substitute_associated_types(ok, type_name)?;
                let new_err = self.substitute_associated_types(err, type_name)?;
                Ok(Type::Result(Box::new(new_ok), Box::new(new_err)))
            }

            // Other types don't contain associated types
            _ => Ok(ty.clone()),
        }
    }
}
