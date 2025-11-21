// Print format string parsing and type-safe formatting

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::Expression;

use super::print_execution::compile_print_variadic;
use crate::codegen_ast::builtins::optimized_print;

pub fn compile_print_call<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    func_name: &str,
    ast_args: &[Expression],
    compiled_args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if ast_args.is_empty() {
        return Err(format!("{}() requires at least one argument", func_name));
    }

    // Check if first argument is a string literal containing '{' (format placeholder)
    let is_format_mode = if let Expression::StringLiteral(s) = &ast_args[0] {
        s.contains('{')
    } else {
        false
    };

    if is_format_mode {
        // Format string mode: print("x = {}, y = {}", 42, 3.14)

        // Check if format string is a literal (for compile-time optimization)
        if let Expression::StringLiteral(fmt_str) = &ast_args[0] {
            // OPTIMIZED PATH: Compile-time format string parsing
            // This generates inline LLVM IR with zero VexValue overhead

            // Infer types of value arguments
            let mut arg_types = Vec::new();
            for arg in &ast_args[1..] {
                let ty = codegen.infer_expression_type(arg)?;
                arg_types.push(ty);
            }

            // Use optimized inline code generation
            return optimized_print::compile_print_fmt_optimized(
                codegen,
                func_name,
                fmt_str,
                &compiled_args[1..],
                &arg_types,
            );
        }

        // Dynamic format strings are not supported in the optimized runtime yet
        return Err("Dynamic format strings are not supported yet. Please use string interpolation or concatenation.".to_string());
    } else {
        // Go-style variadic mode: print("x =", 42, "y =", 3.14)
        // Infer types for all arguments
        let mut arg_types = Vec::new();
        for arg in ast_args {
            let ty = codegen.infer_expression_type(arg)?;
            arg_types.push(ty);
        }
        compile_print_variadic(codegen, func_name, compiled_args, &arg_types)
    }
}
