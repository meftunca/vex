//! Operator overloading logic for binary operations
//!
//! Handles Vec concatenation, builtin contracts, and user-defined operator methods

use crate::{debug_log, debug_println};
use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Check for operator overloading before falling back to builtin operations
    pub(crate) fn check_operator_overloading(
        &mut self,
        left: &Expression,
        op: &BinaryOp,
        right: &Expression,
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        // ⭐ NEW: Operator Overloading - Check if left operand has operator contract
        debug_println!("🔍 Binary op: {:?} {:?} {:?}", left, op, right);
        if let Ok(left_type) = self.infer_expression_type(left) {
            eprintln!("   Left type inferred: {:?}", left_type);

            // ⭐ BUILTIN: Check for Vec + Vec (if both are Vec)
            if let Type::Generic { ref name, .. } = left_type {
                if name == "Vec" && matches!(op, BinaryOp::Add) {
                    if let Ok(right_type) = self.infer_expression_type(right) {
                        if let Type::Generic {
                            name: right_name, ..
                        } = right_type
                        {
                            if right_name == "Vec" {
                                let left_val = self.compile_expression(left)?;
                                let right_val = self.compile_expression(right)?;

                                let concat_fn = self.get_vex_vec_concat();
                                let concat_call = self
                                    .builder
                                    .build_call(
                                        concat_fn,
                                        &[left_val.into(), right_val.into()],
                                        "vec_concat",
                                    )
                                    .map_err(|e| format!("Failed to call vex_vec_concat: {}", e))?;
                                let result = concat_call.try_as_basic_value().unwrap_basic();

                                return Ok(Some(result));
                            }
                        }
                    }
                }
            }

            // ⭐ Strip references to get the actual type for operator lookup
            let actual_type = match left_type {
                Type::Reference(ref inner, _) => inner.as_ref().clone(),
                _ => left_type.clone(),
            };

            // ⭐ Phase 1 Day 3-4: Builtin contract operator dispatch
            if let Type::Named(ref type_name) = actual_type {
                let (contract_name, method_name) = self.binary_op_to_trait(op);
                if !contract_name.is_empty() {
                    // Check if builtin contract exists (e.g., i32 extends Add)
                    use crate::builtin_contracts;
                    if builtin_contracts::has_builtin_contract(type_name, contract_name) {
                        eprintln!(
                            "🎯 Builtin operator contract: {}.{}()",
                            type_name, method_name
                        );

                        // Compile operands
                        let left_val = self.compile_expression(left)?;
                        let right_val = self.compile_expression(right)?;

                        // ⭐ NEW: Dispatch to builtin operator codegen (zero overhead LLVM IR)
                        match builtin_contracts::codegen_builtin_operator(
                            &self.builder,
                            type_name,
                            contract_name,
                            method_name,
                            left_val,
                            right_val,
                        ) {
                            Ok(result) => return Ok(Some(result)),
                            Err(e) => return Err(e.into()),
                        }
                    }

                    // ⭐ NEW: Check for user-defined operator methods by function existence
                    // For binary operators, param_count = 1 (just rhs, receiver not counted in params)
                    let param_count = 1;

                    // CRITICAL: Encode operator name for LLVM compatibility (op+ -> opadd)
                    let method_encoded = Self::encode_operator_name(method_name);
                    let base_method_name = format!("{}_{}", type_name, method_encoded);

                    // Try to get right operand type for type-based lookup (for overloading)
                    let first_arg_type_suffix =
                        if let Ok(arg_type) = self.infer_expression_type(right) {
                            self.generate_type_suffix(&arg_type)
                        } else {
                            String::new()
                        };

                    // ⚠️ CRITICAL FIX: For inline methods, param_count INCLUDES receiver
                    // Binary operator: receiver + rhs = 2 total params
                    let inline_param_count = 2;

                    // Try inline method with type suffix first (for overloaded inline methods)
                    let inline_method_typed = if !first_arg_type_suffix.is_empty() {
                        format!(
                            "{}{}_{}",
                            base_method_name, first_arg_type_suffix, inline_param_count
                        )
                    } else {
                        String::new()
                    };

                    // Fallback: inline method without type suffix
                    let inline_method_name = format!("{}_{}", base_method_name, inline_param_count);

                    // Try external method with type suffix (for overloading support)
                    let external_method_typed =
                        if !first_arg_type_suffix.is_empty() && method_name.starts_with("op") {
                            format!(
                                "{}{}_{}",
                                base_method_name, first_arg_type_suffix, param_count
                            )
                        } else {
                            String::new()
                        };

                    // Fallback: external method without type suffix
                    let external_method_name = if method_name.starts_with("op") {
                        format!("{}_{}", base_method_name, param_count)
                    } else {
                        base_method_name.clone()
                    };

                    // Check all naming schemes: inline typed > inline untyped > external typed > external untyped
                    // Prioritize inline methods (most specific)
                    let method_func_name = if !inline_method_typed.is_empty()
                        && self.functions.contains_key(&inline_method_typed)
                    {
                        inline_method_typed
                    } else if self.functions.contains_key(&inline_method_name) {
                        inline_method_name
                    } else if !external_method_typed.is_empty()
                        && self.functions.contains_key(&external_method_typed)
                    {
                        external_method_typed
                    } else if self.functions.contains_key(&external_method_name) {
                        external_method_name
                    } else {
                        // Default to inline for error reporting (most common case)
                        inline_method_name
                    };

                    eprintln!(
                        "   Checking for method: {} (contract: {})",
                        method_func_name, contract_name
                    );
                    eprintln!(
                        "   has_operator_trait: {}",
                        self.has_operator_trait(type_name, contract_name).is_some()
                    );
                    eprintln!(
                        "   functions.contains_key: {}",
                        self.functions.contains_key(&method_func_name)
                    );

                    // Check if method exists (either in trait_impls OR as external method)
                    if self.has_operator_trait(type_name, contract_name).is_some()
                        || self.functions.contains_key(&method_func_name)
                    {
                        eprintln!(
                            "🎯 User operator contract: {}.{}() → {}",
                            type_name, method_name, method_func_name
                        );
                        eprintln!("   Left: {:?}", left);
                        eprintln!("   Right: {:?}", right);
                        return Ok(Some(self.compile_method_call(
                            left,
                            method_name,
                            &[], // No generic type args for operator overloading
                            &vec![right.clone()],
                            false,
                        )?));
                    } else if !contract_name.is_empty() {
                        // ⚠️ Debug: Operator method not found despite contract name existing
                        eprintln!(
                            "⚠️  Operator method '{}' not found for type '{}' (contract: '{}')",
                            method_name, type_name, contract_name
                        );
                        eprintln!("   Expected function name: {}", method_func_name);
                        eprintln!(
                            "   Trait implementation exists: {}",
                            self.has_operator_trait(type_name, contract_name).is_some()
                        );
                        eprintln!(
                            "   Function registered: {}",
                            self.functions.contains_key(&method_func_name)
                        );
                        eprintln!(
                            "   Hint: Check if contract is implemented and method signature matches"
                        );
                    }
                }
            }
        }

        // No operator overloading found
        Ok(None)
    }
}
