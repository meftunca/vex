// Print function execution (compile_print_variadic)

use crate::codegen_ast::builtins::optimized_print;
use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// Go-style variadic print: print("x =", 42, "y =", 3.14)
/// OPTIMIZED: Uses direct C function dispatch instead of VexValue array
pub(super) fn compile_print_variadic<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    func_name: &str,
    args: &[BasicValueEnum<'ctx>],
    arg_types: &[vex_ast::Type],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Use optimized zero-overhead print path
    // This eliminates VexValue struct allocation (32 bytes per arg)
    // and enables compile-time type dispatch
    optimized_print::compile_print_optimized(codegen, func_name, args, arg_types)
}
