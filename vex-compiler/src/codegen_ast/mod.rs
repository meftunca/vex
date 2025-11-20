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
mod destructors; // Automatic destructors (RAII/Drop trait)
mod diagnostic_helpers; // Diagnostic helper methods for error reporting
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
            variable_concrete_types: HashMap::new(),
            type_constraints: Vec::new(),
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
            namespace_imports: HashMap::new(), // ⭐ NEW: Namespace import aliases
            module_constants: HashMap::new(),  // ⭐ NEW: Module constant registry
            builtins: BuiltinRegistry::new(),
            current_function: None,
            current_function_return_type: None,
            printf_fn: None,
            deferred_statements: Vec::new(),
            loop_context_stack: Vec::new(),
            closure_envs: HashMap::new(),
            closure_variables: HashMap::new(),
            closure_types: HashMap::new(),
            scope_stack: Vec::new(),
            last_compiled_tuple_type: None,
            last_compiled_array_ptr: None,
            current_method_is_mutable: false, // ⭐ NEW: Default to immutable
            diagnostics: DiagnosticEngine::new(), // Initialize diagnostic engine
            span_map,                         // ⭐ NEW: Store span map from parser
            trait_bounds_checker: None,       // ⭐ NEW: Initialized in compile_program
            source_file: source_file.to_string(), // ⭐ NEW: Store source file path
            type_interner: crate::types::interner::TypeInterner::new(), // ⭐ NEW: Type interning for performance
            global_runtime: None, // ⭐ ASYNC: Initialize runtime handle as None
            async_block_counter: 0, // ⭐ ASYNC BLOCKS: Counter for unique names
            async_state_stack: Vec::new(), // ⭐ ASYNC STATE MACHINE: State tracking
            async_state_counter: 0, // ⭐ ASYNC STATE MACHINE: State ID counter
            current_async_resume_fn: None, // ⭐ ASYNC STATE MACHINE: Resume function
            async_resume_blocks: Vec::new(), // ⭐ ASYNC STATE MACHINE: Pre-allocated resume blocks
            async_context: None,  // ⭐ ASYNC: Current async context
            suppress_diagnostics: false, // ⭐ NEW: Default to false
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

    /// ⭐ NEW: Generate comprehensive type suffix for method overloading
    /// Handles ALL type variants including primitives, generics, references, tuples, etc.
    /// Returns a mangled suffix that uniquely identifies the type for method dispatch
    pub(crate) fn generate_type_suffix(&self, ty: &Type) -> String {
        match ty {
            Type::Unknown => "_unknown".to_string(),

            // Primitive integer types
            Type::I8 => "_i8".to_string(),
            Type::I16 => "_i16".to_string(),
            Type::I32 => "_i32".to_string(),
            Type::I64 => "_i64".to_string(),
            Type::I128 => "_i128".to_string(),
            Type::U8 => "_u8".to_string(),
            Type::U16 => "_u16".to_string(),
            Type::U32 => "_u32".to_string(),
            Type::U64 => "_u64".to_string(),
            Type::U128 => "_u128".to_string(),

            // Primitive float types
            Type::F16 => "_f16".to_string(),
            Type::F32 => "_f32".to_string(),
            Type::F64 => "_f64".to_string(),

            // Other primitives
            Type::Bool => "_bool".to_string(),
            Type::String => "_String".to_string(),
            Type::Byte => "_byte".to_string(),

            // Named types (structs, enums, custom types)
            Type::Named(name) => format!("_{}", name),

            // Generic types: Vec<i32> -> _Vec_i32, Map<String,i32> -> _Map_String_i32
            Type::Generic { name, type_args } => {
                let mut suffix = format!("_{}", name);
                for arg in type_args {
                    suffix.push_str(&self.generate_type_suffix(arg));
                }
                suffix
            }

            // Reference types: &T -> _ref_T, &!T -> _refmut_T
            Type::Reference(inner, is_mutable) => {
                let mut suffix = if *is_mutable {
                    "_refmut".to_string()
                } else {
                    "_ref".to_string()
                };
                suffix.push_str(&self.generate_type_suffix(inner));
                suffix
            }

            // Array types: [i32; 5] -> _arr5_i32
            Type::Array(elem, size) => {
                format!("_arr{}{}", size, self.generate_type_suffix(elem))
            }

            // Const array: [T; N] -> _arrN_T
            Type::ConstArray {
                elem_type,
                size_param,
            } => {
                format!("_arr{}{}", size_param, self.generate_type_suffix(elem_type))
            }

            // Slice types: &[T] -> _slice_T, &![T] -> _slicemut_T
            Type::Slice(elem, is_mutable) => {
                let mut suffix = if *is_mutable {
                    "_slicemut".to_string()
                } else {
                    "_slice".to_string()
                };
                suffix.push_str(&self.generate_type_suffix(elem));
                suffix
            }

            // Tuple types: (i32, f64) -> _tuple_i32_f64
            Type::Tuple(elements) => {
                let mut suffix = "_tuple".to_string();
                for elem in elements {
                    suffix.push_str(&self.generate_type_suffix(elem));
                }
                suffix
            }

            // Function types: fn(i32, f64): bool -> _fn_i32_f64_ret_bool
            Type::Function {
                params,
                return_type,
            } => {
                let mut suffix = "_fn".to_string();
                for param in params {
                    suffix.push_str(&self.generate_type_suffix(param));
                }
                suffix.push_str("_ret");
                suffix.push_str(&self.generate_type_suffix(return_type));
                suffix
            }

            // Union types: (i32 | f64) -> _union_i32_f64
            Type::Union(types) => {
                let mut suffix = "_union".to_string();
                for ty in types {
                    suffix.push_str(&self.generate_type_suffix(ty));
                }
                suffix
            }

            // Intersection types: (Display & Clone) -> _inter_Display_Clone
            Type::Intersection(types) => {
                let mut suffix = "_inter".to_string();
                for ty in types {
                    suffix.push_str(&self.generate_type_suffix(ty));
                }
                suffix
            }

            // Raw pointer: *T -> _ptr_T, *const T -> _constptr_T
            Type::RawPtr { inner, is_const } => {
                let mut suffix = if *is_const {
                    "_constptr".to_string()
                } else {
                    "_ptr".to_string()
                };
                suffix.push_str(&self.generate_type_suffix(inner));
                suffix
            }

            // Builtin wrapper types
            Type::Option(inner) => format!("_Option{}", self.generate_type_suffix(inner)),
            Type::Result(ok, err) => format!(
                "_Result{}{}",
                self.generate_type_suffix(ok),
                self.generate_type_suffix(err)
            ),
            Type::Vec(elem) => format!("_Vec{}", self.generate_type_suffix(elem)),
            Type::Box(inner) => format!("_Box{}", self.generate_type_suffix(inner)),
            Type::Channel(inner) => format!("_Channel{}", self.generate_type_suffix(inner)),
            Type::Future(inner) => format!("_Future{}", self.generate_type_suffix(inner)),

            // Special types - use simple names
            Type::Unit => "_unit".to_string(),
            Type::Never => "_never".to_string(),
            Type::Any => "_any".to_string(),
            Type::SelfType => "_Self".to_string(),
            Type::Error => "_error".to_string(),
            Type::Nil => "_nil".to_string(),

            // Associated types: Self::Item -> _Self_Item
            Type::AssociatedType { self_type, name } => {
                format!("{}__{}", self.generate_type_suffix(self_type), name)
            }

            // Complex types - fallback to empty (skip in mangling)
            Type::Conditional { .. } => String::new(),
            Type::Infer(_) => String::new(),
            Type::Typeof(_) => String::new(),
        }
    }
}
