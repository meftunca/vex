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

        // Try builtin registry first (check both PascalCase and lowercase)
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

        // Check if the function exists in LLVM module
        if !self.functions.contains_key(&base_method_name) {
            // Try PascalCase version: TypeName_method or Vec_i32_new
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

            if !self.functions.contains_key(&pascal_method_name) {
                // ⚠️ FALLBACK: For stdlib types (Vec, Box, etc.), try compiler builtin
                // This allows Vec<i32>() to work even without stdlib methods
                let is_stdlib_type = matches!(type_name, "Vec" | "Box" | "String" | "Map" | "Set" | "Channel");
                
                if is_stdlib_type && method == "new" {
                    eprintln!("⚠️  Static method {}.{}() not found, falling back to compiler builtin", type_name, method);
                    
                    // Try builtin registry with PascalCase name
                    let builtin_name = format!("{}.{}", type_name, method);
                    if let Some(builtin_fn) = self.builtins.get(&builtin_name) {
                        let mut arg_vals: Vec<BasicValueEnum> = vec![];
                        for arg in args {
                            let val = self.compile_expression(arg)?;
                            arg_vals.push(val);
                        }
                        return builtin_fn(self, &arg_vals);
                    }
                }
                
                return Err(format!(
                    "Static method {}.{}() not found. Expected function: {} or {}",
                    type_name, method, base_method_name, pascal_method_name
                ));
            }

            // Use PascalCase version
            let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
            for arg in args {
                let val = self.compile_expression(arg)?;
                arg_vals.push(val.into());
            }

            let fn_val = *self.functions.get(&pascal_method_name).unwrap();
            let call_site = self
                .builder
                .build_call(fn_val, &arg_vals, "static_method_call")
                .map_err(|e| format!("Failed to build static method call: {}", e))?;

            return call_site
                .try_as_basic_value()
                .left()
                .ok_or_else(|| "Static method returned void".to_string());
        }

        // Call lowercase version from LLVM module
        let mut arg_vals: Vec<BasicMetadataValueEnum> = vec![];
        for arg in args {
            let val = self.compile_expression(arg)?;
            arg_vals.push(val.into());
        }

        let fn_val = *self.functions.get(&base_method_name).unwrap();
        let call_site = self
            .builder
            .build_call(fn_val, &arg_vals, "static_method_call")
            .map_err(|e| format!("Failed to build static method call: {}", e))?;

        call_site
            .try_as_basic_value()
            .left()
            .ok_or_else(|| "Static method returned void".to_string())
    }
}
