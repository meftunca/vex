pub mod codegen;
pub mod codegen_ast; // Modular LLVM codegen
pub mod module_resolver;

pub use codegen::CodeGen;
pub use codegen_ast::ASTCodeGen;
pub use module_resolver::ModuleResolver;
