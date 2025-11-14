// Expression compilation - identifiers and variable access
use super::ASTCodeGen;
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
        if let Some(ptr) = self.global_constants.get(name) {
            let ty = self
                .global_constant_types
                .get(name)
                .ok_or_else(|| format!("Type for constant {} not found", name))?;

            // Load the constant value
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
                eprintln!("[DEBUG result] Variable 'result' type: {:?}", ty);
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
                    eprintln!("[DEBUG result] Returning pointer without loading");
                }
                return Ok((*ptr).into());
            }

            // Use alignment-aware load to fix memory corruption
            let loaded = self.build_load_aligned(*ty, *ptr, name)?;

          

            return Ok(loaded);
        }

        // Check if this is a global function name (for function pointers)
        if let Some(func_val) = self.functions.get(name) {
            // Return function as a pointer value
            return Ok(func_val.as_global_value().as_pointer_value().into());
        }

        // Variable/function not found - find similar names for suggestion
        let mut candidates: Vec<String> = self.variables.keys().cloned().collect();
        candidates.extend(self.functions.keys().cloned());

        use crate::diagnostics::fuzzy;
        let suggestions = fuzzy::find_similar_names(name, &candidates, 0.7, 3);

        let mut help_msg =
            "Check that the name is spelled correctly and is in scope".to_string();
        if !suggestions.is_empty() {
            help_msg = format!("did you mean `{}`?", suggestions.join("`, `"));
        }

        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: error_codes::UNDEFINED_VARIABLE.to_string(),
            message: format!("Cannot find variable or function `{}` in this scope", name),
            span: Span::unknown(),
            notes: vec![],
            help: Some(help_msg),
            suggestion: None,
        });
        Err(format!("Variable or function {} not found", name))
    }
}