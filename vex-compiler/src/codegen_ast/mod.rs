// Modular LLVM Code Generator for Vex
// Refactored from codegen_ast.rs for better maintainability

// Compiler limits
pub(crate) const MAX_GENERIC_DEPTH: usize = 64; // Maximum nesting depth for generic types (Rust uses 128)

use crate::diagnostics::DiagnosticEngine;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, StructType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;
use vex_ast::*;

// Sub-modules containing impl blocks for ASTCodeGen
mod associated_types; // Associated types resolution
pub mod builtins; // Now a directory module
mod compilation; // Compilation and code generation utilities
mod constants;
mod drop_trait; // Drop trait automatic cleanup (RAII)
mod expressions;
mod ffi;
mod ffi_bridge;
// mod functions;
pub mod analysis;
pub mod enums;
mod functions;
pub mod generics;
mod inline_optimizer;
pub mod memory_management; // Memory allocation and alignment utilities
pub mod metadata;
pub mod methods;
pub mod program;
pub mod registry;
mod scope_management; // Scope and cleanup management
mod statements;
mod string_conversion; // String conversion and formatting
mod struct_def;
pub mod traits;
mod type_analysis; // Type analysis and expression handling
mod types; // functions/{declare,compile,asynchronous}.rs // ASTCodeGen struct definition

pub use struct_def::*;

pub use builtins::BuiltinRegistry;
pub use inline_optimizer::{InlineOptimizer, OptimizationStats};

impl<'ctx> ASTCodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self::new_with_span_map(context, module_name, vex_diagnostics::SpanMap::new())
    }

    pub fn new_with_span_map(
        context: &'ctx Context,
        module_name: &str,
        span_map: vex_diagnostics::SpanMap,
    ) -> Self {
        Self::new_with_source_file(context, module_name, span_map, module_name)
    }

    pub fn new_with_source_file(
        context: &'ctx Context,
        module_name: &str,
        span_map: vex_diagnostics::SpanMap,
        source_file: &str,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        let mut codegen = Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            variable_types: HashMap::new(),
            variable_ast_types: HashMap::new(),
            variable_struct_names: HashMap::new(),
            variable_enum_names: HashMap::new(),
            tuple_variable_types: HashMap::new(),
            function_params: HashMap::new(),
            function_param_types: HashMap::new(),
            global_constants: HashMap::new(),
            global_constant_types: HashMap::new(),
            functions: HashMap::new(),
            function_defs: HashMap::new(),
            struct_ast_defs: HashMap::new(),
            struct_defs: HashMap::new(),
            enum_ast_defs: HashMap::new(),
            type_aliases: HashMap::new(),
            generic_instantiations: HashMap::new(),
            trait_defs: HashMap::new(),
            trait_impls: HashMap::new(),
            associated_type_bindings: HashMap::new(), // ⭐ NEW: Associated type tracking
            destructor_impls: HashMap::new(),         // ⭐ NEW: Destructor trait tracking
            policy_defs: HashMap::new(),
            struct_metadata: HashMap::new(),
            module_namespaces: HashMap::new(),
            builtins: BuiltinRegistry::new(),
            current_function: None,
            printf_fn: None,
            deferred_statements: Vec::new(),
            loop_context_stack: Vec::new(),
            closure_envs: HashMap::new(),
            closure_variables: HashMap::new(),
            scope_stack: Vec::new(),
            last_compiled_tuple_type: None,
            current_method_is_mutable: false, // ⭐ NEW: Default to immutable
            diagnostics: DiagnosticEngine::new(), // Initialize diagnostic engine
            span_map,                         // ⭐ NEW: Store span map from parser
            trait_bounds_checker: None,       // ⭐ NEW: Initialized in compile_program
            source_file: source_file.to_string(), // ⭐ NEW: Store source file path
        };

        // Register Phase 0 builtin types (Vec, Option, Result, Box)
        // Pre-declare external C runtime functions for zero-overhead linking
        builtins::register_builtin_types_phase0(&mut codegen);

        // Register stdlib runtime functions (logger, fs, time, testing)
        builtins::register_stdlib_runtime(&mut codegen);

        // ⭐ NEW: Register built-in destructor implementations
        codegen.register_builtin_destructors();

        codegen
    }

    /// Register a module namespace with its functions
    pub fn register_module_namespace(&mut self, module_name: String, functions: Vec<String>) {
        self.module_namespaces.insert(module_name, functions);
    }

    /// Get reference to diagnostic engine for printing/checking diagnostics
    pub fn diagnostics(&self) -> &DiagnosticEngine {
        &self.diagnostics
    }

    /// Check if there are any diagnostics collected
    pub fn has_diagnostics(&self) -> bool {
        self.diagnostics.has_diagnostics()
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Create FFI bridge for extern declarations
    pub fn create_ffi_bridge(&'ctx self) -> ffi_bridge::FFIBridge<'ctx> {
        ffi_bridge::FFIBridge::new(self.context, &self.module)
    }

    /// Create inline optimizer for zero-cost abstractions
    pub fn create_inline_optimizer(&'ctx self) -> inline_optimizer::InlineOptimizer<'ctx> {
        inline_optimizer::InlineOptimizer::new(&self.module)
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
}
