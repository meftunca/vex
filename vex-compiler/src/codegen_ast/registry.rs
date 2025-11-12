// src/codegen/registry.rs
use super::*;
use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn register_type_alias(&mut self, type_alias: &TypeAlias) -> Result<(), String> {
        // Register ALL type aliases including generic ones
        // Generic aliases will be monomorphized during type resolution
        let resolved_type = self.resolve_type(&type_alias.ty);
        self.type_aliases
            .insert(type_alias.name.clone(), resolved_type);
        
        if !type_alias.type_params.is_empty() {
            eprintln!("📋 Registered generic type alias: {} with {} params", 
                      type_alias.name, type_alias.type_params.len());
        }
        Ok(())
    }

    pub(crate) fn register_struct(&mut self, struct_def: &Struct) -> Result<(), String> {
        use super::StructDef;

        self.struct_ast_defs
            .insert(struct_def.name.clone(), struct_def.clone());

        // Register associated type bindings from struct
        if !struct_def.associated_type_bindings.is_empty() {
            self.register_associated_type_bindings(
                &struct_def.name,
                &struct_def.associated_type_bindings,
            );
        }

        if !struct_def.type_params.is_empty() {
            return Ok(());
        }

        let fields: Vec<(String, Type)> = struct_def
            .fields
            .iter()
            .map(|f| (f.name.clone(), f.ty.clone()))
            .collect();

        self.struct_defs
            .insert(struct_def.name.clone(), StructDef { fields });

        for trait_name in &struct_def.impl_traits {
            let key = (trait_name.clone(), struct_def.name.clone());
            let methods: Vec<Function> = struct_def.methods.clone();
            self.trait_impls.insert(key, methods);
        }

        Ok(())
    }

    pub(crate) fn register_enum(&mut self, enum_def: &Enum) -> Result<(), String> {
        self.enum_ast_defs
            .insert(enum_def.name.clone(), enum_def.clone());
        if !enum_def.type_params.is_empty() {
            return Ok(());
        }
        Ok(())
    }

    pub(crate) fn register_trait(&mut self, trait_def: &Trait) -> Result<(), String> {
        self.trait_defs
            .insert(trait_def.name.clone(), trait_def.clone());
        Ok(())
    }

    pub(crate) fn register_trait_impl(&mut self, trait_impl: &TraitImpl) -> Result<(), String> {
        let type_name = match &trait_impl.for_type {
            Type::Named(name) => name.clone(),
            _ => {
                let type_str = format!("{:?}", trait_impl.for_type);
                self.diagnostics.emit(Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::TYPE_MISMATCH.to_string(),
                    message: "Trait implementations only support named types".to_string(),
                    span: Span::unknown(),
                    notes: vec![format!("Cannot implement trait for type: {}", type_str)],
                    help: Some(
                        "Try implementing the trait for a named struct or enum type".to_string(),
                    ),
                    suggestion: None,
                });
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
            _ => {
                let type_str = format!("{:?}", for_type);
                self.diagnostics.emit(Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::TYPE_MISMATCH.to_string(),
                    message: "Expected named type for trait implementation".to_string(),
                    span: Span::unknown(),
                    notes: vec![format!("Got type: {}", type_str)],
                    help: Some("Trait methods can only be implemented for named types".to_string()),
                    suggestion: None,
                });
                return Err(format!("Expected named type, got: {:?}", for_type));
            }
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

    /// Check if a type implements an operator trait
    /// Returns the method name if trait is implemented
    pub(crate) fn has_operator_trait(&self, type_name: &str, trait_name: &str) -> Option<String> {
        let key = (trait_name.to_string(), type_name.to_string());
        self.trait_impls.get(&key).and_then(|methods| {
            // Operator traits have a single method with standard name
            methods.first().map(|m| m.name.clone())
        })
    }

    /// Get operator trait method name from binary op
    pub(crate) fn binary_op_to_trait(&self, op: &BinaryOp) -> (&'static str, &'static str) {
        match op {
            BinaryOp::Add => ("Add", "add"),
            BinaryOp::Sub => ("Sub", "sub"),
            BinaryOp::Mul => ("Mul", "mul"),
            BinaryOp::Div => ("Div", "div"),
            BinaryOp::Mod => ("Rem", "rem"),
            BinaryOp::Eq => ("Eq", "eq"),
            BinaryOp::NotEq => ("Ne", "ne"),
            BinaryOp::Lt => ("Lt", "lt"),
            BinaryOp::LtEq => ("Le", "le"),
            BinaryOp::Gt => ("Gt", "gt"),
            BinaryOp::GtEq => ("Ge", "ge"),
            BinaryOp::BitAnd => ("BitAnd", "bitand"),
            BinaryOp::BitOr => ("BitOr", "bitor"),
            BinaryOp::BitXor => ("BitXor", "bitxor"),
            BinaryOp::Shl => ("Shl", "shl"),
            BinaryOp::Shr => ("Shr", "shr"),
            _ => ("", ""), // Logical ops don't have traits
        }
    }

    /// Register a policy definition
    pub(crate) fn register_policy(&mut self, policy: &vex_ast::Policy) -> Result<(), String> {
        // Check for policy-trait name collision
        if self.trait_defs.contains_key(&policy.name) {
            return Err(format!(
                "Policy '{}' conflicts with existing trait of the same name",
                policy.name
            ));
        }

        // Check for duplicate policy
        if self.policy_defs.contains_key(&policy.name) {
            return Err(format!("Policy '{}' is already defined", policy.name));
        }

        eprintln!("📋 Registering policy: {}", policy.name);
        self.policy_defs.insert(policy.name.clone(), policy.clone());
        Ok(())
    }

    /// Check for trait-policy name collision when registering trait
    pub(crate) fn check_trait_policy_collision(&self, trait_name: &str) -> Result<(), String> {
        if self.policy_defs.contains_key(trait_name) {
            return Err(format!(
                "Trait '{}' conflicts with existing policy of the same name",
                trait_name
            ));
        }
        Ok(())
    }

    /// Apply policies to struct fields, merging metadata
    pub fn apply_policies_to_struct(&mut self, struct_def: &vex_ast::Struct) -> Result<(), String> {
        use crate::codegen_ast::metadata::{
            apply_policy_hierarchy_to_fields, merge_metadata, parse_metadata,
        };

        if struct_def.policies.is_empty() && !struct_def.fields.iter().any(|f| f.metadata.is_some())
        {
            return Ok(()); // No policies and no inline metadata
        }

        eprintln!("  📋 Processing metadata for struct '{}'", struct_def.name);

        // Get struct field names
        let field_names: Vec<String> = struct_def.fields.iter().map(|f| f.name.clone()).collect();

        // Collect metadata from all policies (with parent resolution)
        let mut merged_metadata: std::collections::HashMap<
            String,
            std::collections::HashMap<String, String>,
        > = std::collections::HashMap::new();

        // Step 1: Apply policies (if any)
        if !struct_def.policies.is_empty() {
            eprintln!("    ├─ Applying {} policies", struct_def.policies.len());

            for policy_name in &struct_def.policies {
                eprintln!("       ├─ Policy '{}'", policy_name);

                // Apply policy with full hierarchy (parents + current)
                let field_results =
                    apply_policy_hierarchy_to_fields(policy_name, &self.policy_defs, &field_names)?;

                // Merge into accumulated metadata
                for (field_name, field_meta, warnings) in field_results {
                    // Print warnings
                    for warning in warnings {
                        eprintln!("       ⚠️  {}", warning);
                    }

                    // Merge metadata from this policy into overall struct metadata
                    let existing = merged_metadata
                        .get(&field_name)
                        .map(|m| m as &std::collections::HashMap<String, String>);

                    let (new_merged, conflicts) = if let Some(existing_meta) = existing {
                        merge_metadata(existing_meta, &field_meta)
                    } else {
                        (field_meta.clone(), vec![])
                    };

                    if !conflicts.is_empty() {
                        eprintln!(
                            "       ⚠️  Conflicts in '{}' (policy '{}' overrides): {:?}",
                            field_name, policy_name, conflicts
                        );
                    }

                    if !new_merged.is_empty() {
                        merged_metadata.insert(field_name, new_merged);
                    }
                }
            }
        }

        // Step 2: Apply inline metadata (overrides policy metadata)
        let has_inline = struct_def.fields.iter().any(|f| f.metadata.is_some());
        if has_inline {
            eprintln!("    ├─ Applying inline metadata (overrides policies)");

            for field in &struct_def.fields {
                if let Some(inline_metadata_str) = &field.metadata {
                    eprintln!(
                        "       ├─ Field '{}': `{}`",
                        field.name, inline_metadata_str
                    );

                    // Parse inline metadata
                    match parse_metadata(inline_metadata_str) {
                        Ok(inline_meta) => {
                            // Merge with existing policy metadata (inline overrides)
                            let existing = merged_metadata
                                .get(&field.name)
                                .map(|m| m as &std::collections::HashMap<String, String>);

                            let (new_merged, conflicts) = if let Some(existing_meta) = existing {
                                merge_metadata(existing_meta, &inline_meta)
                            } else {
                                (inline_meta.clone(), vec![])
                            };

                            if !conflicts.is_empty() {
                                eprintln!("       ⚠️  Inline overrides: {:?}", conflicts);
                            }

                            merged_metadata.insert(field.name.clone(), new_merged);
                        }
                        Err(e) => {
                            eprintln!(
                                "       ❌ Failed to parse inline metadata for '{}': {}",
                                field.name, e
                            );
                        }
                    }
                }
            }
        }

        eprintln!(
            "    └─ ✅ Final metadata for {} fields",
            merged_metadata.len()
        );

        // Store merged_metadata in struct registry for runtime access
        if !merged_metadata.is_empty() {
            self.struct_metadata
                .insert(struct_def.name.clone(), merged_metadata.clone());

            eprintln!(
                "       💾 Stored metadata for struct '{}' ({} fields):",
                struct_def.name,
                merged_metadata.len()
            );

            for (field_name, field_meta) in &merged_metadata {
                let meta_str: Vec<String> = field_meta
                    .iter()
                    .map(|(k, v)| format!("{}:\"{}\"", k, v))
                    .collect();
                eprintln!("          • {} → {{ {} }}", field_name, meta_str.join(", "));
            }
        }

        Ok(())
    }
}
