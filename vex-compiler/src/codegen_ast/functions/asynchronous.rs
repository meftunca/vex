// src/codegen/functions/asynchronous.rs
use super::super::*;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_async_function(&mut self, func: &Function) -> Result<(), String> {
        let fn_name = &func.name;
        let fn_val = *self
            .functions
            .get(fn_name)
            .ok_or_else(|| format!("Async function {} not declared", fn_name))?;

        self.current_function = Some(fn_val);

        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        self.variables.clear();
        self.variable_types.clear();
        self.variable_struct_names.clear();

        for (i, param) in func.params.iter().enumerate() {
            let param_val = fn_val
                .get_nth_param(i as u32)
                .ok_or_else(|| format!("Could not get parameter {} for function {}", i, fn_name))?;

            let param_type = self.ast_type_to_llvm(&param.ty);
            let ptr = self
                .builder
                .build_alloca(param_type, &param.name)
                .map_err(|e| format!("Failed to allocate parameter: {}", e))?;

            self.builder
                .build_store(ptr, param_val)
                .map_err(|e| format!("Failed to store parameter: {}", e))?;

            self.variables.insert(param.name.clone(), ptr);
            self.variable_types.insert(param.name.clone(), param_type);
        }

        self.compile_block(&func.body)?;

        let current_block = self.builder.get_insert_block().unwrap();
        if current_block.get_terminator().is_none() {
            if let Some(ret_ty) = &func.return_type {
                let default_val = self.get_default_value(&self.ast_type_to_llvm(ret_ty));
                self.builder
                    .build_return(Some(&default_val))
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            } else {
                self.builder
                    .build_return(None)
                    .map_err(|e| format!("Failed to build return: {}", e))?;
            }
        }

        Ok(())
    }
}
