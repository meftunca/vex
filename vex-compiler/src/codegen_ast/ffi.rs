// FFI-specific codegen for extern blocks
use super::ASTCodeGen;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::FunctionValue;
use vex_ast::{ExternBlock, ExternFunction};

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile all extern types and functions in a block
    pub fn compile_extern_block(&mut self, block: &ExternBlock) -> Result<(), String> {
        // Register extern types (opaque or aliased)
        for extern_type in &block.types {
            self.register_extern_type(extern_type)?;
        }

        // Declare extern functions
        for func in &block.functions {
            let fn_val = self.declare_extern_function(&block.abi, func)?;
            // Add to function registry (even if already in LLVM module)
            self.functions.insert(func.name.clone(), fn_val);
        }
        Ok(())
    }

    /// Register an extern type (opaque or type alias)
    fn register_extern_type(&mut self, extern_type: &vex_ast::ExternType) -> Result<(), String> {
        // For now, we treat all extern types as opaque pointers
        // Even if they have an alias (type VexDuration = i64), we ignore it
        // because they're usually opaque C structs

        // The actual type resolution happens in ast_type_to_llvm when we see the type used
        // We just need to make sure the parser accepts them

        eprintln!("📋 Registered extern type: {} (opaque)", extern_type.name);
        Ok(())
    }

    /// Declare a single extern function (zero overhead FFI)
    fn declare_extern_function(
        &mut self,
        _abi: &str, // ABI is mostly for documentation; LLVM handles calling conventions
        func: &ExternFunction,
    ) -> Result<FunctionValue<'ctx>, String> {
        // Check if already declared in LLVM module
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
            // No return type = void in LLVM
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
