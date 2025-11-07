pub mod borrow_checker; // v0.9: Borrow checker for safety
                        // pub mod codegen;
pub mod codegen_ast; // Modular LLVM codegen
pub mod diagnostics;
pub mod module_resolver;
pub mod trait_bounds_checker; // Trait bounds verification // v0.9.2: Enhanced error messages with spans and colors

pub use borrow_checker::BorrowChecker;
// pub use codegen::CodeGen;
pub use codegen_ast::ASTCodeGen;
pub use diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
pub use module_resolver::ModuleResolver;
pub use trait_bounds_checker::TraitBoundsChecker;
