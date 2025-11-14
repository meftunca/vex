// statements/let_statement/mod.rs
// Main entry point for let statement compilation

mod pattern;
mod type_inference;
mod type_injection;
mod variable_registration;

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a `let pattern` statement: let (a, b) = expr; or let Point { x, y } = expr;
    pub(crate) fn compile_let_pattern_statement(
        &mut self,
        is_mutable: bool,
        pattern: &Pattern,
        _ty: Option<&Type>,
        value: &Expression,
    ) -> Result<(), String> {
        // Compile the value expression
        let val = self.compile_expression(value)?;

        // Bind pattern variables
        self.compile_pattern_binding(pattern, val)?;

        Ok(())
    }

    /// Compile a `let` statement
    pub(crate) fn compile_let_statement(
        &mut self,
        is_mutable: bool,
        name: &String,
        ty: Option<&Type>,
        value: &Expression,
    ) -> Result<(), String> {
        eprintln!("🔵 compile_let_statement: name='{}', ty={:?}", name, ty);

        // Step 1: Infer struct name from expression if no type annotation
        let struct_name_from_expr = self.infer_struct_name_from_expression(ty, value)?;

        // Step 2: Validate array size if type annotation is array
        self.validate_array_size(ty, value)?;

        // Step 3: Inject type args recursively for nested generics
        let adjusted_value = if let Some(ref type_annotation) = ty {
            self.inject_type_args_recursive(value, type_annotation)?
        } else {
            value.clone()
        };

        // Step 4: Compile the value expression
        let val = self.compile_value_expression(ty, &adjusted_value, name, is_mutable)?;

        // Step 4.5: Check if this is a closure and register it with its environment
        if let Expression::Closure {
            params,
            return_type,
            ..
        } = adjusted_value
        {
            if let BasicValueEnum::PointerValue(fn_ptr) = val {
                // Check if this closure has an environment in closure_envs
                if let Some(env_ptr) = self.closure_envs.get(&fn_ptr).copied() {
                    eprintln!(
                        "📝 Registering closure variable '{}' with environment",
                        name
                    );
                    self.closure_variables
                        .insert(name.clone(), (fn_ptr, env_ptr));
                } else {
                    eprintln!(
                        "📝 Registering closure variable '{}' without environment (pure function)",
                        name
                    );
                    // For closures without environment, still register but with null environment
                    let null_env = self
                        .context
                        .ptr_type(inkwell::AddressSpace::default())
                        .const_null();
                    self.closure_variables
                        .insert(name.clone(), (fn_ptr, null_env));
                }

                // ⭐ Store closure type signature
                let param_types: Vec<Type> = params.iter().map(|p| p.ty.clone()).collect();
                let ret_type = return_type.clone().unwrap_or(Type::I32);
                eprintln!(
                    "  ✨ Stored closure type: {} params, return {:?}",
                    params.len(),
                    &ret_type
                );
                self.closure_types
                    .insert(name.clone(), (param_types, ret_type));
            }
        }

        // Special case: If val is a pointer and variable is already registered, skip step 5-6
        // This happens when compile_value_expression handles large arrays directly
        let early_return_needed = if let BasicValueEnum::PointerValue(_) = val {
            self.variables.contains_key(name)
        } else {
            false
        };

        if !early_return_needed {
            // Step 5: Determine final type
            let (final_var_type, final_llvm_type) =
                self.determine_final_type(ty, val, value, &struct_name_from_expr)?;

            // ⭐ Phase 2/3: Track concrete type BEFORE registering variable
            // Store full AST type (including Generic with type_args) for receiver type resolution
            if let Some(type_annotation) = ty {
                // Explicit type annotation - use it directly
                self.variable_concrete_types
                    .insert(name.clone(), type_annotation.clone());
            } else if let Some(ref struct_name_str) = struct_name_from_expr {
                // Inferred struct type - determine if generic
                if let Some(struct_def) = self.struct_ast_defs.get(struct_name_str) {
                    if !struct_def.type_params.is_empty() {
                        // ⭐ Phase 3: Generic struct without explicit type args - check constructor
                        let type_args_from_value = match value {
                            Expression::Call { type_args, .. } => type_args,
                            Expression::TypeConstructor { type_args, .. } => type_args,
                            _ => &[] as &[Type],
                        };

                        if type_args_from_value.is_empty() {
                            // ⭐ Phase 3.5: Try to infer from constructor args
                            let inferred_from_args = match value {
                                Expression::TypeConstructor { args, .. } if !args.is_empty() => {
                                    // Box(42) - infer T from first arg
                                    if let Ok(arg_type) =
                                        self.infer_expression_type_with_context(&args[0], None)
                                    {
                                        Some(Type::Generic {
                                            name: struct_name_str.clone(),
                                            type_args: vec![arg_type],
                                        })
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };

                            if let Some(concrete_type) = inferred_from_args {
                                // Successfully inferred from constructor args
                                self.variable_concrete_types
                                    .insert(name.clone(), concrete_type);
                            } else {
                                // Vec() or Box() without args - create Unknown placeholder
                                let unknown_type = Type::Generic {
                                    name: struct_name_str.clone(),
                                    type_args: vec![Type::Unknown],
                                };
                                self.variable_concrete_types
                                    .insert(name.clone(), unknown_type);
                            }
                        } else {
                            // Explicit type args - use them
                            if let Ok(inferred_type) =
                                self.infer_expression_type_with_context(value, None)
                            {
                                self.variable_concrete_types
                                    .insert(name.clone(), inferred_type);
                            }
                        }
                    }
                }
            }

            // Step 6: Register the variable (now variable_concrete_types is populated)
            self.register_variable(name, val, &final_var_type, final_llvm_type, is_mutable)?;
        }

        Ok(())
    }
}
