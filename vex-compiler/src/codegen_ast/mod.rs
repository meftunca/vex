// Modular LLVM Code Generator for Vex
// Refactored from codegen_ast.rs for better maintainability

// Compiler limits
pub(crate) const MAX_GENERIC_DEPTH: usize = 64; // Maximum nesting depth for generic types (Rust uses 128)

use crate::diagnostics::{error_codes, DiagnosticEngine, Span};
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
// mod functions;
pub mod analysis;
pub mod enums;
mod functions;
pub mod generics;
pub mod methods;
pub mod program;
pub mod registry;
mod statements;
pub mod traits;
mod types; // functions/{declare,compile,asynchronous}.rs

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

    // Closure environment tracking
    // Maps closure function pointer to its environment pointer
    // Used to pass captured variables when closure is called
    pub(crate) closure_envs: HashMap<PointerValue<'ctx>, PointerValue<'ctx>>,

    // Track which variables hold closures (variable name -> (fn_ptr, env_ptr))
    pub(crate) closure_variables: HashMap<String, (PointerValue<'ctx>, PointerValue<'ctx>)>,

    // Scope tracking for automatic cleanup (Drop trait)
    // Stack of scopes, each scope contains variable names that need cleanup
    // Inner Vec<(var_name, type_name)> tracks variables that need drop calls
    pub(crate) scope_stack: Vec<Vec<(String, String)>>,

    // Tuple type tracking: when compile_tuple_literal is called, store struct type here
    // Let statement reads this to get tuple struct type without recompiling elements
    pub(crate) last_compiled_tuple_type: Option<inkwell::types::StructType<'ctx>>,

    // ⭐ NEW: Method mutability tracking
    // Tracks whether current method is mutable (has ! in signature)
    // Used to validate self! usage in method bodies
    pub(crate) current_method_is_mutable: bool,

    // Diagnostic engine for collecting errors, warnings, and info messages
    pub(crate) diagnostics: DiagnosticEngine,

    // ⭐ NEW: Span tracking for AST nodes (from parser)
    // Maps AST node addresses to their source locations
    pub(crate) span_map: vex_diagnostics::SpanMap,
}

impl<'ctx> ASTCodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        Self::new_with_span_map(context, module_name, vex_diagnostics::SpanMap::new())
    }

    pub fn new_with_span_map(
        context: &'ctx Context,
        module_name: &str,
        span_map: vex_diagnostics::SpanMap,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        let mut codegen = Self {
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
            closure_envs: HashMap::new(),
            closure_variables: HashMap::new(),
            scope_stack: Vec::new(),
            last_compiled_tuple_type: None,
            current_method_is_mutable: false, // ⭐ NEW: Default to immutable
            diagnostics: DiagnosticEngine::new(), // Initialize diagnostic engine
            span_map,                         // ⭐ NEW: Store span map from parser
        };

        // Register Phase 0 builtin types (Vec, Option, Result, Box)
        // Pre-declare external C runtime functions for zero-overhead linking
        builtins::register_builtin_types_phase0(&mut codegen);

        // Register stdlib runtime functions (logger, fs, time, testing)
        builtins::register_stdlib_runtime(&mut codegen);

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

    /// Push a new scope for automatic cleanup tracking
    pub(crate) fn push_scope(&mut self) {
        self.scope_stack.push(Vec::new());
    }

    /// Pop scope and emit cleanup calls for Vec/Box types
    pub(crate) fn pop_scope(&mut self) -> Result<(), String> {
        if let Some(scope_vars) = self.scope_stack.pop() {
            // Emit cleanup calls in reverse order (LIFO)
            for (var_name, type_name) in scope_vars.iter().rev() {
                match type_name.as_str() {
                    "Vec" => {
                        // Call vec_free(var)
                        if let Some(var_ptr) = self.variables.get(var_name) {
                            eprintln!("🧹 Auto-cleanup: vec_free({})", var_name);

                            let vec_opaque_type = self.context.opaque_struct_type("vex_vec_s");
                            let vec_ptr_type =
                                vec_opaque_type.ptr_type(inkwell::AddressSpace::default());

                            let vec_value = self
                                .builder
                                .build_load(vec_ptr_type, *var_ptr, "vec_cleanup_load")
                                .map_err(|e| format!("Failed to load vec for cleanup: {}", e))?;

                            let vec_free_fn = self.get_vex_vec_free();
                            self.builder
                                .build_call(vec_free_fn, &[vec_value.into()], "vec_auto_free")
                                .map_err(|e| format!("Failed to call vec_free: {}", e))?;
                        }
                    }
                    "Box" => {
                        // Call box_free(box_ptr) - load the box pointer and pass it
                        if let Some(var_ptr) = self.variables.get(var_name).copied() {
                            eprintln!("🧹 Auto-cleanup: box_free({})", var_name);

                            // Load the box pointer from the variable
                            let box_ptr_type = self
                                .context
                                .struct_type(
                                    &[
                                        self.context
                                            .i8_type()
                                            .ptr_type(inkwell::AddressSpace::default())
                                            .into(),
                                        self.context.i64_type().into(),
                                    ],
                                    false,
                                )
                                .ptr_type(inkwell::AddressSpace::default());

                            let box_value = self
                                .builder
                                .build_load(box_ptr_type, var_ptr, "box_cleanup_load")
                                .map_err(|e| format!("Failed to load box for cleanup: {}", e))?;

                            let box_free_fn = self.get_vex_box_free();
                            self.builder
                                .build_call(box_free_fn, &[box_value.into()], "box_auto_free")
                                .map_err(|e| format!("Failed to call box_free: {}", e))?;
                        }
                    }
                    "String" => {
                        // Call vex_string_free(vex_string_t*)
                        if let Some(var_ptr) = self.variables.get(var_name).copied() {
                            eprintln!("🧹 Auto-cleanup: vex_string_free({})", var_name);

                            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                            let string_value = self
                                .builder
                                .build_load(ptr_type, var_ptr, "string_cleanup_load")
                                .map_err(|e| format!("Failed to load string for cleanup: {}", e))?;

                            // Declare void vex_string_free(vex_string_t*)
                            let void_fn_type =
                                self.context.void_type().fn_type(&[ptr_type.into()], false);
                            let string_free_fn =
                                self.module
                                    .add_function("vex_string_free", void_fn_type, None);

                            self.builder
                                .build_call(
                                    string_free_fn,
                                    &[string_value.into()],
                                    "string_auto_free",
                                )
                                .map_err(|e| format!("Failed to call vex_string_free: {}", e))?;
                        }
                    }
                    "Map" | "Set" => {
                        // Call vex_map_free(map_ptr) / vex_set_free(set_ptr) - both use map backend
                        if let Some(var_ptr) = self.variables.get(var_name).copied() {
                            eprintln!("🧹 Auto-cleanup: vex_map_free({})", var_name);

                            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                            let map_value = self
                                .builder
                                .build_load(ptr_type, var_ptr, "map_cleanup_load")
                                .map_err(|e| {
                                    format!("Failed to load map/set for cleanup: {}", e)
                                })?;

                            // Declare void vex_map_free(vex_map_t*)
                            let void_fn_type =
                                self.context.void_type().fn_type(&[ptr_type.into()], false);
                            let map_free_fn =
                                self.module.add_function("vex_map_free", void_fn_type, None);

                            self.builder
                                .build_call(map_free_fn, &[map_value.into()], "map_auto_free")
                                .map_err(|e| format!("Failed to call vex_map_free: {}", e))?;
                        }
                    }
                    _ => {
                        // Other types don't need cleanup (yet)
                    }
                }
            }
        }
        Ok(())
    }

    /// Register a variable for automatic cleanup
    pub(crate) fn register_for_cleanup(&mut self, var_name: String, type_name: String) {
        if let Some(current_scope) = self.scope_stack.last_mut() {
            // Only register Vec and Box for now
            if type_name == "Vec" || type_name == "Box" {
                eprintln!("📝 Register for cleanup: {} ({})", var_name, type_name);
                current_scope.push((var_name, type_name));
            }
        }
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

        // v0.9: Mark immutable variables as readonly (optimization hint for LLVM)
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

    /// Unified print/println handler - supports both format strings and variadic mode
    ///
    /// Dispatches to either:
    /// - compile_print_fmt() if first arg is string literal with "{}"
    /// - compile_print_variadic() otherwise (Go-style space-separated)
    pub fn compile_print_call(
        &mut self,
        func_name: &str,
        ast_args: &[Expression],
        compiled_args: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        builtins::compile_print_call(self, func_name, ast_args, compiled_args)
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
            Expression::Call { span_id: _,  func, .. } => {
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
