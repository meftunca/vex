// statements/let_statement/mod.rs
// Main entry point for let statement compilation

mod pattern;
mod type_inference;
mod type_injection;
mod variable_registration;

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a `let pattern` statement: let (a, b) = expr; or let Point { x, y } = expr;
    pub(crate) fn compile_let_pattern_statement(
        &mut self,
        is_mutable: bool,
        pattern: &Pattern,
        _ty: Option<&Type>,
        value: &Expression,
    ) -> Result<(), String> {
        // Compile the value expression
        let val = self.compile_expression(value)?;

        // Bind pattern variables
        self.compile_pattern_binding(pattern, val)?;

        Ok(())
    }

    /// Compile a `let` statement
    pub(crate) fn compile_let_statement(
        &mut self,
        is_mutable: bool,
        name: &String,
        ty: Option<&Type>,
        value: &Expression,
    ) -> Result<(), String> {
        // Step 1: Infer struct name from expression if no type annotation
        let struct_name_from_expr = self.infer_struct_name_from_expression(ty, value)?;

        // Step 2: Validate array size if type annotation is array
        self.validate_array_size(ty, value)?;

        // Step 3: Inject type args recursively for nested generics
        let adjusted_value = if let Some(ref type_annotation) = ty {
            self.inject_type_args_recursive(value, type_annotation)?
        } else {
            value.clone()
        };

        // Step 4: Compile the value expression
        let val = self.compile_value_expression(ty, &adjusted_value, name, is_mutable)?;

        // Step 4.5: Check if this is a closure and register it with its environment
        if let Expression::Closure { .. } = adjusted_value {
            if let BasicValueEnum::PointerValue(fn_ptr) = val {
                // Check if this closure has an environment in closure_envs
                if let Some(env_ptr) = self.closure_envs.get(&fn_ptr).copied() {
                    eprintln!("📝 Registering closure variable '{}' with environment", name);
                    self.closure_variables.insert(name.clone(), (fn_ptr, env_ptr));
                } else {
                    eprintln!("📝 Registering closure variable '{}' without environment (pure function)", name);
                    // For closures without environment, still register but with null environment
                    let null_env = self.context.ptr_type(inkwell::AddressSpace::default()).const_null();
                    self.closure_variables.insert(name.clone(), (fn_ptr, null_env));
                }
            }
        }

        // Special case: If val is a pointer and variable is already registered, skip step 5-6
        // This happens when compile_value_expression handles large arrays directly
        if let BasicValueEnum::PointerValue(_) = val {
            if self.variables.contains_key(name) {
                // Variable already registered by compile_value_expression
                return Ok(());
            }
        }

        // Step 5: Determine final type
        let (final_var_type, final_llvm_type) =
            self.determine_final_type(ty, val, value, &struct_name_from_expr)?;

        // Step 6: Register the variable
        self.register_variable(name, val, &final_var_type, final_llvm_type, is_mutable)?;

        Ok(())
    }
}
