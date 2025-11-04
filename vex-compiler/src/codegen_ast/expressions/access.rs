// Field access, indexing, and f-strings

use super::super::ASTCodeGen;
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
            if let Some(struct_name) = self.variable_struct_names.get(var_name).cloned() {
                eprintln!("   ✅ Found struct: {}", struct_name);
                let var_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Variable {} not found", var_name))?;

                // CRITICAL FIX: After struct variable storage fix, self.variables[name] now holds
                // the DIRECT pointer to the struct (not a pointer to a pointer variable).
                // So we should use var_ptr directly, NOT load it!
                let struct_ptr_val = var_ptr;

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
    fn compile_field_access_with_type(
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

    /// Compile array indexing
    pub(crate) fn compile_index(
        &mut self,
        object: &Expression,
        index: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Get array variable name
        let array_name = if let Expression::Ident(name) = object {
            name
        } else {
            return Err("Complex array expressions not yet supported".to_string());
        };

        // Get array pointer and type (copy to avoid borrow issues)
        let array_ptr = *self
            .variables
            .get(array_name)
            .ok_or_else(|| format!("Array {} not found", array_name))?;
        let array_type = *self
            .variable_types
            .get(array_name)
            .ok_or_else(|| format!("Type for array {} not found", array_name))?;

        // Get element type from array type
        let elem_type = if let BasicTypeEnum::ArrayType(arr_ty) = array_type {
            arr_ty.get_element_type()
        } else {
            return Err(format!("{} is not an array", array_name));
        };

        // Compile index expression
        let index_val = self.compile_expression(index)?;
        let index_int = if let BasicValueEnum::IntValue(iv) = index_val {
            iv
        } else {
            return Err("Index must be integer".to_string());
        };

        // GEP to get element pointer: array[i]
        // First index is 0 (dereference pointer), second is the array index
        let zero = self.context.i32_type().const_int(0, false);

        unsafe {
            let element_ptr = self
                .builder
                .build_in_bounds_gep(array_type, array_ptr, &[zero, index_int], "arrayidx")
                .map_err(|e| format!("Failed to build GEP: {}", e))?;

            // Load the element value
            self.builder
                .build_load(elem_type, element_ptr, "arrayelem")
                .map_err(|e| format!("Failed to load array element: {}", e))
        }
    }

    /// Compile F-string with interpolation
    /// Format: f"text {expr} more text {expr2}"
    pub(crate) fn compile_fstring(
        &mut self,
        template: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Parse the F-string template to extract text parts and expressions
        // For now, implement a simple version that handles {var_name} placeholders

        enum FStringPart {
            Text(String),
            Expr(String),
        }

        let mut result_parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Check if it's an escaped brace {{
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume second {
                    current_text.push('{');
                    continue;
                }

                // Save current text if any
                if !current_text.is_empty() {
                    result_parts.push(FStringPart::Text(current_text.clone()));
                    current_text.clear();
                }

                // Extract expression until }
                let mut expr_str = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    expr_str.push(ch);
                }

                result_parts.push(FStringPart::Expr(expr_str));
            } else if ch == '}' {
                // Check if it's an escaped brace }}
                if chars.peek() == Some(&'}') {
                    chars.next(); // consume second }
                    current_text.push('}');
                    continue;
                }
                current_text.push(ch);
            } else {
                current_text.push(ch);
            }
        }

        // Add remaining text
        if !current_text.is_empty() {
            result_parts.push(FStringPart::Text(current_text));
        }

        // For now, if there are no interpolations, just return as a regular string
        if result_parts
            .iter()
            .all(|p| matches!(p, FStringPart::Text(_)))
        {
            let full_text: String = result_parts
                .iter()
                .filter_map(|p| {
                    if let FStringPart::Text(s) = p {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            let global_str = self
                .builder
                .build_global_string_ptr(&full_text, "fstr")
                .map_err(|e| format!("Failed to create F-string: {}", e))?;
            return Ok(global_str.as_pointer_value().into());
        }

        // TODO: For now, F-strings with interpolation are not fully supported
        // We would need to:
        // 1. Parse each {expression} as a Vex expression
        // 2. Evaluate each expression
        // 3. Convert each result to string (call to_string methods or format functions)
        // 4. Concatenate all parts

        // For now, just return a placeholder string indicating interpolation is needed
        let placeholder = format!("f\"{}\" (interpolation not yet implemented)", template);
        let global_str = self
            .builder
            .build_global_string_ptr(&placeholder, "fstr_placeholder")
            .map_err(|e| format!("Failed to create F-string placeholder: {}", e))?;
        Ok(global_str.as_pointer_value().into())
    }

    /// Get pointer to a struct field for assignment
    pub(crate) fn get_field_pointer(
        &mut self,
        object: &Expression,
        field: &str,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        if let Expression::Ident(var_name) = object {
            // Check if this variable is tracked as a struct
            if let Some(struct_name) = self.variable_struct_names.get(var_name).cloned() {
                let var_ptr = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Variable {} not found", var_name))?;

                let ty = *self
                    .variable_types
                    .get(var_name)
                    .ok_or_else(|| format!("Type for variable {} not found", var_name))?;

                // Load the struct pointer
                let struct_ptr = self
                    .builder
                    .build_load(ty, var_ptr, &format!("{}_ptr", var_name))
                    .map_err(|e| format!("Failed to load struct pointer: {}", e))?;

                let struct_ptr_val = struct_ptr.into_pointer_value();

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

                // Rebuild struct type
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
                        &format!("{}.{}_ptr", var_name, field),
                    )
                    .map_err(|e| format!("Failed to get field pointer: {}", e))?;

                return Ok(field_ptr);
            }
        }

        Err(format!(
            "Cannot get pointer to field {} on non-struct value",
            field
        ))
    }

    /// Get pointer to an array element for assignment
    pub(crate) fn get_index_pointer(
        &mut self,
        object: &Expression,
        index: &Expression,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let array_name = if let Expression::Ident(name) = object {
            name
        } else {
            return Err("Complex array expressions not yet supported".to_string());
        };

        let array_ptr = *self
            .variables
            .get(array_name)
            .ok_or_else(|| format!("Array {} not found", array_name))?;
        let array_type = *self
            .variable_types
            .get(array_name)
            .ok_or_else(|| format!("Type for array {} not found", array_name))?;

        // Compile index expression
        let index_val = self.compile_expression(index)?;
        let index_int = if let BasicValueEnum::IntValue(iv) = index_val {
            iv
        } else {
            return Err("Index must be integer".to_string());
        };

        // GEP to get element pointer
        let zero = self.context.i32_type().const_int(0, false);

        unsafe {
            let element_ptr = self
                .builder
                .build_in_bounds_gep(array_type, array_ptr, &[zero, index_int], "arrayidx_ptr")
                .map_err(|e| format!("Failed to build GEP: {}", e))?;

            Ok(element_ptr)
        }
    }

    /// Get the struct type name of a field from an object expression
    fn get_field_struct_type(
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
                                    return Ok(Some(field_struct_name));
                                }
                            }
                            Type::Generic { name, type_args } => {
                                // Generic struct field - get mangled name
                                match self.instantiate_generic_struct(&name, &type_args) {
                                    Ok(mangled) => return Ok(Some(mangled)),
                                    Err(_) => return Ok(None),
                                }
                            }
                            _ => {}
                        }
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
                            _ => {}
                        }
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Compile field access on an already-loaded struct value
    fn compile_field_access_on_value(
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
            Type::Generic { name, type_args } => {
                // Generic struct (e.g., Box<i32>) - calculate mangled name
                // Use the same mangling strategy as instantiate_generic_struct
                let mangled_type_args: Vec<String> = type_args
                    .iter()
                    .map(|ty| match ty {
                        Type::Named(n) => n.clone(),
                        Type::I32 => "i32".to_string(),
                        Type::I64 => "i64".to_string(),
                        Type::F32 => "f32".to_string(),
                        Type::F64 => "f64".to_string(),
                        Type::Bool => "bool".to_string(),
                        Type::String => "string".to_string(),
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
            .build_load(field_llvm_type, field_ptr, &format!("{}_val", field))
            .map_err(|e| format!("Failed to load field: {}", e))
    }
}
