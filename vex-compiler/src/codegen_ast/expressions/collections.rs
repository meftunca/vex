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
        value: &vex_ast::Expression,
        count: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile the value to repeat
        let elem_value = self.compile_expression(value)?;
        let elem_type = elem_value.get_type();

        // Evaluate count - must be a constant integer
        let count_val = self.compile_expression(count)?;

        // Extract constant count value
        let repeat_count = if let BasicValueEnum::IntValue(int_val) = count_val {
            if let Some(const_val) = int_val.get_zero_extended_constant() {
                const_val
            } else {
                return Err("Array repeat count must be a compile-time constant".to_string());
            }
        } else {
            return Err("Array repeat count must be an integer".to_string());
        };

        if repeat_count == 0 {
            return Err("Array repeat count must be greater than 0".to_string());
        }

        if repeat_count > 10000 {
            return Err("Array repeat count too large (max 10000)".to_string());
        }

        // Create Vec<T> and fill with repeated values
        let vec_new_fn = self.get_vex_vec_new();
        let vec_push_fn = self.get_vex_vec_push();

        // Create new Vec
        let elem_size = match elem_type {
            inkwell::types::BasicTypeEnum::IntType(it) => (it.get_bit_width() / 8) as u64,
            inkwell::types::BasicTypeEnum::FloatType(ft) => {
                if ft == self.context.f32_type() {
                    4
                } else {
                    8
                }
            }
            inkwell::types::BasicTypeEnum::PointerType(_) => 8,
            inkwell::types::BasicTypeEnum::StructType(_) => 8,
            _ => 4,
        };

        let elem_size_val = self.context.i64_type().const_int(elem_size, false);
        let capacity_val = self.context.i64_type().const_int(repeat_count, false);

        let call_site = self
            .builder
            .build_call(
                vec_new_fn,
                &[elem_size_val.into(), capacity_val.into()],
                "vec_repeat",
            )
            .map_err(|e| format!("Failed to call vex_vec_new: {}", e))?;

        let vec_ptr = call_site.try_as_basic_value().unwrap_basic();

        // Push element repeat_count times
        for _ in 0..repeat_count {
            // Cast element to void pointer
            let elem_ptr = if elem_value.is_pointer_value() {
                elem_value.into_pointer_value()
            } else {
                // Allocate temporary for non-pointer values
                let temp = self
                    .builder
                    .build_alloca(
                        elem_type.try_into().map_err(|_| {
                            format!("Failed to convert element type for array repeat")
                        })?,
                        "temp_elem",
                    )
                    .map_err(|e| format!("Failed to allocate temp: {}", e))?;
                self.builder
                    .build_store(temp, elem_value)
                    .map_err(|e| format!("Failed to store temp: {}", e))?;
                temp
            };

            let void_ptr = self
                .builder
                .build_pointer_cast(
                    elem_ptr,
                    self.context.ptr_type(inkwell::AddressSpace::default()),
                    "elem_void_ptr",
                )
                .map_err(|e| format!("Failed to cast element: {}", e))?;

            self.builder
                .build_call(
                    vec_push_fn,
                    &[vec_ptr.into(), void_ptr.into()],
                    "vec_push_repeat",
                )
                .map_err(|e| format!("Failed to call vex_vec_push: {}", e))?;
        }

        Ok(vec_ptr)
    }

    /// Compile map literal
    fn compile_map_literal(
        &mut self,
        entries: &[(vex_ast::Expression, vex_ast::Expression)],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Create new Map using vex_map_new()
        let map_new_fn = self.get_vex_map_new();
        let map_insert_fn = self.get_vex_map_insert();

        let call_site = self
            .builder
            .build_call(map_new_fn, &[], "map_literal")
            .map_err(|e| format!("Failed to call vex_map_new: {}", e))?;

        let map_ptr = call_site.try_as_basic_value().unwrap_basic();

        // Insert each key-value pair
        for (key_expr, value_expr) in entries {
            // Compile key (must be String)
            let key_val = self.compile_expression(key_expr)?;

            // Ensure key is a string pointer
            let key_ptr = if key_val.is_pointer_value() {
                key_val.into_pointer_value()
            } else {
                return Err("Map literal keys must be strings".to_string());
            };

            // Compile value
            let value_val = self.compile_expression(value_expr)?;

            // Cast value to void pointer
            let value_ptr = if value_val.is_pointer_value() {
                value_val.into_pointer_value()
            } else {
                // Allocate temporary for non-pointer values (i64, f64, etc.)
                let temp =
                    self.builder
                        .build_alloca(
                            value_val.get_type().try_into().map_err(|_| {
                                format!("Failed to convert value type for map literal")
                            })?,
                            "temp_value",
                        )
                        .map_err(|e| format!("Failed to allocate temp value: {}", e))?;

                self.builder
                    .build_store(temp, value_val)
                    .map_err(|e| format!("Failed to store temp value: {}", e))?;
                temp
            };

            let void_ptr = self
                .builder
                .build_pointer_cast(
                    value_ptr,
                    self.context.ptr_type(inkwell::AddressSpace::default()),
                    "value_void_ptr",
                )
                .map_err(|e| format!("Failed to cast value: {}", e))?;

            // Call vex_map_insert(map, key, value)
            self.builder
                .build_call(
                    map_insert_fn,
                    &[map_ptr.into(), key_ptr.into(), void_ptr.into()],
                    "map_insert_literal",
                )
                .map_err(|e| format!("Failed to call vex_map_insert: {}", e))?;
        }

        Ok(map_ptr)
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
                32 => iv.get_zero_extended_constant().ok_or_else(|| {
                    "Array repeat count must be a compile-time constant".to_string()
                })? as usize,
                64 => iv.get_zero_extended_constant().ok_or_else(|| {
                    "Array repeat count must be a compile-time constant".to_string()
                })? as usize,
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
