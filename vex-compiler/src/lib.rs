pub mod borrow_checker; // v0.1: Borrow checker for safety
pub mod builtin_contracts; // Builtin contract implementations for primitives
pub mod codegen_ast; // Modular LLVM codegen
pub mod linter; // Static analysis and code quality warnings
pub mod module_resolver;
pub mod prelude; // Embedded Layer 1 prelude (Vex code in compiler binary)
pub mod prelude_loader; // Prelude parser and injection
pub mod resolver; // Platform detection & stdlib resolution
pub mod trait_bounds_checker; // Trait bounds verification
pub mod type_registry; // Builtin type name registry for O(1) lookup
pub mod types; // Type interning and utilities
pub mod utils; // Utility modules (safe arithmetic, etc.)

// Re-export diagnostics from vex-diagnostics crate
pub use vex_diagnostics as diagnostics;

pub use borrow_checker::BorrowChecker;
pub use codegen_ast::ASTCodeGen;
pub use diagnostics::{error_codes, Diagnostic, DiagnosticEngine, ErrorLevel, Span};
pub use linter::{LintRule, Linter, UnusedVariableRule};
pub use module_resolver::ModuleResolver;
pub use prelude_loader::{inject_prelude_into_program, load_embedded_prelude, PreludeLoadError};
pub use resolver::{Arch, Platform, ResolveError, StdlibResolver, Target};
pub use trait_bounds_checker::TraitBoundsChecker;
pub use utils::llvm_safety::{
    emit_bounds_check, emit_null_check, is_pointer_provably_nonnull,
    validate_stack_allocation_size, MAX_STACK_ALLOC_SIZE,
};
pub use utils::safe_arithmetic::{
    safe_array_size, safe_field_index, safe_param_index, CheckedArithmetic, SafeCast,
};
