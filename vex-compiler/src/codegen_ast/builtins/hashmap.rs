// HashMap builtins - SwissTable implementation
// Core operations that need to be builtins for performance

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// Create a new HashMap
/// Usage: let map = hashmap_new(initial_capacity);
pub fn builtin_hashmap_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "hashmap_new expects 1 argument (initial_capacity), got {}",
            args.len()
        ));
    }

    let capacity = args[0].into_int_value();

    // Declare vex_map_new from runtime
    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_map_new = codegen.declare_runtime_fn(
        "vex_map_new",
        &[codegen.context.i64_type().into()], // capacity
        i8_ptr.into(),                        // returns pointer
    );

    // Call vex_map_new(capacity)
    let map_ptr = codegen
        .builder
        .build_call(vex_map_new, &[capacity.into()], "map_new")
        .map_err(|e| format!("Failed to build hashmap_new call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("hashmap_new should return a value")?;

    Ok(map_ptr)
}

/// Insert key-value pair into HashMap
/// Usage: hashmap_insert(map, key, value);
pub fn builtin_hashmap_insert<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err(format!(
            "hashmap_insert expects 3 arguments (map, key, value), got {}",
            args.len()
        ));
    }

    let map_ptr = args[0].into_pointer_value();
    let key = args[1]; // Assuming string key for now
    let value = args[2].into_pointer_value();

    // Declare vex_map_insert from runtime
    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_map_insert = codegen.declare_runtime_fn(
        "vex_map_insert",
        &[
            i8_ptr.into(), // map
            i8_ptr.into(), // key
            i8_ptr.into(), // value
        ],
        codegen.context.bool_type().into(), // returns bool
    );

    // Cast key to i8* if it's a string
    let key_ptr = if let BasicValueEnum::PointerValue(p) = key {
        codegen
            .builder
            .build_pointer_cast(p, i8_ptr, "key_cast")
            .map_err(|e| format!("Failed to cast key: {:?}", e))?
    } else {
        return Err("hashmap_insert: key must be a pointer (string)".to_string());
    };

    // Cast value to i8*
    let value_cast = codegen
        .builder
        .build_pointer_cast(value, i8_ptr, "value_cast")
        .map_err(|e| format!("Failed to cast value: {:?}", e))?;

    // Call vex_map_insert(map, key, value)
    let success = codegen
        .builder
        .build_call(
            vex_map_insert,
            &[map_ptr.into(), key_ptr.into(), value_cast.into()],
            "map_insert",
        )
        .map_err(|e| format!("Failed to build hashmap_insert call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("hashmap_insert should return a value")?;

    Ok(success)
}

/// Get value from HashMap by key
/// Usage: let value = hashmap_get(map, key);
pub fn builtin_hashmap_get<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!(
            "hashmap_get expects 2 arguments (map, key), got {}",
            args.len()
        ));
    }

    let map_ptr = args[0].into_pointer_value();
    let key = args[1];

    // Declare vex_map_get from runtime
    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_map_get = codegen.declare_runtime_fn(
        "vex_map_get",
        &[
            i8_ptr.into(), // map
            i8_ptr.into(), // key
        ],
        i8_ptr.into(), // returns pointer
    );

    // Cast key to i8* if it's a string
    let key_ptr = if let BasicValueEnum::PointerValue(p) = key {
        codegen
            .builder
            .build_pointer_cast(p, i8_ptr, "key_cast")
            .map_err(|e| format!("Failed to cast key: {:?}", e))?
    } else {
        return Err("hashmap_get: key must be a pointer (string)".to_string());
    };

    // Call vex_map_get(map, key)
    let value_ptr = codegen
        .builder
        .build_call(vex_map_get, &[map_ptr.into(), key_ptr.into()], "map_get")
        .map_err(|e| format!("Failed to build hashmap_get call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("hashmap_get should return a value")?;

    Ok(value_ptr)
}

/// Get HashMap length
/// Usage: let len = hashmap_len(map);
pub fn builtin_hashmap_len<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "hashmap_len expects 1 argument (map), got {}",
            args.len()
        ));
    }

    let map_ptr = args[0].into_pointer_value();

    // Declare vex_map_len from runtime
    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_map_len = codegen.declare_runtime_fn(
        "vex_map_len",
        &[i8_ptr.into()],
        codegen.context.i64_type().into(), // returns i64
    );

    // Call vex_map_len(map)
    let len = codegen
        .builder
        .build_call(vex_map_len, &[map_ptr.into()], "map_len")
        .map_err(|e| format!("Failed to build hashmap_len call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("hashmap_len should return a value")?;

    Ok(len)
}

/// Free HashMap
/// Usage: hashmap_free(map);
pub fn builtin_hashmap_free<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "hashmap_free expects 1 argument (map), got {}",
            args.len()
        ));
    }

    let map_ptr = args[0].into_pointer_value();

    // Declare vex_map_free from runtime
    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_map_free = codegen.declare_runtime_fn_void("vex_map_free", &[i8_ptr.into()]);

    // Call vex_map_free(map)
    codegen
        .builder
        .build_call(vex_map_free, &[map_ptr.into()], "map_free")
        .map_err(|e| format!("Failed to build hashmap_free call: {:?}", e))?;

    // Return unit/void (use i32 0 as placeholder)
    Ok(codegen.context.i32_type().const_int(0, false).into())
}

/// Check if HashMap contains key
/// Usage: if hashmap_contains(map, key) { ... }
pub fn builtin_hashmap_contains<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!(
            "hashmap_contains expects 2 arguments (map, key), got {}",
            args.len()
        ));
    }

    // Get value, check if NULL
    let value = builtin_hashmap_get(codegen, args)?;
    let value_ptr = value.into_pointer_value();

    // Check if pointer is null
    let is_not_null = codegen
        .builder
        .build_is_not_null(value_ptr, "contains_check")
        .map_err(|e| format!("Failed to build null check: {:?}", e))?;

    Ok(is_not_null.into())
}

/// Remove key from HashMap
/// Usage: hashmap_remove(map, key);
pub fn builtin_hashmap_remove<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!(
            "hashmap_remove expects 2 arguments (map, key), got {}",
            args.len()
        ));
    }

    let map_ptr = args[0].into_pointer_value();
    let key = args[1];

    // Declare vex_map_remove from runtime
    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_map_remove = codegen.declare_runtime_fn(
        "vex_map_remove",
        &[
            i8_ptr.into(), // map
            i8_ptr.into(), // key
        ],
        codegen.context.bool_type().into(), // returns bool
    );

    // Cast key to i8*
    let key_ptr = if let BasicValueEnum::PointerValue(p) = key {
        codegen
            .builder
            .build_pointer_cast(p, i8_ptr, "key_cast")
            .map_err(|e| format!("Failed to cast key: {:?}", e))?
    } else {
        return Err("hashmap_remove: key must be a pointer (string)".to_string());
    };

    // Call vex_map_remove(map, key)
    let success = codegen
        .builder
        .build_call(
            vex_map_remove,
            &[map_ptr.into(), key_ptr.into()],
            "map_remove",
        )
        .map_err(|e| format!("Failed to build hashmap_remove call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("hashmap_remove should return a value")?;

    Ok(success)
}

/// Clear all entries from HashMap
/// Usage: hashmap_clear(map);
pub fn builtin_hashmap_clear<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "hashmap_clear expects 1 argument (map), got {}",
            args.len()
        ));
    }

    let map_ptr = args[0].into_pointer_value();

    // Declare vex_map_clear from runtime
    let i8_ptr = codegen.context.i8_type().ptr_type(AddressSpace::default());
    let vex_map_clear = codegen.declare_runtime_fn_void("vex_map_clear", &[i8_ptr.into()]);

    // Call vex_map_clear(map)
    codegen
        .builder
        .build_call(vex_map_clear, &[map_ptr.into()], "map_clear")
        .map_err(|e| format!("Failed to build hashmap_clear call: {:?}", e))?;

    // Return unit
    Ok(codegen.context.i32_type().const_int(0, false).into())
}
