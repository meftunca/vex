// Diagnostic helper methods for ASTCodeGen
// Converts common error patterns to structured diagnostics

use super::ASTCodeGen;
use vex_ast::Type;
use vex_diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};

impl<'ctx> ASTCodeGen<'ctx> {
    /// Emit a type mismatch error
    pub(crate) fn emit_type_mismatch(
        &mut self,
        expected: &str,
        found: &str,
        span: Span,
    ) {
        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: error_codes::TYPE_MISMATCH.to_string(),
            message: format!("mismatched types: expected `{}`, found `{}`", expected, found),
            span,
            notes: vec![],
            help: Some(self.suggest_type_conversion(expected, found)),
            suggestion: None,
        });
    }

    /// Emit an undefined variable error
    pub(crate) fn emit_undefined_variable(
        &mut self,
        var_name: &str,
        span: Span,
    ) {
        let similar = self.find_similar_variable_names(var_name);
        
        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: error_codes::UNDEFINED_VARIABLE.to_string(),
            message: format!("cannot find value `{}` in this scope", var_name),
            span,
            notes: vec![],
            help: if !similar.is_empty() {
                Some(format!("did you mean `{}`?", similar.join("`, `")))
            } else {
                None
            },
            suggestion: None,
        });
    }

    /// Emit an argument count mismatch error
    pub(crate) fn emit_argument_count_mismatch(
        &mut self,
        fn_name: &str,
        expected: usize,
        found: usize,
        span: Span,
    ) {
        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: error_codes::ARGUMENT_COUNT.to_string(),
            message: format!(
                "this function takes {} argument{} but {} argument{} {} supplied",
                expected,
                if expected == 1 { "" } else { "s" },
                found,
                if found == 1 { "" } else { "s" },
                if found == 1 { "was" } else { "were" }
            ),
            span,
            notes: vec![format!("function `{}` defined here", fn_name)],
            help: None,
            suggestion: None,
        });
    }

    /// Emit an undefined function error
    pub(crate) fn emit_undefined_function(
        &mut self,
        fn_name: &str,
        span: Span,
    ) {
        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: "E0425".to_string(),
            message: format!("cannot find function `{}` in this scope", fn_name),
            span,
            notes: vec![],
            help: None,
            suggestion: None,
        });
    }

    /// Emit an undefined type error
    pub(crate) fn emit_undefined_type(
        &mut self,
        type_name: &str,
        span: Span,
    ) {
        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: "E0412".to_string(),
            message: format!("cannot find type `{}` in this scope", type_name),
            span,
            notes: vec![],
            help: None,
            suggestion: None,
        });
    }

    /// Emit invalid operation error
    pub(crate) fn emit_invalid_operation(
        &mut self,
        op: &str,
        type_name: &str,
        span: Span,
    ) {
        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: "E0369".to_string(),
            message: format!("cannot apply binary operator `{}` to type `{}`", op, type_name),
            span,
            notes: vec![],
            help: None,
            suggestion: None,
        });
    }

    /// Suggest type conversion based on types
    fn suggest_type_conversion(&self, expected: &str, found: &str) -> String {
        match (expected, found) {
            ("i32", "f64") | ("i64", "f64") => {
                "consider using `.round() as i32` or `.floor() as i32`".to_string()
            }
            ("f64", "i32") | ("f64", "i64") => {
                "consider using `as f64` to convert the integer".to_string()
            }
            ("String", "&str") | ("string", "&str") => {
                "consider using `.to_string()` to convert `&str` to `String`".to_string()
            }
            ("&str", "String") | ("&str", "string") => {
                "consider using `.as_str()` to get a string slice".to_string()
            }
            _ => "types differ".to_string(),
        }
    }

    /// Find similar variable names for suggestions
    fn find_similar_variable_names(&self, target: &str) -> Vec<String> {
        let mut candidates: Vec<String> = self.variables.keys().cloned().collect();
        candidates.extend(self.variable_types.keys().cloned());
        
        vex_diagnostics::fuzzy::find_similar_names(target, &candidates, 0.6, 3)
    }

    /// Emit a generic codegen error with diagnostic
    pub(crate) fn emit_codegen_error(&mut self, message: String, span: Span) {
        self.diagnostics.emit(Diagnostic {
            level: ErrorLevel::Error,
            code: "E0001".to_string(),
            message,
            span,
            notes: vec![],
            help: None,
            suggestion: None,
        });
    }

    /// Check if we should continue compilation despite errors
    /// Returns true if we hit too many errors
    pub(crate) fn should_abort_compilation(&self) -> bool {
        self.diagnostics.diagnostics().iter().filter(|d| d.level == ErrorLevel::Error).count() > 100
    }

    /// Helper: Emit error and return Err for backward compatibility
    pub(crate) fn error_with_diagnostic(
        &mut self,
        message: String,
        span: Span,
    ) -> Result<(), String> {
        self.emit_codegen_error(message.clone(), span);
        Err(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_conversion_suggestions() {
        let ctx = inkwell::context::Context::create();
        let codegen = ASTCodeGen::new(&ctx, "test");
        
        assert!(codegen.suggest_type_conversion("i32", "f64").contains("round"));
        assert!(codegen.suggest_type_conversion("String", "&str").contains("to_string"));
    }
}
