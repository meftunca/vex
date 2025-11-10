// drop_trait.rs
// Drop trait support - RAII automatic cleanup

use super::ASTCodeGen;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Check if a type implements the Drop trait
    pub(crate) fn type_implements_drop(&self, type_name: &str) -> bool {
        let drop_trait_key = ("Drop".to_string(), type_name.to_string());
        self.trait_impls.contains_key(&drop_trait_key)
    }

    /// Call drop() method on all variables in current scope (LIFO order)
    /// Used at scope exit, before return/break/continue
    pub(crate) fn call_scope_drops(&mut self) -> Result<(), String> {
        if let Some(scope) = self.scope_stack.last() {
            // Clone to avoid borrow conflicts
            let drop_vars: Vec<(String, String)> = scope.clone();

            // Drop in LIFO order (reverse)
            for (var_name, type_name) in drop_vars.iter().rev() {
                eprintln!("🗑️  Calling drop on {} (type: {})", var_name, type_name);

                // Get variable pointer
                let var_ptr = if let Some(ptr) = self.variables.get(var_name) {
                    *ptr
                } else {
                    eprintln!("⚠️  Variable {} not found in variables map", var_name);
                    continue;
                };

                // Get drop method name
                let drop_method = format!("{}_drop", type_name);

                // Call drop method if it exists
                if let Some(drop_fn) = self.functions.get(&drop_method) {
                    // Drop methods take &self as receiver
                    self.builder
                        .build_call(
                            *drop_fn,
                            &[inkwell::values::BasicMetadataValueEnum::from(var_ptr)],
                            "drop_call",
                        )
                        .map_err(|e| format!("Failed to call drop: {:?}", e))?;

                    eprintln!("  ✅ Called {}({})", drop_method, var_name);
                } else {
                    eprintln!("  ⚠️  Drop method {} not found", drop_method);
                }
            }
        }

        Ok(())
    }

    /// Push a new scope for tracking Drop variables
    pub(crate) fn push_drop_scope(&mut self) {
        self.scope_stack.push(Vec::new());
        eprintln!("📋 Pushed Drop scope (depth: {})", self.scope_stack.len());
    }

    /// Pop a scope and call drop() on all variables (LIFO order)
    pub(crate) fn pop_drop_scope(&mut self) -> Result<(), String> {
        eprintln!("📋 Popping Drop scope (depth: {})", self.scope_stack.len());

        // Call drops on current scope
        self.call_scope_drops()?;

        // Remove scope
        self.scope_stack.pop();

        Ok(())
    }
}
