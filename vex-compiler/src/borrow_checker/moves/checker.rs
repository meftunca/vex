//! Core MoveChecker structure and item-level checking

use crate::borrow_checker::errors::BorrowResult;
use std::collections::{HashMap, HashSet};
use vex_ast::{Item, Type};

/// Tracks which variables have been moved and are no longer accessible
#[derive(Debug)]
pub struct MoveChecker {
    /// Variables that have been moved (and are now invalid)
    pub(super) moved_vars: HashSet<String>,

    /// Variables that are currently valid
    pub(in crate::borrow_checker) valid_vars: HashSet<String>,

    /// Global variables (extern functions, constants) - never go out of scope
    pub(in crate::borrow_checker) global_vars: HashSet<String>,

    /// Type information for variables (to determine Copy vs Move)
    pub(super) var_types: HashMap<String, Type>,

    /// Builtin function registry for identifying builtin functions
    pub(super) builtin_registry: super::super::builtin_metadata::BuiltinBorrowRegistry,

    /// Current function being checked (for error location tracking)
    pub(super) current_function: Option<String>,
    /// Map of variable -> span_id of the move location (if available)
    pub(super) move_locations: std::collections::HashMap<String, Option<String>>,
}

impl MoveChecker {
    /// Check a top-level item
    pub(super) fn check_item(&mut self, item: &Item) -> BorrowResult<()> {
        match item {
            Item::Function(func) => {
                // Track current function name for error messages
                self.current_function = Some(func.name.clone());

                // Create new scope for function (save only non-global vars)
                let saved_moved = self.moved_vars.clone();
                let saved_types = self.var_types.clone();

                // Save function-local vars only (filter out globals before saving)
                let saved_local_valid: HashSet<String> = self
                    .valid_vars
                    .difference(&self.global_vars)
                    .cloned()
                    .collect();

                // ⭐ CRITICAL FIX: Register receiver (p, self, this, etc.)
                if let Some(ref receiver) = func.receiver {
                    self.valid_vars.insert(receiver.name.clone());
                    self.var_types
                        .insert(receiver.name.clone(), receiver.ty.clone());
                }

                // Function parameters are valid at start
                for param in &func.params {
                    self.valid_vars.insert(param.name.clone());
                    self.var_types.insert(param.name.clone(), param.ty.clone());
                }

                // Check function body
                for stmt in &func.body.statements {
                    self.check_statement(stmt, None)?;
                }

                // Restore local scope (preserve globals)
                self.moved_vars = saved_moved;
                // Restore only local variables - keep global_vars intact
                self.valid_vars = saved_local_valid;
                // Re-add global variables back to valid_vars
                for global in &self.global_vars {
                    self.valid_vars.insert(global.clone());
                }
                self.var_types = saved_types;
                self.current_function = None;

                Ok(())
            }
            Item::ExternBlock(_) => {
                // Extern functions already registered in Phase 0 by BorrowChecker
                // No need to re-register here
                Ok(())
            }
            _ => Ok(()), // No move semantics in type definitions
        }
    }
}
