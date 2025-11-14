// src/codegen/functions/declare.rs
use super::super::*;
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
                Type::Generic { name, .. } => name.clone(), // Generic types like Container<T>
                Type::Reference(inner, _) => match &**inner {
                    Type::Named(name) => name.clone(),
                    Type::Generic { name, .. } => name.clone(),
                    // Handle Vec<T>, Box<T>, etc.
                    Type::Vec(_) => "Vec".to_string(),
                    Type::Box(_) => "Box".to_string(),
                    Type::Option(_) => "Option".to_string(),
                    Type::Result(_, _) => "Result".to_string(),
                    _ => {
                        eprintln!("⚠️  Unsupported receiver type: {:?}", inner);
                        return Err(format!(
                            "Receiver must be a named type or reference to named type, got {:?}",
                            inner
                        ));
                    }
                },
                // Direct Vec/Box without reference
                Type::Vec(_) => "Vec".to_string(),
                Type::Box(_) => "Box".to_string(),
                Type::Option(_) => "Option".to_string(),
                Type::Result(_, _) => "Result".to_string(),
                _ => {
                    eprintln!("⚠️  Unsupported receiver type: {:?}", receiver.ty);
                    return Err(format!(
                        "Receiver must be a named type or reference to named type, got {:?}",
                        receiver.ty
                    ));
                }
            };
            // Check if name is already mangled (imported methods)
            if func.name.starts_with(&format!("{}_", type_name)) {
                func.name.clone()
            } else {
                format!("{}_{}", type_name, func.name)
            }
        } else {
            func.name.clone()
        };

        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();
        if let Some(ref receiver) = func.receiver {
            // ⭐ NEW: External methods (fn (p: Point)) pass receiver BY VALUE
            // Only reference receivers (fn (p: &Point!) are pointers
            let receiver_llvm_type = self.ast_type_to_llvm(&receiver.ty);
            param_types.push(receiver_llvm_type.into());
        }
        for param in &func.params {
            let param_llvm_type = self.ast_type_to_llvm(&param.ty);
            // ⭐ NEW: Struct parameters are now passed BY VALUE (as StructType)
            // ast_type_to_llvm now returns StructType directly for structs
            // This matches the new return-by-value semantics
            param_types.push(param_llvm_type.into());
        }

        // Variadic support: fn format(template: string, args: ...any)
        // In LLVM, variadic functions use is_var_args=true in fn_type
        let is_variadic = func.is_variadic;

        let fn_val = if let Some(ref ty) = func.return_type {
            // ⭐ ASYNC: For async functions, return type is Future<T> (pointer)
            // Wrapper returns Future<T>, resume function returns CoroStatus (i32)
            let mut llvm_ret = if func.is_async {
                // Async wrapper returns Future<T> = void* pointer
                BasicTypeEnum::PointerType(self.context.ptr_type(inkwell::AddressSpace::default()))
            } else {
                self.ast_type_to_llvm(ty)
            };

            eprintln!(
                "🟢 Function {} return type: {:?} → LLVM: {:?}{}",
                fn_name, ty, llvm_ret,
                if func.is_async { " (async -> Future)" } else { "" }
            );

            // ⚠️ CRITICAL FIX: For String return type (Type::String), verify we have PointerType
            if matches!(ty, Type::String) && !matches!(llvm_ret, BasicTypeEnum::PointerType(_)) {
                eprintln!(
                    "⚠️ WARNING: String return type should be PointerType, got {:?}",
                    llvm_ret
                );
                eprintln!("   Forcing to pointer type for str return");
                llvm_ret = BasicTypeEnum::PointerType(
                    self.context.ptr_type(inkwell::AddressSpace::default()),
                );
            }

            let fn_type = match llvm_ret {
                BasicTypeEnum::IntType(t) => t.fn_type(&param_types, is_variadic),
                BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, is_variadic),
                BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, is_variadic),
                BasicTypeEnum::StructType(t) => t.fn_type(&param_types, is_variadic),
                BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, is_variadic),
                _ => {
                    return Err(format!(
                        "Unsupported return type for function {}",
                        func.name
                    ))
                }
            };

            self.module.add_function(&fn_name, fn_type, None)
        } else {
            eprintln!("🟢 Function {} return type: void", fn_name);
            let fn_type = self.context.void_type().fn_type(&param_types, is_variadic);
            self.module.add_function(&fn_name, fn_type, None)
        };
        self.functions.insert(fn_name.clone(), fn_val);
        Ok(fn_val)
    }
}
