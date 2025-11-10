// Type name registry for efficient type checking
// Replaces PascalCase string checks with O(1) lookups

use std::collections::HashSet;
use std::sync::OnceLock;

/// Builtin type names that should be treated as types, not variables
/// Used by borrow checker to skip scope checking for type names
static BUILTIN_TYPE_NAMES: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn get_builtin_types() -> &'static HashSet<&'static str> {
    BUILTIN_TYPE_NAMES.get_or_init(|| {
        let mut types = HashSet::new();

        // Core collection types
        types.insert("Vec");
        types.insert("Box");
        types.insert("Map");
        types.insert("Set");
        types.insert("String");

        // Range types
        types.insert("Range");
        types.insert("RangeInclusive");

        // Concurrency types
        types.insert("Channel");

        // Slice types
        types.insert("Slice");

        // Option & Result
        types.insert("Option");
        types.insert("Result");

        types
    })
}

/// Check if a name is a known builtin type
///
/// # Performance
/// O(1) hash lookup - much faster than PascalCase string check
pub fn is_builtin_type(name: &str) -> bool {
    get_builtin_types().contains(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_types() {
        assert!(is_builtin_type("Vec"));
        assert!(is_builtin_type("Box"));
        assert!(is_builtin_type("Map"));
        assert!(!is_builtin_type("MyType"));
        assert!(!is_builtin_type("my_var"));
    }

    #[test]
    fn test_no_false_positives() {
        // User-defined PascalCase names should NOT be treated as builtins
        assert!(!is_builtin_type("MyStruct"));
        assert!(!is_builtin_type("CustomType"));
        assert!(!is_builtin_type("Point"));
    }
}
