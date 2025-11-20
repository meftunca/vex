// generics/inference.rs
// Type argument inference from function calls

use super::super::*;
use std::collections::HashMap;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn infer_type_args_from_call(
        &mut self,
        func_def: &Function,
        args: &[Expression],
    ) -> Result<Vec<Type>, String> {
        // For functions with multiple type parameters of the same type,
        // we need to infer unique type parameters, not all argument types
        // Example: fn max<T>(a: T, b: T): T has 1 type param T, not 2

        if func_def.type_params.is_empty() {
            return Ok(Vec::new());
        }

        // Build map: type param name -> inferred type
        let mut type_param_map: HashMap<String, Type> = HashMap::new();

        // Match arguments to parameters and infer type parameters
        for (i, param) in func_def.params.iter().enumerate() {
            if i >= args.len() {
                break; // Not enough arguments
            }

            let arg_type = self.infer_expression_type(&args[i])?;
            self.match_type_param(&param.ty, &arg_type, &mut type_param_map);
        }

        // Build type_args vector in the order of func_def.type_params
        let mut type_args = Vec::new();
        for type_param in &func_def.type_params {
            if let Some(inferred_ty) = type_param_map.get(&type_param.name) {
                type_args.push(inferred_ty.clone());
            } else {
                return Err(format!(
                    "Cannot infer type parameter '{}' for function '{}'",
                    type_param.name, func_def.name
                ));
            }
        }

        Ok(type_args)
    }

    /// Match a parameter type pattern against an argument type to infer type parameters
    /// Example: param_ty = T, arg_ty = i32 → map[T] = i32
    ///          param_ty = (T, U), arg_ty = (i32, string) → map[T]=i32, map[U]=string
    fn match_type_param(
        &self,
        param_ty: &Type,
        arg_ty: &Type,
        type_param_map: &mut HashMap<String, Type>,
    ) {
        match param_ty {
            // Single type parameter: T
            Type::Named(name) if name.chars().next().map_or(false, |c| c.is_uppercase()) => {
                // Check if this is a type parameter (starts with uppercase)
                // TODO: Better check - compare with func_def.type_params
                type_param_map.insert(name.clone(), arg_ty.clone());
            }
            // Tuple: (T, U) matched with (i32, string)
            Type::Tuple(param_types) => {
                if let Type::Tuple(arg_types) = arg_ty {
                    for (p_ty, a_ty) in param_types.iter().zip(arg_types.iter()) {
                        self.match_type_param(p_ty, a_ty, type_param_map);
                    }
                }
            }
            // Reference: &T matched with &i32
            Type::Reference(inner_param, _) => {
                if let Type::Reference(inner_arg, _) = arg_ty {
                    self.match_type_param(inner_param, inner_arg, type_param_map);
                }
            }
            // Option<T>, Result<T, E>, Vec<T>
            Type::Option(inner_param) => {
                if let Type::Option(inner_arg) = arg_ty {
                    self.match_type_param(inner_param, inner_arg, type_param_map);
                }
            }
            Type::Result(ok_param, err_param) => {
                if let Type::Result(ok_arg, err_arg) = arg_ty {
                    self.match_type_param(ok_param, ok_arg, type_param_map);
                    self.match_type_param(err_param, err_arg, type_param_map);
                }
            }
            Type::Vec(inner_param) => {
                if let Type::Vec(inner_arg) = arg_ty {
                    self.match_type_param(inner_param, inner_arg, type_param_map);
                }
            }
            // Generic with type args: HashMap<K, V>
            Type::Generic { name, type_args } => {
                if let Type::Generic {
                    name: arg_name,
                    type_args: arg_type_args,
                } = arg_ty
                {
                    if name == arg_name {
                        for (p_ty, a_ty) in type_args.iter().zip(arg_type_args.iter()) {
                            self.match_type_param(p_ty, a_ty, type_param_map);
                        }
                    }
                }
            }
            _ => {
                // Non-generic type, no inference needed
            }
        }
    }

    /// Instantiate all methods of a generic struct with concrete type arguments
    /// This is called when a generic struct is instantiated (e.g., HashMap<str, i32>)
    pub(crate) fn instantiate_struct_methods(
        &mut self,
        struct_name: &str,
        struct_type_params: &[TypeParam],
        type_args: &[Type],
        mangled_struct_name: &str,
    ) -> Result<(), String> {
        // Build type substitution map: K -> str, V -> i32, etc.
        let mut type_subst = HashMap::new();
        for (param, arg) in struct_type_params.iter().zip(type_args.iter()) {
            type_subst.insert(param.name.clone(), arg.clone());
        }

        eprintln!(
            "🔧 Instantiating methods for struct {} -> {}",
            struct_name, mangled_struct_name
        );
        eprintln!("   Type substitution: {:?}", type_subst);

        // Debug: List all function_defs
        eprintln!(
            "   All registered functions ({} total):",
            self.function_defs.len()
        );
        for (name, _) in self.function_defs.iter() {
            eprintln!("      - {}", name);
        }

        // Find all methods for this struct
        // Methods are stored as regular functions with receiver parameter
        let method_names: Vec<String> = self
            .function_defs
            .keys()
            .filter(|name| {
                // Generic struct methods: HashMap_insert, HashMap_get, etc.
                name.starts_with(&format!("{}_", struct_name))
                    && !name.contains("_str_") // Not already instantiated
                    && !name.contains("_i32_")
                    && !name.contains("_i64_")
            })
            .cloned()
            .collect();

        eprintln!("   Found {} methods to instantiate", method_names.len());

        for method_name in method_names {
            let func_def = self.function_defs.get(&method_name).cloned();
            if let Some(func) = func_def {
                eprintln!("   → Method: {}", method_name);

                // Check if this method has a receiver parameter
                // Either has func.receiver field OR first param is named "self"
                let has_receiver = func.receiver.is_some()
                    || func.params.first().map_or(false, |p| p.name == "self");

                if !has_receiver {
                    eprintln!("      ⚠️  Skipping - not a method (no receiver)");
                    continue;
                }

                // Instantiate the method
                let specialized_func = self.substitute_types_in_method(
                    &func,
                    &type_subst,
                    struct_name,
                    mangled_struct_name,
                )?;

                eprintln!(
                    "      ✅ Instantiated: {} -> {}",
                    method_name, specialized_func.name
                );

                // Register the instantiated method in function_defs (AST)
                self.function_defs
                    .insert(specialized_func.name.clone(), specialized_func.clone());

                // ⭐ FIX: Declare and compile the method NOW (not later)
                // Save current context
                let saved_current_function = self.current_function;
                let saved_insert_block = self.builder.get_insert_block();
                let saved_variables = std::mem::take(&mut self.variables);
                let saved_variable_types = std::mem::take(&mut self.variable_types);
                let saved_variable_struct_names = std::mem::take(&mut self.variable_struct_names);

                // Declare the method
                match self.declare_function(&specialized_func) {
                    Ok(fn_val) => {
                        self.functions.insert(specialized_func.name.clone(), fn_val);
                        eprintln!("      → Declared LLVM function");

                        // Compile the method body
                        match self.compile_function(&specialized_func) {
                            Ok(_) => {
                                eprintln!("      ✅ Compiled successfully");
                            }
                            Err(e) => {
                                eprintln!("      ⚠️  Compilation failed: {}", e);
                                eprintln!("         Continuing with next method...");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("      ⚠️  Declaration failed: {}", e);
                        eprintln!("         Continuing with next method...");
                    }
                }

                // Restore context
                self.current_function = saved_current_function;
                self.variables = saved_variables;
                self.variable_types = saved_variable_types;
                self.variable_struct_names = saved_variable_struct_names;

                if let Some(block) = saved_insert_block {
                    self.builder.position_at_end(block);
                }
            }
        }

        Ok(())
    }

    /// Substitute types in a method, including receiver type
    pub(crate) fn substitute_types_in_method(
        &self,
        func: &Function,
        type_subst: &HashMap<String, Type>,
        struct_name: &str,
        mangled_struct_name: &str,
    ) -> Result<Function, String> {
        let mut new_func = func.clone();

        // Clear type parameters (they're now concrete)
        new_func.type_params.clear();

        // ⭐ FIX: Update receiver field with mangled struct name
        if let Some(ref mut receiver) = new_func.receiver {
            receiver.ty = self.substitute_type(&receiver.ty, type_subst);

            // Ensure receiver references the mangled struct
            if let Type::Reference(inner, is_mut) = &receiver.ty {
                match inner.as_ref() {
                    Type::Named(_) | Type::Generic { .. } => {
                        receiver.ty = Type::Reference(
                            Box::new(Type::Named(mangled_struct_name.to_string())),
                            *is_mut,
                        );
                    }
                    _ => {}
                }
            }
        }

        // Substitute parameter types
        for param in &mut new_func.params {
            param.ty = self.substitute_type(&param.ty, type_subst);

            // Also update receiver type if it references the struct
            if let Type::Reference(inner, is_mut) = &param.ty {
                // After substitution, if inner is Named OR still Generic, update to mangled struct
                match inner.as_ref() {
                    Type::Named(_) | Type::Generic { .. } => {
                        // Update struct name to mangled version
                        param.ty = Type::Reference(
                            Box::new(Type::Named(mangled_struct_name.to_string())),
                            *is_mut,
                        );
                    }
                    _ => {}
                }
            } else if let Type::Named(_name) = &param.ty {
                // Direct struct parameter
                param.ty = Type::Named(mangled_struct_name.to_string());
            }
        }

        // Substitute return type
        if let Some(ret_ty) = &new_func.return_type {
            let substituted_ret = self.substitute_type(ret_ty, type_subst);
            eprintln!(
                "      🔄 Return type substitution: {:?} -> {:?}",
                ret_ty, substituted_ret
            );
            new_func.return_type = Some(substituted_ret);
        }

        // Build mangled method name: HashMap_insert -> HashMap_str_i32_insert
        // Extract method name by removing struct prefix
        let struct_prefix = format!("{}_", struct_name);
        let method_suffix = if func.name.starts_with(&struct_prefix) {
            &func.name[struct_prefix.len()..]
        } else {
            // Fallback: use original name if prefix doesn't match
            &func.name
        };

        new_func.name = format!("{}_{}", mangled_struct_name, method_suffix);
        eprintln!("      📛 Specialized method name: {}", new_func.name);

        Ok(new_func)
    }

    pub(crate) fn substitute_types_in_function(
        &self,
        func: &Function,
        type_subst: &HashMap<String, Type>,
    ) -> Result<Function, String> {
        let mut new_func = func.clone();
        new_func.type_params.clear();

        // Substitute receiver type
        if let Some(ref mut receiver) = new_func.receiver {
            receiver.ty = self.substitute_type(&receiver.ty, type_subst);
        }

        // Substitute parameter types
        for param in &mut new_func.params {
            param.ty = self.substitute_type(&param.ty, type_subst);
        }

        // Substitute return type
        if let Some(ret_ty) = &new_func.return_type {
            new_func.return_type = Some(self.substitute_type(ret_ty, type_subst));
        }

        // ⭐ NEW: Substitute types in function body
        new_func.body = self.substitute_types_in_block(&new_func.body, type_subst);

        let type_names: Vec<String> = type_subst
            .values()
            .map(|t| self.type_to_string(t))
            .collect();
        new_func.name = format!("{}_{}", func.name, type_names.join("_"));
        Ok(new_func)
    }

    fn substitute_types_in_block(
        &self,
        block: &Block,
        type_subst: &HashMap<String, Type>,
    ) -> Block {
        Block {
            statements: block
                .statements
                .iter()
                .map(|stmt| self.substitute_types_in_statement(stmt, type_subst))
                .collect(),
        }
    }

    fn substitute_types_in_statement(
        &self,
        stmt: &Statement,
        type_subst: &HashMap<String, Type>,
    ) -> Statement {
        match stmt {
            Statement::Return(Some(expr)) => {
                Statement::Return(Some(self.substitute_types_in_expression(expr, type_subst)))
            }
            Statement::Return(None) => Statement::Return(None),
            Statement::If {
                span_id,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => Statement::If {
                span_id: span_id.clone(),
                condition: self.substitute_types_in_expression(condition, type_subst),
                then_block: self.substitute_types_in_block(then_block, type_subst),
                elif_branches: elif_branches
                    .iter()
                    .map(|(cond, block)| {
                        (
                            self.substitute_types_in_expression(cond, type_subst),
                            self.substitute_types_in_block(block, type_subst),
                        )
                    })
                    .collect(),
                else_block: else_block
                    .as_ref()
                    .map(|b| self.substitute_types_in_block(b, type_subst)),
            },
            Statement::Let {
                name,
                ty,
                value,
                is_mutable,
            } => Statement::Let {
                name: name.clone(),
                ty: ty.as_ref().map(|t| self.substitute_type(t, type_subst)),
                value: self.substitute_types_in_expression(value, type_subst),
                is_mutable: *is_mutable,
            },
            Statement::Expression(expr) => {
                Statement::Expression(self.substitute_types_in_expression(expr, type_subst))
            }
            Statement::While {
                span_id,
                condition,
                body,
            } => Statement::While {
                span_id: span_id.clone(),
                condition: self.substitute_types_in_expression(condition, type_subst),
                body: self.substitute_types_in_block(body, type_subst),
            },
            Statement::For {
                span_id,
                init,
                condition,
                post,
                body,
            } => Statement::For {
                span_id: span_id.clone(),
                init: init
                    .as_ref()
                    .map(|s| Box::new(self.substitute_types_in_statement(s, type_subst))),
                condition: condition
                    .as_ref()
                    .map(|e| self.substitute_types_in_expression(e, type_subst)),
                post: post
                    .as_ref()
                    .map(|s| Box::new(self.substitute_types_in_statement(s, type_subst))),
                body: self.substitute_types_in_block(body, type_subst),
            },
            Statement::ForIn {
                variable,
                iterable,
                body,
            } => Statement::ForIn {
                variable: variable.clone(),
                iterable: self.substitute_types_in_expression(iterable, type_subst),
                body: self.substitute_types_in_block(body, type_subst),
            },
            Statement::Assign { target, value } => Statement::Assign {
                target: self.substitute_types_in_expression(target, type_subst),
                value: self.substitute_types_in_expression(value, type_subst),
            },
            Statement::LetPattern {
                is_mutable,
                pattern,
                ty,
                value,
            } => Statement::LetPattern {
                is_mutable: *is_mutable,
                pattern: pattern.clone(), // TODO: substitute in pattern if needed
                ty: ty.as_ref().map(|t| self.substitute_type(t, type_subst)),
                value: self.substitute_types_in_expression(value, type_subst),
            },
            _ => stmt.clone(),
        }
    }

    fn substitute_types_in_expression(
        &self,
        expr: &Expression,
        type_subst: &HashMap<String, Type>,
    ) -> Expression {
        match expr {
            Expression::StructLiteral {
                name,
                type_args,
                fields,
            } => {
                let new_type_args = type_args
                    .iter()
                    .map(|ty| self.substitute_type(ty, type_subst))
                    .collect();

                let new_fields = fields
                    .iter()
                    .map(|(field_name, field_expr)| {
                        (
                            field_name.clone(),
                            self.substitute_types_in_expression(field_expr, type_subst),
                        )
                    })
                    .collect();

                Expression::StructLiteral {
                    name: name.clone(),
                    type_args: new_type_args,
                    fields: new_fields,
                }
            }
            Expression::Binary {
                span_id,
                left,
                op,
                right,
            } => Expression::Binary {
                span_id: span_id.clone(),
                left: Box::new(self.substitute_types_in_expression(left, type_subst)),
                op: op.clone(),
                right: Box::new(self.substitute_types_in_expression(right, type_subst)),
            },
            Expression::Unary { span_id, op, expr } => Expression::Unary {
                span_id: span_id.clone(),
                op: op.clone(),
                expr: Box::new(self.substitute_types_in_expression(expr, type_subst)),
            },
            Expression::Call {
                span_id,
                func,
                type_args,
                args,
            } => Expression::Call {
                span_id: span_id.clone(),
                func: Box::new(self.substitute_types_in_expression(func, type_subst)),
                type_args: type_args
                    .iter()
                    .map(|ty| self.substitute_type(ty, type_subst))
                    .collect(),
                args: args
                    .iter()
                    .map(|arg| self.substitute_types_in_expression(arg, type_subst))
                    .collect(),
            },
            Expression::MethodCall {
                receiver,
                method,
                type_args,
                args,
                is_mutable_call,
            } => Expression::MethodCall {
                receiver: Box::new(self.substitute_types_in_expression(receiver, type_subst)),
                method: method.clone(),
                type_args: type_args
                    .iter()
                    .map(|ty| self.substitute_type(ty, type_subst))
                    .collect(),
                args: args
                    .iter()
                    .map(|arg| self.substitute_types_in_expression(arg, type_subst))
                    .collect(),
                is_mutable_call: *is_mutable_call,
            },
            Expression::Index { object, index } => Expression::Index {
                object: Box::new(self.substitute_types_in_expression(object, type_subst)),
                index: Box::new(self.substitute_types_in_expression(index, type_subst)),
            },
            Expression::Cast { expr, target_type } => Expression::Cast {
                expr: Box::new(self.substitute_types_in_expression(expr, type_subst)),
                target_type: self.substitute_type(target_type, type_subst),
            },
            Expression::TupleLiteral(elements) => Expression::TupleLiteral(
                elements
                    .iter()
                    .map(|e| self.substitute_types_in_expression(e, type_subst))
                    .collect(),
            ),
            Expression::Array(elements) => Expression::Array(
                elements
                    .iter()
                    .map(|e| self.substitute_types_in_expression(e, type_subst))
                    .collect(),
            ),
            Expression::FieldAccess { object, field } => Expression::FieldAccess {
                object: Box::new(self.substitute_types_in_expression(object, type_subst)),
                field: field.clone(),
            },
            Expression::Reference { is_mutable, expr } => Expression::Reference {
                is_mutable: *is_mutable,
                expr: Box::new(self.substitute_types_in_expression(expr, type_subst)),
            },
            Expression::Deref(expr) => {
                Expression::Deref(Box::new(self.substitute_types_in_expression(expr, type_subst)))
            }
            Expression::Block {
                statements,
                return_expr,
            } => Expression::Block {
                statements: statements
                    .iter()
                    .map(|stmt| self.substitute_types_in_statement(stmt, type_subst))
                    .collect(),
                return_expr: return_expr.as_ref().map(|e| {
                    Box::new(self.substitute_types_in_expression(e, type_subst))
                }),
            },
            _ => expr.clone(),
        }
    }
}
