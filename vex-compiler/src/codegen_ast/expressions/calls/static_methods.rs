// Static method call compilation (Type.method() calls)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile static method calls: Type.method(args) or Type<i32>.method(args)
    pub(crate) fn compile_static_method_call(
        &mut self,
        type_name: &str,
        method: &str,
        type_args: &[Type],
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Handle generic static methods: Vec<i32>.new()
        // Mangle function name with type arguments
        let base_method_name = if !type_args.is_empty() {
            // Generic static method: Vec<i32>.new() → vec_i32_new
            let mut mangled = type_name.to_lowercase();
            for ty in type_args {
                mangled.push('_');
                mangled.push_str(&self.type_to_string(ty));
            }
            mangled.push('_');
            mangled.push_str(method);
            mangled
        } else {
            // Non-generic: Vec.new() → vec_new
            format!("{}_{}", type_name.to_lowercase(), method)
        };

        // Determine PascalCase variant once so we can reuse it below
        let pascal_method_name = if !type_args.is_empty() {
            let mut mangled = type_name.to_string();
            for ty in type_args {
                mangled.push('_');
                mangled.push_str(&self.type_to_string(ty));
            }
            mangled.push('_');
            mangled.push_str(method);
            mangled
        } else {
            format!("{}_{}", type_name, method)
        };

        // Prefer user-defined/static stdlib implementations before compiler builtins
        if let Some(fn_val) = self.functions.get(&base_method_name).copied() {
            let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
            for arg in args {
                let val = self.compile_expression(arg)?;
                arg_vals.push(val.into());
            }

            let call_site = self
                .builder
                .build_call(fn_val, &arg_vals, "static_method_call")
                .map_err(|e| format!("Failed to build static method call: {}", e))?;

            return Ok(call_site.try_as_basic_value().unwrap_basic());
        } else if let Some(fn_val) = self.functions.get(&pascal_method_name).copied() {
            let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
            for arg in args {
                let val = self.compile_expression(arg)?;
                arg_vals.push(val.into());
            }

            let call_site = self
                .builder
                .build_call(fn_val, &arg_vals, "static_method_call")
                .map_err(|e| format!("Failed to build static method call: {}", e))?;

            return Ok(call_site.try_as_basic_value().unwrap_basic());
        }

        // No user-defined version found; try compiler builtin as fallback
        let pascal_builtin_name = format!("{}.{}", type_name, method);
        if let Some(builtin_fn) = self
            .builtins
            .get(&pascal_builtin_name)
            .or_else(|| self.builtins.get(&base_method_name))
        {
            let mut arg_vals: Vec<BasicValueEnum> = vec![];
            for arg in args {
                let val = self.compile_expression(arg)?;
                arg_vals.push(val);
            }

            return builtin_fn(self, &arg_vals);
        }

        let is_stdlib_type = matches!(
            type_name,
            "Vec" | "Box" | "String" | "Map" | "Set" | "Channel"
        );

        if is_stdlib_type && method == "new" {
            eprintln!(
                "⚠️  Static method {}.{}() not found, no compiler builtin registered",
                type_name, method
            );
        }

        Err(format!(
            "Static method {}.{}() not found. Expected function: {} or {}",
            type_name, method, base_method_name, pascal_method_name
        ))
    }
}
