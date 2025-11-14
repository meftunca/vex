// Instance method call compilation (variable and field access receivers)

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Get struct name and receiver value for instance method calls
    pub(crate) fn get_receiver_info(
        &mut self,
        receiver: &Expression,
    ) -> Result<(String, inkwell::values::PointerValue<'ctx>), String> {
        match receiver {
            Expression::Ident(var_name) => {
                eprintln!("🔍 Receiver is identifier: {}", var_name);

                // Check if it's an array first (arrays don't have struct names)
                if let Some(var_type) = self.variable_types.get(var_name) {
                    if let inkwell::types::BasicTypeEnum::ArrayType(_) = var_type {
                        let var_ptr = self
                            .variables
                            .get(var_name)
                            .ok_or_else(|| format!("Array variable {} not found", var_name))?;
                        return Ok(("Array".to_string(), *var_ptr));
                    }
                }

                let struct_name = self
                    .variable_struct_names
                    .get(var_name)
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "Variable {} is not a struct or module, cannot call methods",
                            var_name
                        )
                    })?;

                // Get variable pointer
                let var_ptr = self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Variable {} not found", var_name))?;

                eprintln!("🔍 Receiver var_ptr: {:?}", var_ptr);
                Ok((struct_name, *var_ptr))
            }
            Expression::FieldAccess { object, field } => {
                // Handle field access: self.counter.next()
                eprintln!(
                    "🔧 Method call on field access: {}.{}.{}",
                    if let Expression::Ident(n) = object.as_ref() {
                        n
                    } else {
                        "?"
                    },
                    field,
                    "method"
                );

                // Get the object variable
                if let Expression::Ident(var_name) = object.as_ref() {
                    let object_struct_name =
                        self.variable_struct_names.get(var_name).ok_or_else(|| {
                            format!("Variable {} not found or not a struct", var_name)
                        })?;

                    // Get struct definition
                    let struct_def = self
                        .struct_defs
                        .get(object_struct_name)
                        .ok_or_else(|| format!("Struct {} not found", object_struct_name))?
                        .clone();

                    // Find field index and type
                    let field_index = struct_def
                        .fields
                        .iter()
                        .position(|(name, _)| name == field)
                        .ok_or_else(|| {
                            format!("Field {} not found in struct {}", field, object_struct_name)
                        })?;

                    let field_type = &struct_def.fields[field_index].1;

                    // Get field struct name
                    let field_struct_name = if let Type::Named(name) = field_type {
                        name.clone()
                    } else {
                        return Err(format!("Field {} is not a struct type", field));
                    };

                    // Get object pointer
                    let object_ptr = self
                        .variables
                        .get(var_name)
                        .ok_or_else(|| format!("Variable {} not found", var_name))?;

                    // Get field pointer
                    let field_ptr = self
                        .builder
                        .build_struct_gep(
                            self.ast_type_to_llvm(field_type),
                            *object_ptr,
                            field_index as u32,
                            &format!("{}_field_{}", var_name, field),
                        )
                        .map_err(|e| format!("Failed to GEP field: {}", e))?;

                    eprintln!("  ✅ Field GEP successful, struct: {}", field_struct_name);
                    Ok((field_struct_name, field_ptr))
                } else {
                    Err(
                        "Method calls on field access only supported when object is a variable"
                            .to_string(),
                    )
                }
            }
            _ => {
                Err("Method calls only supported on variables and field access for now".to_string())
            }
        }
    }
}
