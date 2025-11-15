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
                // Special case: For builtin types with constructors, return the type name
                // This ensures variable_struct_names gets populated correctly
                if method == "new" {
                    match potential_type_name.as_str() {
                        "Vec" | "Box" | "Map" | "Set" | "String" | "Channel" => {
                            return Ok(Some(potential_type_name.clone()));
                        }
                        _ => {}
                    }
                }
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
            // Try simple method name first (backward compat)
            let method_func_name = format!("{}_{}", struct_name, method);
            if let Some(func_def) = self.function_defs.get(&method_func_name) {
                return self.extract_return_type_name(func_def);
            }

            // ⭐ NEW: Try type-based overloaded methods
            // Look for any method starting with StructName_method_
            let method_prefix = format!("{}_{}_", struct_name, method);
            for (func_name, func_def) in &self.function_defs {
                if func_name.starts_with(&method_prefix) {
                    // Found an overloaded version, return its type
                    return self.extract_return_type_name(func_def);
                }
            }
        }

        Ok(None)
    }

    fn extract_return_type_name(&self, func_def: &Function) -> Result<Option<String>, String> {
        match &func_def.return_type {
            Some(Type::Named(s_name)) => Ok(Some(s_name.clone())),
            Some(Type::Option(_)) => Ok(Some("Option".to_string())),
            Some(Type::Result(_, _)) => Ok(Some("Result".to_string())),
            _ => Ok(None),
        }
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
            // ⭐ Phase 3: Vec() constructor without type args - create Unknown placeholder
            // This will be resolved later by constraint unification
            if type_args.is_empty() {
                match func_name.as_str() {
                    "Vec" | "Box" | "Option" => {
                        eprintln!("⭐ Phase 3: {}() constructor without type args - will infer from usage", func_name);
                        return Ok(Some(func_name.clone()));
                    }
                    _ => {}
                }
            }

            // Check if this is a type constructor (e.g., Vec<i32>())
            if !type_args.is_empty() {
                // Check stdlib types
                match func_name.as_str() {
                    "Vec" => return Ok(Some("Vec".to_string())),
                    "Box" => return Ok(Some("Box".to_string())),
                    "Option" => return Ok(Some("Option".to_string())),
                    "Result" => return Ok(Some("Result".to_string())),
                    "Map" => return Ok(Some("Map".to_string())),
                    "Set" => return Ok(Some("Set".to_string())),
                    _ => {}
                }

                // Check if it's a registered struct
                if self.struct_defs.contains_key(func_name)
                    || self.struct_ast_defs.contains_key(func_name)
                {
                    return Ok(Some(func_name.clone()));
                }
            }

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
                    // ⭐ NEW: Try to look up actual method implementation first
                    // This handles external operator methods that may not have explicit trait impls
                    if let Ok(right_type) = self.infer_expression_type(right) {
                        if let Some(return_type_name) =
                            self.infer_operator_return_from_method(&left_type, &right_type, op)?
                        {
                            return Ok(Some(return_type_name));
                        }
                    }

                    // Fallback: Check trait-based operator overloading
                    if let Some(_) = self.has_operator_trait(type_name, trait_name) {
                        // For operator overloading, return type is often Self
                        // Check trait definition to see if it returns Self or a specific type
                        if let Ok(Some(return_type_name)) =
                            self.get_operator_return_type(trait_name, &method_name, type_name)
                        {
                            return Ok(Some(return_type_name));
                        }
                        // If trait says Self, return the struct type
                        return Ok(Some(type_name.clone()));
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
        implementing_type: &str,
    ) -> Result<Option<String>, String> {
        if let Some(trait_def) = self.trait_defs.get(trait_name) {
            for method_sig in &trait_def.methods {
                if method_sig.name == method_name {
                    if let Some(return_type) = &method_sig.return_type {
                        match return_type {
                            Type::Named(ret_type_name) => {
                                if ret_type_name == "Self" {
                                    // Self resolves to the implementing type
                                    return Ok(Some(implementing_type.to_string()));
                                } else if self.struct_defs.contains_key(ret_type_name) {
                                    return Ok(Some(ret_type_name.clone()));
                                }
                                // ⭐ NEW: Handle primitive types (bool, i32, etc.)
                                // These don't have struct defs but are valid return types
                                return Ok(Some(ret_type_name.clone()));
                            }
                            Type::I32 => return Ok(Some("i32".to_string())),
                            Type::I64 => return Ok(Some("i64".to_string())),
                            Type::F32 => return Ok(Some("f32".to_string())),
                            Type::F64 => return Ok(Some("f64".to_string())),
                            Type::Bool => return Ok(Some("bool".to_string())),
                            Type::String => return Ok(Some("String".to_string())),
                            _ => {}
                        }
                    }
                    break;
                }
            }
        }
        Ok(None)
    }

    /// Infer operator return type by looking up the actual method implementation
    /// This handles external operator methods that may not have explicit trait impls
    fn infer_operator_return_from_method(
        &self,
        left_type: &Type,
        right_type: &Type,
        op: &BinaryOp,
    ) -> Result<Option<String>, String> {
        if let Type::Named(type_name) = left_type {
            // Get the encoded operator name
            let op_method = match op {
                BinaryOp::Add => "opadd",
                BinaryOp::Sub => "opsub",
                BinaryOp::Mul => "opmul",
                BinaryOp::Div => "opdiv",
                BinaryOp::Mod => "opmod",
                BinaryOp::Eq => "opeq",
                BinaryOp::NotEq => "opne",
                BinaryOp::Lt => "oplt",
                BinaryOp::LtEq => "ople",
                BinaryOp::Gt => "opgt",
                BinaryOp::GtEq => "opge",
                BinaryOp::And => return Ok(None), // Logical ops don't use methods
                BinaryOp::Or => return Ok(None),
                BinaryOp::BitAnd => "opbitand",
                BinaryOp::BitOr => "opbitor",
                BinaryOp::BitXor => "opbitxor",
                BinaryOp::Shl => "opshl",
                BinaryOp::Shr => "opshr",
                _ => return Ok(None), // Other operators not relevant
            };

            // Try to find method with type-based overloading
            // Format: Type_opmethod_typename_paramcount
            let right_type_suffix = self.generate_type_suffix(right_type);
            let method_name_typed = format!("{}_{}{}_1", type_name, op_method, right_type_suffix);

            // Fallback: Try without type suffix
            let method_name_untyped = format!("{}_{}_2", type_name, op_method);

            // Check function definitions
            if let Some(func_def) = self
                .function_defs
                .get(&method_name_typed)
                .or_else(|| self.function_defs.get(&method_name_untyped))
            {
                if let Some(return_type) = &func_def.return_type {
                    match return_type {
                        Type::Named(ret_type_name) => {
                            if ret_type_name == "Self" {
                                return Ok(Some(type_name.clone()));
                            } else if self.struct_defs.contains_key(ret_type_name) {
                                return Ok(Some(ret_type_name.clone()));
                            } else {
                                return Ok(Some(ret_type_name.clone()));
                            }
                        }
                        Type::I32 => return Ok(Some("i32".to_string())),
                        Type::I64 => return Ok(Some("i64".to_string())),
                        Type::F32 => return Ok(Some("f32".to_string())),
                        Type::F64 => return Ok(Some("f64".to_string())),
                        Type::Bool => return Ok(Some("bool".to_string())),
                        Type::String => return Ok(Some("String".to_string())),
                        _ => {}
                    }
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
                let alloca = self.create_entry_block_alloca(
                    name,
                    ty.ok_or("Missing type annotation for array")?,
                    is_mutable,
                )?;
                self.compile_array_repeat_into_buffer(value_expr, count_expr, elem_type, alloca)?;

                let llvm_type = self.ast_type_to_llvm(
                    ty.ok_or("Missing type annotation for array")?,
                );
                self.variables.insert(name.to_string(), alloca);
                self.variable_types.insert(name.to_string(), llvm_type);
                self.variable_ast_types
                    .insert(name.to_string(), (*ty.ok_or("Missing type for variable")?).clone());

                // Return the alloca pointer (already registered)
                return Ok(alloca.into());
            } else if let Expression::Array(elements) = adjusted_value {
                if elements.len() > 100 {
                    let alloca = self.create_entry_block_alloca(
                        name,
                        ty.ok_or("Missing type annotation for large array")?,
                        is_mutable,
                    )?;
                    self.compile_array_literal_into_buffer(elements, elem_type, alloca)?;

                    let llvm_type = self.ast_type_to_llvm(
                        ty.ok_or("Missing type annotation for array")?,
                    );
                    self.variables.insert(name.to_string(), alloca);
                    self.variable_types.insert(name.to_string(), llvm_type);
                    self.variable_ast_types
                        .insert(name.to_string(), (*ty.ok_or("Missing type for variable")?).clone());

                    // Return the alloca pointer (already registered)
                    return Ok(alloca.into());
                }
            }
        }

        self.compile_expression(adjusted_value)
    }
}
