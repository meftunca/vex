// Modular LLVM Code Generator for Vex
// Refactored from codegen_ast.rs for better maintainability

// Compiler limits
pub(crate) const MAX_GENERIC_DEPTH: usize = 64; // Maximum nesting depth for generic types (Rust uses 128)

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
pub mod builtins; // Now a directory module
mod expressions;
mod ffi;
mod functions;
mod statements;
mod types;

pub use builtins::BuiltinRegistry;

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
    pub(crate) variable_enum_names: HashMap<String, String>, // Track enum variable names
    // Track tuple variables separately to know their struct types for pattern matching
    pub(crate) tuple_variable_types: HashMap<String, StructType<'ctx>>,
    // Track function pointer parameters (stored as values, not allocas)
    pub(crate) function_params: HashMap<String, PointerValue<'ctx>>,
    pub(crate) function_param_types: HashMap<String, Type>,
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

    // Defer statement stack (LIFO order)
    pub(crate) deferred_statements: Vec<Statement>,

    // Loop context stack for break/continue
    // Stack of (loop_body_block, loop_merge_block) - last entry is current loop
    pub(crate) loop_context_stack: Vec<(
        inkwell::basic_block::BasicBlock<'ctx>,
        inkwell::basic_block::BasicBlock<'ctx>,
    )>,
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
            variable_enum_names: HashMap::new(),
            tuple_variable_types: HashMap::new(),
            function_params: HashMap::new(),
            function_param_types: HashMap::new(),
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
            deferred_statements: Vec::new(),
            loop_context_stack: Vec::new(),
        }
    }

    /// Register a module namespace with its functions
    pub fn register_module_namespace(&mut self, module_name: String, functions: Vec<String>) {
        self.module_namespaces.insert(module_name, functions);
    }

    /// Execute deferred statements in LIFO order
    /// Called before function exits (return, panic, or end of function)
    /// Note: Does NOT clear the stack - use clear_deferred_statements() at function boundary
    pub(crate) fn execute_deferred_statements(&mut self) -> Result<(), String> {
        // Execute in reverse order (LIFO - Last In First Out)
        // Clone the statements to avoid borrow checker issues
        let statements: Vec<Statement> = self.deferred_statements.iter().rev().cloned().collect();
        for stmt in statements {
            self.compile_statement(&stmt)?;
        }
        Ok(())
    }

    /// Clear deferred statements (called at function boundary)
    pub(crate) fn clear_deferred_statements(&mut self) {
        self.deferred_statements.clear();
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

    /// Get struct name from an expression
    /// Returns the struct type name if the expression evaluates to a struct
    pub(crate) fn get_expression_struct_name(
        &mut self,
        expr: &Expression,
    ) -> Result<Option<String>, String> {
        match expr {
            // Variable: look up in variable_struct_names
            Expression::Ident(var_name) => Ok(self.variable_struct_names.get(var_name).cloned()),
            // Struct literal: directly has name
            Expression::StructLiteral { name, .. } => Ok(Some(name.clone())),
            // Field access: recursively get object's struct, then lookup field type
            Expression::FieldAccess { object, field } => {
                if let Some(object_struct_name) = self.get_expression_struct_name(object)? {
                    // Look up struct definition to get field type
                    // Clone field_type to avoid borrow issues
                    let field_type_opt =
                        self.struct_defs
                            .get(&object_struct_name)
                            .and_then(|struct_def| {
                                struct_def
                                    .fields
                                    .iter()
                                    .find(|(f, _)| f == field)
                                    .map(|(_, t)| t.clone())
                            });

                    if let Some(field_type) = field_type_opt {
                        // Check if field type is a struct
                        match field_type {
                            Type::Named(field_struct_name) => {
                                if self.struct_defs.contains_key(&field_struct_name) {
                                    Ok(Some(field_struct_name))
                                } else {
                                    Ok(None)
                                }
                            }
                            Type::Generic { name, type_args } => {
                                // Generic struct field like Box<i32>
                                // Return the mangled name
                                match self.instantiate_generic_struct(&name, &type_args) {
                                    Ok(mangled_name) => Ok(Some(mangled_name)),
                                    Err(_) => Ok(None),
                                }
                            }
                            _ => Ok(None),
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            // Function call: look up return type in function_defs
            Expression::Call { func, .. } => {
                if let Expression::Ident(func_name) = func.as_ref() {
                    // Clone return_type to avoid borrow issues
                    let return_type_opt = self
                        .function_defs
                        .get(func_name)
                        .and_then(|func_def| func_def.return_type.clone());

                    if let Some(return_type) = return_type_opt {
                        match return_type {
                            Type::Named(struct_name) => {
                                if self.struct_defs.contains_key(&struct_name) {
                                    Ok(Some(struct_name))
                                } else {
                                    Ok(None)
                                }
                            }
                            Type::Generic { name, type_args } => {
                                match self.instantiate_generic_struct(&name, &type_args) {
                                    Ok(mangled_name) => Ok(Some(mangled_name)),
                                    Err(_) => Ok(None),
                                }
                            }
                            _ => Ok(None),
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            // Other expressions don't return structs
            _ => Ok(None),
        }
    }

    /// Calculate nesting depth of a generic type
    /// Example: Box<Box<Box<i32>>> = depth 3
    pub(crate) fn get_generic_depth(&self, ty: &Type) -> usize {
        match ty {
            Type::Generic { type_args, .. } => {
                // Get max depth from all type arguments, add 1 for current level
                let max_arg_depth = type_args
                    .iter()
                    .map(|arg| self.get_generic_depth(arg))
                    .max()
                    .unwrap_or(0);
                1 + max_arg_depth
            }
            Type::Reference(inner, _) => self.get_generic_depth(inner),
            Type::Array(elem, _) => self.get_generic_depth(elem),
            _ => 0,
        }
    }
}
