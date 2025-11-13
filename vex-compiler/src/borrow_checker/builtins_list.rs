// Built-in function registry for borrow checker
// Contains all built-in functions that are always in scope

/// Get the list of all built-in functions that should be registered as always in scope
pub fn get_builtin_functions() -> &'static [&'static str] {
    &[
        // Core builtins
        "print",
        "println",
        "panic",
        "assert",
        "unreachable",
        // Memory builtins
        "alloc",
        "free",
        "realloc",
        "sizeof",
        "alignof",
        // Bit manipulation
        "ctlz",
        "cttz",
        "ctpop",
        "bswap",
        "bitreverse",
        // Overflow checking
        "sadd_overflow",
        "ssub_overflow",
        "smul_overflow",
        // Compiler hints
        "assume",
        "likely",
        "unlikely",
        "prefetch",
        // String functions
        "strlen",
        "strcmp",
        "strcpy",
        "strcat",
        "strdup",
        // Memory operations
        "memcpy",
        "memset",
        "memcmp",
        "memmove",
        // UTF-8 functions
        "utf8_valid",
        "utf8_char_count",
        "utf8_char_at",
        // Array functions
        "array_len",
        "array_get",
        "array_set",
        "array_append",
        // Type reflection
        "typeof",
        "type_id",
        "type_size",
        "type_align",
        "is_int_type",
        "is_float_type",
        "is_pointer_type",
        // HashMap functions
        "hashmap_new",
        "hashmap_insert",
        "hashmap_get",
        "hashmap_len",
        "hashmap_free",
        "hashmap_contains",
        "hashmap_remove",
        "hashmap_clear",
        // Phase 0.4b: Builtin type constructors
        "vec_new",
        "vec_free",
        "box_new",
        "box_free",
        // Phase 0.7: Primitive to string conversions
        "vex_i32_to_string",
        "vex_i64_to_string",
        "vex_u32_to_string",
        "vex_u64_to_string",
        "vex_f32_to_string",
        "vex_f64_to_string",
        "vex_bool_to_string",
        "vex_string_to_string",
    ]
}
