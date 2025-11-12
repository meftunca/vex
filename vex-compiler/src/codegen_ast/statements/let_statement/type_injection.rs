// Type argument injection for nested generic struct literals

use crate::codegen_ast::ASTCodeGen;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Recursively inject type arguments into nested generic struct literals
    /// Handles Box<Box<Box<T>>> with nested StructLiteral { Box { value: StructLiteral { Box { ... } } } }
    pub(crate) fn inject_type_args_recursive(
        &self,
        expr: &Expression,
        target_type: &Type,
    ) -> Result<Expression, String> {
        match expr {
            Expression::StructLiteral {
                name: struct_name,
                type_args: ref literal_type_args,
                fields: ref literal_fields,
            } => {
                // If struct literal has empty type_args and target type is Generic, inject
                let new_type_args = if literal_type_args.is_empty() {
                    match target_type {
                        Type::Generic {
                            name: target_struct_name,
                            type_args: ref target_type_args,
                        } if struct_name == target_struct_name => {
                            eprintln!(
                                "  🔧 Injecting type args into {}: {:?}",
                                struct_name, target_type_args
                            );
                            target_type_args.clone()
                        }
                        Type::Box(inner_type) if struct_name == "Box" => {
                            vec![inner_type.as_ref().clone()]
                        }
                        Type::Vec(inner_type) if struct_name == "Vec" => {
                            vec![inner_type.as_ref().clone()]
                        }
                        Type::Option(inner_type) if struct_name == "Option" => {
                            vec![inner_type.as_ref().clone()]
                        }
                        Type::Result(ok_type, err_type) if struct_name == "Result" => {
                            vec![ok_type.as_ref().clone(), err_type.as_ref().clone()]
                        }
                        _ => literal_type_args.clone(),
                    }
                } else {
                    literal_type_args.clone()
                };

                // Recursively process field values
                let mut new_fields = Vec::new();
                for (field_name, field_expr) in literal_fields.iter() {
                    // Determine expected type for this field
                    let field_target_type = if field_name == "value" && !new_type_args.is_empty() {
                        Some(&new_type_args[0])
                    } else {
                        None
                    };

                    let new_field_expr = if let Some(ft) = field_target_type {
                        self.inject_type_args_recursive(field_expr, ft)?
                    } else {
                        field_expr.clone()
                    };

                    new_fields.push((field_name.clone(), new_field_expr));
                }

                Ok(Expression::StructLiteral {
                    name: struct_name.clone(),
                    type_args: new_type_args,
                    fields: new_fields,
                })
            }
            _ => Ok(expr.clone()),
        }
    }
}
