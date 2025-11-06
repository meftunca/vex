// Builtin function metadata for borrow checker
// Defines which parameters are mutated/borrowed by builtins

use std::collections::HashMap;

/// Describes how a builtin function affects its parameters
#[derive(Debug, Clone, PartialEq)]
pub enum ParamEffect {
    /// Parameter is read-only, no side effects
    ReadOnly,

    /// Parameter is mutated in-place (requires &mut or ownership)
    Mutates,

    /// Parameter is moved (takes ownership)
    Moves,

    /// Parameter is borrowed immutably
    BorrowsImmut,

    /// Parameter is borrowed mutably
    BorrowsMut,
}

/// Metadata about a builtin function's effects
#[derive(Debug, Clone)]
pub struct BuiltinMetadata {
    pub name: &'static str,
    pub param_effects: Vec<ParamEffect>,
    pub returns_borrowed: bool, // Does it return a borrow of input?
}

/// Registry of builtin function metadata for borrow checking
#[derive(Debug)]
pub struct BuiltinBorrowRegistry {
    metadata: HashMap<&'static str, BuiltinMetadata>,
}

impl BuiltinBorrowRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            metadata: HashMap::new(),
        };

        // Core I/O (read-only parameters)
        registry.register("print", vec![ParamEffect::ReadOnly]);
        registry.register("println", vec![ParamEffect::ReadOnly]);
        registry.register("panic", vec![ParamEffect::ReadOnly]);
        registry.register("assert", vec![ParamEffect::ReadOnly, ParamEffect::ReadOnly]);

        // Memory (alloc is special, free moves ownership)
        registry.register("alloc", vec![ParamEffect::ReadOnly]);
        registry.register("free", vec![ParamEffect::Moves]); // Takes ownership
        registry.register("realloc", vec![ParamEffect::Moves, ParamEffect::ReadOnly]);

        // Sizeof/Alignof (compile-time, read-only)
        registry.register("sizeof", vec![ParamEffect::ReadOnly]);
        registry.register("alignof", vec![ParamEffect::ReadOnly]);

        // String operations (most are read-only, strdup allocates)
        registry.register("strlen", vec![ParamEffect::BorrowsImmut]);
        registry.register(
            "strcmp",
            vec![ParamEffect::BorrowsImmut, ParamEffect::BorrowsImmut],
        );
        registry.register(
            "strcpy",
            vec![ParamEffect::BorrowsMut, ParamEffect::BorrowsImmut],
        );
        registry.register(
            "strcat",
            vec![ParamEffect::BorrowsMut, ParamEffect::BorrowsImmut],
        );
        registry.register("strdup", vec![ParamEffect::BorrowsImmut]);

        // Memory operations (memcpy mutates dest, others are read-only)
        registry.register(
            "memcpy",
            vec![
                ParamEffect::BorrowsMut,   // dest
                ParamEffect::BorrowsImmut, // src
                ParamEffect::ReadOnly,     // size
            ],
        );
        registry.register(
            "memset",
            vec![
                ParamEffect::BorrowsMut, // ptr
                ParamEffect::ReadOnly,   // value
                ParamEffect::ReadOnly,   // size
            ],
        );
        registry.register(
            "memcmp",
            vec![
                ParamEffect::BorrowsImmut, // ptr1
                ParamEffect::BorrowsImmut, // ptr2
                ParamEffect::ReadOnly,     // size
            ],
        );
        registry.register(
            "memmove",
            vec![
                ParamEffect::BorrowsMut,   // dest
                ParamEffect::BorrowsImmut, // src
                ParamEffect::ReadOnly,     // size
            ],
        );

        // UTF-8 operations (all read-only)
        registry.register(
            "utf8_valid",
            vec![ParamEffect::BorrowsImmut, ParamEffect::ReadOnly],
        );
        registry.register("utf8_char_count", vec![ParamEffect::BorrowsImmut]);
        registry.register(
            "utf8_char_at",
            vec![ParamEffect::BorrowsImmut, ParamEffect::ReadOnly],
        );

        // Array operations (CRITICAL: most mutate!)
        registry.register("array_len", vec![ParamEffect::BorrowsImmut]);
        registry.register(
            "array_get",
            vec![
                ParamEffect::BorrowsImmut, // array
                ParamEffect::ReadOnly,     // index
                ParamEffect::ReadOnly,     // elem_size
            ],
        );
        registry.register(
            "array_set",
            vec![
                ParamEffect::BorrowsMut, // array (mutates!)
                ParamEffect::ReadOnly,   // index
                ParamEffect::ReadOnly,   // value
                ParamEffect::ReadOnly,   // elem_size
            ],
        );
        registry.register(
            "array_append",
            vec![
                ParamEffect::BorrowsMut, // array (mutates!)
                ParamEffect::ReadOnly,   // value
                ParamEffect::ReadOnly,   // elem_size
            ],
        );

        // HashMap operations (CRITICAL: most mutate!)
        registry.register("hashmap_new", vec![ParamEffect::ReadOnly]);
        registry.register(
            "hashmap_insert",
            vec![
                ParamEffect::BorrowsMut,   // map (mutates!)
                ParamEffect::BorrowsImmut, // key
                ParamEffect::ReadOnly,     // value
            ],
        );
        registry.register(
            "hashmap_get",
            vec![
                ParamEffect::BorrowsImmut, // map
                ParamEffect::BorrowsImmut, // key
            ],
        );
        registry.register("hashmap_len", vec![ParamEffect::BorrowsImmut]);
        registry.register("hashmap_free", vec![ParamEffect::Moves]); // Takes ownership
        registry.register(
            "hashmap_contains",
            vec![
                ParamEffect::BorrowsImmut, // map
                ParamEffect::BorrowsImmut, // key
            ],
        );
        registry.register(
            "hashmap_remove",
            vec![
                ParamEffect::BorrowsMut,   // map (mutates!)
                ParamEffect::BorrowsImmut, // key
            ],
        );
        registry.register("hashmap_clear", vec![ParamEffect::BorrowsMut]); // Mutates

        // Type reflection (all read-only, no side effects)
        registry.register("typeof", vec![ParamEffect::ReadOnly]);
        registry.register("type_id", vec![ParamEffect::ReadOnly]);
        registry.register("type_size", vec![ParamEffect::ReadOnly]);
        registry.register("type_align", vec![ParamEffect::ReadOnly]);
        registry.register("is_int_type", vec![ParamEffect::ReadOnly]);
        registry.register("is_float_type", vec![ParamEffect::ReadOnly]);
        registry.register("is_pointer_type", vec![ParamEffect::ReadOnly]);

        // LLVM intrinsics (all read-only)
        registry.register("ctlz", vec![ParamEffect::ReadOnly]);
        registry.register("cttz", vec![ParamEffect::ReadOnly]);
        registry.register("ctpop", vec![ParamEffect::ReadOnly]);
        registry.register("bswap", vec![ParamEffect::ReadOnly]);
        registry.register("bitreverse", vec![ParamEffect::ReadOnly]);
        registry.register(
            "sadd_overflow",
            vec![ParamEffect::ReadOnly, ParamEffect::ReadOnly],
        );
        registry.register(
            "ssub_overflow",
            vec![ParamEffect::ReadOnly, ParamEffect::ReadOnly],
        );
        registry.register(
            "smul_overflow",
            vec![ParamEffect::ReadOnly, ParamEffect::ReadOnly],
        );

        // Compiler hints (all read-only)
        registry.register("assume", vec![ParamEffect::ReadOnly]);
        registry.register("likely", vec![ParamEffect::ReadOnly]);
        registry.register("unlikely", vec![ParamEffect::ReadOnly]);
        registry.register(
            "prefetch",
            vec![
                ParamEffect::BorrowsImmut, // ptr
                ParamEffect::ReadOnly,     // locality
                ParamEffect::ReadOnly,     // rw
            ],
        );

        // Phase 0.4b: Builtin type constructors (free functions)
        registry.register("vec_new", vec![]); // No args, returns new Vec
        registry.register("vec_with_capacity", vec![ParamEffect::ReadOnly]); // Takes capacity
        registry.register("vec_free", vec![ParamEffect::BorrowsMut]); // Takes &Vec
        registry.register("box_new", vec![ParamEffect::ReadOnly]); // Takes value by copy
        registry.register("box_free", vec![ParamEffect::Moves]); // Takes Box by value
        registry.register("string_new", vec![]); // No args, returns empty String
        registry.register("string_from", vec![ParamEffect::BorrowsImmut]); // Takes string literal
        registry.register("string_free", vec![ParamEffect::Moves]); // Takes String by value
        registry.register("map_new", vec![]); // No args, returns new Map
        registry.register("map_with_capacity", vec![ParamEffect::ReadOnly]); // Takes capacity
        registry.register(
            "map_insert",
            vec![
                ParamEffect::BorrowsMut,
                ParamEffect::BorrowsImmut,
                ParamEffect::ReadOnly,
            ],
        ); // Takes &Map!, key, value
        registry.register(
            "map_get",
            vec![ParamEffect::BorrowsImmut, ParamEffect::BorrowsImmut],
        ); // Takes &Map, key
        registry.register("map_len", vec![ParamEffect::BorrowsImmut]); // Takes &Map
        registry.register("map_free", vec![ParamEffect::Moves]); // Takes Map by value

        // Phase 0.7: Numeric to string conversions (all read-only)
        registry.register("vex_i32_to_string", vec![ParamEffect::ReadOnly]);
        registry.register("vex_i64_to_string", vec![ParamEffect::ReadOnly]);
        registry.register("vex_u32_to_string", vec![ParamEffect::ReadOnly]);
        registry.register("vex_u64_to_string", vec![ParamEffect::ReadOnly]);
        registry.register("vex_f32_to_string", vec![ParamEffect::ReadOnly]);
        registry.register("vex_f64_to_string", vec![ParamEffect::ReadOnly]);

        registry
    }

    fn register(&mut self, name: &'static str, param_effects: Vec<ParamEffect>) {
        self.metadata.insert(
            name,
            BuiltinMetadata {
                name,
                param_effects,
                returns_borrowed: false,
            },
        );
    }

    /// Get metadata for a builtin function
    pub fn get(&self, name: &str) -> Option<&BuiltinMetadata> {
        self.metadata.get(name)
    }

    /// Check if a function is a known builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.metadata.contains_key(name)
    }
}

impl Default for BuiltinBorrowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_registry() {
        let registry = BuiltinBorrowRegistry::new();

        // Test read-only builtins
        let typeof_meta = registry.get("typeof").unwrap();
        assert_eq!(typeof_meta.param_effects[0], ParamEffect::ReadOnly);

        // Test mutating builtins
        let array_append = registry.get("array_append").unwrap();
        assert_eq!(array_append.param_effects[0], ParamEffect::BorrowsMut);

        // Test move builtins
        let free_meta = registry.get("free").unwrap();
        assert_eq!(free_meta.param_effects[0], ParamEffect::Moves);
    }

    #[test]
    fn test_hashmap_builtins() {
        let registry = BuiltinBorrowRegistry::new();

        // hashmap_insert should mutate map
        let insert = registry.get("hashmap_insert").unwrap();
        assert_eq!(insert.param_effects[0], ParamEffect::BorrowsMut);

        // hashmap_get should borrow immutably
        let get = registry.get("hashmap_get").unwrap();
        assert_eq!(get.param_effects[0], ParamEffect::BorrowsImmut);
    }
}
