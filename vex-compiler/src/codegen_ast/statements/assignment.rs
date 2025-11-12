// statements/assignment.rs
// simple assignment and compound assignment

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

/// Helper function to map CompoundOp to trait name
fn compound_op_to_trait(op: &CompoundOp) -> &'static str {
    match op {
        CompoundOp::Add => "Add",
        CompoundOp::Sub => "Sub",
        CompoundOp::Mul => "Mul",
        CompoundOp::Div => "Div",
        CompoundOp::Mod => "Mod",
        CompoundOp::BitAnd => "BitAnd",
        CompoundOp::BitOr => "BitOr",
        CompoundOp::BitXor => "BitXor",
        CompoundOp::Shl => "Shl",
        CompoundOp::Shr => "Shr",
    }
}

/// Helper function to map CompoundOp to method name
fn compound_op_to_method(op: &CompoundOp) -> &'static str {
    match op {
        CompoundOp::Add => "op+",
        CompoundOp::Sub => "op-",
        CompoundOp::Mul => "op*",
        CompoundOp::Div => "op/",
        CompoundOp::Mod => "op%",
        CompoundOp::BitAnd => "op&",
        CompoundOp::BitOr => "op|",
        CompoundOp::BitXor => "op^",
        CompoundOp::Shl => "op<<",
        CompoundOp::Shr => "op>>",
    }
}

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
                    // Special handling for 'self' in external mutable methods
                    // In Golang-style methods with receiver (self: &Type!), 'self' is a parameter (reference)
                    // and should be used directly for field access.
                    let (struct_name, struct_ptr_val) = if var_name == "self" {
                        // For 'self', get the struct name from method context
                        // 'self' is already a parameter pointer, use it directly
                        let self_ptr = *self
                            .variables
                            .get("self")
                            .ok_or_else(|| "self not found in method context".to_string())?;

                        // Get struct name from self's type tracking
                        let struct_name = self
                            .variable_struct_names
                            .get("self")
                            .ok_or_else(|| "self is not a struct".to_string())?
                            .clone();

                        (struct_name, self_ptr)
                    } else {
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
                        (struct_name, var_ptr)
                    };

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
                // First check if this is a user-defined type with IndexMut trait
                if let Expression::Ident(var_name) = &**object {
                    if let Some(struct_name) = self.variable_struct_names.get(var_name).cloned() {
                        // Check for IndexMut trait (key is (trait_name, struct_name))
                        let impl_key = ("IndexMut".to_string(), struct_name.clone());
                        
                        if self.trait_impls.contains_key(&impl_key) {
                            // Call op[]=(index, value)
                            let method_name = format!("{}_op[]=", struct_name);
                            let function = self.module.get_function(&method_name)
                                .ok_or_else(|| format!("IndexMut operator method '{}' not found", method_name))?;
                            
                            // Get self pointer (NOT loaded - pass pointer for mutation)
                            let var_ptr = *self.variables.get(var_name)
                                .ok_or_else(|| format!("Variable {} not found", var_name))?;
                            
                            // Compile index and value
                            let index_val = self.compile_expression(index)?;
                            
                            // Call op[]=(self_ptr, index, value) - pass pointer for mutation
                            self.builder.build_call(
                                function,
                                &[var_ptr.into(), index_val.into(), val.into()],
                                "index_assign"
                            )
                            .map_err(|e| format!("Failed to call index assignment operator: {}", e))?;
                            
                            return Ok(());
                        }
                        
                        // Fallback to builtin Map handling
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
        // Compound assignment: x += expr
        // Strategy:
        // 1. For primitives: direct LLVM IR (x = x + expr)
        // 2. For structs: call trait method if available, otherwise desugar
        
        // First, check if target is a struct with operator trait
        let target_type = self.infer_expression_type(target)?;
        
        // Check if we should use trait dispatch
        if let Type::Named(type_name) = &target_type {
            if self.struct_defs.contains_key(type_name) {
                // This is a struct - try trait dispatch
                let trait_name = compound_op_to_trait(op);
                
                if let Some(_) = self.has_operator_trait(type_name, trait_name) {
                    // Use trait method call: result = x.op+(y)
                    let method_name = compound_op_to_method(op);
                    
                    // Call the binary operator method
                    let result = self.compile_method_call(
                        target,
                        method_name,
                        &[], // No generic type args
                        &vec![value.clone()],
                        false,
                    )?;
                    
                    // Store result back to target
                    match target {
                        Expression::Ident(name) => {
                            let ptr = self
                                .variables
                                .get(name)
                                .ok_or_else(|| format!("Variable {} not found", name))?;
                            self.builder
                                .build_store(*ptr, result)
                                .map_err(|e| format!("Failed to store compound assignment: {}", e))?;
                            return Ok(());
                        }
                        Expression::FieldAccess { object, field } => {
                            let field_ptr = self.get_field_pointer(object, field)?;
                            self.builder
                                .build_store(field_ptr, result)
                                .map_err(|e| format!("Failed to store field compound assignment: {}", e))?;
                            return Ok(());
                        }
                        _ => {
                            return Err("Unsupported compound assignment target for struct".to_string());
                        }
                    }
                }
            }
        }
        
        // Fallback: builtin types or structs without trait
        // Desugar to: x = x + expr
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
                CompoundOp::Mod => self
                    .builder
                    .build_int_signed_rem(lhs, rhs, "mod_tmp")
                    .map_err(|e| format!("Failed to build mod: {}", e))?
                    .into(),
                CompoundOp::BitAnd => self
                    .builder
                    .build_and(lhs, rhs, "and_tmp")
                    .map_err(|e| format!("Failed to build bitwise and: {}", e))?
                    .into(),
                CompoundOp::BitOr => self
                    .builder
                    .build_or(lhs, rhs, "or_tmp")
                    .map_err(|e| format!("Failed to build bitwise or: {}", e))?
                    .into(),
                CompoundOp::BitXor => self
                    .builder
                    .build_xor(lhs, rhs, "xor_tmp")
                    .map_err(|e| format!("Failed to build bitwise xor: {}", e))?
                    .into(),
                CompoundOp::Shl => self
                    .builder
                    .build_left_shift(lhs, rhs, "shl_tmp")
                    .map_err(|e| format!("Failed to build left shift: {}", e))?
                    .into(),
                CompoundOp::Shr => self
                    .builder
                    .build_right_shift(lhs, rhs, false, "shr_tmp")
                    .map_err(|e| format!("Failed to build right shift: {}", e))?
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
                CompoundOp::Mod => {
                    return Err("Modulo operator not supported for floats".to_string());
                }
                CompoundOp::BitAnd
                | CompoundOp::BitOr
                | CompoundOp::BitXor
                | CompoundOp::Shl
                | CompoundOp::Shr => {
                    return Err("Bitwise operators not supported for floats".to_string());
                }
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
