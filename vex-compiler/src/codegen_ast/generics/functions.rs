// generics/functions.rs
// Generic function instantiation

use super::super::*;
use std::collections::HashMap;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn instantiate_generic_function(
        &mut self,
        func_def: &Function,
        type_args: &[Type],
    ) -> Result<inkwell::values::FunctionValue<'ctx>, String> {
        let type_names: Vec<String> = type_args.iter().map(|t| self.type_to_string(t)).collect();

        // Include const params in mangled name if present
        let mangled_name = if func_def.const_params.is_empty() {
            format!("{}_{}", func_def.name, type_names.join("_"))
        } else {
            // For now, const params are not specialized - they're just in signature
            // Future: Add const values to mangled name when instantiated with concrete values
            format!("{}_{}", func_def.name, type_names.join("_"))
        };

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

            // ⭐ NEW: Check where clause constraints
            if !func_def.where_clause.is_empty() {
                use crate::trait_bounds_checker::TraitBoundsChecker;
                let type_substitutions =
                    TraitBoundsChecker::build_type_substitutions(&func_def.type_params, type_args);
                checker.check_where_clause(&func_def.where_clause, &type_substitutions)?;
                eprintln!(
                    "✅ Where clause validated for {}::<{}>",
                    func_def.name,
                    type_names.join(", ")
                );
            }
        }

        // Build type substitution map, using defaults for missing type args
        let mut type_subst = HashMap::new();
        for (i, type_param) in func_def.type_params.iter().enumerate() {
            let concrete_type = if let Some(provided_type) = type_args.get(i) {
                provided_type.clone()
            } else if let Some(ref default_type) = type_param.default_type {
                // Use default type (Self will be substituted later if needed)
                default_type.clone()
            } else {
                return Err(format!(
                    "Missing type argument for parameter '{}' in function '{}'",
                    type_param.name, func_def.name
                ));
            };
            type_subst.insert(type_param.name.clone(), concrete_type);
        }

        let subst_func = self.substitute_types_in_function(func_def, &type_subst)?;

        // ⭐ NEW: If return type is a generic struct, instantiate it first
        if let Some(ref ret_type) = subst_func.return_type {
            if let Type::Named(struct_name) = ret_type {
                // Check if this is a monomorphized generic struct (e.g., Container_i32)
                if struct_name.contains('_') {
                    // Extract base name and type args from mangled name
                    let parts: Vec<&str> = struct_name.split('_').collect();
                    if parts.len() >= 2 {
                        let base_name = parts[0];
                        // Check if base struct is generic
                        if let Some(base_struct) = self.struct_ast_defs.get(base_name) {
                            if !base_struct.type_params.is_empty() {
                                // This is a monomorphized generic struct, ensure it exists
                                if !self.struct_defs.contains_key(struct_name) {
                                    // Reconstruct type args from mangled name
                                    let type_arg_str = parts[1..].join("_");
                                    eprintln!(
                                        "🔧 Instantiating return type struct {} from {}",
                                        struct_name, type_arg_str
                                    );
                                    // Parse ALL Vex types from mangled name
                                    let concrete_type_args: Vec<Type> = parts[1..]
                                        .iter()
                                        .filter_map(|&part| match part {
                                            // Integer types
                                            "i8" => Some(Type::I8),
                                            "i16" => Some(Type::I16),
                                            "i32" => Some(Type::I32),
                                            "i64" => Some(Type::I64),
                                            "i128" => Some(Type::I128),
                                            "u8" => Some(Type::U8),
                                            "u16" => Some(Type::U16),
                                            "u32" => Some(Type::U32),
                                            "u64" => Some(Type::U64),
                                            "u128" => Some(Type::U128),
                                            // Float types
                                            "f16" => Some(Type::F16),
                                            "f32" => Some(Type::F32),
                                            "f64" => Some(Type::F64),
                                            // Other primitives
                                            "bool" => Some(Type::Bool),
                                            "string" => Some(Type::String),
                                            "byte" => Some(Type::Byte),
                                            // User-defined types (if not matched above)
                                            _ => {
                                                // Check if it's a known user struct
                                                if self.struct_ast_defs.contains_key(part) {
                                                    Some(Type::Named(part.to_string()))
                                                } else {
                                                    None
                                                }
                                            }
                                        })
                                        .collect();

                                    if !concrete_type_args.is_empty() {
                                        let _ = self.instantiate_generic_struct(
                                            base_name,
                                            &concrete_type_args,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let saved_current_function = self.current_function;
        let saved_insert_block = self.builder.get_insert_block();
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_variable_types = std::mem::take(&mut self.variable_types);
        let saved_variable_struct_names = std::mem::take(&mut self.variable_struct_names);

        // ⭐ CRITICAL: Store type substitution map for use during compilation
        // This allows infer_expression_type() to resolve generic parameters like T → I32
        // Example: When compiling "if a < b", identifiers "a" and "b" need concrete types
        let saved_type_subst =
            std::mem::replace(&mut self.active_type_substitutions, type_subst.clone());

        // ⭐ NEW: Store instantiated function in function_defs for type inference
        self.function_defs
            .insert(mangled_name.clone(), subst_func.clone());

        let fn_val = self.declare_function(&subst_func)?;
        self.functions.insert(mangled_name.clone(), fn_val);

        self.compile_function(&subst_func)?;

        // ⭐ CRITICAL: Restore previous type substitution context
        self.active_type_substitutions = saved_type_subst;

        self.current_function = saved_current_function;
        self.variables = saved_variables;
        self.variable_types = saved_variable_types;
        self.variable_struct_names = saved_variable_struct_names;

        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        Ok(fn_val)
    }
}
