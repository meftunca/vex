// Safe arithmetic operations to prevent integer overflow and truncation bugs
//
// This module provides checked arithmetic operations and safe type conversions
// to prevent security vulnerabilities and undefined behavior.
//
// Related: critique/04_BOUNDS_OVERFLOW_PROTECTION.md

use std::fmt;

/// Error type for arithmetic operations
#[derive(Debug, Clone)]
pub struct ArithmeticError {
    pub message: String,
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Arithmetic error: {}", self.message)
    }
}

impl std::error::Error for ArithmeticError {}

impl From<ArithmeticError> for String {
    fn from(err: ArithmeticError) -> String {
        err.message
    }
}

/// Trait providing checked arithmetic operations
pub trait CheckedArithmetic: Sized {
    /// Safely add two values, returning error on overflow
    fn safe_add(&self, rhs: Self) -> Result<Self, ArithmeticError>;
    
    /// Safely multiply two values, returning error on overflow
    fn safe_mul(&self, rhs: Self) -> Result<Self, ArithmeticError>;
    
    /// Safely subtract two values, returning error on overflow
    fn safe_sub(&self, rhs: Self) -> Result<Self, ArithmeticError>;
}

/// Trait for safe type conversions
pub trait SafeCast<T> {
    /// Safely cast to target type, returning error if value doesn't fit
    fn safe_cast(&self) -> Result<T, ArithmeticError>;
}

// Implementations for usize
impl CheckedArithmetic for usize {
    fn safe_add(&self, rhs: Self) -> Result<Self, ArithmeticError> {
        self.checked_add(rhs).ok_or_else(|| ArithmeticError {
            message: format!("Overflow in addition: {} + {}", self, rhs),
        })
    }
    
    fn safe_mul(&self, rhs: Self) -> Result<Self, ArithmeticError> {
        self.checked_mul(rhs).ok_or_else(|| ArithmeticError {
            message: format!("Overflow in multiplication: {} * {}", self, rhs),
        })
    }
    
    fn safe_sub(&self, rhs: Self) -> Result<Self, ArithmeticError> {
        self.checked_sub(rhs).ok_or_else(|| ArithmeticError {
            message: format!("Overflow in subtraction: {} - {}", self, rhs),
        })
    }
}

impl SafeCast<u32> for usize {
    fn safe_cast(&self) -> Result<u32, ArithmeticError> {
        u32::try_from(*self).map_err(|_| ArithmeticError {
            message: format!("Cannot cast {} (usize) to u32: value too large", self),
        })
    }
}

impl SafeCast<i32> for usize {
    fn safe_cast(&self) -> Result<i32, ArithmeticError> {
        i32::try_from(*self).map_err(|_| ArithmeticError {
            message: format!("Cannot cast {} (usize) to i32: value too large", self),
        })
    }
}

// Implementations for u32
impl CheckedArithmetic for u32 {
    fn safe_add(&self, rhs: Self) -> Result<Self, ArithmeticError> {
        self.checked_add(rhs).ok_or_else(|| ArithmeticError {
            message: format!("Overflow in addition: {} + {}", self, rhs),
        })
    }
    
    fn safe_mul(&self, rhs: Self) -> Result<Self, ArithmeticError> {
        self.checked_mul(rhs).ok_or_else(|| ArithmeticError {
            message: format!("Overflow in multiplication: {} * {}", self, rhs),
        })
    }
    
    fn safe_sub(&self, rhs: Self) -> Result<Self, ArithmeticError> {
        self.checked_sub(rhs).ok_or_else(|| ArithmeticError {
            message: format!("Overflow in subtraction: {} - {}", self, rhs),
        })
    }
}

impl SafeCast<usize> for u32 {
    fn safe_cast(&self) -> Result<usize, ArithmeticError> {
        Ok(*self as usize) // u32 always fits in usize on 32-bit+ platforms
    }
}

/// Helper function to safely compute parameter index with offset
///
/// This is a common pattern in LLVM function parameter access where we need to
/// add an offset (for receiver, environment pointer, etc.) to the parameter index.
///
/// # Arguments
/// * `index` - Base parameter index (from iteration)
/// * `offset` - Offset to add (0 for regular functions, 1 for methods with receiver)
///
/// # Returns
/// * `Ok(u32)` - Safe parameter index for LLVM get_nth_param
/// * `Err` - If the computation overflows or result doesn't fit in u32
///
/// # Example
/// ```rust
/// let param_idx = safe_param_index(i, param_offset)?;
/// let llvm_param = fn_val.get_nth_param(param_idx);
/// ```
pub fn safe_param_index(index: usize, offset: usize) -> Result<u32, ArithmeticError> {
    index.safe_add(offset)?.safe_cast()
}

/// Safely cast usize to u32 for array sizes
///
/// LLVM array_type() requires u32, but Rust collections use usize.
/// This prevents silent truncation on 64-bit platforms.
///
/// # Example
/// ```rust
/// let array_type = i32_type.array_type(safe_array_size(elements.len())?);
/// ```
pub fn safe_array_size(size: usize) -> Result<u32, ArithmeticError> {
    size.safe_cast()
}

/// Safely cast usize to u32 for struct field indices
///
/// LLVM build_struct_gep() requires u32 field index.
///
/// # Example
/// ```rust
/// let field_ptr = builder.build_struct_gep(struct_ty, ptr, safe_field_index(i)?)?;
/// ```
pub fn safe_field_index(index: usize) -> Result<u32, ArithmeticError> {
    index.safe_cast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_add_success() {
        assert_eq!(5usize.safe_add(3).unwrap(), 8);
        assert_eq!(100u32.safe_add(50).unwrap(), 150);
    }

    #[test]
    fn test_safe_add_overflow() {
        let result = usize::MAX.safe_add(1);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Overflow"));
    }

    #[test]
    fn test_safe_cast_u32_success() {
        let value = 42usize;
        let result: u32 = value.safe_cast().unwrap();
        assert_eq!(result, 42u32);
    }

    #[test]
    fn test_safe_cast_u32_overflow() {
        let value = (u32::MAX as usize) + 1;
        let result: Result<u32, _> = value.safe_cast();
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_param_index() {
        assert_eq!(safe_param_index(0, 0).unwrap(), 0);
        assert_eq!(safe_param_index(5, 1).unwrap(), 6);
        assert_eq!(safe_param_index(10, 2).unwrap(), 12);
    }

    #[test]
    fn test_safe_param_index_overflow() {
        let result = safe_param_index(usize::MAX, 1);
        assert!(result.is_err());
    }
}
