// Expression compilation - collections (arrays, maps, tuples)
use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile array literal expressions
    pub(crate) fn compile_array_dispatch(
        &mut self,
        elements: &[vex_ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_array_literal(elements)
    }

    /// Compile array repeat literal expressions
    pub(crate) fn compile_array_repeat_dispatch(
        &mut self,
        value: &vex_ast::Expression,
        count: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_array_repeat_literal(value, count)
    }

    /// Compile map literal expressions
    pub(crate) fn compile_map_dispatch(
        &mut self,
        entries: &[(vex_ast::Expression, vex_ast::Expression)],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_map_literal(entries)
    }

    /// Compile tuple literal expressions
    pub(crate) fn compile_tuple_dispatch(
        &mut self,
        elements: &[vex_ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        self.compile_tuple_literal(elements)
    }

    /// Compile array literal - proper array compilation
    fn compile_array_literal(
        &mut self,
        elements: &[vex_ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if elements.is_empty() {
            return Err("Empty array literals not supported".to_string());
        }

        // Compile first element to determine type
        let first_val = self.compile_expression(&elements[0])?;
        let elem_type = first_val.get_type();

        // Create array type
        let array_type = match elem_type {
            BasicTypeEnum::IntType(it) => it.array_type(elements.len() as u32),
            BasicTypeEnum::FloatType(ft) => ft.array_type(elements.len() as u32),
            BasicTypeEnum::ArrayType(at) => at.array_type(elements.len() as u32),
            _ => return Err(format!("Unsupported array element type: {:?}", elem_type)),
        };

        // Allocate on stack
        let array_ptr = self
            .builder
            .build_alloca(array_type, "arrayliteral")
            .map_err(|e| format!("Failed to allocate array: {}", e))?;

        // Store elements
        self.store_array_elements(array_ptr, array_type, elements, 0)?;

        // Return the array value (load from pointer)
        self.builder
            .build_load(array_type, array_ptr, "arrayval")
            .map_err(|e| format!("Failed to load array: {}", e))
    }

    /// Helper to store array elements into a pointer
    fn store_array_elements(
        &mut self,
        array_ptr: inkwell::values::PointerValue<'ctx>,
        array_type: inkwell::types::ArrayType<'ctx>,
        elements: &[vex_ast::Expression],
        start_index: usize,
    ) -> Result<(), String> {
        let zero = self.context.i32_type().const_int(0, false);
        for (i, elem_expr) in elements.iter().enumerate() {
            let elem_val = self.compile_expression(elem_expr)?;
            let index = self
                .context
                .i32_type()
                .const_int((start_index + i) as u64, false);

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
        Ok(())
    }

    /// Compile array repeat literal
    fn compile_array_repeat_literal(
        &mut self,
        _value: &vex_ast::Expression,
        _count: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // TODO: Implement array repeat literal compilation
        Err("Array repeat literals not yet implemented".to_string())
    }

    /// Compile map literal
    fn compile_map_literal(
        &mut self,
        _entries: &[(vex_ast::Expression, vex_ast::Expression)],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // TODO: Implement map literal compilation
        Err("Map literals not yet implemented".to_string())
    }

    /// Compile tuple literal
    fn compile_tuple_literal(
        &mut self,
        elements: &[vex_ast::Expression],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile each element
        let mut compiled_elements = Vec::new();
        for element in elements {
            let compiled = self.compile_expression(element)?;
            compiled_elements.push(compiled);
        }

        // Create a struct type for the tuple
        let element_types: Vec<_> = compiled_elements.iter().map(|v| v.get_type()).collect();

        let tuple_struct_type = self.context.struct_type(&element_types, false);

        // Allocate space for the tuple
        let tuple_ptr = self
            .builder
            .build_alloca(tuple_struct_type, "tuple")
            .map_err(|e| format!("Failed to allocate tuple: {}", e))?;

        // Store each element
        for (i, element_value) in compiled_elements.into_iter().enumerate() {
            let field_ptr = self
                .builder
                .build_struct_gep(
                    tuple_struct_type,
                    tuple_ptr,
                    i as u32,
                    &format!("tuple_field_{}", i),
                )
                .map_err(|e| format!("Failed to get tuple field pointer: {}", e))?;

            self.builder
                .build_store(field_ptr, element_value)
                .map_err(|e| format!("Failed to store tuple field: {}", e))?;
        }

        // Store the tuple type for later use (e.g., in let statements)
        self.last_compiled_tuple_type = Some(tuple_struct_type);

        // Return the tuple pointer as the result
        Ok(tuple_ptr.into())
    }

    /// Compile array literal directly into a pre-allocated buffer
    pub(crate) fn compile_array_literal_into_buffer(
        &mut self,
        elements: &[vex_ast::Expression],
        elem_type: &vex_ast::Type,
        buffer_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> Result<(), String> {
        let elem_llvm_type = self.ast_type_to_llvm(elem_type);

        for (i, element) in elements.iter().enumerate() {
            let compiled_element = self.compile_expression(element)?;

            // Get pointer to array element at index i
            let elem_ptr = unsafe {
                self.builder.build_gep(
                    elem_llvm_type,
                    buffer_ptr,
                    &[self.context.i32_type().const_int(i as u64, false)],
                    &format!("array_elem_{}", i),
                )
            }
            .map_err(|e| format!("Failed to get array element pointer: {}", e))?;

            self.builder
                .build_store(elem_ptr, compiled_element)
                .map_err(|e| format!("Failed to store array element: {}", e))?;
        }

        Ok(())
    }

    /// Compile array repeat literal directly into a pre-allocated buffer
    pub(crate) fn compile_array_repeat_into_buffer(
        &mut self,
        value_expr: &vex_ast::Expression,
        count_expr: &vex_ast::Expression,
        elem_type: &vex_ast::Type,
        buffer_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> Result<(), String> {
        let elem_llvm_type = self.ast_type_to_llvm(elem_type);

        // Compile the value and count
        let value = self.compile_expression(value_expr)?;
        let count = self.compile_expression(count_expr)?;

        // For now, assume count is a compile-time constant
        // TODO: Handle runtime count with a loop
        let count_const = match count {
            BasicValueEnum::IntValue(iv) => match iv.get_type().get_bit_width() {
                32 => iv.get_zero_extended_constant().unwrap_or(0) as usize,
                64 => iv.get_zero_extended_constant().unwrap_or(0) as usize,
                _ => return Err("Unsupported count type for array repeat".to_string()),
            },
            _ => return Err("Array repeat count must be a constant integer".to_string()),
        };

        // Store the value in each array element
        for i in 0..count_const {
            let elem_ptr = unsafe {
                self.builder.build_gep(
                    elem_llvm_type,
                    buffer_ptr,
                    &[self.context.i32_type().const_int(i as u64, false)],
                    &format!("array_elem_{}", i),
                )
            }
            .map_err(|e| format!("Failed to get array element pointer: {}", e))?;

            self.builder
                .build_store(elem_ptr, value)
                .map_err(|e| format!("Failed to store array element: {}", e))?;
        }

        Ok(())
    }
}
