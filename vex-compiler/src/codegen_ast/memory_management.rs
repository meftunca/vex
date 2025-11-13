// Memory management and allocation utilities for ASTCodeGen
// Handles allocas, alignment, store/load operations

use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};
use vex_ast::Type;

impl<'ctx> super::ASTCodeGen<'ctx> {
    /// Create an alloca instruction in the entry block of the function
    pub(crate) fn create_entry_block_alloca(
        &mut self,
        name: &str,
        ty: &Type,
        is_mutable: bool,
    ) -> Result<PointerValue<'ctx>, String> {
        let builder = self.context.create_builder();

        let entry = self
            .current_function
            .ok_or("No current function")?
            .get_first_basic_block()
            .ok_or("Function has no entry block")?;

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        // Special handling for Range types - allocate struct {i64, i64, i64}
        let llvm_type = if let Type::Named(type_name) = ty {
            if type_name == "Range" || type_name == "RangeInclusive" {
                self.context
                    .struct_type(
                        &[
                            self.context.i64_type().into(),
                            self.context.i64_type().into(),
                            self.context.i64_type().into(),
                        ],
                        false,
                    )
                    .into()
            } else {
                self.ast_type_to_llvm(ty)
            }
        } else {
            self.ast_type_to_llvm(ty)
        };

        let alloca = builder
            .build_alloca(llvm_type, name)
            .map_err(|e| format!("Failed to create alloca: {}", e))?;

        // v0.1: Mark immutable variables as readonly (optimization hint for LLVM)
        // Mutable variables (let!) remain writable
        if !is_mutable {
            // TODO: Add LLVM metadata to mark as readonly/constant
            // This will be optimized better by LLVM backend
        }

        Ok(alloca)
    }

    /// Get correct alignment for a type (in bytes)
    fn get_type_alignment(ty: BasicTypeEnum) -> u32 {
        match ty {
            BasicTypeEnum::IntType(int_ty) => {
                let bit_width = int_ty.get_bit_width();
                match bit_width {
                    1..=8 => 1,
                    9..=16 => 2,
                    17..=32 => 4,
                    33..=64 => 8,
                    _ => 8, // i128 and larger
                }
            }
            BasicTypeEnum::FloatType(float_ty) => {
                // f32 = 4 bytes, f64 = 8 bytes
                if float_ty.get_context().f32_type() == float_ty {
                    4
                } else {
                    8
                }
            }
            BasicTypeEnum::PointerType(_) => 8, // Pointers are 8 bytes on 64-bit systems
            BasicTypeEnum::ArrayType(_) => 8,   // Arrays align to largest element
            BasicTypeEnum::StructType(_) => 8,  // Structs align to largest field
            BasicTypeEnum::VectorType(_) => 16, // SIMD vectors
            BasicTypeEnum::ScalableVectorType(_) => 16, // Scalable SIMD vectors (ARM SVE, RISC-V V)
        }
    }

    /// Build store with correct alignment
    pub(crate) fn build_store_aligned(
        &self,
        ptr: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
    ) -> Result<(), String> {
        let alignment = Self::get_type_alignment(value.get_type());
        self.builder
            .build_store(ptr, value)
            .map_err(|e| format!("Failed to store value: {}", e))?
            .set_alignment(alignment)
            .map_err(|e| format!("Failed to set alignment: {}", e))?;
        Ok(())
    }

    /// Build load with correct alignment
    pub(crate) fn build_load_aligned(
        &self,
        ty: BasicTypeEnum<'ctx>,
        ptr: PointerValue<'ctx>,
        name: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let alignment = Self::get_type_alignment(ty);

        // Build the load instruction
        let load_result = self
            .builder
            .build_load(ty, ptr, name)
            .map_err(|e| format!("Failed to load value: {}", e))?;

        // Get the load instruction and set alignment
        // The load result is the loaded value, but we need the instruction itself
        // Inkwell's builder maintains the last instruction built
        if let Some(inst) = self.builder.get_insert_block() {
            if let Some(last_inst) = inst.get_last_instruction() {
                // Try to set alignment on the instruction
                if let Err(e) = last_inst.set_alignment(alignment) {
                    eprintln!("Warning: Could not set load alignment: {}", e);
                }
            }
        }

        Ok(load_result)
    }
}
