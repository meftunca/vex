// Compilation and code generation utilities for ASTCodeGen
// Handles object file generation, verification, and default values

use inkwell::types::BasicTypeEnum;
use inkwell::values::BasicValueEnum;
use inkwell::{
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};
use std::path::Path;

impl<'ctx> super::ASTCodeGen<'ctx> {
    /// Unified print/println handler - supports both format strings and variadic mode
    ///
    /// Dispatches to either:
    /// - compile_print_fmt() if first arg is string literal with "{}"
    /// - compile_print_variadic() otherwise (Go-style space-separated)
    pub fn compile_print_call(
        &mut self,
        func_name: &str,
        ast_args: &[vex_ast::Expression],
        compiled_args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        super::builtins::compile_print_call(self, func_name, ast_args, compiled_args)
    }

    /// Compile to object file
    pub fn compile_to_object(&self, output_path: &Path) -> Result<(), String> {
        self.compile_to_object_with_opt(output_path, OptimizationLevel::Default)
    }

    pub fn compile_to_object_with_opt(
        &self,
        output_path: &Path,
        opt_level: OptimizationLevel,
    ) -> Result<(), String> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| format!("Failed to initialize native target: {}", e))?;

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| format!("Failed to get target from triple: {}", e))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                opt_level,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine")?;

        target_machine
            .write_to_file(&self.module, FileType::Object, output_path)
            .map_err(|e| format!("Failed to write object file: {}", e))
    }

    pub fn verify_and_print(&self) -> Result<(), String> {
        if let Err(e) = self.module.verify() {
            return Err(format!("Module verification failed: {}", e));
        }

        println!(
            "\n🔍 Generated LLVM IR:\n{}",
            self.module.print_to_string().to_string()
        );
        Ok(())
    }

    /// Get default/zero value for a type
    pub(crate) fn get_default_value(&self, ty: &BasicTypeEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match ty {
            BasicTypeEnum::IntType(int_ty) => int_ty.const_zero().into(),
            BasicTypeEnum::FloatType(float_ty) => float_ty.const_zero().into(),
            BasicTypeEnum::PointerType(ptr_ty) => ptr_ty.const_null().into(),
            BasicTypeEnum::StructType(struct_ty) => struct_ty.const_zero().into(),
            BasicTypeEnum::ArrayType(array_ty) => array_ty.const_zero().into(),
            _ => self.context.i32_type().const_zero().into(),
        }
    }
}
