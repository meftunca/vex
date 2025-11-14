// Type constraint unification algorithm
// Phase 1 & Phase 3: Resolve Type::Unknown placeholders to concrete types

use crate::codegen_ast::{ASTCodeGen, TypeConstraint};
use vex_ast::Type;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Unification algorithm: resolve all Type::Unknown to concrete types
    /// Returns Err if any Unknown types remain after constraint solving
    pub fn unify_types(&mut self) -> Result<(), String> {
        // Iterate constraints and update variable_concrete_types
        for constraint in &self.type_constraints {
            match constraint {
                TypeConstraint::Equal(t1, t2) => {
                    // If t1 is Unknown and t2 is concrete, resolve t1 → t2
                    if matches!(t1, Type::Unknown) && !matches!(t2, Type::Unknown) {
                        // Find variables with t1 and replace with t2
                        let mut vars_to_update = Vec::new();
                        for (var_name, var_type) in &self.variable_concrete_types {
                            if var_type == t1 {
                                vars_to_update.push(var_name.clone());
                            }
                        }
                        for var_name in vars_to_update {
                            self.variable_concrete_types.insert(var_name, t2.clone());
                        }
                    }
                    // Symmetric case
                    else if matches!(t2, Type::Unknown) && !matches!(t1, Type::Unknown) {
                        let mut vars_to_update = Vec::new();
                        for (var_name, var_type) in &self.variable_concrete_types {
                            if var_type == t2 {
                                vars_to_update.push(var_name.clone());
                            }
                        }
                        for var_name in vars_to_update {
                            self.variable_concrete_types.insert(var_name, t1.clone());
                        }
                    }
                }

                TypeConstraint::MethodReceiver {
                    receiver_name,
                    inferred_receiver_type,
                    ..
                } => {
                    // Update receiver variable type if it contains Unknown
                    if let Some(current_type) = self.variable_concrete_types.get(receiver_name) {
                        if self.contains_unknown(current_type) {
                            self.variable_concrete_types
                                .insert(receiver_name.clone(), inferred_receiver_type.clone());
                        }
                    }
                }

                TypeConstraint::Assignment {
                    var_name,
                    expr_type,
                } => {
                    // Variable type must match expression type
                    if let Some(current_type) = self.variable_concrete_types.get(var_name) {
                        if matches!(current_type, Type::Unknown) {
                            self.variable_concrete_types
                                .insert(var_name.clone(), expr_type.clone());
                        }
                    }
                }
            }
        }

        // Verify no Unknown types remain
        for (var_name, var_type) in &self.variable_concrete_types {
            if self.contains_unknown(var_type) {
                return Err(format!(
                    "Cannot infer complete type for variable '{}': {:?}",
                    var_name, var_type
                ));
            }
        }

        Ok(())
    }
}
