pub mod borrow_checker; // v0.1: Borrow checker for safety
pub mod codegen_ast; // Modular LLVM codegen
pub mod module_resolver;
pub mod resolver; // Platform detection & stdlib resolution
pub mod trait_bounds_checker; // Trait bounds verification
pub mod type_registry; // Builtin type name registry for O(1) lookup

// Re-export diagnostics from vex-diagnostics crate
pub use vex_diagnostics as diagnostics;

pub use borrow_checker::BorrowChecker;
pub use codegen_ast::ASTCodeGen;
pub use diagnostics::{error_codes, Diagnostic, DiagnosticEngine, ErrorLevel, Span};
pub use module_resolver::ModuleResolver;
pub use resolver::{Arch, Platform, ResolveError, StdlibResolver, Target};
pub use trait_bounds_checker::TraitBoundsChecker;
