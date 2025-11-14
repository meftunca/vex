//! Pointer loading logic for binary operations
//!
//! Handles loading struct/enum values from pointers before performing binary operations

use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Load struct/enum values from pointers if needed for binary operations
    pub(crate) fn load_operands_if_needed(
        &mut self,
        lhs: BasicValueEnum<'ctx>,
        rhs: BasicValueEnum<'ctx>,
        left: &Expression,
        right: &Expression,
    ) -> Result<(BasicValueEnum<'ctx>, BasicValueEnum<'ctx>), String> {
        // ⚡ CRITICAL: If operands are pointers to structs/enums, load them first
        // Variables for structs/enums return pointers (zero-copy semantics)
        // But binary ops need actual struct values for field comparison
        let lhs = if lhs.is_pointer_value() {
            let ptr = lhs.into_pointer_value();
            // Check if this is a struct/enum variable that needs loading
            if let Expression::Ident(var_name) = left {
                // Check if it's a builtin enum (Option, Result) tracked in variable_struct_names
                if let Some(type_name) = self.variable_struct_names.get(var_name) {
                    if type_name.starts_with("Option") || type_name.starts_with("Result") {
                        // Builtin enum - use stored StructType from variable_types
                        if let Some(var_type) = self.variable_types.get(var_name) {
                            self.builder
                                .build_load(*var_type, ptr, "lhs_enum_loaded")
                                .map_err(|e| format!("Failed to load left enum: {}", e))?
                        } else {
                            lhs.into()
                        }
                    } else if self.struct_defs.contains_key(type_name) {
                        // User-defined struct - build type from definition
                        let struct_def = self.struct_defs.get(type_name).unwrap().clone();
                        let field_types: Vec<_> = struct_def
                            .fields
                            .iter()
                            .map(|(_, ty)| self.ast_type_to_llvm(ty))
                            .collect();
                        let struct_type = self.context.struct_type(&field_types, false);
                        self.builder
                            .build_load(struct_type, ptr, "lhs_struct_loaded")
                            .map_err(|e| format!("Failed to load left struct: {}", e))?
                    } else {
                        lhs.into()
                    }
                }
                // Check user-defined enums
                else if self.variable_enum_names.contains_key(var_name) {
                    if let Some(var_type) = self.variable_types.get(var_name) {
                        self.builder
                            .build_load(*var_type, ptr, "lhs_enum_loaded")
                            .map_err(|e| format!("Failed to load left enum: {}", e))?
                    } else {
                        lhs.into()
                    }
                } else {
                    lhs.into()
                }
            } else {
                lhs.into()
            }
        } else {
            lhs
        };

        let rhs = if rhs.is_pointer_value() {
            let ptr = rhs.into_pointer_value();
            if let Expression::Ident(var_name) = right {
                // Check builtin enum
                if let Some(type_name) = self.variable_struct_names.get(var_name) {
                    if type_name.starts_with("Option") || type_name.starts_with("Result") {
                        if let Some(var_type) = self.variable_types.get(var_name) {
                            self.builder
                                .build_load(*var_type, ptr, "rhs_enum_loaded")
                                .map_err(|e| format!("Failed to load right enum: {}", e))?
                        } else {
                            rhs.into()
                        }
                    } else if self.struct_defs.contains_key(type_name) {
                        let struct_def = self.struct_defs.get(type_name).unwrap().clone();
                        let field_types: Vec<_> = struct_def
                            .fields
                            .iter()
                            .map(|(_, ty)| self.ast_type_to_llvm(ty))
                            .collect();
                        let struct_type = self.context.struct_type(&field_types, false);
                        self.builder
                            .build_load(struct_type, ptr, "rhs_struct_loaded")
                            .map_err(|e| format!("Failed to load right struct: {}", e))?
                    } else {
                        rhs.into()
                    }
                }
                // Check user-defined enums
                else if self.variable_enum_names.contains_key(var_name) {
                    if let Some(var_type) = self.variable_types.get(var_name) {
                        self.builder
                            .build_load(*var_type, ptr, "rhs_enum_loaded")
                            .map_err(|e| format!("Failed to load right enum: {}", e))?
                    } else {
                        rhs.into()
                    }
                } else {
                    rhs.into()
                }
            } else {
                rhs.into()
            }
        } else {
            rhs
        };

        Ok((lhs, rhs))
    }
}
