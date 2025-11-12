// Array/slice indexing and pointer operations

use crate::codegen_ast::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile array/map indexing: arr[i] or map[key]
    pub(crate) fn compile_index(
        &mut self,
        object: &Expression,
        index: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Handle complex object expressions (e.g., self.field[index])
        let (base_ptr, var_type, struct_name_opt) = match object {
            Expression::Ident(name) => {
                // Simple variable: arr[i]
                let var_ptr = *self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                let var_ty = *self
                    .variable_types
                    .get(name)
                    .ok_or_else(|| format!("Type for {} not found", name))?;
                let struct_name = self.variable_struct_names.get(name).cloned();
                (var_ptr, var_ty, struct_name)
            }
            Expression::FieldAccess {
                object: inner_obj,
                field,
            } => {
                // Field access: self.field[index] or obj.field[index]
                let field_ptr = self.get_field_pointer(inner_obj, field)?;

                // Get the field type by examining the struct definition
                let struct_name_key = if let Expression::Ident(name) = inner_obj.as_ref() {
                    self.variable_struct_names.get(name).cloned()
                } else {
                    None
                };

                if let Some(struct_name) = struct_name_key {
                    if let Some(struct_def) = self.struct_defs.get(&struct_name) {
                        // Find field in struct definition to get its type
                        let field_ast_type = struct_def
                            .fields
                            .iter()
                            .find(|f| f.0 == *field)
                            .map(|f| &f.1)
                            .ok_or_else(|| {
                                format!("Field {} not found in struct {}", field, struct_name)
                            })?;

                        let field_llvm_type = self.ast_type_to_llvm(field_ast_type);

                        // Load the field value (it should be an array)
                        let field_val = self
                            .builder
                            .build_load(field_llvm_type, field_ptr, "field_load")
                            .map_err(|e| format!("Failed to load field: {}", e))?;

                        // Store in a temporary for indexing
                        let temp_alloca = self
                            .builder
                            .build_alloca(field_llvm_type, "temp_array")
                            .map_err(|e| format!("Failed to allocate temp: {}", e))?;
                        self.builder
                            .build_store(temp_alloca, field_val)
                            .map_err(|e| format!("Failed to store temp: {}", e))?;

                        (temp_alloca, field_llvm_type, None)
                    } else {
                        return Err(format!("Struct definition {} not found", struct_name));
                    }
                } else {
                    return Err("Cannot index field of non-struct object".to_string());
                }
            }
            _ => {
                return Err("Complex indexing expressions not yet fully supported".to_string());
            }
        };

        // Check if this is a Map or Vec
        if let Some(ref struct_name) = struct_name_opt {
            if struct_name == "Map" {
                // Map indexing: map[key] -> map.get(key)
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let map_ptr = self
                    .builder
                    .build_load(ptr_type, base_ptr, "map_ptr")
                    .map_err(|e| format!("Failed to load map pointer: {}", e))?;

                // Compile key expression
                let key = self.compile_expression(index)?;

                // Call vex_map_get(map, key)
                let vex_map_get = self.declare_runtime_fn(
                    "vex_map_get",
                    &[ptr_type.into(), ptr_type.into()],
                    ptr_type.into(),
                );

                return self
                    .builder
                    .build_call(vex_map_get, &[map_ptr.into(), key.into()], "map_get")
                    .map_err(|e| format!("Failed to call vex_map_get: {}", e))?
                    .try_as_basic_value()
                    .left()
                    .ok_or("map_get should return a value".to_string());
            }
            
            // Vec indexing: v[i] -> v.get(i) -> *vex_vec_get(v, i)
            if struct_name.starts_with("Vec") {
                // Get Vec pointer (already stored directly in variables)
                let vec_ptr = base_ptr;

                // Compile index expression
                let index_val = self.compile_expression(index)?;
                let index_i64 = if let BasicValueEnum::IntValue(iv) = index_val {
                    if iv.get_type().get_bit_width() < 64 {
                        self.builder
                            .build_int_z_extend(iv, self.context.i64_type(), "index_i64")
                            .map_err(|e| format!("Failed to extend index: {}", e))?
                    } else {
                        iv
                    }
                } else {
                    return Err("Vec index must be integer".to_string());
                };

                // Call vex_vec_get(vec, index) -> void*
                let get_fn = self.get_vex_vec_get();
                let elem_ptr = self
                    .builder
                    .build_call(get_fn, &[vec_ptr.into(), index_i64.into()], "vec_get")
                    .map_err(|e| format!("Failed to call vex_vec_get: {}", e))?
                    .try_as_basic_value()
                    .left()
                    .ok_or("vex_vec_get should return pointer".to_string())?;

                // Cast void* to element type pointer and load
                // Extract element type from Vec<T> - struct_name is like "Vec_i32"
                let elem_type_str = struct_name.strip_prefix("Vec_").unwrap_or("i32");
                let elem_llvm_type: BasicTypeEnum = match elem_type_str {
                    "i32" => self.context.i32_type().into(),
                    "i64" => self.context.i64_type().into(),
                    "f32" => self.context.f32_type().into(),
                    "f64" => self.context.f64_type().into(),
                    "bool" => self.context.bool_type().into(),
                    _ => self.context.i32_type().into(), // Default fallback
                };

                let typed_ptr = self
                    .builder
                    .build_pointer_cast(
                        elem_ptr.into_pointer_value(),
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                        "typed_ptr",
                    )
                    .map_err(|e| format!("Failed to cast element pointer: {}", e))?;

                // Load the element value
                return self
                    .builder
                    .build_load(elem_llvm_type, typed_ptr, "vec_elem")
                    .map_err(|e| format!("Failed to load vec element: {}", e));
            }
        }

        // Check if this is string indexing (v0.1.2) - only for simple variables
        if let Expression::Ident(var_name) = object {
            if let BasicTypeEnum::PointerType(_) = var_type {
                // Could be string - check if it's actually a string variable
                // For now, assume pointer + integer index = string indexing
                // Better: track string type explicitly in type system

                // Check if index is a Range expression (string slicing)
                if let Expression::Range { start, end } = index {
                    return self.compile_string_slice(var_name, start.as_ref(), end.as_ref());
                }

                // Single index - could be string byte access
                // Try string indexing first, fall back to array if it fails
                if let Ok(string_byte) = self.try_compile_string_index(var_name, index) {
                    return Ok(string_byte);
                }
            }
        }

        // Array indexing
        // Get element type from array type
        let elem_type = if let BasicTypeEnum::ArrayType(arr_ty) = var_type {
            arr_ty.get_element_type()
        } else {
            return Err("Indexing target is not an array".to_string());
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
                .build_in_bounds_gep(var_type, base_ptr, &[zero, index_int], "arrayidx")
                .map_err(|e| format!("Failed to build GEP: {}", e))?;

            // Load the element value
            self.builder
                .build_load(elem_type, element_ptr, "arrayelem")
                .map_err(|e| format!("Failed to load array element: {}", e))
        }
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

    /// Get pointer to an array element or map entry for assignment
    pub(crate) fn get_index_pointer(
        &mut self,
        object: &Expression,
        index: &Expression,
    ) -> Result<inkwell::values::PointerValue<'ctx>, String> {
        let var_name = if let Expression::Ident(name) = object {
            name
        } else {
            return Err("Complex indexing expressions not yet supported".to_string());
        };

        // Check if this is a Map - for Map, we don't return pointer, we handle in assignment
        if let Some(struct_name) = self.variable_struct_names.get(var_name) {
            if struct_name == "Map" {
                return Err(
                    "Map indexing assignment should use map.insert() - handled specially"
                        .to_string(),
                );
            }
        }

        let array_ptr = *self
            .variables
            .get(var_name)
            .ok_or_else(|| format!("Array {} not found", var_name))?;
        let array_type = *self
            .variable_types
            .get(var_name)
            .ok_or_else(|| format!("Type for array {} not found", var_name))?;

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

    /// Try to compile string indexing: text[3] -> vex_string_get_byte(text, 3)
    /// Returns Ok if successful, Err if not a string
    fn try_compile_string_index(
        &mut self,
        var_name: &str,
        index: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Get string pointer
        let string_ptr_var = *self
            .variables
            .get(var_name)
            .ok_or_else(|| format!("Variable {} not found", var_name))?;

        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let string_ptr = self
            .builder
            .build_load(ptr_type, string_ptr_var, &format!("{}_ptr", var_name))
            .map_err(|e| format!("Failed to load string pointer: {}", e))?;

        // Compile index
        let index_val = self.compile_expression(index)?;
        let index_int = if let BasicValueEnum::IntValue(iv) = index_val {
            // Convert to i64 for runtime function
            if iv.get_type().get_bit_width() < 64 {
                self.builder
                    .build_int_z_extend(iv, self.context.i64_type(), "index_i64")
                    .map_err(|e| format!("Failed to extend index: {}", e))?
            } else {
                iv
            }
        } else {
            return Err("Index must be integer".to_string());
        };

        // Call vex_string_index(str, index) -> u8
        let vex_string_index = self.declare_runtime_fn(
            "vex_string_index",
            &[ptr_type.into(), self.context.i64_type().into()],
            self.context.i8_type().into(),
        );

        let result = self
            .builder
            .build_call(
                vex_string_index,
                &[string_ptr.into(), index_int.into()],
                "string_byte",
            )
            .map_err(|e| format!("Failed to call vex_string_index: {}", e))?
            .try_as_basic_value()
            .left()
            .ok_or("vex_string_index should return a value".to_string())?;

        Ok(result)
    }

    /// Compile string slicing: text[0..5] -> vex_string_substr(text, 0, 5)
    fn compile_string_slice(
        &mut self,
        var_name: &str,
        start: Option<&Box<Expression>>,
        end: Option<&Box<Expression>>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Get string pointer
        let string_ptr_var = *self
            .variables
            .get(var_name)
            .ok_or_else(|| format!("Variable {} not found", var_name))?;

        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let string_ptr = self
            .builder
            .build_load(ptr_type, string_ptr_var, &format!("{}_ptr", var_name))
            .map_err(|e| format!("Failed to load string pointer: {}", e))?;

        // Compile start index (default: 0)
        let start_val = if let Some(start_expr) = start {
            let val = self.compile_expression(start_expr.as_ref())?;
            if let BasicValueEnum::IntValue(iv) = val {
                if iv.get_type().get_bit_width() < 64 {
                    self.builder
                        .build_int_z_extend(iv, self.context.i64_type(), "start_i64")
                        .map_err(|e| format!("Failed to extend start: {}", e))?
                } else {
                    iv
                }
            } else {
                return Err("Slice start must be integer".to_string());
            }
        } else {
            // No start means [..end] -> start from 0
            self.context.i64_type().const_int(0, false)
        };

        // Compile end index (default: -1 meaning "to end")
        let end_val = if let Some(end_expr) = end {
            let val = self.compile_expression(end_expr.as_ref())?;
            if let BasicValueEnum::IntValue(iv) = val {
                if iv.get_type().get_bit_width() < 64 {
                    self.builder
                        .build_int_z_extend(iv, self.context.i64_type(), "end_i64")
                        .map_err(|e| format!("Failed to extend end: {}", e))?
                } else {
                    iv
                }
            } else {
                return Err("Slice end must be integer".to_string());
            }
        } else {
            // No end means [start..] -> use -1 (sentinel for "to end")
            self.context.i64_type().const_int((-1i64) as u64, true)
        };

        // Call vex_string_substr(str, start, end) -> char*
        let vex_string_substr = self.declare_runtime_fn(
            "vex_string_substr",
            &[
                ptr_type.into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
            ],
            ptr_type.into(),
        );

        let result = self
            .builder
            .build_call(
                vex_string_substr,
                &[string_ptr.into(), start_val.into(), end_val.into()],
                "string_slice",
            )
            .map_err(|e| format!("Failed to call vex_string_substr: {}", e))?
            .try_as_basic_value()
            .left()
            .ok_or("vex_string_substr should return a value".to_string())?;

        Ok(result)
    }
}
