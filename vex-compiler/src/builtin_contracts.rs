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
    )
}

/// Get the method name for a builtin contract
pub fn get_builtin_contract_method(contract_name: &str) -> Option<&'static str> {
    match contract_name {
        "Display" => Some("to_string"),
        "Debug" => Some("debug"),
        "Clone" => Some("clone"),
        "Eq" => Some("eq"),
        _ => None,
    }
}

/// Codegen for builtin contract method calls
/// Returns None if not a builtin contract method (caller should handle user implementations)
pub fn codegen_builtin_contract_method<'ctx>(
    _type_name: &str,
    _contract_name: &str,
    _method_name: &str,
    receiver: BasicValueEnum<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Option<BasicValueEnum<'ctx>> {
    // TODO: Implement actual codegen for builtin contracts
    // For now, just return the receiver (for Clone on primitives)
    // This will be expanded in later phases
    Some(receiver)
}

