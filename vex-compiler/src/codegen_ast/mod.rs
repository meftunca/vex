// Modular LLVM Code Generator for Vex
// Refactored from codegen_ast.rs for better maintainability

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::Path;
use vex_ast::*;

// Sub-modules containing impl blocks for ASTCodeGen
mod builtins;
mod expressions;
mod ffi;
mod functions;
mod statements;
mod types;

use builtins::BuiltinRegistry;

/// Struct definition metadata
#[derive(Debug, Clone)]
pub struct StructDef {
    pub fields: Vec<(String, Type)>, // Field name and type
}

pub struct ASTCodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,

    // Symbol tables
    pub(crate) variables: HashMap<String, PointerValue<'ctx>>,
    pub(crate) variable_types: HashMap<String, BasicTypeEnum<'ctx>>,
    pub(crate) variable_struct_names: HashMap<String, String>,
    // Track tuple variables separately to know their struct types for pattern matching
    pub(crate) tuple_variable_types: HashMap<String, StructType<'ctx>>,
    pub(crate) functions: HashMap<String, FunctionValue<'ctx>>,
    pub(crate) function_defs: HashMap<String, Function>,
    pub(crate) struct_ast_defs: HashMap<String, Struct>,
    pub(crate) struct_defs: HashMap<String, StructDef>,
    pub(crate) enum_ast_defs: HashMap<String, Enum>,
    pub(crate) type_aliases: HashMap<String, Type>,
    pub(crate) generic_instantiations: HashMap<(String, Vec<String>), String>,

    // Trait definitions: trait_name -> Trait
    pub(crate) trait_defs: HashMap<String, Trait>,
    // Trait implementations: (trait_name, type_name) -> Vec<Function>
    pub(crate) trait_impls: HashMap<(String, String), Vec<Function>>,

    // Module namespace tracking
    // Maps module names to their imported functions: "io" -> ["print", "println"]
    pub(crate) module_namespaces: HashMap<String, Vec<String>>,

    // Builtin functions registry
    pub(crate) builtins: BuiltinRegistry<'ctx>,

    pub(crate) current_function: Option<FunctionValue<'ctx>>,
    pub(crate) printf_fn: Option<FunctionValue<'ctx>>,
}

impl<'ctx> ASTCodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            variable_types: HashMap::new(),
            variable_struct_names: HashMap::new(),
            tuple_variable_types: HashMap::new(),
            functions: HashMap::new(),
            function_defs: HashMap::new(),
            struct_ast_defs: HashMap::new(),
            struct_defs: HashMap::new(),
            enum_ast_defs: HashMap::new(),
            type_aliases: HashMap::new(),
            generic_instantiations: HashMap::new(),
            trait_defs: HashMap::new(),
            trait_impls: HashMap::new(),
            module_namespaces: HashMap::new(),
            builtins: BuiltinRegistry::new(),
            current_function: None,
            printf_fn: None,
        }
    }

    /// Register a module namespace with its functions
    pub fn register_module_namespace(&mut self, module_name: String, functions: Vec<String>) {
        self.module_namespaces.insert(module_name, functions);
    }

    /// Declare printf for output
    pub(crate) fn declare_printf(&mut self) -> FunctionValue<'ctx> {
        if let Some(printf) = self.printf_fn {
            return printf;
        }

        let i8_ptr_type = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let printf_type = self.context.i32_type().fn_type(&[i8_ptr_type.into()], true);
        let printf = self.module.add_function("printf", printf_type, None);
        self.printf_fn = Some(printf);
        printf
    }

    /// Generate a call to printf
    pub(crate) fn build_printf(
        &mut self,
        format: &str,
        args: &[BasicValueEnum<'ctx>],
    ) -> Result<(), String> {
        let printf = self.declare_printf();

        // Create format string global
        let format_str = self
            .builder
            .build_global_string_ptr(format, "fmt")
            .map_err(|e| format!("Failed to create format string: {}", e))?;

        // Build arguments: [format_ptr, ...args]
        let mut printf_args: Vec<BasicMetadataValueEnum> =
            vec![format_str.as_pointer_value().into()];
        for arg in args {
            printf_args.push((*arg).into());
        }

        // Call printf
        self.builder
            .build_call(printf, &printf_args, "printf_call")
            .map_err(|e| format!("Failed to call printf: {}", e))?;

        Ok(())
    }

    /// Create an alloca instruction in the entry block of the function
    pub(crate) fn create_entry_block_alloca(
        &self,
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

        let llvm_type = self.ast_type_to_llvm(ty);
        let alloca = builder
            .build_alloca(llvm_type, name)
            .map_err(|e| format!("Failed to create alloca: {}", e))?;

        // v0.9: Mark immutable variables as readonly (optimization hint for LLVM)
        // Mutable variables (let!) remain writable
        if !is_mutable {
            // TODO: Add LLVM metadata to mark as readonly/constant
            // This will be optimized better by LLVM backend
        }

        Ok(alloca)
    }

    /// Compile to object file
    pub fn compile_to_object(&self, output_path: &Path) -> Result<(), String> {
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
                OptimizationLevel::Default,
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
