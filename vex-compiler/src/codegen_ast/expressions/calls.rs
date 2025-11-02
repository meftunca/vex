// Function and method calls

use super::super::ASTCodeGen;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile function call
    pub(crate) fn compile_call(
        &mut self,
        func_expr: &Expression,
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Get function name
        let func_name = if let Expression::Ident(name) = func_expr {
            name
        } else {
            return Err("Complex function calls not yet supported".to_string());
        };

        // Special handling for print() builtin (no newline)
        if func_name == "print" {
            if args.len() != 1 {
                return Err("print() takes exactly one argument".to_string());
            }

            let val = self.compile_expression(&args[0])?;

            // Determine format string based on type (NO newline)
            match val {
                BasicValueEnum::IntValue(_) => {
                    self.build_printf("%d", &[val])?;
                }
                BasicValueEnum::FloatValue(_) => {
                    self.build_printf("%f", &[val])?;
                }
                BasicValueEnum::PointerValue(_) => {
                    // String (i8* pointer)
                    self.build_printf("%s", &[val])?;
                }
                _ => {
                    return Err(format!("print() doesn't support this type yet: {:?}", val));
                }
            }

            // Return 0 as dummy value
            return Ok(self.context.i32_type().const_int(0, false).into());
        }

        // Special handling for println() builtin (with newline)
        if func_name == "println" {
            if args.len() != 1 {
                return Err("println() takes exactly one argument".to_string());
            }

            let val = self.compile_expression(&args[0])?;

            // Determine format string based on type (WITH newline)
            match val {
                BasicValueEnum::IntValue(_) => {
                    self.build_printf("%d\n", &[val])?;
                }
                BasicValueEnum::FloatValue(_) => {
                    self.build_printf("%f\n", &[val])?;
                }
                BasicValueEnum::PointerValue(_) => {
                    // String (i8* pointer)
                    self.build_printf("%s\n", &[val])?;
                }
                _ => {
                    return Err(format!(
                        "println() doesn't support this type yet: {:?}",
                        val
                    ));
                }
            }

            // Return 0 as dummy value
            return Ok(self.context.i32_type().const_int(0, false).into());
        }

        // Compile arguments
        let mut arg_vals: Vec<BasicMetadataValueEnum> = Vec::new();
        for arg in args {
            let val = self.compile_expression(arg)?;
            arg_vals.push(val.into());
        }

        // Check if this is a generic function that needs instantiation
        let fn_val = if let Some(fn_val) = self.functions.get(func_name) {
            *fn_val
        } else if let Some(func_def) = self.function_defs.get(func_name).cloned() {
            // Check if it's a generic function
            if !func_def.type_params.is_empty() {
                // Infer type arguments from arguments
                // For now, simple approach: use argument types
                let type_args = self.infer_type_args_from_call(&func_def, args)?;

                // Instantiate generic function
                self.instantiate_generic_function(&func_def, &type_args)?
            } else {
                return Err(format!("Function {} not declared", func_name));
            }
        } else {
            return Err(format!("Function {} not found", func_name));
        };

        // Build call
        let call_site = self
            .builder
            .build_call(fn_val, &arg_vals, "call")
            .map_err(|e| format!("Failed to build call: {}", e))?;

        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Function call returned void".to_string())
    }

    /// Compile method call: obj.method(args)
    /// Methods are compiled as functions with the receiver as the first argument
    pub(crate) fn compile_method_call(
        &mut self,
        receiver: &Expression,
        method: &str,
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Check if this is a module-level function call (io.print, log.info, etc.)
        if let Expression::Ident(module_name) = receiver {
            // Check if this is a known module namespace
            if let Some(module_funcs) = self.module_namespaces.get(module_name) {
                // This is a module namespace, check if the method exists
                if module_funcs.contains(&method.to_string()) {
                    // Found! Call the function directly
                    let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                    for arg in args {
                        let val = self.compile_expression(arg)?;
                        arg_vals.push(val.into());
                    }

                    let fn_val = *self.functions.get(method).ok_or_else(|| {
                        format!("Module function {} not found in LLVM module", method)
                    })?;

                    let call_site = self
                        .builder
                        .build_call(fn_val, &arg_vals, "modulecall")
                        .map_err(|e| format!("Failed to build module call: {}", e))?;

                    return call_site
                        .try_as_basic_value()
                        .left()
                        .ok_or_else(|| "Module function returned void".to_string());
                }
            }

            // Legacy: Try old-style module_function naming
            let module_func_name = format!("{}_{}", module_name, method);
            if self.functions.contains_key(&module_func_name) {
                let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
                for arg in args {
                    let val = self.compile_expression(arg)?;
                    arg_vals.push(val.into());
                }

                let fn_val = *self.functions.get(&module_func_name).unwrap();
                let call_site = self
                    .builder
                    .build_call(fn_val, &arg_vals, "modulecall")
                    .map_err(|e| format!("Failed to build module call: {}", e))?;

                return call_site
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| "Module function returned void".to_string());
            }
        }

        // Get struct type from receiver (for actual method calls)
        let struct_name = if let Expression::Ident(var_name) = receiver {
            self.variable_struct_names
                .get(var_name)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "Variable {} is not a struct or module, cannot call methods",
                        var_name
                    )
                })?
        } else {
            return Err("Method calls only supported on variables for now".to_string());
        };

        // Construct method function name: StructName_method
        let method_func_name = format!("{}_{}", struct_name, method);

        // Check if method function exists (either as a struct method or trait method)
        let final_method_name = if self.functions.contains_key(&method_func_name) {
            // Found as struct method
            method_func_name
        } else {
            // Try to find trait method: TypeName_TraitName_methodName
            // Search all trait impls for this type
            let mut found_trait_method = None;
            for ((trait_name, type_name), _) in &self.trait_impls {
                if type_name == &struct_name {
                    let trait_method_name = format!("{}_{}_{}", type_name, trait_name, method);
                    if self.functions.contains_key(&trait_method_name) {
                        found_trait_method = Some(trait_method_name);
                        break;
                    }
                }
            }

            if let Some(trait_method) = found_trait_method {
                trait_method
            } else {
                return Err(format!(
                    "Method '{}' not found for struct '{}' (neither as struct method nor trait method)",
                    method, struct_name
                ));
            }
        };

        // Compile receiver (this will be the first argument)
        let receiver_val = self.compile_expression(receiver)?;

        // Compile other arguments
        let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![receiver_val.into()];
        for arg in args {
            let val = self.compile_expression(arg)?;
            arg_vals.push(val.into());
        }

        // Get method function
        let fn_val = *self
            .functions
            .get(&final_method_name)
            .ok_or_else(|| format!("Method function {} not found", final_method_name))?;

        // Build call
        let call_site = self
            .builder
            .build_call(fn_val, &arg_vals, "methodcall")
            .map_err(|e| format!("Failed to build method call: {}", e))?;

        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Method call returned void".to_string())
    }
}
