// src/codegen/functions/declare.rs
use super::super::*;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::FunctionValue;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn declare_function(
        &mut self,
        func: &Function,
    ) -> Result<FunctionValue<'ctx>, String> {
        println!("🔍 DECLARE_FUNCTION: entering for {}", func.name);
        println!("🔍 DECLARE_FUNCTION: is_async = {}", func.is_async);
        // If receiver exists, ensure that `Self` in the function signature is
        // substituted for the actual concrete type. This prevents `SelfType`
        // from leaking into LLVM function declarations.
        let mut func_for_decl = func.clone();
        if let Some(ref receiver) = func_for_decl.receiver {
            if let Some(struct_name) = self.extract_struct_name_from_receiver(&receiver.ty) {
                // Replace Self in receiver, params, and return_type
                func_for_decl.receiver = Some(vex_ast::Receiver {
                    name: receiver.name.clone(),
                    is_mutable: receiver.is_mutable,
                    ty: Self::replace_self_in_type(&receiver.ty, struct_name.as_str()),
                });

                for param in &mut func_for_decl.params {
                    param.ty = Self::replace_self_in_type(&param.ty, struct_name.as_str());
                }

                if let Some(rt) = &mut func_for_decl.return_type {
                    *rt = Self::replace_self_in_type(rt, struct_name.as_str());
                }
            }
        }

        let fn_name = if let Some(ref receiver) = func_for_decl.receiver {
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

        eprintln!(
            "🔍 DEBUG: func.name='{}', fn_name='{}', func.is_async={}",
            func.name, fn_name, func.is_async
        );

        // ⭐ SPECIAL HANDLING: async main() should be declared as regular main
        let is_async_main = func.name == "main" && func.is_async;
        let effective_is_async = func.is_async; // Keep async for main too
        eprintln!("🔍 DEBUG: func.name='{}', fn_name='{}', func.is_async={}, is_async_main={}, effective_is_async={}", 
            func.name, fn_name, func.is_async, is_async_main, effective_is_async);
        eprintln!(
            "🔍 Function {}: is_async={}, is_async_main={}, effective_is_async={}",
            func.name, func.is_async, is_async_main, effective_is_async
        );

        let mut param_types = vec![];
        if fn_name == "main" && !func.is_async {
            // int main(int argc, char **argv) - only for sync main (NOT async main)
            let i32_type = self.context.i32_type();
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

            param_types.push(i32_type.into()); // argc
            param_types.push(ptr_type.into()); // argv (char**)
        }

        if let Some(ref receiver) = func_for_decl.receiver {
            // ⭐ NEW: External methods (fn (p: Point)) pass receiver BY VALUE
            // Only reference receivers (fn (p: &Point!) are pointers
            let receiver_llvm_type = self.ast_type_to_llvm(&receiver.ty);
            param_types.push(receiver_llvm_type.into());
        }
        for param in &func_for_decl.params {
            let param_llvm_type = self.ast_type_to_llvm(&param.ty);
            // ⭐ NEW: Struct parameters are now passed BY VALUE (as StructType)
            // ast_type_to_llvm now returns StructType directly for structs
            // This matches the new return-by-value semantics
            param_types.push(param_llvm_type.into());
        }

        // Variadic support: fn format(template: string, args: ...any)
        // In LLVM, variadic functions use is_var_args=true in fn_type
        let is_variadic = func.is_variadic;

        let fn_val = if let Some(ref ty) = func_for_decl.return_type {
            println!("🔍 ABOUT TO CHECK RETURN TYPE for {}", fn_name);
            eprintln!("🔍 Function {} return type check: {:?}", fn_name, ty);

            // ⭐ SPECIAL: Type::Nil should be treated as void (no return value)
            if matches!(ty, Type::Nil) {
                eprintln!("🟢 Function {} return type: nil (void)", fn_name);
                let fn_type = self.context.void_type().fn_type(&param_types, is_variadic);
                let fn_val = self.module.add_function(&fn_name, fn_type, None);
                self.functions.insert(fn_name.clone(), fn_val);
                return Ok(fn_val);
            }

            // If this is a method and return type is SelfType, replace with concrete struct
            let actual_ret_ty = if matches!(ty, vex_ast::Type::SelfType) {
                if let Some(ref receiver) = func_for_decl.receiver {
                    if let Some(struct_name) = self.extract_struct_name_from_receiver(&receiver.ty)
                    {
                        vex_ast::Type::Named(struct_name)
                    } else {
                        ty.clone()
                    }
                } else {
                    ty.clone()
                }
            } else {
                ty.clone()
            };
            // ⭐ ASYNC: For async functions, return type is Future<T> (pointer)
            // Wrapper returns Future<T>, resume function returns CoroStatus (i32)
            let mut llvm_ret = if effective_is_async {
                // Async wrapper returns Future<T> = void* pointer
                BasicTypeEnum::PointerType(self.context.ptr_type(inkwell::AddressSpace::default()))
            } else {
                self.ast_type_to_llvm(&actual_ret_ty)
            };

            eprintln!(
                "🟢 Declaring function {} with return AST type: {:?}",
                fn_name, actual_ret_ty
            );
            eprintln!(
                "🟢 Converted to LLVM type: {:?}{}",
                llvm_ret,
                if effective_is_async {
                    " (async -> Future)"
                } else {
                    ""
                }
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

        // ⭐ CRITICAL FIX: For function overloading, generate mangled name with parameter types
        // This allows multiple functions with same base name but different signatures
        let mangled_fn_name = if !func.params.is_empty() && !func.type_params.is_empty() {
            // Generic function - use base name (will be instantiated with type args later)
            fn_name.clone()
        } else if !func.params.is_empty() {
            // Non-generic function with parameters - mangle with parameter types
            let mut param_suffix = String::new();
            for param in &func.params {
                param_suffix.push_str(&self.generate_type_suffix(&param.ty));
            }
            let mangled = format!("{}{}", fn_name, param_suffix);
            eprintln!("🔧 Function overload: {} → {}", fn_name, mangled);

            // Register with BOTH names:
            // 1. Mangled name for type-specific lookup
            self.functions.insert(mangled.clone(), fn_val);
            // 2. Base name for generic lookup (will be overridden by last overload, but that's OK)
            fn_name.clone()
        } else {
            fn_name.clone()
        };

        self.functions.insert(mangled_fn_name, fn_val);
        Ok(fn_val)
    }
}
