use super::super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn ast_function_type_to_llvm(
        &self,
        params: &[Type],
        return_type: &Type,
    ) -> Result<inkwell::types::FunctionType<'ctx>, String> {
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = params
            .iter()
            .map(|t| self.ast_type_to_llvm(t).into())
            .collect();

        let ret_llvm = self.ast_type_to_llvm(return_type);

        // Create function type based on return type
        let fn_type = match ret_llvm {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            _ => return Err("Unsupported return type for function".to_string()),
        };

        Ok(fn_type)
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
                    128 => Ok(Type::I128),
                    1 => Ok(Type::Bool),
                    _ => {
                        // Note: Cannot emit diagnostic here due to &self limitation
                        // This is an internal error that shouldn't normally occur
                        Err(format!(
                            "Unsupported integer bit width: {} (internal error)",
                            bit_width
                        ))
                    }
                }
            }
            BasicTypeEnum::FloatType(float_ty) => {
                // Check float type based on LLVM representation
                if float_ty == self.context.f16_type() {
                    Ok(Type::F16)
                } else if float_ty == self.context.f32_type() {
                    Ok(Type::F32)
                } else if float_ty == self.context.f64_type() {
                    Ok(Type::F64)
                } else {
                    // Fallback to f64 if unknown
                    Ok(Type::F64)
                }
            }
            BasicTypeEnum::PointerType(_) => Ok(Type::Named("str".to_string())), // String literals are str (pointer to C string)
            BasicTypeEnum::ArrayType(arr_ty) => {
                let elem_ty = arr_ty.get_element_type();
                let size = arr_ty.len() as usize;

                // Recursively infer element type
                let elem_ast_ty = match elem_ty {
                    BasicTypeEnum::IntType(int_ty) => {
                        let bit_width = int_ty.get_bit_width();
                        match bit_width {
                            1 => Type::Bool,
                            8 => Type::I8,
                            16 => Type::I16,
                            32 => Type::I32,
                            64 => Type::I64,
                            128 => Type::I128,
                            _ => {
                                return Err(format!(
                                    "Unsupported array element int type with bit width {}",
                                    bit_width
                                ))
                            }
                        }
                    }
                    BasicTypeEnum::FloatType(float_ty) => {
                        if float_ty == self.context.f16_type() {
                            Type::F16
                        } else if float_ty == self.context.f32_type() {
                            Type::F32
                        } else if float_ty == self.context.f64_type() {
                            Type::F64
                        } else {
                            Type::F64
                        }
                    }
                    BasicTypeEnum::ArrayType(inner_arr_ty) => {
                        // Nested array: [[i32; 2]; 2]
                        let inner_elem_ty = inner_arr_ty.get_element_type();
                        let inner_size = inner_arr_ty.len() as usize;

                        let inner_ast_ty = match inner_elem_ty {
                            BasicTypeEnum::IntType(int_ty) => {
                                let bit_width = int_ty.get_bit_width();
                                match bit_width {
                                    1 => Type::Bool,
                                    8 => Type::I8,
                                    16 => Type::I16,
                                    32 => Type::I32,
                                    64 => Type::I64,
                                    128 => Type::I128,
                                    _ => {
                                        return Err(format!(
                                        "Unsupported nested array element type with bit width {}",
                                        bit_width
                                    ))
                                    }
                                }
                            }
                            _ => return Err("Unsupported nested array element type".to_string()),
                        };
                        Type::Array(Box::new(inner_ast_ty), inner_size)
                    }
                    _ => return Err(format!("Unsupported array element type: {:?}", elem_ty)),
                };
                Ok(Type::Array(Box::new(elem_ast_ty), size))
            }
            BasicTypeEnum::StructType(_) => {
                // For struct types, we can't fully infer without metadata
                // For now, use a placeholder named type
                Ok(Type::Named("AnonymousStruct".to_string()))
            }
            _ => Err("Cannot infer type from LLVM type".to_string()),
        }
    }
}
