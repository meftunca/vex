// Scope and cleanup management for ASTCodeGen
// Handles deferred statements, RAII cleanup, and scope management

use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use vex_ast::Statement;

impl<'ctx> super::ASTCodeGen<'ctx> {
    /// Execute deferred statements in LIFO order
    /// Called before function exits (return, panic, or end of function)
    /// Note: Does NOT clear the stack - use clear_deferred_statements() at function boundary
    pub(crate) fn execute_deferred_statements(&mut self) -> Result<(), String> {
        // Execute in reverse order (LIFO - Last In First Out)
        // Clone the statements to avoid borrow checker issues
        let statements: Vec<Statement> = self.deferred_statements.iter().rev().cloned().collect();
        for stmt in statements {
            self.compile_statement(&stmt)?;
        }
        Ok(())
    }

    /// Clear deferred statements (called at function boundary)
    pub(crate) fn clear_deferred_statements(&mut self) {
        self.deferred_statements.clear();
    }

    /// Push a new scope for automatic cleanup tracking
    pub(crate) fn push_scope(&mut self) {
        self.scope_stack.push(Vec::new());
        self.push_drop_scope(); // Also push Drop trait scope
    }

    /// Register built-in types that implement Destructor trait
    pub(crate) fn register_builtin_destructors(&mut self) {
        // Built-in types that need cleanup at scope exit
        self.destructor_impls
            .insert("Vec".to_string(), "vec_free".to_string());
        self.destructor_impls
            .insert("Box".to_string(), "box_free".to_string());
        self.destructor_impls
            .insert("String".to_string(), "vex_string_free".to_string());
        self.destructor_impls
            .insert("Map".to_string(), "vex_map_free".to_string());
        self.destructor_impls
            .insert("Set".to_string(), "vex_map_free".to_string()); // Set uses Map backend

        eprintln!(
            "📝 Registered {} built-in destructor implementations",
            self.destructor_impls.len()
        );
    }

    /// Pop scope and emit cleanup calls for types implementing Destructor trait
    pub(crate) fn pop_scope(&mut self) -> Result<(), String> {
        // Call Drop trait drops first
        self.pop_drop_scope()?;

        if let Some(scope_vars) = self.scope_stack.pop() {
            // Emit cleanup calls in reverse order (LIFO - last allocated, first freed)
            for (var_name, type_name) in scope_vars.iter().rev() {
                // Check if this type implements Destructor trait (has a registered cleanup function)
                if let Some(cleanup_func_name) = self.destructor_impls.get(type_name) {
                    eprintln!(
                        "🧹 Auto-cleanup: {}({}) [trait-based]",
                        cleanup_func_name, var_name
                    );

                    // Get variable pointer
                    let var_ptr = match self.variables.get(var_name) {
                        Some(ptr) => *ptr,
                        None => {
                            eprintln!("⚠️  Variable {} not found, skipping cleanup", var_name);
                            continue;
                        }
                    };

                    // Call the appropriate cleanup function based on type
                    match type_name.as_str() {
                        "Vec" => {
                            let vec_opaque_type = self.context.opaque_struct_type("vex_vec_s");
                            let vec_ptr_type =
                                vec_opaque_type.ptr_type(inkwell::AddressSpace::default());

                            let vec_value = self
                                .builder
                                .build_load(vec_ptr_type, var_ptr, "vec_cleanup_load")
                                .map_err(|e| format!("Failed to load vec for cleanup: {}", e))?;

                            let vec_free_fn = self.get_vex_vec_free();
                            self.builder
                                .build_call(vec_free_fn, &[vec_value.into()], "vec_auto_free")
                                .map_err(|e| format!("Failed to call vec_free: {}", e))?;
                        }
                        "Box" => {
                            let box_ptr_type = self
                                .context
                                .struct_type(
                                    &[
                                        self.context
                                            .i8_type()
                                            .ptr_type(inkwell::AddressSpace::default())
                                            .into(),
                                        self.context.i64_type().into(),
                                    ],
                                    false,
                                )
                                .ptr_type(inkwell::AddressSpace::default());

                            let box_value = self
                                .builder
                                .build_load(box_ptr_type, var_ptr, "box_cleanup_load")
                                .map_err(|e| format!("Failed to load box for cleanup: {}", e))?;

                            let box_free_fn = self.get_vex_box_free();
                            self.builder
                                .build_call(box_free_fn, &[box_value.into()], "box_auto_free")
                                .map_err(|e| format!("Failed to call box_free: {}", e))?;
                        }
                        "String" => {
                            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                            let string_value = self
                                .builder
                                .build_load(ptr_type, var_ptr, "string_cleanup_load")
                                .map_err(|e| format!("Failed to load string for cleanup: {}", e))?;

                            let void_fn_type =
                                self.context.void_type().fn_type(&[ptr_type.into()], false);
                            let string_free_fn =
                                self.module
                                    .add_function("vex_string_free", void_fn_type, None);

                            self.builder
                                .build_call(
                                    string_free_fn,
                                    &[string_value.into()],
                                    "string_auto_free",
                                )
                                .map_err(|e| format!("Failed to call vex_string_free: {}", e))?;
                        }
                        "Map" | "Set" => {
                            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                            let map_value = self
                                .builder
                                .build_load(ptr_type, var_ptr, "map_cleanup_load")
                                .map_err(|e| {
                                    format!("Failed to load map/set for cleanup: {}", e)
                                })?;

                            let void_fn_type =
                                self.context.void_type().fn_type(&[ptr_type.into()], false);
                            let map_free_fn =
                                self.module.add_function("vex_map_free", void_fn_type, None);

                            self.builder
                                .build_call(map_free_fn, &[map_value.into()], "map_auto_free")
                                .map_err(|e| format!("Failed to call vex_map_free: {}", e))?;
                        }
                        _ => {
                            eprintln!("⚠️  Cleanup for type {} not yet implemented", type_name);
                        }
                    }
                } else {
                    // Type doesn't implement Destructor trait - no cleanup needed
                    eprintln!("  → Type {} has no destructor, skipping cleanup", type_name);
                }
            }
        }
        Ok(())
    }

    /// Register a variable for automatic cleanup (if it implements Destructor trait)
    pub(crate) fn register_for_cleanup(&mut self, var_name: String, type_name: String) {
        // Check if this type implements Destructor trait
        if self.destructor_impls.contains_key(&type_name) {
            if let Some(current_scope) = self.scope_stack.last_mut() {
                eprintln!(
                    "📝 Register for cleanup: {} ({}) [trait-based]",
                    var_name, type_name
                );
                current_scope.push((var_name, type_name));
            }
        } else {
            eprintln!(
                "  → Type {} has no destructor, not registering for cleanup",
                type_name
            );
        }
    }
}
