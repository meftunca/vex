// statements/assignment.rs
// simple assignment and compound assignment

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn compile_assign_statement(
        &mut self,
        target: &Expression,
        value: &Expression,
    ) -> Result<(), String> {
        let val = self.compile_expression(value)?;

        match target {
            // Simple variable assignment: x = value
            Expression::Ident(name) => {
                let ptr = self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                self.builder
                    .build_store(*ptr, val)
                    .map_err(|e| format!("Failed to assign: {}", e))?;
            }

            // Field assignment: obj.field = value
            Expression::FieldAccess { object, field } => {
                // Get field pointer using similar logic to compile_field_access
                if let Expression::Ident(var_name) = &**object {
                    // Get struct name from tracking
                    let struct_name = self
                        .variable_struct_names
                        .get(var_name)
                        .ok_or_else(|| format!("Variable {} is not a struct", var_name))?
                        .clone();

                    // Get variable pointer
                    let var_ptr = *self
                        .variables
                        .get(var_name)
                        .ok_or_else(|| format!("Variable {} not found", var_name))?;

                    // After struct variable storage fix, variables[name] now holds the DIRECT pointer.
                    let struct_ptr_val = var_ptr;

                    // Get struct definition
                    let struct_def = self
                        .struct_defs
                        .get(&struct_name)
                        .cloned()
                        .ok_or_else(|| format!("Struct '{}' not found", struct_name))?;

                    // Find field index
                    let field_index = struct_def
                        .fields
                        .iter()
                        .position(|(name, _)| name == field)
                        .ok_or_else(|| {
                            format!("Field '{}' not found in struct '{}'", field, struct_name)
                        })? as u32;

                    // Build struct type
                    let field_types: Vec<inkwell::types::BasicTypeEnum> = struct_def
                        .fields
                        .iter()
                        .map(|(_, ty)| self.ast_type_to_llvm(ty))
                        .collect();
                    let struct_type = self.context.struct_type(&field_types, false);

                    // Get field pointer
                    let field_ptr = self
                        .builder
                        .build_struct_gep(
                            struct_type,
                            struct_ptr_val,
                            field_index,
                            &format!("{}.{}", var_name, field),
                        )
                        .map_err(|e| format!("Failed to get field pointer: {}", e))?;

                    // Store value
                    self.builder
                        .build_store(field_ptr, val)
                        .map_err(|e| format!("Failed to store field: {}", e))?;
                } else {
                    return Err("Complex field assignment not yet supported".to_string());
                }
            }

            // Index assignment: arr[i] = value or map[key] = value
            Expression::Index { object, index } => {
                // Check if this is a Map
                if let Expression::Ident(var_name) = &**object {
                    if let Some(struct_name) = self.variable_struct_names.get(var_name) {
                        if struct_name == "Map" {
                            // Map indexing: map[key] = value -> map.insert(key, value)
                            let map_ptr_var = *self
                                .variables
                                .get(var_name)
                                .ok_or_else(|| format!("Map {} not found", var_name))?;

                            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                            let map_ptr = self
                                .builder
                                .build_load(ptr_type, map_ptr_var, &format!("{}_ptr", var_name))
                                .map_err(|e| format!("Failed to load map pointer: {}", e))?;

                            // Compile key and value
                            let key = self.compile_expression(index)?;

                            // Call vex_map_insert(map, key, value)
                            let vex_map_insert = self.declare_runtime_fn(
                                "vex_map_insert",
                                &[ptr_type.into(), ptr_type.into(), ptr_type.into()],
                                self.context.bool_type().into(),
                            );

                            self.builder
                                .build_call(
                                    vex_map_insert,
                                    &[map_ptr.into(), key.into(), val.into()],
                                    "map_insert",
                                )
                                .map_err(|e| format!("Failed to call vex_map_insert: {}", e))?;

                            return Ok(());
                        }
                    }
                }

                // Array indexing - not yet supported
                return Err("Array index assignment not yet supported".to_string());
            }

            _ => {
                return Err(
                    "Complex assignment targets not yet supported (array indexing, etc.)"
                        .to_string(),
                );
            }
        }

        Ok(())
    }

    pub(crate) fn compile_compound_assign_statement(
        &mut self,
        target: &Expression,
        op: &CompoundOp,
        value: &Expression,
    ) -> Result<(), String> {
        // Compound assignment: x += expr is equivalent to x = x + expr
        // First load the current value
        let current_val = self.compile_expression(target)?;
        let rhs_val = self.compile_expression(value)?;

        // Perform the operation based on type
        let result: BasicValueEnum = if current_val.is_int_value() {
            let lhs = current_val.into_int_value();
            let rhs = rhs_val.into_int_value();
            match op {
                CompoundOp::Add => self
                    .builder
                    .build_int_add(lhs, rhs, "add_tmp")
                    .map_err(|e| format!("Failed to build add: {}", e))?
                    .into(),
                CompoundOp::Sub => self
                    .builder
                    .build_int_sub(lhs, rhs, "sub_tmp")
                    .map_err(|e| format!("Failed to build sub: {}", e))?
                    .into(),
                CompoundOp::Mul => self
                    .builder
                    .build_int_mul(lhs, rhs, "mul_tmp")
                    .map_err(|e| format!("Failed to build mul: {}", e))?
                    .into(),
                CompoundOp::Div => self
                    .builder
                    .build_int_signed_div(lhs, rhs, "div_tmp")
                    .map_err(|e| format!("Failed to build div: {}", e))?
                    .into(),
            }
        } else if current_val.is_float_value() {
            let lhs = current_val.into_float_value();
            let rhs = rhs_val.into_float_value();
            match op {
                CompoundOp::Add => self
                    .builder
                    .build_float_add(lhs, rhs, "add_tmp")
                    .map_err(|e| format!("Failed to build add: {}", e))?
                    .into(),
                CompoundOp::Sub => self
                    .builder
                    .build_float_sub(lhs, rhs, "sub_tmp")
                    .map_err(|e| format!("Failed to build sub: {}", e))?
                    .into(),
                CompoundOp::Mul => self
                    .builder
                    .build_float_mul(lhs, rhs, "mul_tmp")
                    .map_err(|e| format!("Failed to build mul: {}", e))?
                    .into(),
                CompoundOp::Div => self
                    .builder
                    .build_float_div(lhs, rhs, "div_tmp")
                    .map_err(|e| format!("Failed to build div: {}", e))?
                    .into(),
            }
        } else {
            return Err("Invalid type for compound assignment".to_string());
        };

        // Store the result back
        match target {
            Expression::Ident(name) => {
                // Simple variable assignment
                let ptr = self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                self.builder
                    .build_store(*ptr, result)
                    .map_err(|e| format!("Failed to store compound assignment: {}", e))?;
            }
            Expression::FieldAccess { object, field } => {
                // Field access assignment: obj.field += value
                let field_ptr = self.get_field_pointer(object, field)?;
                self.builder
                    .build_store(field_ptr, result)
                    .map_err(|e| format!("Failed to store field compound assignment: {}", e))?;
            }
            Expression::Index { object, index } => {
                // Array index assignment: arr[i] += value
                let elem_ptr = self.get_index_pointer(object, index)?;
                self.builder
                    .build_store(elem_ptr, result)
                    .map_err(|e| format!("Failed to store array compound assignment: {}", e))?;
            }
            _ => {
                return Err("This compound assignment target is not yet supported".to_string());
            }
        }

        Ok(())
    }
}
