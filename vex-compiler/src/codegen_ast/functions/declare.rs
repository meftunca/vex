// src/codegen/functions/declare.rs
use super::super::*;
use crate::{debug_log, debug_println};
use inkwell::types::BasicTypeEnum;
use inkwell::values::FunctionValue;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn declare_function(
        &mut self,
        func: &Function,
    ) -> Result<FunctionValue<'ctx>, String> {
        debug_println!("🔍 DECLARE_FUNCTION: entering for {}", func.name);
        debug_println!("🔍 DECLARE_FUNCTION: is_async = {}", func.is_async);

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

        let fn_name = if func_for_decl.is_static {
            let type_name = func_for_decl
                .static_type
                .as_ref()
                .ok_or_else(|| "Static method missing type name".to_string())?;

            if func_for_decl.name.starts_with(&format!("{}_", type_name)) {
                func_for_decl.name.clone()
            } else {
                let encoded_method_name = Self::encode_operator_name(&func_for_decl.name);
                format!("{}_{}", type_name, encoded_method_name)
            }
        } else if let Some(ref receiver) = func_for_decl.receiver {
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
            // For methods, func.name is already mangled by program.rs or compile_function
            // Just use it directly
            func.name.clone()
        } else {
            func.name.clone()
        };

        debug_log!(
            "🔍 DEBUG: func.name='{}', fn_name='{}', func.is_async={}",
            func.name,
            fn_name,
            func.is_async
        );

        // ⭐ COMPUTE MANGLED NAME EARLY: For regular functions with params, mangle before LLVM declaration
        // Methods are already pre-mangled by program.rs, generics use base name
        let mangled_llvm_name = if !func.params.is_empty() 
            && !func.type_params.is_empty() {
            // Generic function - use base name (will be instantiated with type args later)
            fn_name.clone()
        } else if !func.params.is_empty() && func_for_decl.receiver.is_none() {
            // Non-generic regular function with parameters - mangle now
            let mangled = self.mangle_function_name(&fn_name, &func.params, true);
            eprintln!("🔧 Function overload (LLVM): {} → {}", fn_name, mangled);
            mangled
        } else {
            // Methods (pre-mangled), no-param functions, or special cases
            fn_name.clone()
        };

        // ⭐ SPECIAL HANDLING: async main() should be declared as regular main
        let is_async_main = func.name == "main" && func.is_async;
        let effective_is_async = func.is_async; // Keep async for main too
        debug_log!("🔍 DEBUG: func.name='{}', fn_name='{}', func.is_async={}, is_async_main={}, effective_is_async={}", 
            func.name, fn_name, func.is_async, is_async_main, effective_is_async);
        debug_log!(
            "🔍 Function {}: is_async={}, is_async_main={}, effective_is_async={}",
            func.name,
            func.is_async,
            is_async_main,
            effective_is_async
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
            debug_println!("🔍 ABOUT TO CHECK RETURN TYPE for {}", fn_name);
            debug_log!("🔍 Function {} return type check: {:?}", fn_name, ty);

            // ⭐ SPECIAL: Type::Nil should be treated as void (no return value)
            if matches!(ty, Type::Nil) {
                debug_log!("🟢 Function {} return type: nil (void)", mangled_llvm_name);
                let fn_type = self.context.void_type().fn_type(&param_types, is_variadic);
                let fn_val = self.module.add_function(&mangled_llvm_name, fn_type, None);
                self.functions.insert(mangled_llvm_name.clone(), fn_val);
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
                mangled_llvm_name, actual_ret_ty
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

            // ⭐ LINKAGE TRICK: Control visibility for Dead Code Elimination (DCE)
            // If LINKAGE_MODE != "EXTERNAL", we make non-exported functions Internal.
            // Internal functions can be deleted by LLVM if they are not used.
            // main() is always External.
            use inkwell::module::Linkage;
            
            let linkage_mode = std::env::var("LINKAGE_MODE").unwrap_or_default();
            let force_external = linkage_mode == "EXTERNAL";
            
            let linkage = if fn_name == "main" {
                Some(Linkage::External)
            } else if force_external {
                Some(Linkage::External)
            } else if func.is_exported {
                Some(Linkage::External)
            } else {
                // "Trick": Make it internal so GlobalDCE can remove it if unused
                Some(Linkage::Internal)
            };

            self.module.add_function(&fn_name, fn_type, linkage)
        } else {
            // ⭐ SPECIAL HANDLING: main() must always return i32 (0) for C runtime compatibility
            if fn_name == "main" && !func.is_async {
                eprintln!(
                    "🟢 Function {} return type: void -> i32 (forced for main)",
                    fn_name
                );
                let i32_type = self.context.i32_type();
                let fn_type = i32_type.fn_type(&param_types, is_variadic);
                self.module.add_function(&mangled_llvm_name, fn_type, Some(inkwell::module::Linkage::External))
            } else {
                eprintln!("🟢 Function {} return type: void", fn_name);
                let fn_type = self.context.void_type().fn_type(&param_types, is_variadic);
                
                // ⭐ LINKAGE TRICK (Void functions)
                use inkwell::module::Linkage;
                let linkage_mode = std::env::var("LINKAGE_MODE").unwrap_or_default();
                let force_external = linkage_mode == "EXTERNAL";
                
                let linkage = if fn_name == "main" {
                    Some(Linkage::External)
                } else if force_external {
                    Some(Linkage::External)
                } else if func.is_exported {
                    Some(Linkage::External)
                } else {
                    Some(Linkage::Internal)
                };
                
                self.module.add_function(&mangled_llvm_name, fn_type, linkage)
            }
        };

        // Register in functions map with BOTH mangled name AND base name
        // This enables both direct calls (double_i32_1) and function pointers (double)
        self.register_function(&func_for_decl.name, &mangled_llvm_name, fn_val);
        
        Ok(fn_val)
    }
}
