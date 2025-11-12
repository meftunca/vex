// Type inference from expressions for let statements

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Infer struct name from expression when no type annotation is provided
    pub(crate) fn infer_struct_name_from_expression(
        &mut self,
        ty: Option<&Type>,
        value: &Expression,
    ) -> Result<Option<String>, String> {
        if ty.is_some() {
            return Ok(None);
        }

        eprintln!("  → Type inference needed, analyzing expression...");

        let result = match value {
            Expression::StructLiteral {
                name: s_name,
                type_args,
                ..
            } => self.infer_from_struct_literal(s_name, type_args)?,

            Expression::MethodCall {
                receiver, method, ..
            } => self.infer_from_method_call(receiver, method)?,

            Expression::TypeConstructor {
                type_name,
                type_args,
                ..
            } => self.infer_from_type_constructor(type_name, type_args)?,

            Expression::Range { .. } => Some("Range".to_string()),
            Expression::RangeInclusive { .. } => Some("RangeInclusive".to_string()),
            Expression::Array(_) | Expression::ArrayRepeat(_, _) => None,

            Expression::Call {
                func, type_args, ..
            } => self.infer_from_call(func, type_args)?,

            Expression::FieldAccess { object, field } => {
                self.get_field_struct_type(object, field).ok().flatten()
            }

            Expression::MapLiteral(_) => Some("Map".to_string()),

            Expression::Binary {
                left, op, right, ..
            } => self.infer_from_binary_op(left, op, right)?,

            Expression::EnumLiteral { enum_name, .. } => {
                if enum_name == "Option" || enum_name == "Result" {
                    Some(enum_name.clone())
                } else {
                    None
                }
            }

            Expression::Unary { expr, op, .. } => self.infer_from_unary_op(expr, op)?,

            _ => None,
        };

        Ok(result)
    }

    fn infer_from_struct_literal(
        &mut self,
        s_name: &str,
        type_args: &[Type],
    ) -> Result<Option<String>, String> {
        if !type_args.is_empty() {
            match self.instantiate_generic_struct(s_name, type_args) {
                Ok(mangled_name) => Ok(Some(mangled_name)),
                Err(_) => Ok(None),
            }
        } else if self.struct_defs.contains_key(s_name) {
            Ok(Some(s_name.to_string()))
        } else {
            Ok(None)
        }
    }

    fn infer_from_method_call(
        &mut self,
        receiver: &Expression,
        method: &str,
    ) -> Result<Option<String>, String> {
        // Check for static method calls: Type.new() -> Type
        if let Expression::Ident(potential_type_name) = receiver {
            let is_type_name = potential_type_name
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);

            let is_not_variable = !self.variables.contains_key(potential_type_name);

            if is_type_name && is_not_variable {
                return Ok(Some(potential_type_name.clone()));
            }
        }

        // Check for builtin method calls with known return types
        if let Expression::Ident(var_name) = receiver {
            if let Some(struct_name) = self.variable_struct_names.get(var_name) {
                if struct_name == "Vec" && method == "as_slice" {
                    return Ok(Some("Slice".to_string()));
                }
            }
        }

        // Get struct type from receiver and method signature
        self.infer_method_return_type(receiver, method)
    }

    fn infer_method_return_type(
        &self,
        receiver: &Expression,
        method: &str,
    ) -> Result<Option<String>, String> {
        let struct_name = if let Expression::Ident(var_name) = receiver {
            self.variable_struct_names.get(var_name).cloned()
        } else {
            None
        };

        if let Some(struct_name) = struct_name {
            let method_func_name = format!("{}_{}", struct_name, method);
            if let Some(func_def) = self.function_defs.get(&method_func_name) {
                if let Some(Type::Named(s_name)) = &func_def.return_type {
                    return Ok(Some(s_name.clone()));
                } else if let Some(Type::Option(_)) = &func_def.return_type {
                    return Ok(Some("Option".to_string()));
                } else if let Some(Type::Result(_, _)) = &func_def.return_type {
                    return Ok(Some("Result".to_string()));
                }
            }
        }

        Ok(None)
    }

    fn infer_from_type_constructor(
        &mut self,
        type_name: &str,
        type_args: &[Type],
    ) -> Result<Option<String>, String> {
        if !type_args.is_empty() {
            if let Ok(mangled_name) = self.instantiate_generic_struct(type_name, type_args) {
                Ok(Some(mangled_name))
            } else {
                Ok(Some(type_name.to_string()))
            }
        } else {
            Ok(Some(type_name.to_string()))
        }
    }

    fn infer_from_call(
        &mut self,
        func: &Expression,
        type_args: &[Type],
    ) -> Result<Option<String>, String> {
        if let Expression::Ident(func_name) = func {
            // Check for builtin constructors
            match func_name.as_str() {
                "vec_new" | "vec_with_capacity" => return Ok(Some("Vec".to_string())),
                "box_new" => return Ok(Some("Box".to_string())),
                "string_new" | "string_from" => return Ok(Some("String".to_string())),
                "map_new" | "map_with_capacity" | "hashmap_new" => {
                    return Ok(Some("Map".to_string()))
                }
                "set_new" | "set_with_capacity" => return Ok(Some("Set".to_string())),
                "range_new" => return Ok(Some("Range".to_string())),
                "range_inclusive_new" => return Ok(Some("RangeInclusive".to_string())),
                "channel_new" => return Ok(Some("Channel".to_string())),
                _ => {}
            }

            // Regular function
            if let Some(func_def) = self.function_defs.get(func_name).cloned() {
                return self.infer_from_function_def(&func_def, func_name, type_args);
            }
        }

        Ok(None)
    }

    fn infer_from_function_def(
        &self,
        func_def: &Function,
        func_name: &str,
        type_args: &[Type],
    ) -> Result<Option<String>, String> {
        // Check if this is a generic function with explicit type args
        if !func_def.type_params.is_empty() && !type_args.is_empty() {
            return self.infer_from_generic_function(func_def.clone(), func_name, type_args);
        }

        // Non-generic function
        match &func_def.return_type {
            Some(Type::Named(s_name)) => {
                if self.struct_defs.contains_key(s_name) || self.enum_ast_defs.contains_key(s_name)
                {
                    Ok(Some(s_name.clone()))
                } else {
                    Ok(None)
                }
            }
            Some(Type::Generic { name: gen_name, .. }) => {
                if self.enum_ast_defs.contains_key(gen_name)
                    || self.struct_defs.contains_key(gen_name)
                {
                    Ok(Some(gen_name.clone()))
                } else {
                    Ok(None)
                }
            }
            Some(Type::Result(_, _)) => Ok(Some("Result".to_string())),
            Some(Type::Option(_)) => Ok(Some("Option".to_string())),
            _ => Ok(None),
        }
    }

    fn infer_from_generic_function(
        &self,
        func_def: Function,
        func_name: &str,
        type_args: &[Type],
    ) -> Result<Option<String>, String> {
        let type_names: Vec<String> = type_args.iter().map(|t| self.type_to_string(t)).collect();
        let mangled_func = format!("{}_{}", func_name, type_names.join("_"));

        if let Some(inst_func_def) = self.function_defs.get(&mangled_func) {
            match &inst_func_def.return_type {
                Some(Type::Named(s_name)) => {
                    if self.struct_defs.contains_key(s_name) {
                        Ok(Some(s_name.clone()))
                    } else {
                        Ok(None)
                    }
                }
                Some(Type::Generic {
                    name: gen_name,
                    type_args: gen_args,
                }) => {
                    if !gen_args.is_empty() {
                        let arg_names: Vec<String> =
                            gen_args.iter().map(|t| self.type_to_string(t)).collect();
                        let mangled_struct = format!("{}_{}", gen_name, arg_names.join("_"));

                        // Note: Instantiation will happen during compile_expression
                        // We just predict the mangled name here

                        Ok(Some(mangled_struct))
                    } else {
                        Ok(None)
                    }
                }
                _ => Ok(None),
            }
        } else {
            // Predict the mangled struct name from return type
            if let Some(Type::Generic { name: gen_name, .. }) = &func_def.return_type {
                let mangled_struct = format!("{}_{}", gen_name, type_names.join("_"));
                Ok(Some(mangled_struct))
            } else {
                Ok(None)
            }
        }
    }

    fn infer_from_binary_op(
        &mut self,
        left: &Expression,
        op: &BinaryOp,
        right: &Expression,
    ) -> Result<Option<String>, String> {
        // Check for operator overloading
        if let Ok(left_type) = self.infer_expression_type(left) {
            if let Type::Named(type_name) = &left_type {
                let (trait_name, method_name) = self.binary_op_to_trait(op);
                if !trait_name.is_empty() {
                    if let Some(_) = self.has_operator_trait(type_name, trait_name) {
                        return self.get_operator_return_type(trait_name, &method_name);
                    }
                }
            }
        }

        // Check for Vec + Vec concatenation
        if matches!(op, BinaryOp::Add) {
            if let Ok(left_type) = self.infer_expression_type(left) {
                if let Type::Generic { name, type_args } = left_type {
                    if name == "Vec" {
                        if let Ok(right_type) = self.infer_expression_type(right) {
                            if let Type::Generic {
                                name: right_name, ..
                            } = right_type
                            {
                                if right_name == "Vec" && !type_args.is_empty() {
                                    let mangled =
                                        format!("Vec_{}", self.type_to_string(&type_args[0]));
                                    return Ok(Some(mangled));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn get_operator_return_type(
        &self,
        trait_name: &str,
        method_name: &str,
    ) -> Result<Option<String>, String> {
        if let Some(trait_def) = self.trait_defs.get(trait_name) {
            for method_sig in &trait_def.methods {
                if method_sig.name == method_name {
                    if let Some(return_type) = &method_sig.return_type {
                        if let Type::Named(ret_type_name) = return_type {
                            if self.struct_defs.contains_key(ret_type_name) {
                                return Ok(Some(ret_type_name.clone()));
                            }
                        }
                    }
                    break;
                }
            }
        }
        Ok(None)
    }

    fn infer_from_unary_op(
        &mut self,
        expr: &Expression,
        op: &UnaryOp,
    ) -> Result<Option<String>, String> {
        if let Ok(expr_type) = self.infer_expression_type(expr) {
            if let Type::Named(type_name) = &expr_type {
                let (trait_name, _method_name) = self.unary_op_to_trait(op);
                if !trait_name.is_empty() {
                    if let Some(_) = self.has_operator_trait(type_name, trait_name) {
                        return Ok(Some(type_name.clone()));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Validate array size if type annotation is array
    pub(crate) fn validate_array_size(
        &self,
        ty: Option<&Type>,
        value: &Expression,
    ) -> Result<(), String> {
        if let Some(Type::Array(_, annotated_size)) = ty {
            let actual_size = match value {
                Expression::Array(elements) => Some(elements.len()),
                Expression::ArrayRepeat(_, repeat_size_expr) => {
                    if let Expression::IntLiteral(n) = repeat_size_expr.as_ref() {
                        Some(*n as usize)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(actual_size) = actual_size {
                if actual_size != *annotated_size {
                    return Err(format!(
                        "Array size mismatch: literal has {} elements but type annotation specifies [T; {}]",
                        actual_size, annotated_size
                    ));
                }
            }
        }
        Ok(())
    }

    /// Compile the value expression with special handling for Vec and Array types
    pub(crate) fn compile_value_expression(
        &mut self,
        ty: Option<&Type>,
        adjusted_value: &Expression,
        name: &str,
        is_mutable: bool,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Special case: Array literal → Vec<T> conversion
        if let Some(Type::Vec(elem_type)) = ty {
            if matches!(adjusted_value, Expression::Array(_)) {
                if let Expression::Array(elements) = adjusted_value {
                    return self.compile_vec_from_array_literal(elements, elem_type);
                }
            }
        }

        // Special case: Array with type annotation [T; N]
        if let Some(Type::Array(elem_type, _)) = ty {
            if let Expression::ArrayRepeat(value_expr, count_expr) = adjusted_value {
                let alloca = self.create_entry_block_alloca(name, ty.unwrap(), is_mutable)?;
                self.compile_array_repeat_into_buffer(value_expr, count_expr, elem_type, alloca)?;

                let llvm_type = self.ast_type_to_llvm(ty.unwrap());
                self.variables.insert(name.to_string(), alloca);
                self.variable_types.insert(name.to_string(), llvm_type);

                // Return a dummy value since we've already registered the variable
                return Ok(self.context.i32_type().const_zero().into());
            } else if let Expression::Array(elements) = adjusted_value {
                if elements.len() > 100 {
                    let alloca = self.create_entry_block_alloca(name, ty.unwrap(), is_mutable)?;
                    self.compile_array_literal_into_buffer(elements, elem_type, alloca)?;

                    let llvm_type = self.ast_type_to_llvm(ty.unwrap());
                    self.variables.insert(name.to_string(), alloca);
                    self.variable_types.insert(name.to_string(), llvm_type);
                    self.variable_ast_types
                        .insert(name.to_string(), (*ty.unwrap()).clone());

                    return Ok(self.context.i32_type().const_zero().into());
                }
            }
        }

        self.compile_expression(adjusted_value)
    }
}
