use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{PointerValue, BasicValueEnum};
use anyhow::Result;

pub struct SafetyConfig {
    pub null_checks: bool,
    pub bounds_checks: bool,
    pub overflow_checks: bool,
    pub alignment_checks: bool,
}

pub trait SafeLLVMBuilder<'ctx> {
    fn safe_alloca(&self, ty: BasicTypeEnum<'ctx>, name: &str, max_size: usize) -> Result<PointerValue<'ctx>>;
    fn safe_load(&self, ty: BasicTypeEnum<'ctx>, ptr: PointerValue<'ctx>, name: &str) -> Result<BasicValueEnum<'ctx>>;
    fn safe_gep(&self, ty: StructType<'ctx>, ptr: PointerValue<'ctx>, indices: &[u32]) -> Result<PointerValue<'ctx>>;
}

impl<'ctx> SafeLLVMBuilder<'ctx> for Builder<'ctx> {
    fn safe_alloca(&self, ty: BasicTypeEnum<'ctx>, name: &str, max_size: usize) -> Result<PointerValue<'ctx>> {
        let size_val = ty.size_of().ok_or_else(|| anyhow::anyhow!("Cannot get size of type"))?;
        let size_int = size_val.const_to_int();
        let size = size_int.get_zero_extended_constant().ok_or_else(|| anyhow::anyhow!("Cannot get constant value"))? as usize;
        if size > max_size {
            return Err(anyhow::anyhow!("Allocation too large: {} > {}", size, max_size));
        }

        let ptr = self.build_alloca(ty, name)?;

        // Emit LLVM metadata for alignment
        if let Some(inst) = ptr.as_instruction_value() {
            let align_val = ty.size_of().ok_or_else(|| anyhow::anyhow!("Cannot get size for alignment"))?;
            let align_int = align_val.const_to_int();
            let align = align_int.get_zero_extended_constant().ok_or_else(|| anyhow::anyhow!("Cannot get alignment constant"))? as u32;
            inst.set_alignment(align);
        }

        Ok(ptr)
    }

    fn safe_load(&self, ty: BasicTypeEnum<'ctx>, ptr: PointerValue<'ctx>, name: &str) -> Result<BasicValueEnum<'ctx>> {
        // In a full implementation, add null checks here
        Ok(self.build_load(ty, ptr, name)?)
    }

    fn safe_gep(&self, ty: StructType<'ctx>, ptr: PointerValue<'ctx>, indices: &[u32]) -> Result<PointerValue<'ctx>> {
        // In a full implementation, add bounds checks
        Ok(self.build_struct_gep(ty, ptr, indices[0], "gep")?)
    }
}