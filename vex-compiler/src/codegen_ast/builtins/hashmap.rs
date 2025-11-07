// HashMap builtins - SwissTable implementation
// Core operations that need to be builtins for performance

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::AddressSpace;

/// Create a new HashMap
/// Usage: let map = map_new(initial_capacity);
pub fn builtin_hashmap_new<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Default capacity if no args
    let capacity = if args.is_empty() {
        codegen.context.i64_type().const_int(16, false)
    } else if args.len() == 1 {
        args[0].into_int_value()
    } else {
        return Err(format!(
            "map_new expects 0 or 1 argument (initial_capacity), got {}",
            args.len()
        ));
    };

    // Use vex_map_create (heap-allocated, like Vec)
    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_map_create = codegen.declare_runtime_fn(
        "vex_map_create",
        &[codegen.context.i64_type().into()], // capacity
        ptr_type.into(),                      // returns pointer
    );

    // Call vex_map_create(capacity)
    let map_ptr = codegen
        .builder
        .build_call(vex_map_create, &[capacity.into()], "map_create")
        .map_err(|e| format!("Failed to build map_new call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("map_new should return a value")?;

    Ok(map_ptr)
}

/// Insert key-value pair into HashMap
/// Usage: map_insert(map, key, value);
pub fn builtin_hashmap_insert<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 3 {
        return Err(format!(
            "map_insert expects 3 arguments (map, key, value), got {}",
            args.len()
        ));
    }

    // args[0] is the map variable (ptr to VexMap*)
    // Need to load it to get the actual VexMap*
    let map_var_ptr = args[0].into_pointer_value();
    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let map_ptr = codegen
        .builder
        .build_load(ptr_type, map_var_ptr, "map_load")
        .map_err(|e| format!("Failed to load map pointer: {:?}", e))?
        .into_pointer_value();

    let key = args[1];
    let value = args[2].into_pointer_value();

    // Declare vex_map_insert from runtime
    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let vex_map_insert = codegen.declare_runtime_fn(
        "vex_map_insert",
        &[
            ptr_type.into(), // map
            ptr_type.into(), // key
            ptr_type.into(), // value
        ],
        codegen.context.bool_type().into(), // returns bool
    );

    // Cast key to ptr if it's a string
    let key_ptr = if let BasicValueEnum::PointerValue(p) = key {
        p
    } else {
        return Err("map_insert: key must be a pointer (string)".to_string());
    };

    // Call vex_map_insert(map, key, value)
    let success = codegen
        .builder
        .build_call(
            vex_map_insert,
            &[map_ptr.into(), key_ptr.into(), value.into()],
            "map_insert",
        )
        .map_err(|e| format!("Failed to build map_insert call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("map_insert should return a value")?;

    Ok(success)
}

/// Get value from HashMap by key
/// Usage: let value = map_get(map, key);
pub fn builtin_hashmap_get<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!(
            "map_get expects 2 arguments (map, key), got {}",
            args.len()
        ));
    }

    // Load map pointer from variable
    let map_var_ptr = args[0].into_pointer_value();
    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let map_ptr = codegen
        .builder
        .build_load(ptr_type, map_var_ptr, "map_load")
        .map_err(|e| format!("Failed to load map pointer: {:?}", e))?
        .into_pointer_value();

    let key = args[1];

    // Declare vex_map_get from runtime
    let vex_map_get = codegen.declare_runtime_fn(
        "vex_map_get",
        &[
            ptr_type.into(), // map
            ptr_type.into(), // key
        ],
        ptr_type.into(), // returns pointer
    );

    // Cast key to ptr if it's a string
    let key_ptr = if let BasicValueEnum::PointerValue(p) = key {
        p
    } else {
        return Err("map_get: key must be a pointer (string)".to_string());
    };

    // Call vex_map_get(map, key)
    let value_ptr = codegen
        .builder
        .build_call(vex_map_get, &[map_ptr.into(), key_ptr.into()], "map_get")
        .map_err(|e| format!("Failed to build map_get call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("map_get should return a value")?;

    Ok(value_ptr)
}

/// Get HashMap length
/// Usage: let len = map_len(map);
pub fn builtin_hashmap_len<'ctx>(
    codegen: &mut ASTCodeGen<'ctx>,
    args: &[BasicValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "map_len expects 1 argument (map), got {}",
            args.len()
        ));
    }

    // Load map pointer from variable
    let map_var_ptr = args[0].into_pointer_value();
    let ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let map_ptr = codegen
        .builder
        .build_load(ptr_type, map_var_ptr, "map_load")
        .map_err(|e| format!("Failed to load map pointer: {:?}", e))?
        .into_pointer_value();

    // Declare vex_map_len from runtime
    let vex_map_len = codegen.declare_runtime_fn(
        "vex_map_len",
        &[ptr_type.into()],
        codegen.context.i64_type().into(), // returns i64
    );

    // Call vex_map_len(map)
    let len = codegen
        .builder
        .build_call(vex_map_len, &[map_ptr.into()], "map_len")
        .map_err(|e| format!("Failed to build map_len call: {:?}", e))?
        .try_as_basic_value()
        .left()
        .ok_or("map_len should return a value")?;

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
