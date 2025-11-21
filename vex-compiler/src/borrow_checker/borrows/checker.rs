//! Main borrow checker structure and core logic

use super::tracking::{Borrow, BorrowKind};
use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use std::collections::{HashMap, HashSet};
use vex_ast::{Expression, Item};

/// Tracks active borrows to enforce borrow rules
#[derive(Debug)]
pub struct BorrowRulesChecker {
    /// Active borrows: reference -> borrow info
    pub(super) active_borrows: HashMap<String, Vec<Borrow>>,

    /// Variables that are currently borrowed (cannot be moved or mutated)
    pub(super) borrowed_vars: HashMap<String, Vec<BorrowKind>>,

    /// Valid variables (includes extern functions)
    pub(in crate::borrow_checker) valid_vars: HashSet<String>,

    /// Registry of builtin function metadata
    pub(super) builtin_registry: super::super::builtin_metadata::BuiltinBorrowRegistry,

    /// Current function being checked (for error location tracking)
    pub(super) current_function: Option<String>,

    /// Track if we're inside an unsafe block
    pub(super) in_unsafe_block: bool,
}

impl BorrowRulesChecker {
    /// Check a top-level item
    pub(super) fn check_item(&mut self, item: &Item) -> BorrowResult<()> {
        match item {
            Item::Function(func) => {
                // Track current function name for error messages
                self.current_function = Some(func.name.clone());

                // Create new scope for function
                let saved_borrows = self.active_borrows.clone();
                let saved_borrowed = self.borrowed_vars.clone();

                // Check function body
                for stmt in &func.body.statements {
                    self.check_statement(stmt, None)?;
                }

                // Restore scope
                self.active_borrows = saved_borrows;
                self.borrowed_vars = saved_borrowed;

                Ok(())
            }
            _ => Ok(()), // No borrow rules in type definitions
        }
    }

    /// Check if a variable can be borrowed with the given kind
    pub(super) fn check_can_borrow(&self, var: &str, kind: BorrowKind, _new_borrow_loc: Option<String>) -> BorrowResult<()> {
        if let Some(existing_borrows) = self.borrowed_vars.get(var) {
            match kind {
                BorrowKind::Mutable => {
                    // Cannot borrow as mutable if ANY borrows exist
                    if !existing_borrows.is_empty() {
                        // Try to find a borrow location for an existing borrow (if recorded)
                        let mut existing_loc: Option<String> = None;
                        for (_ref_name, borrows) in &self.active_borrows {
                            for b in borrows {
                                if b.variable == var {
                                    if let Some(loc) = &b.location {
                                        existing_loc = Some(loc.clone());
                                        break;
                                    }
                                }
                            }
                            if existing_loc.is_some() {
                                break;
                            }
                        }
                        if existing_borrows.contains(&BorrowKind::Mutable) {
                            return Err(BorrowError::MutableBorrowWhileBorrowed {
                                variable: var.to_string(),
                                existing_borrow: existing_loc,
                                new_borrow: _new_borrow_loc.clone(),
                            });
                        } else {
                            return Err(BorrowError::MutableBorrowWhileBorrowed {
                                variable: var.to_string(),
                                existing_borrow: existing_loc.or(Some("immutably borrowed".to_string())),
                                new_borrow: _new_borrow_loc.clone(),
                            });
                        }
                    }
                }
                BorrowKind::Immutable => {
                    // Cannot borrow as immutable if mutable borrow exists
                    if existing_borrows.contains(&BorrowKind::Mutable) {
                        // Find an active mutable borrow location for better diagnostics
                        let mut mutable_loc: Option<String> = None;
                        for (_ref_name, borrows) in &self.active_borrows {
                            for b in borrows {
                                if b.variable == var && b.kind == BorrowKind::Mutable {
                                    if let Some(loc) = &b.location {
                                        mutable_loc = Some(loc.clone());
                                        break;
                                    }
                                }
                            }
                            if mutable_loc.is_some() {
                                break;
                            }
                        }
                        return Err(BorrowError::ImmutableBorrowWhileMutableBorrowed {
                            variable: var.to_string(),
                            mutable_borrow: mutable_loc,
                            new_borrow: None,
                        });
                    }
                    // Multiple immutable borrows are OK
                }
            }
        }

        Ok(())
    }

    /// Create a new borrow
    pub(super) fn create_borrow(
        &mut self,
        ref_name: String,
        var: String,
        kind: BorrowKind,
        location: Option<String>,
    ) -> BorrowResult<()> {
        // First check if this borrow is allowed
        self.check_can_borrow(&var, kind.clone(), location.clone())?;

        // Track the borrow
        self.active_borrows
            .entry(ref_name)
            .or_insert_with(Vec::new)
            .push(Borrow {
                kind: kind.clone(),
                variable: var.clone(),
                location,
            });

        // Mark the variable as borrowed
        self.borrowed_vars
            .entry(var)
            .or_insert_with(Vec::new)
            .push(kind);

        Ok(())
    }

    /// Check if an expression is likely a raw pointer
    /// Heuristic: check if it's a cast to ptr or a call to alloc/malloc
    pub(super) fn is_likely_raw_pointer(expr: &Expression) -> bool {
        match expr {
            Expression::Cast { target_type, .. } => {
                // Check if casting to ptr/RawPtr type
                matches!(target_type, vex_ast::Type::RawPtr { .. })
            }
            Expression::Call { func, .. } => {
                // Check if calling alloc/malloc/etc
                if let Expression::Ident(name) = func.as_ref() {
                    matches!(name.as_str(), "malloc" | "alloc" | "realloc" | "calloc")
                } else {
                    false
                }
            }
            Expression::Ident(_) => {
                // Could be a raw pointer variable, but we can't tell without type info
                // Be conservative - don't require unsafe for simple idents
                false
            }
            _ => false,
        }
    }
}
