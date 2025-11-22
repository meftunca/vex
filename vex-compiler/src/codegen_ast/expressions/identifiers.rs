// Expression compilation - identifiers and variable access
use super::ASTCodeGen;
use crate::debug_println;
use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile identifier expressions (variables, functions, constants)
    pub(crate) fn compile_identifier(
        &mut self,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Check if this is a builtin function first (before variables/functions)
        // This ensures builtins like print() work inside method bodies
        if self.builtins.is_builtin(name) {
            // This is a builtin function - return a dummy function pointer
            // The actual builtin will be called via compile_call()
            // For now, return a zero pointer (builtins are handled specially)
            return Err(format!(
                "Builtin function '{}' cannot be used as a value (must be called directly)",
                name
            ));
        }

        // Check global constants FIRST (never cleared during function compilation)
        // First check if we have the value directly (for constant folding)
        if let Some(val) = self.module_constants.get(name) {
            return Ok(*val);
        }

        // Fallback to loading from pointer if value not cached (should not happen for simple constants)
        if let Some(ptr) = self.global_constants.get(name) {
            // For global constants, try to return the initializer value (LLVM constant) instead of loading
            // This allows const folding: `const X = 60 * SECOND` where SECOND is also a const
            if let Some(global_var) = self.module.get_global(name) {
                if let Some(initializer) = global_var.get_initializer() {
                    eprintln!("🔍 Returning const initializer for {}", name);
                    // Return the constant initializer directly for const folding
                    return Ok(initializer);
                } else {
                    eprintln!("⚠️ Global {} has no initializer!", name);
                }
            } else {
                eprintln!("⚠️ Module has no global named {}", name);
            }

            // Fallback: load if no initializer (shouldn't happen for constants but handles edge cases)
            let ty = self
                .global_constant_types
                .get(name)
                .ok_or_else(|| format!("Type for constant {} not found", name))?;

            eprintln!("⚠️ Falling back to load for const {}", name);
            let loaded = self.build_load_aligned(*ty, *ptr, name)?;
            return Ok(loaded);
        }

        // Check if this is a function pointer parameter first
        if let Some(fn_ptr) = self.function_params.get(name) {
            // Return function pointer directly
            return Ok((*fn_ptr).into());
        }

        // Check if this is a variable (includes regular parameters)
        if let Some(ptr) = self.variables.get(name) {
            let ty = self
                .variable_types
                .get(name)
                .ok_or_else(|| format!("Type for variable {} not found", name))?;

            if name == "result" {
                debug_println!("[DEBUG result] Variable 'result' type: {:?}", ty);
                eprintln!(
                    "[DEBUG result] Is in variable_struct_names: {}",
                    self.variable_struct_names.contains_key(name)
                );
            }

            // IMPORTANT: For struct variables, return the pointer directly (zero-copy)
            // Struct types in LLVM are already pointers (ast_type_to_llvm returns pointer for structs)
            if self.variable_struct_names.contains_key(name) {
                // This is a struct variable - return pointer without loading
                if name == "result" {
                    debug_println!("[DEBUG result] Returning pointer without loading");
                }
                return Ok((*ptr).into());
            }

            // CRITICAL: For ptr and str type variables, LOAD the pointer value from alloca
            // ptr/str are stored as "ptr" in alloca, so we need to load them
            if let Some(ast_type) = self.variable_ast_types.get(name) {
                if matches!(ast_type, vex_ast::Type::Named(n) if n == "ptr" || n == "str") {
                    // Load the pointer value from the alloca
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let loaded_ptr = self
                        .builder
                        .build_load(ptr_type, *ptr, &format!("{}_ptr", name))
                        .map_err(|e| format!("Failed to load ptr/str variable: {}", e))?;
                    return Ok(loaded_ptr);
                }
            }

            // Use alignment-aware load to fix memory corruption
            let loaded = self.build_load_aligned(*ty, *ptr, name)?;

            return Ok(loaded);
        }

        // Check if this is a global function name (for function pointers)
        // Use centralized lookup that handles BOTH base names and mangled names
        if let Some(func_val) = self.lookup_function(name) {
            // Return function as a pointer value
            return Ok(func_val.as_global_value().as_pointer_value().into());
        }

        // Variable/function not found - find similar names for suggestion
        if self.suppress_diagnostics {
            return Err(format!(
                "Cannot find variable or function '{}' in this scope",
                name
            ));
        }

        let mut candidates: Vec<String> = self.variables.keys().cloned().collect();
        candidates.extend(self.functions.keys().cloned());

        use crate::diagnostics::fuzzy;
        let suggestions = fuzzy::find_similar_names(name, &candidates, 0.7, 3);

        let mut help_msg = "Check that the name is spelled correctly and is in scope".to_string();
        if !suggestions.is_empty() {
            help_msg = format!("did you mean `{}`?", suggestions.join("`, `"));
        }

        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: error_codes::UNDEFINED_VARIABLE.to_string(),
            message: format!("Cannot find variable or function `{}` in this scope", name),
            span: Span::unknown(),
            primary_label: Some("undefined name".to_string()),
            notes: vec![],
            help: Some(help_msg),
            suggestion: suggestions.get(0).map(|s| vex_diagnostics::Suggestion {
                message: format!("use `{}`", s),
                replacement: s.clone(),
                span: Span::unknown(),
            }),
            related: Vec::new(),
        });
        Err(format!("Variable or function {} not found", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_compile_identifier_undefined_suggestion() {
        let ctx = Context::create();
        let mut codegen = ASTCodeGen::new(&ctx, "test");
        // register a known function
        let fn_type = codegen.context.void_type().fn_type(&[], false);
        let print_fn = codegen.module.add_function("print", fn_type, None);
        codegen.functions.insert("print".to_string(), print_fn);

        let res = codegen.compile_identifier("prinnt");
        assert!(res.is_err());
        let diags = codegen.diagnostics.diagnostics();
        assert!(!diags.is_empty());
        let diag = diags.last().unwrap();
        assert!(diag.primary_label.is_some());
        assert_eq!(diag.primary_label.as_ref().unwrap(), "undefined name");
        assert!(diag.suggestion.is_some());
        assert_eq!(diag.suggestion.as_ref().unwrap().replacement, "print");
    }
}
