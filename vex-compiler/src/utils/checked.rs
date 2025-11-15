use anyhow::{anyhow, Result};

pub trait CheckedArithmetic {
    fn safe_add(&self, rhs: Self) -> Result<Self> where Self: Sized;
    fn safe_mul(&self, rhs: Self) -> Result<Self> where Self: Sized;
    fn safe_cast_u32(&self) -> Result<u32>;
}

impl CheckedArithmetic for usize {
    fn safe_add(&self, rhs: Self) -> Result<Self> {
        self.checked_add(rhs).ok_or_else(|| anyhow!("Overflow: {} + {}", self, rhs))
    }

    fn safe_mul(&self, rhs: Self) -> Result<Self> {
        self.checked_mul(rhs).ok_or_else(|| anyhow!("Overflow: {} * {}", self, rhs))
    }

    fn safe_cast_u32(&self) -> Result<u32> {
        u32::try_from(*self).map_err(|_| anyhow!("Cannot cast {} to u32", self))
    }
}