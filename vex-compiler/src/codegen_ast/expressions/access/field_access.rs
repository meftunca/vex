// Struct field access compilation

use crate::codegen_ast::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile field access: obj.field
    pub(crate) fn compile_field_access(
        &mut self,
        object: &Expression,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Phase 0.8: Check if this is tuple field access (field is numeric: "0", "1", "2", etc.)
        if let Ok(tuple_index) = field.parse::<u32>() {
            return self.compile_tuple_field_access(object, tuple_index);
        }

        // Handle chained field access: outer.value.value
        // Strategy: Recursively compile object expression, determine its struct type

        // Case 1: Nested field access (chained)
        if let Expression::FieldAccess {
            object: inner_obj,
            field: inner_field,
        } = object
        {
            // Recursively get intermediate value AND its struct type
            let (intermediate_value, intermediate_struct_name) =
                self.compile_field_access_with_type(inner_obj, inner_field)?;

            if let Some(struct_name) = intermediate_struct_name {
                // Now access field on the intermediate struct value
                return self.compile_field_access_on_value(intermediate_value, &struct_name, field);
            } else {
                return Err(format!(
                    "Cannot determine struct type for chained field access"
                ));
            }
        }

        // Case 2: Simple variable field access
        if let Expression::Ident(var_name) = object {
            let func_name = self
                .current_function
                .map(|f| f.get_name().to_string_lossy().to_string())
                .unwrap_or_else(|| "None".to_string());
            eprintln!(
                "🔍 Field access: {}.{} (in function: {})",
                var_name, field, func_name
            );
            eprintln!(
                "   variables = {:?}",
                self.variables.keys().collect::<Vec<_>>()
            );
            eprintln!(
                "   variable_struct_names = {:?}",
                self.variable_struct_names
            );
            // Check if this variable is tracked as a struct
            let maybe_struct_name = self.variable_struct_names.get(var_name).cloned();

            if maybe_struct_name.is_none() {
                eprintln!("   ⚠️ Variable not tracked in variable_struct_names");
                eprintln!("   → Checking type annotation...");

                // Fallback: Try to infer from type annotation stored during Let statement
                // This happens when type annotation is present: let inner: Box<i32> = ...
                // In such cases, we might have tracked the type but not the struct name
                // Look at variable_types to see if we can infer the struct name
                if let Some(var_type) = self.variable_types.get(var_name) {
                    eprintln!("   → Found type in variable_types: {:?}", var_type);
                    // For now, we can't easily extract struct name from LLVM type
                    // This is a limitation - we need better type tracking
                }
            }

            if let Some(struct_name) = maybe_struct_name {
                eprintln!("   ✅ Found struct: {}", struct_name);
                let var_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Variable {} not found", var_name))?;

                // CRITICAL FIX: After struct variable storage fix, self.variables[name] now holds
                // the DIRECT pointer to the struct (not a pointer to a pointer variable).
                // So we should use var_ptr directly, NOT load it!
                let struct_ptr_val = var_ptr;

                // AUTO-DEREF: Only for specific cases (Box<T>, external method receivers)
                // NOT for regular struct variables on the stack
                // Check if this variable is 'self' in an external method (which is already a pointer parameter)
                let is_external_self = var_name == "self"; // 'self' is always a parameter, never needs deref

                if !is_external_self {
                    // For non-self variables, check if we need auto-deref (Box<T> case)
                    // This is complex, for now skip auto-deref for all non-self variables
                    // TODO: Add proper AST type tracking to determine when to auto-deref
                    eprintln!("   ℹ️  No auto-deref for variable '{}'", var_name);
                }

                // Get struct definition from registry
                let struct_def = self
                    .struct_defs
                    .get(&struct_name)
                    .cloned()
                    .ok_or_else(|| format!("Struct '{}' not found in registry", struct_name))?;

                // Find field index
                let field_index = struct_def
                    .fields
                    .iter()
                    .position(|(name, _)| name == field)
                    .ok_or_else(|| {
                        format!("Field '{}' not found in struct '{}'", field, struct_name)
                    })? as u32;

                // Rebuild struct type from definition
                let field_types: Vec<BasicTypeEnum> = struct_def
                    .fields
                    .iter()
                    .map(|(_, ty)| self.ast_type_to_llvm(ty))
                    .collect();
                let struct_type = self.context.struct_type(&field_types, false);

                // Get pointer to field
                let field_ptr = self
                    .builder
                    .build_struct_gep(
                        struct_type,
                        struct_ptr_val,
                        field_index,
                        &format!("{}.{}", var_name, field),
                    )
                    .map_err(|e| format!("Failed to get field pointer: {}", e))?;

                // Get field type from AST definition (critical for nested generics!)
                let (_, field_ast_type) = &struct_def.fields[field_index as usize];
                let field_llvm_type = self.ast_type_to_llvm(field_ast_type);

                // CRITICAL: Check if field is a struct type - return pointer for zero-copy
                match field_ast_type {
                    Type::Named(field_type_name) => {
                        if self.struct_defs.contains_key(field_type_name) {
                            return Ok(field_ptr.into());
                        }
                    }
                    Type::Generic { name, type_args } => {
                        let mangled_name = self.type_to_string(field_ast_type);
                        if self.struct_defs.contains_key(&mangled_name) {
                            return Ok(field_ptr.into());
                        }
                    }
                    Type::Box(_) | Type::Vec(_) | Type::Option(_) | Type::Result(_, _) => {
                        // Phase 0 builtin types - always structs, return pointer
                        return Ok(field_ptr.into());
                    }
                    _ => {}
                }

                // Load and return field value
                return self
                    .builder
                    .build_load(field_llvm_type, field_ptr, &format!("{}_val", field))
                    .map_err(|e| format!("Failed to load field: {}", e));
            }
        }

        eprintln!("❌ Field access FAILED for field: {}", field);
        Err(format!("Cannot access field {} on non-struct value", field))
    }

    /// Compile field access and return both value and struct type name (for chained access)
    pub(super) fn compile_field_access_with_type(
        &mut self,
        object: &Expression,
        field: &str,
    ) -> Result<(BasicValueEnum<'ctx>, Option<String>), String> {
        // Compile the field access to get the value
        let value = self.compile_field_access(object, field)?;

        // Determine the struct type of the resulting value
        let struct_type = self.get_field_struct_type(object, field)?;

        Ok((value, struct_type))
    }

    /// Compile field access on an already-loaded struct value
    pub(super) fn compile_field_access_on_value(
        &mut self,
        struct_value: BasicValueEnum<'ctx>,
        struct_name: &str,
        field: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Get struct definition
        let struct_def = self
            .struct_defs
            .get(struct_name)
            .cloned()
            .ok_or_else(|| format!("Struct '{}' not found", struct_name))?;

        // Find field index
        let field_index = struct_def
            .fields
            .iter()
            .position(|(name, _)| name == field)
            .ok_or_else(|| format!("Field '{}' not found in struct '{}'", field, struct_name))?
            as u32;

        // Value must be pointer to struct
        if !struct_value.is_pointer_value() {
            return Err(format!("Expected pointer value for struct field access"));
        }

        let struct_ptr = struct_value.into_pointer_value();

        // Rebuild struct type
        let field_types: Vec<BasicTypeEnum> = struct_def
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
                struct_ptr,
                field_index,
                &format!("field_{}", field),
            )
            .map_err(|e| format!("Failed to get field pointer: {}", e))?;

        // Get field type and load
        let (_, field_ast_type) = &struct_def.fields[field_index as usize];
        let field_llvm_type = self.ast_type_to_llvm(field_ast_type);

        // CRITICAL: If field is a struct type, return pointer (zero-copy)
        // Otherwise load the value normally
        match field_ast_type {
            Type::Named(field_type_name) => {
                if self.struct_defs.contains_key(field_type_name) {
                    // Field is a struct - return pointer without loading (zero-copy)
                    return Ok(field_ptr.into());
                }
            }
            Type::Box(_) | Type::Vec(_) | Type::Option(_) | Type::Result(_, _) => {
                // Phase 0 builtin types - always structs, return pointer
                return Ok(field_ptr.into());
            }
            Type::Generic { name, type_args } => {
                // Generic struct (e.g., Box<i32>) - calculate mangled name
                // Use the same mangling strategy as instantiate_generic_struct
                let mangled_type_args: Vec<String> = type_args
                    .iter()
                    .map(|ty| match ty {
                        Type::Named(n) => n.clone(),
                        Type::I8 => "i8".to_string(),
                        Type::I16 => "i16".to_string(),
                        Type::I32 => "i32".to_string(),
                        Type::I64 => "i64".to_string(),
                        Type::I128 => "i128".to_string(),
                        Type::U8 => "u8".to_string(),
                        Type::U16 => "u16".to_string(),
                        Type::U32 => "u32".to_string(),
                        Type::U64 => "u64".to_string(),
                        Type::U128 => "u128".to_string(),
                        Type::F16 => "f16".to_string(),
                        Type::F32 => "f32".to_string(),
                        Type::F64 => "f64".to_string(),
                        Type::Bool => "bool".to_string(),
                        Type::String => "string".to_string(),
                        Type::Byte => "byte".to_string(),
                        Type::Nil => "nil".to_string(),
                        Type::Error => "error".to_string(),
                        Type::Generic {
                            name: gn,
                            type_args: gta,
                        } => {
                            // Nested generic: Box<Box<i32>> -> Box_Box_i32
                            let inner_mangled: Vec<String> = gta
                                .iter()
                                .map(|t| format!("{:?}", t).replace("\"", ""))
                                .collect();
                            format!("{}_{}", gn, inner_mangled.join("_"))
                        }
                        _ => format!("{:?}", ty),
                    })
                    .collect();
                let mangled_name = format!("{}_{}", name, mangled_type_args.join("_"));

                if self.struct_defs.contains_key(&mangled_name) {
                    // Field is a generic struct - return pointer without loading
                    return Ok(field_ptr.into());
                }
            }
            _ => {}
        }

        // Non-struct field: load normally
        self.builder
            .build_load(field_llvm_type, field_ptr, "field_val")
            .map_err(|e| format!("Failed to load field: {}", e))
    }

    /// Phase 0.8: Compile tuple field access: tuple.0, tuple.1, etc.
    pub(super) fn compile_tuple_field_access(
        &mut self,
        object: &Expression,
        field_index: u32,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile the tuple expression
        let tuple_val = self.compile_expression(object)?;

        // Tuple is represented as a pointer to anonymous struct
        if !tuple_val.is_pointer_value() {
            return Err("Tuple field access requires pointer value".to_string());
        }

        let tuple_ptr = tuple_val.into_pointer_value();

        // CRITICAL: With LLVM 15+ opaque pointers, we need to reconstruct the tuple type
        // We'll inspect the tuple variable to get its type
        // This is a temporary workaround - ideally we'd track tuple types separately

        // For direct variable access (tuple.0), get type from variable_types
        let tuple_struct_type = if let Expression::Ident(var_name) = object {
            if let Some(var_type) = self.variable_types.get(var_name) {
                if let BasicTypeEnum::StructType(st) = var_type {
                    *st
                } else {
                    return Err(format!(
                        "Variable '{}' is not a tuple (struct type)",
                        var_name
                    ));
                }
            } else {
                return Err(format!("Variable '{}' type not found", var_name));
            }
        } else {
            // For complex expressions, we need to store tuple metadata
            // For now, return an error
            return Err(
                "Tuple field access only supported on direct variables for now".to_string(),
            );
        };

        // Check if field index is within bounds
        let field_count = tuple_struct_type.count_fields();
        if field_index >= field_count {
            return Err(format!(
                "Tuple field index {} out of bounds (tuple has {} fields)",
                field_index, field_count
            ));
        }

        // Get pointer to field
        let field_ptr = self
            .builder
            .build_struct_gep(
                tuple_struct_type,
                tuple_ptr,
                field_index,
                &format!("tuple_field_{}", field_index),
            )
            .map_err(|e| format!("Failed to get tuple field pointer: {}", e))?;

        // Get field type
        let field_type = tuple_struct_type
            .get_field_type_at_index(field_index)
            .ok_or_else(|| format!("Cannot get field type at index {}", field_index))?;

        // Load and return field value
        self.builder
            .build_load(field_type, field_ptr, "tuple_field_val")
            .map_err(|e| format!("Failed to load tuple field: {}", e))
    }

    /// Get the struct type name of a field from an object expression
    pub(crate) fn get_field_struct_type(
        &mut self,
        object: &Expression,
        field: &str,
    ) -> Result<Option<String>, String> {
        match object {
            Expression::Ident(var_name) => {
                // Get struct name of variable
                if let Some(struct_name) = self.variable_struct_names.get(var_name).cloned() {
                    // Look up field type in struct definition (clone to avoid borrow issues)
                    let field_type_opt = self
                        .struct_defs
                        .get(&struct_name)
                        .and_then(|def| def.fields.iter().find(|(f, _)| f == field))
                        .map(|(_, t)| t.clone());

                    if let Some(field_type) = field_type_opt {
                        // Check if field is a struct type
                        match field_type {
                            Type::Named(field_struct_name) => {
                                if self.struct_defs.contains_key(&field_struct_name) {
                                    eprintln!(
                                        "  → Field type is Named struct: {}",
                                        field_struct_name
                                    );
                                    return Ok(Some(field_struct_name));
                                }
                            }
                            Type::Generic { name, type_args } => {
                                // Generic struct field - get mangled name
                                eprintln!("  → Field type is Generic: {}<??>", name);
                                match self.instantiate_generic_struct(&name, &type_args) {
                                    Ok(mangled) => {
                                        eprintln!("  → Mangled name: {}", mangled);
                                        return Ok(Some(mangled));
                                    }
                                    Err(e) => {
                                        eprintln!("  → Instantiation failed: {}", e);
                                        return Ok(None);
                                    }
                                }
                            }
                            Type::Box(inner_ty) => {
                                // Phase 0: Box<T> builtin type
                                eprintln!("  → Field type is Box<?>");
                                let mangled =
                                    format!("Box_{}", self.type_to_string(inner_ty.as_ref()));
                                eprintln!("  → Box mangled: {}", mangled);
                                if self.struct_defs.contains_key(&mangled) {
                                    return Ok(Some(mangled));
                                }
                            }
                            _ => {
                                eprintln!("  → Field type is not struct: {:?}", field_type);
                            }
                        }
                    } else {
                        eprintln!("  → Field not found in struct definition");
                    }
                }
                Ok(None)
            }
            Expression::FieldAccess {
                object: inner_obj,
                field: inner_field,
            } => {
                // CRITICAL FIX: For chained field access, we need to:
                // 1. Get the type of the inner field access (e.g., b3.value -> Box<Box<i32>>)
                // 2. Then get the type of 'field' from that result type

                // First, get the struct type of the inner field access
                let inner_struct_type = self.get_field_struct_type(inner_obj, inner_field)?;

                if let Some(inner_struct_name) = inner_struct_type {
                    // Now look up 'field' in that struct type
                    let field_type_opt = self
                        .struct_defs
                        .get(&inner_struct_name)
                        .and_then(|def| def.fields.iter().find(|(f, _)| f == field))
                        .map(|(_, t)| t.clone());

                    if let Some(field_type) = field_type_opt {
                        match field_type {
                            Type::Named(field_struct_name) => {
                                if self.struct_defs.contains_key(&field_struct_name) {
                                    return Ok(Some(field_struct_name));
                                }
                            }
                            Type::Generic { name, type_args } => {
                                match self.instantiate_generic_struct(&name, &type_args) {
                                    Ok(mangled) => return Ok(Some(mangled)),
                                    Err(_) => return Ok(None),
                                }
                            }
                            Type::Box(inner_ty) => {
                                // Chained Box<T> access
                                let mangled =
                                    format!("Box_{}", self.type_to_string(inner_ty.as_ref()));
                                if self.struct_defs.contains_key(&mangled) {
                                    return Ok(Some(mangled));
                                }
                            }
                            Type::Vec(inner_ty) => {
                                let mangled =
                                    format!("Vec_{}", self.type_to_string(inner_ty.as_ref()));
                                if self.struct_defs.contains_key(&mangled) {
                                    return Ok(Some(mangled));
                                }
                            }
                            Type::Option(inner_ty) => {
                                let mangled =
                                    format!("Option_{}", self.type_to_string(inner_ty.as_ref()));
                                if self.struct_defs.contains_key(&mangled) {
                                    return Ok(Some(mangled));
                                }
                            }
                            Type::Result(ok_ty, err_ty) => {
                                let mangled = format!(
                                    "Result_{}_{}",
                                    self.type_to_string(ok_ty.as_ref()),
                                    self.type_to_string(err_ty.as_ref())
                                );
                                if self.struct_defs.contains_key(&mangled) {
                                    return Ok(Some(mangled));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
