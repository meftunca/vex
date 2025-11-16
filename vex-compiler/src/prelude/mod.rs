/// Embedded Vex prelude sources (Layer 1 - Self-hosted)
///
/// These files are compiled directly into the Vex compiler binary.
/// The prelude provides core types and traits that are always available
/// to all Vex programs without explicit imports.
///
/// Dead code elimination: LLVM will optimize away unused prelude functions
/// at link time, so including the full prelude has no runtime cost.

/// Core prelude module containing fundamental types and traits
pub const LIB: &str = include_str!("lib.vx");

/// Vec<T> - Dynamic array implementation
pub const VEC: &str = include_str!("vec.vx");

/// Option<T> - Optional value type (Some/None)
pub const OPTION: &str = include_str!("option.vx");

/// Result<T, E> - Error handling type (Ok/Err)
pub const RESULT: &str = include_str!("result.vx");

/// Box<T> - Heap-allocated smart pointer
pub const BOX: &str = include_str!("box.vx");

/// Operator traits (Add, Sub, Mul, Div, etc.)
pub const OPS: &str = include_str!("ops.vx");

/// Builtin type contracts and assertions
pub const BUILTIN_CONTRACTS: &str = include_str!("builtin_contracts.vx");

/// Get all embedded prelude modules as (module_name, source_code) pairs
///
/// Returns modules in the correct initialization order:
/// 1. lib.vx - Core traits (Display, Clone, Debug, Default)
/// 2. ops.vx - Operator traits
/// 3. builtin_contracts.vx - Type contracts
/// 4. option.vx - Option<T> type
/// 5. result.vx - Result<T, E> type
/// 6. vec.vx - Vec<T> type
/// 7. box.vx - Box<T> type
pub fn get_embedded_prelude() -> Vec<(&'static str, &'static str)> {
    vec![
        ("core::lib", LIB),
        ("core::ops", OPS),
        ("core::builtin_contracts", BUILTIN_CONTRACTS),
        ("core::option", OPTION),
        ("core::result", RESULT),
        ("core::vec", VEC),
        ("core::box", BOX),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_sources_not_empty() {
        assert!(!LIB.is_empty(), "lib.vx should not be empty");
        assert!(!VEC.is_empty(), "vec.vx should not be empty");
        assert!(!OPTION.is_empty(), "option.vx should not be empty");
        assert!(!RESULT.is_empty(), "result.vx should not be empty");
        assert!(!BOX.is_empty(), "box.vx should not be empty");
        assert!(!OPS.is_empty(), "ops.vx should not be empty");
        assert!(
            !BUILTIN_CONTRACTS.is_empty(),
            "builtin_contracts.vx should not be empty"
        );
    }

    #[test]
    fn test_get_embedded_prelude_count() {
        let modules = get_embedded_prelude();
        assert_eq!(modules.len(), 7, "Should have 7 prelude modules");
    }

    #[test]
    fn test_prelude_module_names() {
        let modules = get_embedded_prelude();
        let names: Vec<&str> = modules.iter().map(|(name, _)| *name).collect();

        assert_eq!(names[0], "core::lib");
        assert_eq!(names[1], "core::ops");
        assert_eq!(names[2], "core::builtin_contracts");
        assert_eq!(names[3], "core::option");
        assert_eq!(names[4], "core::result");
        assert_eq!(names[5], "core::vec");
        assert_eq!(names[6], "core::box");
    }
}
