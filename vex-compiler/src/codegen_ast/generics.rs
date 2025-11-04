// Generic type instantiation and monomorphization

use super::ASTCodeGen;
use inkwell::values::FunctionValue;
use std::collections::HashMap;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Instantiate a generic function with concrete types
    /// Creates a monomorphized version: identity<i32> -> identity_i32
    pub(crate) fn instantiate_generic_function(
        &mut self,
        func_def: &Function,
        type_args: &[Type],
    ) -> Result<FunctionValue<'ctx>, String> {
        // Generate mangled name: identity_i32, pair_i32_f64
        let type_names: Vec<String> = type_args.iter().map(|t| self.type_to_string(t)).collect();
        let mangled_name = format!("{}_{}", func_def.name, type_names.join("_"));

        // Check if already instantiated
        if let Some(fn_val) = self.functions.get(&mangled_name) {
            return Ok(*fn_val);
        }

        // Create type substitution map: T -> i32, U -> f64
        let mut type_subst = HashMap::new();
        for (i, type_param) in func_def.type_params.iter().enumerate() {
            if let Some(concrete_type) = type_args.get(i) {
                type_subst.insert(type_param.name.clone(), concrete_type.clone());
            }
        }

        // Substitute types in function signature
        let subst_func = self.substitute_types_in_function(func_def, &type_subst)?;

        // Save current compilation state INCLUDING builder position
        let saved_current_function = self.current_function;
        let saved_insert_block = self.builder.get_insert_block();
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_variable_types = std::mem::take(&mut self.variable_types);
        let saved_variable_struct_names = std::mem::take(&mut self.variable_struct_names);

        // Declare and compile the specialized function
        use super::declaration::ASTCodeGen as DeclCodeGen;
        let fn_val = DeclCodeGen::declare_function(self, &subst_func)?;
        self.functions.insert(mangled_name.clone(), fn_val);

        // Compile body
        use super::compilation::ASTCodeGen as CompCodeGen;
        CompCodeGen::compile_function(self, &subst_func)?;

        // Restore compilation state INCLUDING builder position
        self.current_function = saved_current_function;
        self.variables = saved_variables;
        self.variable_types = saved_variable_types;
        self.variable_struct_names = saved_variable_struct_names;

        // Restore builder position to where we were
        if let Some(block) = saved_insert_block {
            self.builder.position_at_end(block);
        }

        Ok(fn_val)
    }

    /// Instantiate a generic struct with concrete types
    /// Creates a monomorphized version: Box<i32> -> Box_i32
    pub(crate) fn instantiate_generic_struct(
        &mut self,
        struct_name: &str,
        type_args: &[Type],
    ) -> Result<String, String> {
        use super::StructDef;

        // Check depth limit for all type arguments
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

        // Check if already instantiated (memoization)
        let type_arg_strings: Vec<String> =
            type_args.iter().map(|t| self.type_to_string(t)).collect();
        let cache_key = (struct_name.to_string(), type_arg_strings.clone());

        if let Some(mangled_name) = self.generic_instantiations.get(&cache_key) {
            return Ok(mangled_name.clone());
        }

        // Get the generic struct definition
        let struct_ast = self
            .struct_ast_defs
            .get(struct_name)
            .cloned()
            .ok_or_else(|| format!("Generic struct '{}' not found", struct_name))?;

        // Check type parameter count
        if struct_ast.type_params.len() != type_args.len() {
            return Err(format!(
                "Struct '{}' expects {} type parameters, got {}",
                struct_name,
                struct_ast.type_params.len(),
                type_args.len()
            ));
        }

        // Create type substitution map: T -> i32, U -> f64
        let mut type_subst = HashMap::new();
        for (param, arg) in struct_ast.type_params.iter().zip(type_args.iter()) {
            type_subst.insert(param.name.clone(), arg.clone());
        }

        // Generate mangled name: Box<i32> -> Box_i32
        let mangled_name = format!("{}_{}", struct_name, type_arg_strings.join("_"));

        // Substitute types in struct fields
        let specialized_fields: Vec<(String, Type)> = struct_ast
            .fields
            .iter()
            .map(|f| {
                let substituted_ty = self.substitute_type(&f.ty, &type_subst);
                (f.name.clone(), substituted_ty)
            })
            .collect();

        // Register the specialized struct
        self.struct_defs.insert(
            mangled_name.clone(),
            StructDef {
                fields: specialized_fields,
            },
        );

        // Cache the instantiation
        self.generic_instantiations
            .insert(cache_key, mangled_name.clone());

        Ok(mangled_name)
    }

    /// Track struct name for a parameter (handles both Named and Generic types)
    pub(crate) fn track_param_struct_name(&mut self, param_name: &str, param_ty: &Type) {
        match param_ty {
            Type::Named(struct_name) => {
                if self.struct_defs.contains_key(struct_name) {
                    self.variable_struct_names
                        .insert(param_name.to_string(), struct_name.clone());
                }
            }
            Type::Generic { name, type_args } => {
                // Generic struct parameter: Pair<i32, i32>
                // Instantiate to get mangled name: Pair_i32_i32
                if let Ok(mangled_name) = self.instantiate_generic_struct(name, type_args) {
                    self.variable_struct_names
                        .insert(param_name.to_string(), mangled_name);
                }
            }
            _ => {}
        }
    }

    /// Infer type arguments from function call arguments
    /// Simple version: just infer from argument types
    pub(crate) fn infer_type_args_from_call(
        &mut self,
        _func_def: &Function,
        args: &[Expression],
    ) -> Result<Vec<Type>, String> {
        // Infer type from each argument - matches order of type params
        let mut type_args = Vec::new();

        for arg in args {
            let arg_type = self.infer_expression_type(arg)?;
            type_args.push(arg_type);
        }

        Ok(type_args)
    }

    /// Substitute type parameters in a function
    pub(crate) fn substitute_types_in_function(
        &self,
        func: &Function,
        type_subst: &HashMap<String, Type>,
    ) -> Result<Function, String> {
        let mut new_func = func.clone();

        // Clear type parameters (no longer generic)
        new_func.type_params.clear();

        // Substitute in parameter types
        for param in &mut new_func.params {
            param.ty = self.substitute_type(&param.ty, type_subst);
        }

        // Substitute in return type
        if let Some(ret_ty) = &new_func.return_type {
            new_func.return_type = Some(self.substitute_type(ret_ty, type_subst));
        }

        // Update function name with mangled name
        let type_names: Vec<String> = type_subst
            .values()
            .map(|t| self.type_to_string(t))
            .collect();
        new_func.name = format!("{}_{}", func.name, type_names.join("_"));

        Ok(new_func)
    }
}

