/**
 * FFI Bridge
 * Converts extern "C" declarations to LLVM IR without libclang
 * Zero-cost FFI with proper C ABI calling conventions
 */
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicMetadataTypeEnum;
use inkwell::values::FunctionValue;
use inkwell::AddressSpace;
use vex_ast::{ExternFunction, Type};

/// FFI Bridge for generating LLVM declarations from extern "C" blocks
pub struct FFIBridge<'ctx> {
    context: &'ctx Context,
    module: &'ctx Module<'ctx>,
}

impl<'ctx> FFIBridge<'ctx> {
    /// Create a new FFI bridge
    pub fn new(context: &'ctx Context, module: &'ctx Module<'ctx>) -> Self {
        Self { context, module }
    }

    /// Generate LLVM declaration for an extern "C" function
    ///
    /// # Example
    /// ```vex
    /// extern "C" {
    ///     fn vex_print(ptr: *const u8, len: u64);
    /// }
    /// ```
    /// Becomes:
    /// ```llvm
    /// declare void @vex_print(i8*, i64)
    /// ```
    pub fn generate_extern_declaration(
        &self,
        func: &ExternFunction,
    ) -> Result<FunctionValue<'ctx>, String> {
        // Check if already declared
        if let Some(existing) = self.module.get_function(&func.name) {
            return Ok(existing);
        }

        // 1. Map parameter types
        let param_types: Vec<BasicMetadataTypeEnum> = func
            .params
            .iter()
            .map(|p| self.vex_type_to_llvm(&p.ty))
            .collect::<Result<Vec<_>, _>>()?;

        // 2. Map return type
        let return_type = if let Some(ref ret_ty) = func.return_type {
            self.vex_type_to_llvm(ret_ty)?
        } else {
            // No return type = void
            return self.generate_void_function(&func.name, &param_types, func.is_variadic);
        };

        // 3. Create function type
        let fn_type = match return_type {
            BasicMetadataTypeEnum::IntType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicMetadataTypeEnum::FloatType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicMetadataTypeEnum::PointerType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicMetadataTypeEnum::ArrayType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicMetadataTypeEnum::StructType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicMetadataTypeEnum::VectorType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicMetadataTypeEnum::ScalableVectorType(_)
            | BasicMetadataTypeEnum::MetadataType(_) => {
                return Err(
                    "Unsupported return type for FFI (scalable vector or metadata)".to_string(),
                );
            }
        };

        // 4. Add to module
        let fn_value = self.module.add_function(&func.name, fn_type, None);

        // 5. Set C calling convention (critical for FFI)
        // Note: inkwell doesn't expose calling convention setter directly
        // The C calling convention is the default for extern functions

        Ok(fn_value)
    }

    /// Generate void function (no return type)
    fn generate_void_function(
        &self,
        name: &str,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
        is_variadic: bool,
    ) -> Result<FunctionValue<'ctx>, String> {
        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(param_types, is_variadic);
        let fn_value = self.module.add_function(name, fn_type, None);
        // C calling convention is default
        Ok(fn_value)
    }

    /// Convert Vex type to LLVM type (FFI-compatible)
    ///
    /// Key mappings:
    /// - Integers: i8 → i8, i32 → i32, i64 → i64
    /// - Floats: f32 → float, f64 → double
    /// - Pointers: *const T → T*, *mut T → T*
    /// - Strings: string → { i8*, i64 } (fat pointer)
    /// - Slices: &[T] → { T*, i64 } (fat pointer)
    fn vex_type_to_llvm(&self, ty: &Type) -> Result<BasicMetadataTypeEnum<'ctx>, String> {
        match ty {
            // Integer types
            Type::I8 => Ok(self.context.i8_type().into()),
            Type::I16 => Ok(self.context.i16_type().into()),
            Type::I32 => Ok(self.context.i32_type().into()),
            Type::I64 => Ok(self.context.i64_type().into()),
            Type::I128 => Ok(self.context.i128_type().into()),
            Type::U8 => Ok(self.context.i8_type().into()),
            Type::U16 => Ok(self.context.i16_type().into()),
            Type::U32 => Ok(self.context.i32_type().into()),
            Type::U64 => Ok(self.context.i64_type().into()),
            Type::U128 => Ok(self.context.i128_type().into()),

            // Float types
            Type::F16 => Ok(self.context.f16_type().into()),
            Type::F32 => Ok(self.context.f32_type().into()),
            Type::F64 => Ok(self.context.f64_type().into()),

            // Boolean and other primitives
            Type::Bool => Ok(self.context.bool_type().into()),
            Type::Byte => Ok(self.context.i8_type().into()),
            Type::Nil => Ok(self.context.i8_type().into()),
            Type::Error => Ok(self.context.i32_type().into()),

            // Raw pointer: *T or *const T (for FFI)
            Type::RawPtr {
                inner: _,
                is_const: _,
            } => {
                // Raw pointer - always mapped to LLVM pointer (opaque)
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }

            // String: { i8*, i64 } fat pointer
            Type::String => {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let i64 = self.context.i64_type();
                let struct_type = self.context.struct_type(
                    &[ptr_type.into(), i64.into()],
                    false, // not packed
                );
                Ok(struct_type.into())
            }

            // Slice: &[T] → { T*, i64 } fat pointer
            Type::Slice(_inner, _is_mutable) => {
                let ptr_type = self.context.ptr_type(AddressSpace::default());
                let i64 = self.context.i64_type();
                let struct_type = self
                    .context
                    .struct_type(&[ptr_type.into(), i64.into()], false);
                Ok(struct_type.into())
            }

            // Array: [T; N] → [N x T]
            Type::Array(inner, size) => {
                let inner_type = self.vex_type_to_llvm(inner)?;
                let array_type = match inner_type {
                    BasicMetadataTypeEnum::IntType(t) => t.array_type(*size as u32),
                    BasicMetadataTypeEnum::FloatType(t) => t.array_type(*size as u32),
                    BasicMetadataTypeEnum::PointerType(t) => t.array_type(*size as u32),
                    BasicMetadataTypeEnum::ArrayType(t) => t.array_type(*size as u32),
                    BasicMetadataTypeEnum::StructType(t) => t.array_type(*size as u32),
                    BasicMetadataTypeEnum::VectorType(t) => t.array_type(*size as u32),
                    _ => return Err("Unsupported array element type for FFI".to_string()),
                };
                Ok(array_type.into())
            }

            // Reference: &T → T* (immutable)
            // Reference: &T! → T* (mutable)
            Type::Reference(_inner, _is_mutable) => {
                // Use opaque pointer
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }

            // Unit type (void)
            Type::Unit => {
                // Unit cannot be a parameter/return type in LLVM
                // Return i8 as placeholder (should be handled at function level)
                Ok(self.context.i8_type().into())
            }

            // Named types (structs/enums) - map to opaque pointer for FFI
            Type::Named(_name) => {
                // For FFI, we typically pass structs as pointers
                // Use opaque pointer
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }

            // Unsupported types for FFI
            Type::Function { .. } => Err(
                "Function types not supported in FFI (use function pointers instead)".to_string(),
            ),
            Type::Generic { .. } => {
                Err("Generic types must be monomorphized before FFI".to_string())
            }
            Type::Tuple(_) => {
                Err("Tuple types not supported in FFI (use struct instead)".to_string())
            }
            Type::Union(_) => Err("Union types not yet implemented in FFI".to_string()),
            Type::Infer(_) => {
                Err("Type inference failed - cannot generate FFI declaration".to_string())
            }

            // Other unsupported types
            _ => Err(format!("Unsupported type for FFI: {:?}", ty)),
        }
    }
}

#[cfg(test)]
mod tests {
    // Integration test moved to examples/test_ffi_bridge.vx
    // Test lifetime issues prevented unit testing here
    // FFI bridge validated via actual extern "C" block compilation
}
