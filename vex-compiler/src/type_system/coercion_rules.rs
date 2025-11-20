//! Type coercion rules for Vex language
//!
//! Defines safe/unsafe/forbidden type coercions with automatic upcast support.
//! Prevents lossy downcasts except in unsafe{} blocks (with warnings).

use vex_ast::Type;

/// Classification of type coercions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoercionKind {
    /// Safe widening cast (i8 → i64, f32 → f64)
    /// Always allowed, no data loss
    Safe,

    /// Unsafe narrowing cast (i64 → i32, f64 → f32)
    /// Forbidden by default, allowed in unsafe{} with warning
    Unsafe,

    /// Completely forbidden (int ↔ float, signed ↔ unsigned)
    /// Requires explicit `as` cast even in unsafe{}
    Forbidden,
}

/// Coercion policy decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoercionPolicy {
    /// Perform implicit cast
    Allow,

    /// Emit compile-time warning
    Warn,

    /// Emit compile-time error
    Error,
}

/// Classify coercion between two types
pub fn classify_coercion(from: &Type, to: &Type) -> CoercionKind {
    use Type::*;

    match (from, to) {
        // ========== IDENTITY ==========
        (a, b) if a == b => CoercionKind::Safe,

        // ========== INTEGER UPCASTS (SAFE) ==========
        // Signed → Wider Signed
        (I8, I16 | I32 | I64 | I128) => CoercionKind::Safe,
        (I16, I32 | I64 | I128) => CoercionKind::Safe,
        (I32, I64 | I128) => CoercionKind::Safe,
        (I64, I128) => CoercionKind::Safe,

        // Unsigned → Wider Unsigned
        (U8, U16 | U32 | U64 | U128) => CoercionKind::Safe,
        (U16, U32 | U64 | U128) => CoercionKind::Safe,
        (U32, U64 | U128) => CoercionKind::Safe,
        (U64, U128) => CoercionKind::Safe,

        // ========== INTEGER DOWNCASTS (UNSAFE) ==========
        // Signed → Narrower Signed
        (I16, I8) => CoercionKind::Unsafe,
        (I32, I8 | I16) => CoercionKind::Unsafe,
        (I64, I8 | I16 | I32) => CoercionKind::Unsafe,
        (I128, I8 | I16 | I32 | I64) => CoercionKind::Unsafe,

        // Unsigned → Narrower Unsigned
        (U16, U8) => CoercionKind::Unsafe,
        (U32, U8 | U16) => CoercionKind::Unsafe,
        (U64, U8 | U16 | U32) => CoercionKind::Unsafe,
        (U128, U8 | U16 | U32 | U64) => CoercionKind::Unsafe,

        // ========== FLOAT UPCASTS (SAFE) ==========
        (F16, F32 | F64) => CoercionKind::Safe,
        (F32, F64) => CoercionKind::Safe,

        // ========== FLOAT DOWNCASTS (UNSAFE) ==========
        (F32, F16) => CoercionKind::Unsafe,
        (F64, F16 | F32) => CoercionKind::Unsafe,

        // ========== CROSS-TYPE CONVERSIONS (FORBIDDEN) ==========
        // Signed ↔ Unsigned (sign change)
        (I8 | I16 | I32 | I64 | I128, U8 | U16 | U32 | U64 | U128) => CoercionKind::Forbidden,
        (U8 | U16 | U32 | U64 | U128, I8 | I16 | I32 | I64 | I128) => CoercionKind::Forbidden,

        // Int ↔ Float (representation change)
        (I8 | I16 | I32 | I64 | I128, F16 | F32 | F64) => CoercionKind::Forbidden,
        (U8 | U16 | U32 | U64 | U128, F16 | F32 | F64) => CoercionKind::Forbidden,
        (F16 | F32 | F64, I8 | I16 | I32 | I64 | I128) => CoercionKind::Forbidden,
        (F16 | F32 | F64, U8 | U16 | U32 | U64 | U128) => CoercionKind::Forbidden,

        // ========== NON-NUMERIC TYPES ==========
        _ => CoercionKind::Forbidden,
    }
}

/// Determine coercion policy based on kind and unsafe context
pub fn coercion_policy(kind: CoercionKind, in_unsafe_block: bool) -> CoercionPolicy {
    match (kind, in_unsafe_block) {
        (CoercionKind::Safe, _) => CoercionPolicy::Allow,
        (CoercionKind::Unsafe, true) => CoercionPolicy::Warn,
        (CoercionKind::Unsafe, false) => CoercionPolicy::Error,
        (CoercionKind::Forbidden, _) => CoercionPolicy::Error,
    }
}

/// Check if target type is wider than source type
pub fn is_wider_type(from: &Type, to: &Type) -> bool {
    matches!(classify_coercion(from, to), CoercionKind::Safe)
}

/// Get wider type between two types (for binary op alignment)
pub fn wider_type<'a>(left: &'a Type, right: &'a Type) -> Result<&'a Type, String> {
    if left == right {
        return Ok(left);
    }

    if is_wider_type(left, right) {
        Ok(right) // right is wider
    } else if is_wider_type(right, left) {
        Ok(left) // left is wider
    } else {
        Err(format!(
            "cannot determine wider type between {:?} and {:?}: incompatible types",
            left, right
        ))
    }
}

/// Format diagnostic message for coercion error
pub fn format_coercion_error(from: &Type, to: &Type, kind: CoercionKind) -> String {
    match kind {
        CoercionKind::Unsafe => {
            format!(
                "cannot implicitly downcast {:?} to {:?}: data loss possible\n\nhelp: use explicit cast `as {:?}` or wrap in unsafe{{}}",
                from, to, to
            )
        }
        CoercionKind::Forbidden => {
            format!(
                "cannot convert {:?} to {:?}: incompatible types\n\nhelp: use explicit cast `as {:?}`",
                from, to, to
            )
        }
        CoercionKind::Safe => unreachable!("Safe coercions should not generate errors"),
    }
}

/// Format diagnostic message for coercion warning (unsafe context)
pub fn format_coercion_warning(from: &Type, to: &Type) -> String {
    format!(
        "unsafe downcast from {:?} to {:?}: data loss may occur\n\nhelp: consider using explicit cast `as {:?}` for clarity",
        from, to, to
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use vex_ast::Type::*;

    #[test]
    fn test_safe_integer_upcasts() {
        assert_eq!(classify_coercion(&I8, &I16), CoercionKind::Safe);
        assert_eq!(classify_coercion(&I8, &I64), CoercionKind::Safe);
        assert_eq!(classify_coercion(&I32, &I64), CoercionKind::Safe);
        assert_eq!(classify_coercion(&U8, &U32), CoercionKind::Safe);
    }

    #[test]
    fn test_unsafe_integer_downcasts() {
        assert_eq!(classify_coercion(&I64, &I32), CoercionKind::Unsafe);
        assert_eq!(classify_coercion(&I32, &I8), CoercionKind::Unsafe);
        assert_eq!(classify_coercion(&U64, &U16), CoercionKind::Unsafe);
    }

    #[test]
    fn test_forbidden_sign_change() {
        assert_eq!(classify_coercion(&I32, &U32), CoercionKind::Forbidden);
        assert_eq!(classify_coercion(&U64, &I64), CoercionKind::Forbidden);
    }

    #[test]
    fn test_forbidden_int_float() {
        assert_eq!(classify_coercion(&I32, &F32), CoercionKind::Forbidden);
        assert_eq!(classify_coercion(&F64, &I64), CoercionKind::Forbidden);
    }

    #[test]
    fn test_float_upcasts() {
        assert_eq!(classify_coercion(&F32, &F64), CoercionKind::Safe);
    }

    #[test]
    fn test_float_downcasts() {
        assert_eq!(classify_coercion(&F64, &F32), CoercionKind::Unsafe);
    }

    #[test]
    fn test_wider_type() {
        assert_eq!(wider_type(&I8, &I64).unwrap(), &I64);
        assert_eq!(wider_type(&I64, &I32).unwrap(), &I64);
        assert_eq!(wider_type(&F32, &F64).unwrap(), &F64);
    }

    #[test]
    fn test_coercion_policy() {
        // Safe: always allow
        assert_eq!(
            coercion_policy(CoercionKind::Safe, false),
            CoercionPolicy::Allow
        );
        assert_eq!(
            coercion_policy(CoercionKind::Safe, true),
            CoercionPolicy::Allow
        );

        // Unsafe: error outside unsafe{}, warn inside
        assert_eq!(
            coercion_policy(CoercionKind::Unsafe, false),
            CoercionPolicy::Error
        );
        assert_eq!(
            coercion_policy(CoercionKind::Unsafe, true),
            CoercionPolicy::Warn
        );

        // Forbidden: always error
        assert_eq!(
            coercion_policy(CoercionKind::Forbidden, false),
            CoercionPolicy::Error
        );
        assert_eq!(
            coercion_policy(CoercionKind::Forbidden, true),
            CoercionPolicy::Error
        );
    }
}
