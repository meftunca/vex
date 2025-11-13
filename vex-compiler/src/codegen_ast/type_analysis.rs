// Type analysis and expression handling utilities for ASTCodeGen
// Handles struct name resolution, generic depth calculation

use vex_ast::{Expression, Type};

impl<'ctx> super::ASTCodeGen<'ctx> {
    /// Get struct name from an expression
    /// Returns the struct type name if the expression evaluates to a struct
    pub(crate) fn get_expression_struct_name(
        &mut self,
        expr: &Expression,
    ) -> Result<Option<String>, String> {
        match expr {
            // Variable: look up in variable_struct_names
            Expression::Ident(var_name) => Ok(self.variable_struct_names.get(var_name).cloned()),
            // Struct literal: directly has name
            Expression::StructLiteral { name, .. } => Ok(Some(name.clone())),
            // Field access: recursively get object's struct, then lookup field type
            Expression::FieldAccess { object, field } => {
                if let Some(object_struct_name) = self.get_expression_struct_name(object)? {
                    // Look up struct definition to get field type
                    // Clone field_type to avoid borrow issues
                    let field_type_opt =
                        self.struct_defs
                            .get(&object_struct_name)
                            .and_then(|struct_def| {
                                struct_def
                                    .fields
                                    .iter()
                                    .find(|(f, _)| f == field)
                                    .map(|(_, t)| t.clone())
                            });

                    if let Some(field_type) = field_type_opt {
                        // Check if field type is a struct
                        match field_type {
                            Type::Named(field_struct_name) => {
                                if self.struct_defs.contains_key(&field_struct_name) {
                                    Ok(Some(field_struct_name))
                                } else {
                                    Ok(None)
                                }
                            }
                            Type::Generic { name, type_args } => {
                                // Generic struct field like Box<i32>
                                // Return the mangled name
                                match self.instantiate_generic_struct(&name, &type_args) {
                                    Ok(mangled_name) => Ok(Some(mangled_name)),
                                    Err(_) => Ok(None),
                                }
                            }
                            _ => Ok(None),
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            // Function call: look up return type in function_defs
            Expression::Call {
                span_id: _, func, ..
            } => {
                if let Expression::Ident(func_name) = func.as_ref() {
                    // Clone return_type to avoid borrow issues
                    let return_type_opt = self
                        .function_defs
                        .get(func_name)
                        .and_then(|func_def| func_def.return_type.clone());

                    if let Some(return_type) = return_type_opt {
                        match return_type {
                            Type::Named(struct_name) => {
                                if self.struct_defs.contains_key(&struct_name) {
                                    Ok(Some(struct_name))
                                } else {
                                    Ok(None)
                                }
                            }
                            Type::Generic { name, type_args } => {
                                match self.instantiate_generic_struct(&name, &type_args) {
                                    Ok(mangled_name) => Ok(Some(mangled_name)),
                                    Err(_) => Ok(None),
                                }
                            }
                            _ => Ok(None),
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            // Type constructor: treat like function call
            Expression::TypeConstructor { type_name, .. } => {
                // Type_new() might return the type itself
                // Check if it's a struct
                if self.struct_defs.contains_key(type_name) {
                    Ok(Some(type_name.clone()))
                } else {
                    Ok(None)
                }
            }
            // Other expressions don't return structs
            _ => Ok(None),
        }
    }

    /// Calculate nesting depth of a generic type
    /// Example: Box<Box<Box<i32>>> = depth 3
    pub(crate) fn get_generic_depth(&self, ty: &Type) -> usize {
        match ty {
            Type::Generic { type_args, .. } => {
                // Get max depth from all type arguments, add 1 for current level
                let max_arg_depth = type_args
                    .iter()
                    .map(|arg| self.get_generic_depth(arg))
                    .max()
                    .unwrap_or(0);
                1 + max_arg_depth
            }
            Type::Reference(inner, _) => self.get_generic_depth(inner),
            Type::Array(elem, _) => self.get_generic_depth(elem),
            _ => 0,
        }
    }
}
