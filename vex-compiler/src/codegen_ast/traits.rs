// src/codegen/traits.rs
use super::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_trait_impl_method(
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

        let fn_val = *self
            .functions
            .get(&mangled_name)
            .ok_or_else(|| format!("Trait impl method {} not found", mangled_name))?;

        self.current_function = Some(fn_val);

        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();

        let mut param_offset = 0;

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

            let struct_name_opt = match &receiver.ty {
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

            if let Some(name) = struct_name_opt {
                self.variable_struct_names.insert("self".to_string(), name);
            }

            param_offset = 1;
        }

        for (i, param) in method.params.iter().enumerate() {
            let param_idx = crate::safe_param_index(i, param_offset)
                .map_err(|e| format!("Parameter index overflow for {}: {}", param.name, e))?;
            let param_val = fn_val
                .get_nth_param(param_idx)
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

            self.track_param_struct_name(&param.name, &param.ty);
        }

        // Compile method body
        let mut last_expr_value: Option<BasicValueEnum> = None;

        for (i, stmt) in method.body.statements.iter().enumerate() {
            let is_last = i == method.body.statements.len() - 1;

            // If last statement is expression and method has non-void return, save for implicit return
            if is_last && matches!(stmt, Statement::Expression(_)) && method.return_type.is_some() {
                if let Statement::Expression(expr) = stmt {
                    // Check if return type is void/nil
                    let is_void_return = matches!(method.return_type.as_ref(), Some(Type::Nil));
                    
                    if is_void_return {
                        // Void/nil function: compile expression as statement (for side effects)
                        self.compile_statement(stmt)?;
                    } else {
                        // Non-void function: save expression value for implicit return
                        let val = self.compile_expression(expr)?;
                        last_expr_value = Some(val);
                        continue; // Don't compile as statement
                    }
                }
            } else {
                self.compile_statement(stmt)?;
            }
        }

        // If we have a last expression value and block is not terminated, use implicit return
        if let Some(return_val) = last_expr_value {
            let is_terminated = if let Some(bb) = self.builder.get_insert_block() {
                bb.get_terminator().is_some()
            } else {
                false
            };

            if !is_terminated {
                eprintln!("🔄 Implicit return from last expression in function body");
                self.builder
                    .build_return(Some(&return_val))
                    .map_err(|e| format!("Failed to build implicit return: {}", e))?;
            }
        }

        // Check if block needs explicit terminator

        // Check if function needs explicit return
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            // Determine if void/nil return
            let is_void_or_nil = method.return_type.is_none()
                || matches!(method.return_type.as_ref(), Some(Type::Nil));

            if is_void_or_nil {
                // Void/nil function - add implicit void return
                self.builder
                    .build_return(None)
                    .map_err(|e| format!("Failed to build void return: {}", e))?;
            } else if method.return_type.is_some() {
                return Err(format!("Function {} must return a value", mangled_name));
            }
        }

        Ok(())
    }
}
