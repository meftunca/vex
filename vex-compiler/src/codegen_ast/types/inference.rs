use super::super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// ⭐ Phase 1: Enhanced type inference with context propagation
    /// 
    /// This enhances the existing `infer_expression_type` by using:
    /// 1. variable_concrete_types for known variable types
    /// 2. expected_type for bidirectional inference
    /// 3. Type::Unknown placeholders for deferred resolution
    pub fn infer_expression_type_with_context(
        &mut self,
        expr: &Expression,
        expected_type: Option<&Type>,
    ) -> Result<Type, String> {
        match expr {
            Expression::Ident(name) => {
                // Check variable_concrete_types first (Phase 1 addition)
                if let Some(ty) = self.variable_concrete_types.get(name) {
                    return Ok(ty.clone());
                }
                
                // Fallback to existing variable_ast_types
                if let Some(ty) = self.variable_ast_types.get(name) {
                    return Ok(ty.clone());
                }
                
                // Fallback to basic type inference
                self.infer_expression_type(expr)
            }
            
            Expression::Call { func, type_args, args, .. } => {
                // Check if this is a constructor call: Vec<i32>() or Vec()
                if let Expression::Ident(name) = func.as_ref() {
                    if self.struct_ast_defs.contains_key(name) {
                        // This is a struct constructor
                        if !type_args.is_empty() {
                            // Explicit type args: Vec<i32>()
                            return Ok(Type::Generic {
                                name: name.clone(),
                                type_args: type_args.clone(),
                            });
                        } else {
                            // No type args: Vec()
                            // Try to infer from expected_type
                            if let Some(Type::Generic { name: expected_name, type_args: expected_args }) = expected_type {
                                if expected_name == name {
                                    return Ok(expected_type.unwrap().clone());
                                }
                            }
                            
                            // Cannot infer yet - return placeholder with Unknown
                            // Will be resolved in unification phase
                            if let Some(struct_def) = self.struct_ast_defs.get(name) {
                                let unknown_args = vec![Type::Unknown; struct_def.type_params.len()];
                                return Ok(Type::Generic {
                                    name: name.clone(),
                                    type_args: unknown_args,
                                });
                            }
                            
                            // Not a generic struct - just named type
                            return Ok(Type::Named(name.clone()));
                        }
                    }
                }
                
                // Fallback to existing logic
                self.infer_expression_type(expr)
            }
            
            Expression::MethodCall { receiver, method, args, .. } => {
                // Infer receiver type (this may contain Unknown)
                let receiver_type = self.infer_expression_type_with_context(receiver, None)?;
                
                // Try to infer more specific type from method signature
                // This is deferred - we just return receiver_type for now
                Ok(receiver_type)
            }
            
            _ => {
                // For all other expressions, use existing inference
                self.infer_expression_type(expr)
            }
        }
    }
    
    /// Check if a type contains Type::Unknown
    pub fn contains_unknown(&self, ty: &Type) -> bool {
        match ty {
            Type::Unknown => true,
            Type::Generic { type_args, .. } => {
                type_args.iter().any(|arg| self.contains_unknown(arg))
            }
            Type::Function { params, return_type } => {
                params.iter().any(|p| self.contains_unknown(p)) ||
                self.contains_unknown(return_type)
            }
            Type::Tuple(types) => {
                types.iter().any(|t| self.contains_unknown(t))
            }
            Type::Array(elem_ty, _) => self.contains_unknown(elem_ty),
            Type::RawPtr { inner, .. } => self.contains_unknown(inner),
            Type::Reference(inner, _) => self.contains_unknown(inner),
            Type::Vec(elem_ty) => self.contains_unknown(elem_ty),
            Type::Box(inner_ty) => self.contains_unknown(inner_ty),
            Type::Option(inner_ty) => self.contains_unknown(inner_ty),
            Type::Result(ok_ty, err_ty) => {
                self.contains_unknown(ok_ty) || self.contains_unknown(err_ty)
            }
            Type::Channel(elem_ty) => self.contains_unknown(elem_ty),
            _ => false,
        }
    }
    
    pub(crate) fn infer_expression_type(&self, expr: &Expression) -> Result<Type, String> {
        let result = match expr {
            Expression::IntLiteral(_) => Ok(Type::I32),
            Expression::BigIntLiteral(_) => Ok(Type::I128), // Large integers default to i128
            Expression::FloatLiteral(_) => Ok(Type::F64),
            Expression::StringLiteral(_) => Ok(Type::String),
            Expression::FStringLiteral(_) => Ok(Type::String),
            Expression::BoolLiteral(_) => Ok(Type::Bool),
            Expression::MapLiteral(_) => Ok(Type::Named("Map".to_string())),
            Expression::Array(elements) => {
                // Array literal [1, 2, 3] is a Vec<T>
                if elements.is_empty() {
                    return Ok(Type::Generic {
                        name: "Vec".to_string(),
                        type_args: vec![Type::I32], // Default to Vec<i32>
                    });
                }
                // Infer element type from first element
                let elem_type = self.infer_expression_type(&elements[0])?;
                Ok(Type::Generic {
                    name: "Vec".to_string(),
                    type_args: vec![elem_type],
                })
            }
            Expression::Ident(name) => {
                // Check if we have AST type information first (most accurate)
                if let Some(ast_type) = self.variable_ast_types.get(name) {
                    return Ok(ast_type.clone());
                }

                // Check if this is a struct variable
                if let Some(struct_name) = self.variable_struct_names.get(name) {
                    // Handle mangled generic types (e.g., "Vec_i32" -> Vec<i32>)
                    if struct_name.starts_with("Vec_") {
                        let elem_type_str = &struct_name["Vec_".len()..];
                        let elem_type = match elem_type_str {
                            "i32" => Type::I32,
                            "i64" => Type::I64,
                            "f32" => Type::F32,
                            "f64" => Type::F64,
                            _ => Type::I32, // Fallback
                        };
                        return Ok(Type::Generic {
                            name: "Vec".to_string(),
                            type_args: vec![elem_type],
                        });
                    }
                    // Handle other generic types similarly
                    if struct_name.starts_with("Box_") {
                        let inner_type_str = &struct_name["Box_".len()..];
                        let inner_type = match inner_type_str {
                            "i32" => Type::I32,
                            "i64" => Type::I64,
                            _ => Type::I32,
                        };
                        return Ok(Type::Generic {
                            name: "Box".to_string(),
                            type_args: vec![inner_type],
                        });
                    }
                    return Ok(Type::Named(struct_name.clone()));
                }

                // Try to get type from variable
                if let Some(llvm_type) = self.variable_types.get(name) {
                    // Convert LLVM type back to AST type (simplified)
                    match llvm_type {
                        BasicTypeEnum::IntType(_) => Ok(Type::I32),
                        BasicTypeEnum::FloatType(_) => Ok(Type::F64),
                        _ => Ok(Type::I32), // Fallback
                    }
                } else {
                    Ok(Type::I32) // Default fallback
                }
            }
            Expression::Reference { expr, is_mutable } => {
                // For references (&x), return Reference type wrapping the inner type
                let inner_type = self.infer_expression_type(expr)?;
                Ok(Type::Reference(Box::new(inner_type), *is_mutable))
            }
            Expression::FieldAccess { object, field } => {
                // Infer type of field access (self.x, obj.field)
                let base_type = self.infer_expression_type(object)?;
                
                // Get struct definition to find field type
                // Handle both Named and Reference(Named)
                let struct_name = match &base_type {
                    Type::Named(name) => name.clone(),
                    Type::Reference(inner, _) => {
                        match &**inner {
                            Type::Named(name) => name.clone(),
                            _ => return Ok(Type::I32), // Fallback
                        }
                    }
                    _ => return Ok(Type::I32), // Fallback
                };
                
                if let Some(struct_def) = self.struct_ast_defs.get(&struct_name) {
                    // Find field in struct definition
                    for field_def in &struct_def.fields {
                        if field_def.name == *field {
                            return Ok(field_def.ty.clone());
                        }
                    }
                }
                
                Ok(Type::I32) // Fallback if field not found
            }
            Expression::Binary { left, op, .. } => {
                // For binary operators, infer based on left operand and operator
                let left_type = self.infer_expression_type(left)?;
                
                match op {
                    // Comparison operators always return bool
                    BinaryOp::Eq | BinaryOp::NotEq | 
                    BinaryOp::Lt | BinaryOp::LtEq |
                    BinaryOp::Gt | BinaryOp::GtEq => {
                        Ok(Type::Bool)
                    }
                    // Logical operators return bool
                    BinaryOp::And | BinaryOp::Or => {
                        Ok(Type::Bool)
                    }
                    // Range operators need special handling (TODO: return Range<T>)
                    BinaryOp::Range | BinaryOp::RangeInclusive => {
                        Ok(left_type) // Placeholder
                    }
                    // Null coalesce returns left type
                    BinaryOp::NullCoalesce => {
                        Ok(left_type)
                    }
                    // Arithmetic/bitwise operators preserve operand type
                    _ => Ok(left_type)
                }
            }
            Expression::Unary { expr, .. } => {
                // For unary operators, return operand type
                // (Neg returns same type, Not returns bool, BitNot returns same type)
                self.infer_expression_type(expr)
            }
            Expression::MethodCall { receiver, method, .. } => {
                // Infer return type of method call
                // First get receiver type
                let receiver_type = self.infer_expression_type(receiver)?;
                
                // Get struct name
                let struct_name = match &receiver_type {
                    Type::Named(name) => name.clone(),
                    Type::Generic { name, .. } => name.clone(),
                    _ => return Ok(Type::I32), // Fallback for primitives
                };
                
                // Look up method signature in struct definition
                if let Some(struct_def) = self.struct_ast_defs.get(&struct_name) {
                    for method_def in &struct_def.methods {
                        if method_def.name == *method {
                            return Ok(method_def.return_type.clone().unwrap_or(Type::I32));
                        }
                    }
                }
                
                // Check trait implementations
                for (impl_type, impl_trait) in self.trait_impls.keys() {
                    if impl_type == &struct_name {
                        if let Some(trait_def) = self.trait_defs.get(impl_trait) {
                            for method_sig in &trait_def.methods {
                                if method_sig.name == *method {
                                    return Ok(method_sig.return_type.clone().unwrap_or(Type::I32));
                                }
                            }
                        }
                    }
                }
                
                Ok(Type::I32) // Fallback
            }
            Expression::Typeof(_) => {
                // typeof always returns string
                Ok(Type::String)
            }
            Expression::Call { func, .. } => {
                // Infer return type of function call
                match func.as_ref() {
                    Expression::Ident(func_name) => {
                        // Builtin string conversion functions return String
                        if func_name == "i32_to_string" || func_name == "f64_to_string" || 
                           func_name == "bool_to_string" || func_name.ends_with("_to_string") {
                            return Ok(Type::String);
                        }
                        
                        // Check function definitions we've compiled
                        if let Some(func_def) = self.function_defs.get(func_name.as_str()) {
                            return Ok(func_def.return_type.clone().unwrap_or(Type::I32));
                        }
                        
                        Ok(Type::I32) // Fallback
                    }
                    _ => Ok(Type::I32),
                }
            }
            _ => Ok(Type::I32), // Default for complex expressions
        };
        result
    }

}
