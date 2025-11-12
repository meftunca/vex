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
        let array_type = match elem_type_llvm {
            BasicTypeEnum::IntType(it) => it.array_type(elements.len() as u32),
            BasicTypeEnum::FloatType(ft) => ft.array_type(elements.len() as u32),
            BasicTypeEnum::ArrayType(at) => at.array_type(elements.len() as u32),
            _ => return Err(format!("Unsupported array element type: {:?}", elem_type_llvm)),
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
            count_const as u32
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
                _ => return Err(format!("Unsupported array element type: {:?}", elem_type_llvm)),
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
            count_const as u32
        } else {
            return Err("Array repeat count must be a compile-time constant".to_string());
        };

        // Create array type
        let array_type = match elem_type {
            BasicTypeEnum::IntType(it) => it.array_type(count_u32),
            BasicTypeEnum::FloatType(ft) => ft.array_type(count_u32),
            BasicTypeEnum::ArrayType(at) => at.array_type(count_u32),
            _ => return Err(format!("Unsupported array element type for repeat: {:?}", elem_type)),
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
            // Even if type_args is empty, this might be a generic struct reference
            // Check if it's a known generic struct that needs instantiation
            if let Some(_ast_def) = self.struct_ast_defs.get(struct_name) {
                if !_ast_def.type_params.is_empty() {
                    // This is a generic struct but no type args provided
                    eprintln!(
                        "⚠️  Generic struct '{}' used without type args in literal",
                        struct_name
                    );
                    return Err(format!(
                        "Generic struct '{}' requires type arguments",
                        struct_name
                    ));
                }
            }
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

            // CRITICAL FIX: If field value is a StructLiteral with empty type_args
            // but field_ty is Generic, inject type_args recursively
            let adjusted_field_expr = if let Expression::StructLiteral {
                name: lit_name,
                type_args: ref lit_type_args,
                fields: ref lit_fields,
            } = &field_expr.1
            {
                if lit_type_args.is_empty() {
                    if let Type::Generic {
                        name: expected_name,
                        type_args: ref expected_args,
                    } = field_ty
                    {
                        if lit_name == expected_name {
                            eprintln!(
                                "  🔧 Recursively injecting type args into nested struct literal: {:?}",
                                expected_args
                            );
                            Expression::StructLiteral {
                                name: lit_name.clone(),
                                type_args: expected_args.clone(),
                                fields: lit_fields.clone(),
                            }
                        } else {
                            field_expr.1.clone()
                        }
                    } else {
                        field_expr.1.clone()
                    }
                } else {
                    field_expr.1.clone()
                }
            } else {
                field_expr.1.clone()
            };

            let field_val = self.compile_expression(&adjusted_field_expr)?;
            field_types.push(self.ast_type_to_llvm(field_ty));

            // CRITICAL FIX: If field is a struct type and field_val is a pointer,
            // we need to load the struct value (memcpy) instead of storing the pointer
            let actual_field_val = match field_ty {
                Type::Named(type_name) if self.struct_defs.contains_key(type_name) => {
                    // Field is a user-defined struct
                    if field_val.is_pointer_value() {
                        // Load the struct value from pointer
                        let field_type = self.ast_type_to_llvm(field_ty);
                        self.builder
                            .build_load(field_type, field_val.into_pointer_value(), "struct_val")
                            .map_err(|e| format!("Failed to load struct field value: {}", e))?
                    } else {
                        field_val
                    }
                }
                Type::Generic { .. } => {
                    // Generic struct - use mangled name
                    let mangled_name = self.type_to_string(field_ty);
                    if self.struct_defs.contains_key(&mangled_name) && field_val.is_pointer_value()
                    {
                        let field_type = self.ast_type_to_llvm(field_ty);
                        self.builder
                            .build_load(
                                field_type,
                                field_val.into_pointer_value(),
                                "generic_struct_val",
                            )
                            .map_err(|e| {
                                format!("Failed to load generic struct field value: {}", e)
                            })?
                    } else {
                        field_val
                    }
                }
                _ => field_val,
            };

            field_values.push(actual_field_val);
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

        // Save struct type for Let statement to read
        self.last_compiled_tuple_type = Some(tuple_struct_type);

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

    /// Compile builtin enum literals (Option, Result) - Phase 0.4
    pub(crate) fn compile_builtin_enum_literal(
        &mut self,
        enum_name: &str,
        variant: &str,
        data: &Vec<Expression>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        eprintln!(
            "📗 compile_builtin_enum_literal: {}::{}, data.len={}",
            enum_name,
            variant,
            data.len()
        );
        match enum_name {
            "Option" => {
                // Option<T> = { i32 tag, T value }
                // Some(x) = tag=0, None = tag=1
                let tag_value = if variant == "Some" { 0 } else { 1 };
                let tag = self.context.i32_type().const_int(tag_value, false);

                if variant == "None" {
                    // None: Create Option<i32> with tag=0 and zero value
                    // Default to i32 for now (proper type inference from context later)
                    let default_value_type = self.context.i32_type();
                    let zero_value = default_value_type.const_zero();

                    let option_struct_type = self.context.struct_type(
                        &[self.context.i32_type().into(), default_value_type.into()],
                        false,
                    );

                    let option_ptr = self
                        .builder
                        .build_alloca(option_struct_type, "option_none")
                        .map_err(|e| format!("Failed to allocate Option::None: {}", e))?;

                    // Store tag=0
                    let tag_ptr = self
                        .builder
                        .build_struct_gep(option_struct_type, option_ptr, 0, "tag_ptr")
                        .map_err(|e| format!("Failed to GEP tag: {}", e))?;
                    self.builder
                        .build_store(tag_ptr, tag)
                        .map_err(|e| format!("Failed to store tag: {}", e))?;

                    // Store zero value
                    let value_ptr = self
                        .builder
                        .build_struct_gep(option_struct_type, option_ptr, 1, "value_ptr")
                        .map_err(|e| format!("Failed to GEP value: {}", e))?;
                    self.builder
                        .build_store(value_ptr, zero_value)
                        .map_err(|e| format!("Failed to store zero value: {}", e))?;

                    // Load and return
                    let option_value = self
                        .builder
                        .build_load(option_struct_type, option_ptr, "option_none_value")
                        .map_err(|e| format!("Failed to load Option::None: {}", e))?;

                    return Ok(option_value);
                } else {
                    // Some(value): Compile value and create struct
                    let value_expr = data
                        .first()
                        .ok_or_else(|| "Some() requires a value argument".to_string())?;
                    let value = self.compile_expression(value_expr)?;
                    let value_type = value.get_type();

                    // Create Option<T> struct: { i32, T }
                    let option_struct_type = self
                        .context
                        .struct_type(&[self.context.i32_type().into(), value_type], false);

                    // Allocate on stack
                    let option_ptr = self
                        .builder
                        .build_alloca(option_struct_type, "option_some")
                        .map_err(|e| format!("Failed to allocate Option: {}", e))?;

                    // Store tag (field 0)
                    let tag_ptr = self
                        .builder
                        .build_struct_gep(option_struct_type, option_ptr, 0, "tag_ptr")
                        .map_err(|e| format!("Failed to GEP tag: {}", e))?;
                    self.builder
                        .build_store(tag_ptr, tag)
                        .map_err(|e| format!("Failed to store tag: {}", e))?;

                    // Store value (field 1)
                    let value_ptr = self
                        .builder
                        .build_struct_gep(option_struct_type, option_ptr, 1, "value_ptr")
                        .map_err(|e| format!("Failed to GEP value: {}", e))?;
                    self.builder
                        .build_store(value_ptr, value)
                        .map_err(|e| format!("Failed to store value: {}", e))?;

                    // Load and return struct value
                    let option_value = self
                        .builder
                        .build_load(option_struct_type, option_ptr, "option_value")
                        .map_err(|e| format!("Failed to load Option: {}", e))?;

                    eprintln!("📗 Returning Some(...) as struct: {:?}", option_value);
                    Ok(option_value)
                }
            }
            "Result" => {
                // Result<T, E> = { i32 tag, union { T, E } }
                // Err = tag=0, Ok = tag=1
                let tag_value = if variant == "Err" { 0 } else { 1 };
                let tag = self.context.i32_type().const_int(tag_value, false);

                // Compile data value
                let value_expr = data
                    .first()
                    .ok_or_else(|| format!("{}() requires a value argument", variant))?;
                let value = self.compile_expression(value_expr)?;
                let value_type = value.get_type();

                // Try to infer Result<T,E> type from current function's return type
                let result_struct_type = if let Some(func) = self.current_function {
                    if let Some(func_def) = self
                        .function_defs
                        .get(&func.get_name().to_str().unwrap().to_string())
                    {
                        if let Some(Type::Result(ok_ty, err_ty)) = &func_def.return_type {
                            // Use proper union type from ast_type_to_llvm
                            if let BasicTypeEnum::StructType(st) =
                                self.ast_type_to_llvm(&Type::Result(ok_ty.clone(), err_ty.clone()))
                            {
                                st
                            } else {
                                // Fallback: use value_type directly
                                self.context.struct_type(
                                    &[self.context.i32_type().into(), value_type],
                                    false,
                                )
                            }
                        } else {
                            // Not a Result return type, use value_type
                            self.context
                                .struct_type(&[self.context.i32_type().into(), value_type], false)
                        }
                    } else {
                        // Function def not found, use value_type
                        self.context
                            .struct_type(&[self.context.i32_type().into(), value_type], false)
                    }
                } else {
                    // Outside function context, use value_type
                    self.context
                        .struct_type(&[self.context.i32_type().into(), value_type], false)
                };

                // Allocate on stack
                let result_ptr = self
                    .builder
                    .build_alloca(
                        result_struct_type,
                        &format!("result_{}", variant.to_lowercase()),
                    )
                    .map_err(|e| format!("Failed to allocate Result: {}", e))?;

                // Store tag (field 0)
                let tag_ptr = self
                    .builder
                    .build_struct_gep(result_struct_type, result_ptr, 0, "tag_ptr")
                    .map_err(|e| format!("Failed to GEP tag: {}", e))?;
                self.builder
                    .build_store(tag_ptr, tag)
                    .map_err(|e| format!("Failed to store tag: {}", e))?;

                // Store value (field 1)
                let value_ptr = self
                    .builder
                    .build_struct_gep(result_struct_type, result_ptr, 1, "value_ptr")
                    .map_err(|e| format!("Failed to GEP value: {}", e))?;

                // Get union field type
                let union_field_type = result_struct_type
                    .get_field_type_at_index(1)
                    .ok_or_else(|| "Result struct missing value field".to_string())?;

                // Cast or store value based on type compatibility
                if value_type == union_field_type {
                    // Same type, direct store
                    self.builder
                        .build_store(value_ptr, value)
                        .map_err(|e| format!("Failed to store value: {}", e))?;
                } else {
                    // Different types: bitcast if needed (ptr types, sizes match)
                    // For now, try direct store (LLVM will error if incompatible)
                    self.builder
                        .build_store(value_ptr, value)
                        .map_err(|e| format!("Failed to store value (type mismatch): {}", e))?;
                }

                // Load and return struct value
                let result_value = self
                    .builder
                    .build_load(result_struct_type, result_ptr, "result_value")
                    .map_err(|e| format!("Failed to load Result: {}", e))?;

                Ok(result_value)
            }
            _ => Err(format!("Unknown builtin enum: {}", enum_name)),
        }
    }

    /// Compile map literal: {"key": value, "key2": value2}
    pub(crate) fn compile_map_literal(
        &mut self,
        entries: &[(Expression, Expression)],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Create a new Map
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let capacity = self.context.i64_type().const_int(
            if entries.is_empty() {
                8
            } else {
                entries.len() as u64 * 2
            },
            false,
        );

        let vex_map_create = self.declare_runtime_fn(
            "vex_map_create",
            &[self.context.i64_type().into()],
            ptr_type.into(),
        );

        let map_ptr = self
            .builder
            .build_call(vex_map_create, &[capacity.into()], "map_create")
            .map_err(|e| format!("Failed to create map: {}", e))?
            .try_as_basic_value()
            .left()
            .ok_or("map_create should return a value")?;

        // Insert each entry
        if !entries.is_empty() {
            let vex_map_insert = self.declare_runtime_fn(
                "vex_map_insert",
                &[ptr_type.into(), ptr_type.into(), ptr_type.into()],
                self.context.bool_type().into(),
            );

            for (key_expr, value_expr) in entries {
                let key = self.compile_expression(key_expr)?;
                let value = self.compile_expression(value_expr)?;

                self.builder
                    .build_call(
                        vex_map_insert,
                        &[map_ptr.into(), key.into(), value.into()],
                        "map_insert",
                    )
                    .map_err(|e| format!("Failed to insert map entry: {}", e))?;
            }
        }

        Ok(map_ptr)
    }
}
