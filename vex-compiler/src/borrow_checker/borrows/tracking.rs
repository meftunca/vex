//! Borrow tracking structures and utilities

use std::collections::HashMap;

/// Type of borrow (for tracking)
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowKind {
    Immutable,
    Mutable,
}

/// Information about an active borrow
#[derive(Debug, Clone)]
pub(crate) struct Borrow {
    pub kind: BorrowKind,
    pub variable: String, // Which variable is being borrowed from
}

/// Borrow tracking state
#[derive(Debug, Clone)]
pub(crate) struct BorrowState {
    /// Active borrows: reference -> borrow info
    pub active_borrows: HashMap<String, Vec<Borrow>>,

    /// Variables that are currently borrowed (cannot be moved or mutated)
    pub borrowed_vars: HashMap<String, Vec<BorrowKind>>,
}

impl BorrowState {
    pub fn new() -> Self {
        Self {
            active_borrows: HashMap::new(),
            borrowed_vars: HashMap::new(),
        }
    }
}
