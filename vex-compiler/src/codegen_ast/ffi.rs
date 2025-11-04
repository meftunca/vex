// FFI-specific codegen for extern blocks
use super::ASTCodeGen;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::FunctionValue;
use vex_ast::{ExternBlock, ExternFunction};

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile all extern functions in a block
    pub fn compile_extern_block(&mut self, block: &ExternBlock) -> Result<(), String> {
        // TODO: Check attributes for #[link(name = "...")] and #[cfg(...)]

        for func in &block.functions {
            self.declare_extern_function(&block.abi, func)?;
        }

        Ok(())
    }

    /// Declare a single extern function (zero overhead FFI)
    fn declare_extern_function(
        &mut self,
        _abi: &str, // ABI is mostly for documentation; LLVM handles calling conventions
        func: &ExternFunction,
    ) -> Result<FunctionValue<'ctx>, String> {
        // Check if already declared
        if let Some(existing) = self.module.get_function(&func.name) {
            return Ok(existing);
        }

        // Convert parameter types to LLVM types
        let mut param_types: Vec<BasicMetadataTypeEnum> = Vec::new();
        for param in &func.params {
            let llvm_ty = self.ast_type_to_llvm(&param.ty);
            param_types.push(llvm_ty.into());
        }

        // Convert return type
        let ret_type = if let Some(ref ty) = func.return_type {
            self.ast_type_to_llvm(ty)
        } else {
            // void return defaults to i32 in Vex
            BasicTypeEnum::IntType(self.context.i32_type())
        };

        // Create function type
        let fn_type = match ret_type {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, func.is_variadic),
            BasicTypeEnum::ScalableVectorType(t) => t.fn_type(&param_types, func.is_variadic),
        };

        // Add function to module with external linkage
        let fn_val = self.module.add_function(&func.name, fn_type, None);

        // Store in symbol table so Vex code can call it
        self.functions.insert(func.name.clone(), fn_val);

        Ok(fn_val)
    }
}
