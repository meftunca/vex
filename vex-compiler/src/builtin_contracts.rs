// Builtin contract implementations for primitive types
// This module provides compiler-generated contract methods for i32, f64, bool, string

use inkwell::values::BasicValueEnum;

/// Check if a type has a builtin contract implementation
pub fn has_builtin_contract(type_name: &str, contract_name: &str) -> bool {
    matches!(
        (type_name, contract_name),
        ("i32" | "i64" | "i8" | "i16" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "string", "Display")
        | ("i32" | "i64" | "i8" | "i16" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "string", "Clone")
        | ("i32" | "i64" | "i8" | "i16" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "string", "Eq")
        | ("i32" | "i64" | "i8" | "i16" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "string", "Debug")
        | ("i32" | "i64" | "f64", "Add")
        | ("i32" | "i64" | "f64", "Sub")
        | ("i32" | "i64" | "f64", "Mul")
        | ("i32" | "i64" | "f64", "Div")
        | ("i32" | "i64" | "f64", "Rem")
    )
}

/// Get the method name for a builtin contract
pub fn get_builtin_contract_method(contract_name: &str) -> Option<&'static str> {
    match contract_name {
        "Display" => Some("to_string"),
        "Debug" => Some("debug"),
        "Clone" => Some("clone"),
        "Eq" => Some("eq"),
        "Add" => Some("add"),
        "Sub" => Some("sub"),
        "Mul" => Some("mul"),
        "Div" => Some("div"),
        "Rem" => Some("rem"),
        _ => None,
    }
}

/// Codegen for builtin contract method calls
/// Returns None if not a builtin contract method (caller should handle user implementations)
pub fn codegen_builtin_contract_method<'ctx>(
    type_name: &str,
    contract_name: &str,
    method_name: &str,
    receiver: BasicValueEnum<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Option<BasicValueEnum<'ctx>> {
    match (type_name, contract_name, method_name) {
        // Clone: return receiver (bitwise copy for primitives)
        (_, "Clone", "clone") => {
            Some(receiver)
        }
        
        // Arithmetic operators: Add, Sub, Mul, Div, Rem
        // Return None to let binary_ops.rs handle LLVM codegen with existing fallback
        ("i32" | "i64", "Add", "add") if args.len() == 1 => None,
        ("i32" | "i64", "Sub", "sub") if args.len() == 1 => None,
        ("i32" | "i64", "Mul", "mul") if args.len() == 1 => None,
        ("i32" | "i64", "Div", "div") if args.len() == 1 => None,
        ("i32" | "i64", "Rem", "rem") if args.len() == 1 => None,
        
        ("f64", "Add", "add") if args.len() == 1 => None,
        ("f64", "Sub", "sub") if args.len() == 1 => None,
        ("f64", "Mul", "mul") if args.len() == 1 => None,
        ("f64", "Div", "div") if args.len() == 1 => None,
        ("f64", "Rem", "rem") if args.len() == 1 => None,
        
        // Display: TODO - needs runtime string conversion functions
        (_, "Display", "to_string") => {
            eprintln!("⚠️ Display.to_string not yet implemented - needs runtime functions");
            None
        }
        
        // Debug: TODO
        (_, "Debug", "debug") => {
            eprintln!("⚠️ Debug.debug not yet implemented");
            None
        }
        
        // Eq: TODO
        (_, "Eq", "eq") => {
            eprintln!("⚠️ Eq.eq not yet implemented");
            None
        }
        
        _ => None,
    }
}

