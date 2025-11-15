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
        let array_size = crate::safe_array_size(elements.len())
            .map_err(|e| format!("Array literal too large: {}", e))?;
        let array_type = match elem_type {
            BasicTypeEnum::IntType(it) => it.array_type(array_size),
            BasicTypeEnum::FloatType(ft) => ft.array_type(array_size),
            BasicTypeEnum::ArrayType(at) => at.array_type(array_size),
            _ => return Err(format!("Unsupported array element type: {:?}", elem_type)),
        };

        // Allocate on stack
        let array_ptr = self
            .builder
            .build_alloca(array_type, "arrayliteral")
            .map_err(|e| format!("Failed to allocate array: {}", e))?;

        // Store elements
        self.store_array_elements(array_ptr, array_type, elements, 0)?;

        // Return the array value
        self.builder
            .build_load(array_type, array_ptr, "arrayval")
            .map_err(|e| format!("Failed to load array: {}", e))
    }

    /// Compile array literal directly into a pre-allocated buffer (optimization)
    pub(crate) fn compile_array_literal_into_buffer(
        &mut self,
        elements: &[Expression],
        elem_type_ast: &Type,
        dest_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> Result<(), String> {
        if elements.is_empty() {
            return Err("Empty array literals not supported".to_string());
        }

        // Get LLVM type
        let elem_type_llvm = self.ast_type_to_llvm(elem_type_ast);

        // Create array type
        let array_size = crate::safe_array_size(elements.len())
            .map_err(|e| format!("Array literal too large: {}", e))?;
        let array_type = match elem_type_llvm {
            BasicTypeEnum::IntType(it) => it.array_type(array_size),
            BasicTypeEnum::FloatType(ft) => ft.array_type(array_size),
            BasicTypeEnum::ArrayType(at) => at.array_type(array_size),
            _ => {
                return Err(format!(
                    "Unsupported array element type: {:?}",
                    elem_type_llvm
                ))
            }
        };

        // Store elements directly into dest_ptr
        self.store_array_elements(dest_ptr, array_type, elements, 0)?;

        Ok(())
    }

    /// Helper to store array elements into a pointer
    fn store_array_elements(
        &mut self,
        array_ptr: inkwell::values::PointerValue<'ctx>,
        array_type: inkwell::types::ArrayType<'ctx>,
        elements: &[Expression],
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

    /// Compile array repeat literal: [value; count]
    pub(crate) fn compile_array_repeat_literal(
        &mut self,
        value_expr: &Expression,
        count_expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile the value to determine element type
        let value_val = self.compile_expression(value_expr)?;
        let elem_type = value_val.get_type();

        self.compile_array_repeat_internal(value_val, elem_type, count_expr)
    }

    /// Compile array repeat with explicit element type from type annotation
    pub(crate) fn compile_array_repeat_with_type(
        &mut self,
        value_expr: &Expression,
        count_expr: &Expression,
        elem_type_ast: &Type,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Get LLVM type from AST type annotation
        let elem_type_llvm = self.ast_type_to_llvm(elem_type_ast);

        // Compile value and cast if necessary
        let value_val = self.compile_expression(value_expr)?;
        let value_type = value_val.get_type();

        // Cast value to match target element type
        let casted_value = if value_type != elem_type_llvm {
            match (value_val, elem_type_llvm) {
                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::IntType(target_int)) => {
                    if int_val.get_type().get_bit_width() < target_int.get_bit_width() {
                        self.builder
                            .build_int_s_extend(int_val, target_int, "elem_sext")
                            .map_err(|e| format!("Failed to extend: {}", e))?
                            .into()
                    } else if int_val.get_type().get_bit_width() > target_int.get_bit_width() {
                        self.builder
                            .build_int_truncate(int_val, target_int, "elem_trunc")
                            .map_err(|e| format!("Failed to truncate: {}", e))?
                            .into()
                    } else {
                        value_val
                    }
                }
                _ => value_val, // No cast needed or unsupported
            }
        } else {
            value_val
        };

        self.compile_array_repeat_internal(casted_value, elem_type_llvm, count_expr)
    }

    /// Compile array repeat directly into a pre-allocated buffer (optimization for large arrays)
    pub(crate) fn compile_array_repeat_into_buffer(
        &mut self,
        value_expr: &Expression,
        count_expr: &Expression,
        elem_type_ast: &Type,
        dest_ptr: inkwell::values::PointerValue<'ctx>,
    ) -> Result<(), String> {
        // Get LLVM type from AST type annotation
        let elem_type_llvm = self.ast_type_to_llvm(elem_type_ast);

        // Compile value and cast if necessary
        let value_val = self.compile_expression(value_expr)?;
        let value_type = value_val.get_type();

        // Cast value to match target element type
        let casted_value = if value_type != elem_type_llvm {
            match (value_val, elem_type_llvm) {
                (BasicValueEnum::IntValue(int_val), BasicTypeEnum::IntType(target_int)) => {
                    if int_val.get_type().get_bit_width() < target_int.get_bit_width() {
                        self.builder
                            .build_int_s_extend(int_val, target_int, "elem_sext")
                            .map_err(|e| format!("Failed to extend: {}", e))?
                            .into()
                    } else if int_val.get_type().get_bit_width() > target_int.get_bit_width() {
                        self.builder
                            .build_int_truncate(int_val, target_int, "elem_trunc")
                            .map_err(|e| format!("Failed to truncate: {}", e))?
                            .into()
                    } else {
                        value_val
                    }
                }
                _ => value_val,
            }
        } else {
            value_val
        };

        // Compile count
        let count_val = self.compile_expression(count_expr)?;
        let count_int = match count_val {
            BasicValueEnum::IntValue(iv) => iv,
            _ => return Err("Array repeat count must be an integer".to_string()),
        };

        let count_u32 = if let Some(count_const) = count_int.get_zero_extended_constant() {
            crate::safe_array_size(count_const as usize)
                .map_err(|e| format!("Array repeat count overflow (line 218): {}", e))?
        } else {
            return Err("Array repeat count must be a compile-time constant".to_string());
        };

        // Check if value is zero for memset optimization
        let is_zero_fill = if let BasicValueEnum::IntValue(int_val) = casted_value {
            int_val.is_null()
        } else if let BasicValueEnum::FloatValue(_) = casted_value {
            false
        } else {
            false
        };

        if is_zero_fill && count_u32 > 100 {
            // Use memset directly into dest_ptr
            let i8_type = self.context.i8_type();
            let size_type = self.context.i64_type();

            let elem_size = match elem_type_llvm {
                BasicTypeEnum::IntType(it) => it.size_of(),
                BasicTypeEnum::FloatType(ft) => ft.size_of(),
                _ => return Err("Unsupported type for memset".to_string()),
            };
            let count_val_sized = size_type.const_int(count_u32 as u64, false);
            let total_size = self
                .builder
                .build_int_mul(elem_size, count_val_sized, "total_size")
                .map_err(|e| format!("Failed to calculate size: {}", e))?;

            let memset_fn = self.get_or_declare_memset();
            let zero_byte = i8_type.const_int(0, false);
            let is_volatile = self.context.bool_type().const_int(0, false);

            self.builder
                .build_call(
                    memset_fn,
                    &[
                        dest_ptr.into(),
                        zero_byte.into(),
                        total_size.into(),
                        is_volatile.into(),
                    ],
                    "memset_direct",
                )
                .map_err(|e| format!("Failed to call memset: {}", e))?;
        } else {
            // Use loop to initialize dest_ptr directly
            let array_type = match elem_type_llvm {
                BasicTypeEnum::IntType(it) => it.array_type(count_u32),
                BasicTypeEnum::FloatType(ft) => ft.array_type(count_u32),
                BasicTypeEnum::ArrayType(at) => at.array_type(count_u32),
                _ => {
                    return Err(format!(
                        "Unsupported array element type: {:?}",
                        elem_type_llvm
                    ))
                }
            };

            let zero = self.context.i32_type().const_int(0, false);
            for i in 0..count_u32 {
                let index = self.context.i32_type().const_int(i as u64, false);

                unsafe {
                    let elem_ptr = self
                        .builder
                        .build_in_bounds_gep(
                            array_type,
                            dest_ptr,
                            &[zero, index],
                            &format!("elem{}", i),
                        )
                        .map_err(|e| format!("Failed to build GEP: {}", e))?;

                    self.builder
                        .build_store(elem_ptr, casted_value)
                        .map_err(|e| format!("Failed to store: {}", e))?;
                }
            }
        }

        Ok(())
    }

    /// Internal helper for array repeat compilation
    fn compile_array_repeat_internal(
        &mut self,
        value_val: BasicValueEnum<'ctx>,
        elem_type: BasicTypeEnum<'ctx>,
        count_expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile the count expression
        let count_val = self.compile_expression(count_expr)?;
        let count_int = match count_val {
            BasicValueEnum::IntValue(iv) => iv,
            _ => return Err("Array repeat count must be an integer".to_string()),
        };

        // Get the count as a constant if possible, otherwise use dynamic size
        let count_u32 = if let Some(count_const) = count_int.get_zero_extended_constant() {
            crate::safe_array_size(count_const as usize)
                .map_err(|e| format!("Array repeat count overflow (line 319): {}", e))?
        } else {
            return Err("Array repeat count must be a compile-time constant".to_string());
        };

        // Create array type
        let array_type = match elem_type {
            BasicTypeEnum::IntType(it) => it.array_type(count_u32),
            BasicTypeEnum::FloatType(ft) => ft.array_type(count_u32),
            BasicTypeEnum::ArrayType(at) => at.array_type(count_u32),
            _ => {
                return Err(format!(
                    "Unsupported array element type for repeat: {:?}",
                    elem_type
                ))
            }
        };

        // Allocate on stack
        let array_ptr = self
            .builder
            .build_alloca(array_type, "arrayrepeat")
            .map_err(|e| format!("Failed to allocate array: {}", e))?;

        // OPTIMIZATION: Use memset for large arrays filled with zeros
        // For [0; N] where N > 100, use memset instead of loop
        let is_zero_fill = if let BasicValueEnum::IntValue(int_val) = value_val {
            int_val.is_null()
        } else if let BasicValueEnum::FloatValue(_float_val) = value_val {
            // Check if float is 0.0 - conservative, keep loop for floats
            false
        } else {
            false
        };

        if is_zero_fill && count_u32 > 100 {
            // Use memset for efficient zero-initialization
            let i8_type = self.context.i8_type();
            let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
            let size_type = self.context.i64_type();

            // Cast array pointer to i8*
            let array_i8_ptr = self
                .builder
                .build_pointer_cast(array_ptr, i8_ptr_type, "array_i8_ptr")
                .map_err(|e| format!("Failed to cast pointer: {}", e))?;

            // Calculate total size in bytes
            let elem_size = match elem_type {
                BasicTypeEnum::IntType(it) => it.size_of(),
                BasicTypeEnum::FloatType(ft) => ft.size_of(),
                _ => return Err("Unsupported type for memset".to_string()),
            };
            let count_val = size_type.const_int(count_u32 as u64, false);
            let total_size = self
                .builder
                .build_int_mul(elem_size, count_val, "total_size")
                .map_err(|e| format!("Failed to calculate total size: {}", e))?;

            // Call memset(ptr, 0, size)
            let memset_fn = self.get_or_declare_memset();
            let zero_byte = i8_type.const_int(0, false);
            let is_volatile = self.context.bool_type().const_int(0, false);

            self.builder
                .build_call(
                    memset_fn,
                    &[
                        array_i8_ptr.into(),
                        zero_byte.into(),
                        total_size.into(),
                        is_volatile.into(),
                    ],
                    "memset_array",
                )
                .map_err(|e| format!("Failed to call memset: {}", e))?;
        } else {
            // Original loop for small arrays or non-zero values
            let zero = self.context.i32_type().const_int(0, false);
            for i in 0..count_u32 {
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
                        .build_store(elem_ptr, value_val)
                        .map_err(|e| format!("Failed to store element: {}", e))?;
                }
            }
        }

        // Return the array value
        // NOTE: For large arrays, LLVM optimizer will convert load/store to memcpy
        // We don't need to return pointer manually - let LLVM handle it
        self.builder
            .build_load(array_type, array_ptr, "arrayrepeatval")
            .map_err(|e| format!("Failed to load array: {}", e))
    }
}
