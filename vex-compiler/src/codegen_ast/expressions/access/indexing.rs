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
        // Get variable name
        let var_name = if let Expression::Ident(name) = object {
            name
        } else {
            return Err("Complex indexing expressions not yet supported".to_string());
        };

        // Check if this is a Map
        if let Some(struct_name) = self.variable_struct_names.get(var_name) {
            if struct_name == "Map" {
                // Map indexing: map[key] -> map.get(key)
                let map_ptr_var = *self
                    .variables
                    .get(var_name)
                    .ok_or_else(|| format!("Map {} not found", var_name))?;

                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let map_ptr = self
                    .builder
                    .build_load(ptr_type, map_ptr_var, &format!("{}_ptr", var_name))
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
        }

        // Array indexing
        let array_ptr = *self
            .variables
            .get(var_name)
            .ok_or_else(|| format!("Array {} not found", var_name))?;
        let array_type = *self
            .variable_types
            .get(var_name)
            .ok_or_else(|| format!("Type for array {} not found", var_name))?;

        // Get element type from array type
        let elem_type = if let BasicTypeEnum::ArrayType(arr_ty) = array_type {
            arr_ty.get_element_type()
        } else {
            return Err(format!("{} is not an array", var_name));
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
}
