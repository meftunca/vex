// src/codegen/generics.rs
use super::*;
use vex_ast::*;
use std::collections::HashMap;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn instantiate_generic_function(
        &mut self,
        func_def: &Function,
        type_args: &[Type],
    ) -> Result<inkwell::values::FunctionValue<'ctx>, String> {
        let type_names: Vec<String> = type_args.iter().map(|t| self.type_to_string(t)).collect();
        let mangled_name = format!("{}_{}", func_def.name, type_names.join("_"));

        if let Some(fn_val) = self.functions.get(&mangled_name) {
            return Ok(*fn_val);
        }

        let mut type_subst = HashMap::new();
        for (i, type_param) in func_def.type_params.iter().enumerate() {
            if let Some(concrete_type) = type_args.get(i) {
                type_subst.insert(type_param.name.clone(), concrete_type.clone());
            }
        }

        let subst_func = self.substitute_types_in_function(func_def, &type_subst)?;

        let saved_current_function = self.current_function;
        let saved_insert_block = self.builder.get_insert_block();
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_variable_types = std::mem::take(&mut self.variable_types);
        let saved_variable_struct_names = std::mem::take(&mut self.variable_struct_names);

        let fn_val = self.declare_function(&subst_func)?;
        self.functions.insert(mangled_name.clone(), fn_val);

        self.compile_function(&subst_func)?;

        self.current_function = saved_current_function;
        self.variables = saved_variables;
        self.variable_types = saved_variable_types;
        self.variable_struct_names = saved_variable_struct_names;

        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        Ok(fn_val)
    }

    pub(crate) fn instantiate_generic_struct(
        &mut self,
        struct_name: &str,
        type_args: &[Type],
    ) -> Result<String, String> {
        // uses MAX_GENERIC_DEPTH from super
        for type_arg in type_args {
            let depth = self.get_generic_depth(type_arg);
            if depth > super::MAX_GENERIC_DEPTH {
                return Err(format!(
                    "Generic type nesting too deep (depth {}, max {}): {}",
                    depth,
                    super::MAX_GENERIC_DEPTH,
                    self.type_to_string(type_arg)
                ));
            }
        }

        let type_arg_strings: Vec<String> =
            type_args.iter().map(|t| self.type_to_string(t)).collect();
        let cache_key = (struct_name.to_string(), type_arg_strings.clone());

        if let Some(mangled_name) = self.generic_instantiations.get(&cache_key) {
            return Ok(mangled_name.clone());
        }

        let struct_ast = self
            .struct_ast_defs
            .get(struct_name)
            .cloned()
            .ok_or_else(|| format!("Generic struct '{}' not found", struct_name))?;

        if struct_ast.type_params.len() != type_args.len() {
            return Err(format!(
                "Struct '{}' expects {} type parameters, got {}",
                struct_name,
                struct_ast.type_params.len(),
                type_args.len()
            ));
        }

        let mut type_subst = HashMap::new();
        for (param, arg) in struct_ast.type_params.iter().zip(type_args.iter()) {
            type_subst.insert(param.name.clone(), arg.clone());
        }

        let mangled_name = format!("{}_{}", struct_name, type_arg_strings.join("_"));

        let specialized_fields: Vec<(String, Type)> = struct_ast
            .fields
            .iter()
            .map(|f| {
                let substituted_ty = self.substitute_type(&f.ty, &type_subst);
                (f.name.clone(), substituted_ty)
            })
            .collect();

        // StructDef type lives in super (as in the original file's usage)
        use super::StructDef;
        self.struct_defs.insert(
            mangled_name.clone(),
            StructDef { fields: specialized_fields },
        );

        self.generic_instantiations.insert(cache_key, mangled_name.clone());

        Ok(mangled_name)
    }

    pub(crate) fn infer_type_args_from_call(
        &mut self,
        _func_def: &Function,
        args: &[Expression],
    ) -> Result<Vec<Type>, String> {
        let mut type_args = Vec::new();
        for arg in args {
            let arg_type = self.infer_expression_type(arg)?;
            type_args.push(arg_type);
        }
        Ok(type_args)
    }

    fn substitute_types_in_function(
        &self,
        func: &Function,
        type_subst: &HashMap<String, Type>,
    ) -> Result<Function, String> {
        let mut new_func = func.clone();
        new_func.type_params.clear();
        for param in &mut new_func.params {
            param.ty = self.substitute_type(&param.ty, type_subst);
        }
        if let Some(ret_ty) = &new_func.return_type {
            new_func.return_type = Some(self.substitute_type(ret_ty, type_subst));
        }
        let type_names: Vec<String> = type_subst.values().map(|t| self.type_to_string(t)).collect();
        new_func.name = format!("{}_{}", func.name, type_names.join("_"));
        Ok(new_func)
    }
}
