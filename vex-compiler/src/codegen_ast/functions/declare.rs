// src/codegen/functions/declare.rs
use super::super::*;
use vex_ast::*;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::FunctionValue;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn declare_function(
        &mut self,
        func: &Function,
    ) -> Result<FunctionValue<'ctx>, String> {
        let fn_name = if let Some(ref receiver) = func.receiver {
            let type_name = match &receiver.ty {
                Type::Named(name) => name.clone(),
                Type::Reference(inner, _) => {
                    if let Type::Named(name) = &**inner { name.clone() } else {
                        return Err("Receiver must be a named type or reference to named type".to_string());
                    }
                }
                _ => {
                    return Err("Receiver must be a named type or reference to named type".to_string());
                }
            };
            format!("{}_{}", type_name, func.name)
        } else { func.name.clone() };

        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();
        if let Some(ref receiver) = func.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }
        for param in &func.params {
            let param_llvm_type = self.ast_type_to_llvm(&param.ty);
            let is_struct = match &param.ty {
                Type::Named(type_name) => self.struct_defs.contains_key(type_name),
                _ => false,
            };
            if is_struct {
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                param_types.push(ptr_type.into());
            } else {
                param_types.push(param_llvm_type.into());
            }
        }

        let ret_type = if let Some(ref ty) = func.return_type {
            let llvm_ret = self.ast_type_to_llvm(ty);
            eprintln!("🟢 Function {} return type: {:?} → LLVM: {:?}", fn_name, ty, llvm_ret);
            llvm_ret
        } else {
            BasicTypeEnum::IntType(self.context.i32_type())
        };

        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            _ => return Err(format!("Unsupported return type for function {}", func.name)),
        };

        let fn_val = self.module.add_function(&fn_name, fn_type, None);
        self.functions.insert(fn_name.clone(), fn_val);
        Ok(fn_val)
    }
}
