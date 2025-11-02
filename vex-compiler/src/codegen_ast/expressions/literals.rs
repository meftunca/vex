// Literal expressions (arrays, structs, tuples)

use super::super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile array literal
    pub(crate) fn compile_array_literal(
        &mut self,
        elements: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if elements.is_empty() {
            return Err("Empty array literals not supported".to_string());
        }

        // Compile first element to determine type
        let first_val = self.compile_expression(&elements[0])?;
        let elem_type = first_val.get_type();

        // Create array type
        let array_type = if let BasicTypeEnum::IntType(it) = elem_type {
            it.array_type(elements.len() as u32)
        } else if let BasicTypeEnum::FloatType(ft) = elem_type {
            ft.array_type(elements.len() as u32)
        } else {
            return Err("Unsupported array element type".to_string());
        };

        // Allocate on stack
        let array_ptr = self
            .builder
            .build_alloca(array_type, "arrayliteral")
            .map_err(|e| format!("Failed to allocate array: {}", e))?;

        // Store each element
        let zero = self.context.i32_type().const_int(0, false);
        for (i, elem_expr) in elements.iter().enumerate() {
            let elem_val = self.compile_expression(elem_expr)?;
            let index = self.context.i32_type().const_int(i as u64, false);

            unsafe {
                let elem_ptr = self
                    .builder
                    .build_in_bounds_gep(
                        array_type,
                        array_ptr,
                        &[zero, index],
                        &format!("elem{}", i),
                    )
                    .map_err(|e| format!("Failed to build GEP: {}", e))?;

                self.builder
                    .build_store(elem_ptr, elem_val)
                    .map_err(|e| format!("Failed to store element: {}", e))?;
            }
        }

        // Return the array pointer as a value
        // Note: This returns the array, not a pointer to it
        self.builder
            .build_load(array_type, array_ptr, "arrayval")
            .map_err(|e| format!("Failed to load array: {}", e))
    }

    /// Compile struct literal: TypeName { field1: val1, field2: val2 } or Box<i32> { value: 10 }
    pub(crate) fn compile_struct_literal(
        &mut self,
        struct_name: &str,
        type_args: &[Type],
        fields: &[(String, Expression)],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Check if this is a generic struct instantiation
        let actual_struct_name = if !type_args.is_empty() {
            // Box<i32> -> Box_i32
            // Need to instantiate the generic struct first
            self.instantiate_generic_struct(struct_name, type_args)?
        } else {
            struct_name.to_string()
        };

        // Get struct definition from registry (clone to avoid borrow issues)
        let struct_def = self
            .struct_defs
            .get(&actual_struct_name)
            .cloned()
            .ok_or_else(|| format!("Struct '{}' not found in registry", actual_struct_name))?;

        // Build field types and values in the order defined in the struct
        let mut field_types = Vec::new();
        let mut field_values = Vec::new();

        for (field_name, field_ty) in &struct_def.fields {
            // Find the field value in the literal
            let field_expr = fields
                .iter()
                .find(|(name, _)| name == field_name)
                .ok_or_else(|| format!("Missing field '{}' in struct literal", field_name))?;

            let field_val = self.compile_expression(&field_expr.1)?;
            field_types.push(self.ast_type_to_llvm(field_ty));
            field_values.push(field_val);
        }

        // 2. Create struct type from registry definition
        let struct_type = self.context.struct_type(&field_types, false);

        // 3. Allocate struct on stack
        let struct_ptr = self
            .builder
            .build_alloca(struct_type, &format!("{}_literal", struct_name))
            .map_err(|e| format!("Failed to allocate struct: {}", e))?;

        // 4. Store each field
        for (i, field_val) in field_values.iter().enumerate() {
            let field_ptr = self
                .builder
                .build_struct_gep(struct_type, struct_ptr, i as u32, &format!("field_{}", i))
                .map_err(|e| format!("Failed to build struct GEP: {}", e))?;

            self.builder
                .build_store(field_ptr, *field_val)
                .map_err(|e| format!("Failed to store field: {}", e))?;
        }

        // 5. Return the struct POINTER (zero-copy semantics!)
        // We return the pointer, not the value - no copy!
        // The caller can use this pointer directly
        Ok(struct_ptr.into())
    }

    /// Compile tuple literal: (val1, val2, val3, ...)
    /// Tuples are implemented as anonymous structs with fields named field_0, field_1, etc.
    pub(crate) fn compile_tuple_literal(
        &mut self,
        elements: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if elements.is_empty() {
            // Empty tuple () - unit type
            // We'll represent it as a zero-sized struct (i8 as placeholder)
            let unit_type = self.context.struct_type(&[], false);
            let unit_ptr = self
                .builder
                .build_alloca(unit_type, "unit_tuple")
                .map_err(|e| format!("Failed to allocate unit tuple: {}", e))?;
            return Ok(unit_ptr.into());
        }

        // Compile all elements and collect their types
        let mut element_values = Vec::new();
        let mut element_types = Vec::new();

        for elem_expr in elements.iter() {
            let elem_val = self.compile_expression(elem_expr)?;
            element_types.push(elem_val.get_type());
            element_values.push(elem_val);
        }

        // Create anonymous struct type for the tuple
        let tuple_struct_type = self.context.struct_type(&element_types, false);

        // Allocate tuple on stack
        let tuple_ptr = self
            .builder
            .build_alloca(tuple_struct_type, "tuple_literal")
            .map_err(|e| format!("Failed to allocate tuple: {}", e))?;

        // Store each element
        for (i, elem_val) in element_values.iter().enumerate() {
            let field_ptr = self
                .builder
                .build_struct_gep(
                    tuple_struct_type,
                    tuple_ptr,
                    i as u32,
                    &format!("field_{}", i),
                )
                .map_err(|e| format!("Failed to build tuple GEP: {}", e))?;

            self.builder
                .build_store(field_ptr, *elem_val)
                .map_err(|e| format!("Failed to store tuple element: {}", e))?;
        }

        // Return the tuple pointer
        Ok(tuple_ptr.into())
    }
}
