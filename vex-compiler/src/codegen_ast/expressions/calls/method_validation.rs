// Method validation (mutability checking, external vs inline methods)

use crate::codegen_ast::ASTCodeGen;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Validate method call mutability and external method rules
    pub(crate) fn validate_method_call(
        &mut self,
        method_name: &str,
        method: &str,
        is_mutable_call: bool,
    ) -> Result<(), String> {
        // ⭐ Contract-based mutability validation
        // Check if the method is declared as mutable and if it's an external method
        let (method_is_mutable, is_external_method) = self
            .function_defs
            .get(method_name)
            .map(|func| {
                let is_external = func
                    .receiver
                    .as_ref()
                    .map_or(false, |r| matches!(r.ty, Type::Reference(_, _)));
                (func.is_mutable, is_external)
            })
            .unwrap_or((false, false));

        // CONTRACT RULES:
        // 1. External methods (Golang-style with &Type! receiver): NO '!' at call site
        // 2. Inline methods: '!' is OPTIONAL (compiler validates mutability at compile time)

        if is_external_method {
            // External method: '!' suffix is FORBIDDEN
            if is_mutable_call {
                return Err(format!(
                    "External method '{}' cannot use '!' suffix at call site (Golang-style methods don't use '!')",
                    method
                ));
            }
            // For external methods, ignore mutability check (receiver handles it)
        } else {
            // Inline method: '!' suffix is OPTIONAL but recommended for clarity
            // We validate mutability at compile time regardless of '!' presence
            if method_is_mutable && !is_mutable_call {
                eprintln!(
                    "ℹ️  Inline mutable method '{}' called without '!' suffix (allowed but not recommended)",
                    method
                );
                // Don't error, just warn
            }

            if !method_is_mutable && is_mutable_call {
                return Err(format!(
                    "Method '{}' is immutable, cannot use '!' suffix at call site",
                    method
                ));
            }
        }

        Ok(())
    }
}
