// src/codegen/methods.rs
use super::*;
use vex_ast::*;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn declare_struct_method(
        &mut self,
        struct_name: &str,
        method: &Function,
    ) -> Result<(), String> {
        let mangled_name = format!("{}_{}", struct_name, method.name);

        let mut param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = Vec::new();
        if let Some(ref receiver) = method.receiver {
            param_types.push(self.ast_type_to_llvm(&receiver.ty).into());
        }
        for param in &method.params {
            param_types.push(self.ast_type_to_llvm(&param.ty).into());
        }

        let ret_type = if let Some(ref ty) = method.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            inkwell::types::BasicTypeEnum::IntType(self.context.i32_type())
        };

        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_types, false),
        };

        let fn_val = self.module.add_function(&mangled_name, fn_type, None);
        self.functions.insert(mangled_name.clone(), fn_val);

        let mut mangled_method = method.clone();
        mangled_method.name = mangled_name.clone();
        self.function_defs.insert(mangled_name, mangled_method);
        Ok(())
    }

    pub(crate) fn compile_struct_method(
        &mut self,
        struct_name: &str,
        method: &Function,
    ) -> Result<(), String> {
        let mangled_name = format!("{}_{}", struct_name, method.name);
        let fn_val = *self
            .functions
            .get(&mangled_name)
            .ok_or_else(|| format!("Struct method {} not found", mangled_name))?;

        self.current_function = Some(fn_val);

        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();

        let mut param_offset = 0;
        if let Some(ref receiver) = method.receiver {
            let param_val = fn_val.get_nth_param(0).ok_or("Missing receiver parameter")?;
            let receiver_ty = self.ast_type_to_llvm(&receiver.ty);

            let alloca = self
                .builder
                .build_alloca(receiver_ty, "self")
                .map_err(|e| format!("Failed to create receiver alloca: {}", e))?;

            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store receiver: {}", e))?;

            self.variables.insert("self".to_string(), alloca);
            self.variable_types.insert("self".to_string(), receiver_ty);

            let struct_name_opt = match &receiver.ty {
                Type::Named(name) => Some(name.clone()),
                Type::Reference(inner, _) => {
                    if let Type::Named(name) = &**inner { Some(name.clone()) } else { None }
                }
                _ => None,
            };

            eprintln!(
                "🔧 compile_struct_method: struct={}, receiver.ty={:?}, struct_name_opt={:?}",
                struct_name, receiver.ty, struct_name_opt
            );

            if let Some(name) = struct_name_opt {
                eprintln!("   ✅ Tracking 'self' as struct: {}", name);
                self.variable_struct_names.insert("self".to_string(), name);
            } else {
                eprintln!("   ❌ No struct name extracted from receiver type!");
            }

            param_offset = 1;
        }

        for (i, param) in method.params.iter().enumerate() {
            let param_val = fn_val
                .get_nth_param((i + param_offset) as u32)
                .ok_or_else(|| format!("Missing parameter {}", param.name))?;

            let param_ty = self.ast_type_to_llvm(&param.ty);
            let alloca = self
                .builder
                .build_alloca(param_ty, &param.name)
                .map_err(|e| format!("Failed to create parameter alloca: {}", e))?;

            self.builder
                .build_store(alloca, param_val)
                .map_err(|e| format!("Failed to store parameter: {}", e))?;

            self.variables.insert(param.name.clone(), alloca);
            self.variable_types.insert(param.name.clone(), param_ty);
            self.track_param_struct_name(&param.name, &param.ty);
        }

        for stmt in &method.body.statements {
            self.compile_statement(stmt)?;
        }

        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            if method.return_type.is_some() {
                return Err(format!("Function {} must return a value", mangled_name));
            } else {
                let ret_val = self.context.i32_type().const_int(0, false);
                self.builder
                    .build_return(Some(&ret_val))
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            }
        }

        Ok(())
    }

    pub(crate) fn track_param_struct_name(&mut self, param_name: &str, param_ty: &Type) {
        match param_ty {
            Type::Named(struct_name) => {
                if self.struct_defs.contains_key(struct_name) {
                    self.variable_struct_names
                        .insert(param_name.to_string(), struct_name.clone());
                }
            }
            Type::Generic { name, type_args } => {
                if let Ok(mangled_name) = self.instantiate_generic_struct(name, type_args) {
                    self.variable_struct_names
                        .insert(param_name.to_string(), mangled_name);
                }
            }
            _ => {}
        }
    }
}
