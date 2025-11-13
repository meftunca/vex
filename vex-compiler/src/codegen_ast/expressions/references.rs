// Expression compilation - references and dereferencing
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile reference expressions (& and &!)
    pub(crate) fn compile_reference_dispatch(
        &mut self,
        is_mutable: bool,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Take a reference to an expression: &expr or &expr!
        // This creates a pointer to the value
        match expr {
            vex_ast::Expression::Ident(name) => {
                // For identifiers, return the pointer directly (don't load)
                let ptr = self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                Ok((*ptr).into())
            }
            _ => {
                // For other expressions, compile them, store in temporary, return pointer
                let value = self.compile_expression(expr)?;
                let value_type = value.get_type();
                let temp_name = if is_mutable {
                    "ref_temp_mut"
                } else {
                    "ref_temp"
                };
                let temp_ptr = self
                    .builder
                    .build_alloca(value_type, temp_name)
                    .map_err(|e| format!("Failed to allocate reference temporary: {}", e))?;
                self.builder
                    .build_store(temp_ptr, value)
                    .map_err(|e| format!("Failed to store reference temporary: {}", e))?;
                Ok(temp_ptr.into())
            }
        }
    }

    /// Compile dereference expressions (*ptr)
    pub(crate) fn compile_dereference_dispatch(
        &mut self,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Dereference a pointer: *ptr
        // Try to infer the inner type from the expression
        match expr {
            vex_ast::Expression::Ident(name) => {
                // For identifiers, we can load using the stored LLVM type
                let ptr = self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                let var_type = self
                    .variable_types
                    .get(name)
                    .ok_or_else(|| format!("Type for variable {} not found", name))?;

                // Load the pointer value first (variables store the reference)
                let ptr_loaded = self
                    .builder
                    .build_load(*var_type, *ptr, &format!("{}_ptr", name))
                    .map_err(|e| format!("Failed to load pointer variable: {}", e))?;

                if !ptr_loaded.is_pointer_value() {
                    return Err(format!("Cannot dereference non-pointer variable {}", name));
                }

                // Now dereference the pointer
                // For references, the inner type is what we need to load
                // Since we don't track AST types, we'll use a heuristic:
                // Try common types (i32, i64, f64, bool)
                // TODO: Add proper AST type tracking for variables
                let deref_ptr = ptr_loaded.into_pointer_value();

                // Try to determine the pointee type
                // For now, default to i32 (most common case)
                let inner_type = self.context.i32_type();
                let loaded = self
                    .builder
                    .build_load(inner_type, deref_ptr, "deref")
                    .map_err(|e| format!("Failed to dereference pointer: {}", e))?;
                Ok(loaded)
            }
            _ => {
                // For other expressions, compile and dereference
                let ptr_value = self.compile_expression(expr)?;
                if !ptr_value.is_pointer_value() {
                    return Err("Cannot dereference non-pointer value".to_string());
                }
                let ptr = ptr_value.into_pointer_value();

                // Default to i32 for now
                let inner_type = self.context.i32_type();
                let loaded = self
                    .builder
                    .build_load(inner_type, ptr, "deref")
                    .map_err(|e| format!("Failed to dereference pointer: {}", e))?;
                Ok(loaded)
            }
        }
    }
}
