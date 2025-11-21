// ASTCodeGen struct definition
// Separated from mod.rs for better organization

// ASTCodeGen struct definition
// Separated from mod.rs for better organization

use super::functions::asynchronous::AsyncContext;
use super::*;

/// Struct definition metadata
#[derive(Debug, Clone)]
pub struct StructDef {
    pub fields: Vec<(String, Type)>, // Field name and type
}

/// Type constraint for type inference unification
/// Collected during first compilation pass, resolved in unification phase
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    /// Two types must be equal: T = i32
    Equal(Type, Type),

    /// Method receiver type constraint
    /// receiver_expr (e.g., "v") must have type that supports method with arg_types
    MethodReceiver {
        receiver_name: String,
        method_name: String,
        arg_types: Vec<Type>,
        inferred_receiver_type: Type,
    },

    /// Assignment constraint: variable = expression
    /// Variable type must match expression type
    Assignment { var_name: String, expr_type: Type },
}

pub struct ASTCodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,

    // Symbol tables
    pub(crate) variables: HashMap<String, PointerValue<'ctx>>,
    pub(crate) variable_types: HashMap<String, BasicTypeEnum<'ctx>>,
    pub(crate) variable_ast_types: HashMap<String, Type>, // Track AST types for correct print() formatting
    pub(crate) variable_struct_names: HashMap<String, String>,
    pub(crate) variable_enum_names: HashMap<String, String>, // Track enum variable names

    // ⭐ Phase 1: Type Inference - Variable type tracking with full AST types
    // Maps variable name → concrete AST Type (including Generic with type_args)
    // Used for method call receiver type resolution and generic instantiation
    pub variable_concrete_types: HashMap<String, Type>,

    // ⭐ Phase 1: Type Inference - Type constraint collection
    // Collects constraints during first pass, resolved in unification phase
    pub type_constraints: Vec<TypeConstraint>,

    // ⭐ GENERIC INSTANTIATION: Active type substitutions during generic function compilation
    // Maps type parameter names to their concrete types during instantiation
    // Example: {"T" → Type::I32} when compiling min_i32 from min<T>
    // CRITICAL: Used by infer_expression_type() to resolve generic type parameters
    pub active_type_substitutions: HashMap<String, Type>,
    // Track tuple variables separately to know their struct types for pattern matching
    pub(crate) tuple_variable_types: HashMap<String, StructType<'ctx>>,
    // Track function pointer parameters (stored as values, not allocas)
    pub(crate) function_params: HashMap<String, PointerValue<'ctx>>,
    pub(crate) function_param_types: HashMap<String, Type>,
    // Global constants (never cleared during function compilation)
    pub(crate) global_constants: HashMap<String, PointerValue<'ctx>>,
    pub(crate) global_constant_types: HashMap<String, BasicTypeEnum<'ctx>>,
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

    // ⭐ NEW: Associated type bindings for trait implementations
    // Maps (type_name, assoc_type_name) -> concrete_type
    // Example: ("Counter", "Item") -> Type::I32
    pub(crate) associated_type_bindings: HashMap<(String, String), Type>,

    // ⭐ NEW: Destructor trait tracking
    // Types that implement Destructor trait and need cleanup at scope exit
    // Maps type_name -> cleanup_function_name (e.g., "Vec" -> "vec_free")
    pub(crate) destructor_impls: HashMap<String, String>,

    // Policy definitions: policy_name -> Policy
    pub(crate) policy_defs: HashMap<String, vex_ast::Policy>,

    // Struct metadata: struct_name -> (field_name -> metadata_map)
    pub(crate) struct_metadata: HashMap<String, HashMap<String, HashMap<String, String>>>,

    // Module namespace tracking
    // Maps module names to their imported functions: "io" -> ["print", "println"]
    pub(crate) module_namespaces: HashMap<String, Vec<String>>,

    // ⭐ NEW: Namespace import aliases: alias -> module_name
    // Example: import * as math from "math" → namespace_imports["math"] = "math"
    pub(crate) namespace_imports: HashMap<String, String>,

    // ⭐ NEW: Module constants registry
    // Stores compiled constant values from imported modules
    // Key format: "module_name::CONST_NAME" or direct "CONST_NAME"
    pub(crate) module_constants: HashMap<String, BasicValueEnum<'ctx>>,

    // ⭐ NEW: Module constant type tracking
    // Stores AST types for module constants for proper type inference
    // Used by infer_expression_type() for namespace access (math.PI)
    pub(crate) module_constant_types: HashMap<String, Type>,

    // Builtin functions registry
    pub(crate) builtins: BuiltinRegistry<'ctx>,

    pub(crate) current_function: Option<FunctionValue<'ctx>>,
    pub(crate) current_function_return_type: Option<Type>,
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

    // ⭐ NEW: Track closure types for proper type inference
    // Maps variable name -> (param_types, return_type)
    // Used when calling closures to know the correct signature
    pub(crate) closure_types: HashMap<String, (Vec<Type>, Type)>,

    // Scope tracking for automatic cleanup (Drop trait)
    // Stack of scopes, each scope contains variable names that need cleanup
    // Inner Vec<(var_name, type_name)> tracks variables that need drop calls
    pub(crate) scope_stack: Vec<Vec<(String, String)>>,

    // Tuple type tracking: when compile_tuple_literal is called, store struct type here
    // Let statement reads this to get tuple struct type without recompiling elements
    pub(crate) last_compiled_tuple_type: Option<inkwell::types::StructType<'ctx>>,

    // ⭐ Array pointer tracking: when compile_array_literal is called, store pointer here
    // Cast/reference operations can use the pointer instead of re-allocating
    pub(crate) last_compiled_array_ptr: Option<inkwell::values::PointerValue<'ctx>>,

    // ⭐ NEW: Method mutability tracking
    // Tracks whether current method is mutable (has ! in signature)
    // Used to validate self! usage in method bodies
    pub(crate) current_method_is_mutable: bool,

    // ⭐ NEW: Unsafe context tracking
    // Tracks whether current code is inside unsafe{} block
    // Used to downgrade downcast errors to warnings
    pub(crate) is_in_unsafe_block: bool,

    // Diagnostic engine for collecting errors, warnings, and info messages
    pub(crate) diagnostics: DiagnosticEngine,

    // ⭐ NEW: Span tracking for AST nodes (from parser)
    // Maps AST node addresses to their source locations
    pub(crate) span_map: vex_diagnostics::SpanMap,

    // ⭐ NEW: Trait bounds checker for generic constraints
    pub(crate) trait_bounds_checker: Option<crate::trait_bounds_checker::TraitBoundsChecker>,

    // ⭐ NEW: Source file path for resolving relative imports
    pub(crate) source_file: String,

    // ⭐ NEW: Type interning for performance optimization
    // Reduces memory usage and clone overhead for common types
    pub(crate) type_interner: crate::types::interner::TypeInterner,

    // ⭐ ASYNC/AWAIT: Global runtime handle for spawning async tasks
    // Initialized in main() when async functions exist
    pub(crate) global_runtime: Option<PointerValue<'ctx>>,

    // ⭐ ASYNC BLOCKS: Counter for generating unique async block function names
    pub(crate) async_block_counter: u32,

    // ⭐ ASYNC STATE MACHINE: State tracking for await points
    // Stack of (state_struct_ptr, state_field_ptr, next_state_id)
    pub(crate) async_state_stack: Vec<(PointerValue<'ctx>, PointerValue<'ctx>, u32)>,

    // ⭐ ASYNC STATE MACHINE: Counter for generating unique state IDs
    pub(crate) async_state_counter: u32,

    // ⭐ ASYNC STATE MACHINE: Current async function's resume function
    pub(crate) current_async_resume_fn: Option<FunctionValue<'ctx>>,

    // ⭐ ASYNC STATE MACHINE: Pre-allocated resume blocks for await points
    // Maps state_id -> BasicBlock for resume continuation
    pub(crate) async_resume_blocks: Vec<inkwell::basic_block::BasicBlock<'ctx>>,

    // ⭐ ASYNC: Current async context for await compilation
    pub(crate) async_context: Option<AsyncContext>,

    // ⭐ NEW: Suppress diagnostics during speculative compilation (e.g. constants)
    pub(crate) suppress_diagnostics: bool,
}
