// Set<T> builtin functions - wrapper around Map<T, ()>

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;

/// set_new() - Create empty Set (wraps Map)
pub fn builtin_set_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    _args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Set is just Map<T, ()>, so call map_new()
    super::hashmap::builtin_hashmap_new(codegen, _args)
}

/// Create a new Set<T> with capacity (wraps Map<T,()>)
pub fn builtin_set_with_capacity<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Delegate to hashmap_new (which accepts capacity argument)
    super::hashmap::builtin_hashmap_new(codegen, args)
}

/// set_insert(set, value) - Insert value into Set
pub fn builtin_set_insert<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("set_insert() requires 2 arguments (set, value)".to_string());
    }

    // Insert with dummy value () - we only care about keys
    // For now, just call map_insert with a zero value
    let set_ptr = args[0];
    let value = args[1];

    // Create dummy unit value (i8 zero)
    let dummy_unit = codegen.context.i8_type().const_zero();

    // Call map_insert(set, value, dummy_unit)
    super::hashmap::builtin_hashmap_insert(codegen, &[set_ptr, value, dummy_unit.into()])
}

/// set_contains(set, value) - Check if Set contains value
pub fn builtin_set_contains<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("set_contains() requires 2 arguments (set, value)".to_string());
    }

    // Just call map_get and check if result is non-null
    let result = super::hashmap::builtin_hashmap_get(codegen, args)?;

    // Check if result pointer is non-null
    if let BasicValueEnum::PointerValue(ptr) = result {
        let is_not_null = codegen
            .builder
            .build_is_not_null(ptr, "set_contains_check")
            .map_err(|e| format!("Failed to build is_not_null: {}", e))?;

        Ok(is_not_null.into())
    } else {
        Err("set_contains: expected pointer from map_get".to_string())
    }
}

/// set_remove(set, value) - Remove value from Set
pub fn builtin_set_remove<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err("set_remove() requires 2 arguments (set, value)".to_string());
    }

    // Delegate to map_remove
    super::hashmap::builtin_hashmap_remove(codegen, args)
}

/// set_len(set) - Get Set size
pub fn builtin_set_len<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("set_len() requires 1 argument (set)".to_string());
    }

    // Delegate to map_len
    super::hashmap::builtin_hashmap_len(codegen, args)
}

/// set_clear(set) - Clear all values from Set
pub fn builtin_set_clear<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err("set_clear() requires 1 argument (set)".to_string());
    }

    // Delegate to map_clear
    super::hashmap::builtin_hashmap_clear(codegen, args)
}
