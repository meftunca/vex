// Type conversion and inference
// Contains ast_type_to_llvm, infer_expression_type and related methods

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use std::collections::HashMap;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Convert AST Type to LLVM BasicTypeEnum
    pub(crate) fn ast_type_to_llvm(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::I8 => BasicTypeEnum::IntType(self.context.i8_type()),
            Type::I16 => BasicTypeEnum::IntType(self.context.i16_type()),
            Type::I32 => BasicTypeEnum::IntType(self.context.i32_type()),
            Type::I64 => BasicTypeEnum::IntType(self.context.i64_type()),
            Type::U8 => BasicTypeEnum::IntType(self.context.i8_type()),
            Type::U16 => BasicTypeEnum::IntType(self.context.i16_type()),
            Type::U32 => BasicTypeEnum::IntType(self.context.i32_type()),
            Type::U64 => BasicTypeEnum::IntType(self.context.i64_type()),
            Type::F32 => BasicTypeEnum::FloatType(self.context.f32_type()),
            Type::F64 => BasicTypeEnum::FloatType(self.context.f64_type()),
            Type::Bool => BasicTypeEnum::IntType(self.context.bool_type()),
            Type::String => {
                // String as i8* (C-style string pointer)
                BasicTypeEnum::PointerType(
                    self.context
                        .i8_type()
                        .ptr_type(inkwell::AddressSpace::default()),
                )
            }
            Type::Nil => {
                // Nil as void/i8 (placeholder)
                // In LLVM, we use i8 as a minimal type
                BasicTypeEnum::IntType(self.context.i8_type())
            }
            Type::Array(elem_ty, size) => {
                let elem_llvm = self.ast_type_to_llvm(elem_ty);
                match elem_llvm {
                    BasicTypeEnum::IntType(it) => {
                        BasicTypeEnum::ArrayType(it.array_type(*size as u32))
                    }
                    BasicTypeEnum::FloatType(ft) => {
                        BasicTypeEnum::ArrayType(ft.array_type(*size as u32))
                    }
                    BasicTypeEnum::ArrayType(at) => {
                        BasicTypeEnum::ArrayType(at.array_type(*size as u32))
                    }
                    _ => BasicTypeEnum::IntType(self.context.i32_type()), // fallback
                }
            }
            Type::Reference(inner_ty, _is_mutable) => {
                // Reference type: &T or &mut T
                // In LLVM, references are just pointers
                let inner_llvm = self.ast_type_to_llvm(inner_ty);
                match inner_llvm {
                    BasicTypeEnum::IntType(it) => {
                        BasicTypeEnum::PointerType(it.ptr_type(inkwell::AddressSpace::default()))
                    }
                    BasicTypeEnum::FloatType(ft) => {
                        BasicTypeEnum::PointerType(ft.ptr_type(inkwell::AddressSpace::default()))
                    }
                    BasicTypeEnum::ArrayType(at) => {
                        BasicTypeEnum::PointerType(at.ptr_type(inkwell::AddressSpace::default()))
                    }
                    BasicTypeEnum::StructType(st) => {
                        BasicTypeEnum::PointerType(st.ptr_type(inkwell::AddressSpace::default()))
                    }
                    BasicTypeEnum::PointerType(pt) => {
                        // Already a pointer, just return it
                        BasicTypeEnum::PointerType(pt)
                    }
                    _ => BasicTypeEnum::PointerType(
                        self.context
                            .i32_type()
                            .ptr_type(inkwell::AddressSpace::default()),
                    ),
                }
            }
            Type::Named(name) => {
                // Handle custom struct types
                // IMPORTANT: Structs are always passed by pointer for zero-copy semantics

                // Check if this struct is registered
                if let Some(struct_def) = self.struct_defs.get(name) {
                    // Build struct type from registry
                    let field_types: Vec<BasicTypeEnum> = struct_def
                        .fields
                        .iter()
                        .map(|(_, field_ty)| self.ast_type_to_llvm(field_ty))
                        .collect();

                    let struct_ty = self.context.struct_type(&field_types, false);

                    // Return pointer to struct (zero-copy!)
                    BasicTypeEnum::PointerType(struct_ty.ptr_type(inkwell::AddressSpace::default()))
                } else if name == "AnonymousStruct" {
                    // Placeholder for inferred structs
                    BasicTypeEnum::IntType(self.context.i32_type())
                } else {
                    // Unknown named type, default to i32
                    // TODO: Better error handling
                    BasicTypeEnum::IntType(self.context.i32_type())
                }
            }
            Type::Union(types) => {
                // Union type: T1 | T2 | T3
                // For now, use the first type as the LLVM representation
                // TODO: Implement proper tagged union with discriminator
                if let Some(first_type) = types.first() {
                    self.ast_type_to_llvm(first_type)
                } else {
                    // Empty union, fallback to i32
                    BasicTypeEnum::IntType(self.context.i32_type())
                }
            }
            _ => {
                // Default to i32 for unsupported types
                BasicTypeEnum::IntType(self.context.i32_type())
            }
        }
    }

    /// Infer AST type from LLVM type (for type inference)
    pub(crate) fn infer_ast_type_from_llvm(
        &self,
        llvm_type: BasicTypeEnum<'ctx>,
    ) -> Result<Type, String> {
        match llvm_type {
            BasicTypeEnum::IntType(int_ty) => {
                let bit_width = int_ty.get_bit_width();
                match bit_width {
                    8 => Ok(Type::I8),
                    16 => Ok(Type::I16),
                    32 => Ok(Type::I32),
                    64 => Ok(Type::I64),
                    1 => Ok(Type::Bool),
                    _ => Err(format!("Unsupported integer bit width: {}", bit_width)),
                }
            }
            BasicTypeEnum::FloatType(float_ty) => {
                // Check if f32 or f64 based on LLVM representation
                if float_ty == self.context.f32_type() {
                    Ok(Type::F32)
                } else {
                    Ok(Type::F64)
                }
            }
            BasicTypeEnum::PointerType(_) => Ok(Type::String), // Assume string for now
            BasicTypeEnum::ArrayType(arr_ty) => {
                let elem_ty = arr_ty.get_element_type();
                let size = arr_ty.len() as usize;

                // Recursively infer element type
                if let BasicTypeEnum::IntType(int_ty) = elem_ty {
                    let bit_width = int_ty.get_bit_width();
                    let elem_ast_ty = match bit_width {
                        32 => Type::I32,
                        64 => Type::I64,
                        _ => return Err("Unsupported array element type".to_string()),
                    };
                    Ok(Type::Array(Box::new(elem_ast_ty), size))
                } else {
                    Err("Unsupported array element type".to_string())
                }
            }
            BasicTypeEnum::StructType(_) => {
                // For struct types, we can't fully infer without metadata
                // For now, use a placeholder named type
                Ok(Type::Named("AnonymousStruct".to_string()))
            }
            _ => Err("Cannot infer type from LLVM type".to_string()),
        }
    }

    /// Infer expression type for type inference
    pub(crate) fn infer_expression_type(&self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::IntLiteral(_) => Ok(Type::I32),
            Expression::FloatLiteral(_) => Ok(Type::F64),
            Expression::StringLiteral(_) => Ok(Type::String),
            Expression::FStringLiteral(_) => Ok(Type::String),
            Expression::BoolLiteral(_) => Ok(Type::Bool),
            Expression::Ident(name) => {
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
            _ => Ok(Type::I32), // Default for complex expressions
        }
    }

    /// Convert Type to string for mangling
    pub(crate) fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Named(name) => name.clone(),
            Type::Generic { name, .. } => name.clone(),
            _ => "unknown".to_string(),
        }
    }

    /// Substitute type parameters in a type
    pub(crate) fn substitute_type(&self, ty: &Type, type_subst: &HashMap<String, Type>) -> Type {
        match ty {
            Type::Named(name) => {
                // Check if this is a type parameter
                if let Some(concrete_ty) = type_subst.get(name) {
                    concrete_ty.clone()
                } else {
                    ty.clone()
                }
            }
            Type::Reference(inner, is_mut) => {
                Type::Reference(Box::new(self.substitute_type(inner, type_subst)), *is_mut)
            }
            Type::Array(elem_ty, size) => {
                Type::Array(Box::new(self.substitute_type(elem_ty, type_subst)), *size)
            }
            _ => ty.clone(),
        }
    }

    /// Resolve a type, expanding type aliases
    pub(crate) fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Named(name) => {
                // Check if this is a type alias
                if let Some(resolved) = self.type_aliases.get(name) {
                    // Recursively resolve in case the alias points to another alias
                    self.resolve_type(resolved)
                } else {
                    ty.clone()
                }
            }
            // For complex types, recursively resolve components
            Type::Array(inner, size) => Type::Array(Box::new(self.resolve_type(inner)), *size),
            Type::Reference(inner, is_mut) => {
                Type::Reference(Box::new(self.resolve_type(inner)), *is_mut)
            }
            Type::Generic { name, type_args } => Type::Generic {
                name: name.clone(),
                type_args: type_args.iter().map(|t| self.resolve_type(t)).collect(),
            },
            // Primitive types don't need resolution
            _ => ty.clone(),
        }
    }
}
