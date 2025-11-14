// Expression compilation - references and dereferencing
use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::Type;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile reference expressions (& and &!)
    pub(crate) fn compile_reference_dispatch(
        &mut self,
        is_mutable: bool,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Take a reference to an expression: &expr or &expr!
        // This creates a pointer to the value
        match expr {
            vex_ast::Expression::Ident(name) => {
                // Check if this is an array - if so, treat as slice reference
                let var_type = self
                    .variable_types
                    .get(name)
                    .ok_or_else(|| format!("Type for variable {} not found", name))?;

                // If the variable is an array, create a slice struct
                if let inkwell::types::BasicTypeEnum::ArrayType(arr_ty) = var_type {
                    // Create slice struct: { i8* data, i64 len, i64 elem_size }
                    let ptr_ty = self.context.ptr_type(inkwell::AddressSpace::default());
                    let size_ty = self.context.i64_type();
                    let slice_struct_ty = self.context.struct_type(
                        &[
                            ptr_ty.into(),  // void *data
                            size_ty.into(), // size_t len
                            size_ty.into(), // size_t elem_size
                        ],
                        false,
                    );

                    // Get array pointer
                    let arr_ptr = self
                        .variables
                        .get(name)
                        .ok_or_else(|| format!("Variable {} not found", name))?;

                    // Cast array to i8* for data field
                    let data_ptr = self
                        .builder
                        .build_pointer_cast(*arr_ptr, ptr_ty, "slice_data")
                        .map_err(|e| format!("Failed to cast array to slice data: {}", e))?;

                    // Get array length
                    let arr_len = arr_ty.len() as u64;
                    let len_val = size_ty.const_int(arr_len, false);

                    // Get element size from actual element type
                    let elem_ty = arr_ty.get_element_type();
                    // Calculate element size in bytes
                    let elem_size = match elem_ty {
                        inkwell::types::BasicTypeEnum::IntType(it) => {
                            (it.get_bit_width() / 8) as u64
                        }
                        inkwell::types::BasicTypeEnum::FloatType(ft) => {
                            if ft == self.context.f32_type() {
                                4
                            } else {
                                8
                            }
                        }
                        inkwell::types::BasicTypeEnum::PointerType(_) => 8, // 64-bit pointers
                        inkwell::types::BasicTypeEnum::StructType(_) => 8,  // Default struct size
                        _ => 4,                                             // Default
                    };
                    let elem_size_val = size_ty.const_int(elem_size, false);

                    // Build slice struct
                    let undef = slice_struct_ty.get_undef();
                    let with_data = self
                        .builder
                        .build_insert_value(undef, data_ptr, 0, "slice_with_data")
                        .map_err(|e| format!("Failed to insert data: {}", e))?;
                    let with_len = self
                        .builder
                        .build_insert_value(with_data, len_val, 1, "slice_with_len")
                        .map_err(|e| format!("Failed to insert len: {}", e))?;
                    let slice_val = self
                        .builder
                        .build_insert_value(with_len, elem_size_val, 2, "slice_complete")
                        .map_err(|e| format!("Failed to insert elem_size: {}", e))?;

                    // Return slice struct value directly
                    // Convert AggregateValueEnum to StructValue to BasicValueEnum
                    let struct_val: inkwell::values::StructValue = slice_val.into_struct_value();
                    return Ok(struct_val.into());
                }

                // For non-arrays (identifiers), return the pointer directly (don't load)
                let ptr = self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                Ok((*ptr).into())
            }
            _ => {
                // For other expressions, compile them, store in temporary, return pointer
                let value = self.compile_expression(expr)?;
                let value_type = value.get_type();
                let temp_name = if is_mutable {
                    "ref_temp_mut"
                } else {
                    "ref_temp"
                };
                let temp_ptr = self
                    .builder
                    .build_alloca(value_type, temp_name)
                    .map_err(|e| format!("Failed to allocate reference temporary: {}", e))?;
                self.builder
                    .build_store(temp_ptr, value)
                    .map_err(|e| format!("Failed to store reference temporary: {}", e))?;
                Ok(temp_ptr.into())
            }
        }
    }

    /// Compile dereference expressions (*ptr)
    pub(crate) fn compile_dereference_dispatch(
        &mut self,
        expr: &vex_ast::Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Dereference a pointer: *ptr
        // Try to infer the inner type from the expression
        match expr {
            vex_ast::Expression::Ident(name) => {
                // For identifiers, we can load using the stored LLVM type
                let ptr = self
                    .variables
                    .get(name)
                    .ok_or_else(|| format!("Variable {} not found", name))?;
                let var_type = self
                    .variable_types
                    .get(name)
                    .ok_or_else(|| format!("Type for variable {} not found", name))?;

                // Load the pointer value first (variables store the reference)
                let ptr_loaded = self
                    .builder
                    .build_load(*var_type, *ptr, &format!("{}_ptr", name))
                    .map_err(|e| format!("Failed to load pointer variable: {}", e))?;

                if !ptr_loaded.is_pointer_value() {
                    return Err(format!("Cannot dereference non-pointer variable {}", name));
                }

                // Now dereference the pointer
                // For references, the inner type is what we need to load
                // Since we don't track AST types, we'll use a heuristic:
                // Try common types (i32, i64, f64, bool)
                // TODO: Add proper AST type tracking for variables
                let deref_ptr = ptr_loaded.into_pointer_value();

                // Try to determine the pointee type from AST type tracking
                let inner_type = if let Some(ast_type) = self.variable_ast_types.get(name) {
                    // Check if it's a reference type
                    match ast_type {
                        Type::Reference(inner, _) => self.ast_type_to_llvm(inner),
                        Type::RawPtr { inner, .. } => self.ast_type_to_llvm(inner),
                        _ => {
                            // Variable is not a reference in AST, try to infer
                            self.context.i32_type().into()
                        }
                    }
                } else {
                    // No AST type info - default to i32
                    self.context.i32_type().into()
                };

                // build_load expects AnyTypeEnum, but inner_type is BasicTypeEnum
                // We need to use match to extract the correct type
                let loaded = match inner_type {
                    inkwell::types::BasicTypeEnum::IntType(it) => self
                        .builder
                        .build_load(it, deref_ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::FloatType(ft) => self
                        .builder
                        .build_load(ft, deref_ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::PointerType(pt) => self
                        .builder
                        .build_load(pt, deref_ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::StructType(st) => self
                        .builder
                        .build_load(st, deref_ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::ArrayType(at) => self
                        .builder
                        .build_load(at, deref_ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::VectorType(vt) => self
                        .builder
                        .build_load(vt, deref_ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::ScalableVectorType(svt) => self
                        .builder
                        .build_load(svt, deref_ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                };
                Ok(loaded)
            }
            _ => {
                // For other expressions, compile and dereference
                let ptr_value = self.compile_expression(expr)?;
                if !ptr_value.is_pointer_value() {
                    return Err("Cannot dereference non-pointer value".to_string());
                }
                let ptr = ptr_value.into_pointer_value();

                // Try to infer type from expression
                let inner_type = if let Ok(expr_type) = self.infer_expression_type(expr) {
                    match &expr_type {
                        Type::Reference(inner, _) => self.ast_type_to_llvm(inner),
                        Type::RawPtr { inner, .. } => self.ast_type_to_llvm(inner),
                        _ => self.context.i32_type().into(),
                    }
                } else {
                    self.context.i32_type().into()
                };

                // Use match to handle different BasicTypeEnum variants
                let loaded = match inner_type {
                    inkwell::types::BasicTypeEnum::IntType(it) => self
                        .builder
                        .build_load(it, ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::FloatType(ft) => self
                        .builder
                        .build_load(ft, ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::PointerType(pt) => self
                        .builder
                        .build_load(pt, ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::StructType(st) => self
                        .builder
                        .build_load(st, ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::ArrayType(at) => self
                        .builder
                        .build_load(at, ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::VectorType(vt) => self
                        .builder
                        .build_load(vt, ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                    inkwell::types::BasicTypeEnum::ScalableVectorType(svt) => self
                        .builder
                        .build_load(svt, ptr, "deref")
                        .map_err(|e| format!("Failed to dereference pointer: {}", e))?,
                };
                Ok(loaded)
            }
        }
    }
}
