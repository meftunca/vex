pub mod borrow_checker; // v0.9: Borrow checker for safety
pub mod codegen;
pub mod codegen_ast; // Modular LLVM codegen
pub mod module_resolver;

pub use borrow_checker::BorrowChecker;
pub use codegen::CodeGen;
pub use codegen_ast::ASTCodeGen;
pub use module_resolver::ModuleResolver;
