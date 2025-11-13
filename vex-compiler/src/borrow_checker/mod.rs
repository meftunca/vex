// Vex Borrow Checker Module
// Phase 1: Basic Immutability Check
// Phase 2: Move Semantics
// Phase 3: Borrow Rules
// Phase 4: Lifetime Analysis

pub mod borrows;
pub mod builtin_metadata;
pub mod builtins_list;
pub mod closure_analysis;
pub mod closure_traits;
pub mod errors;
pub mod immutability;
pub mod lifetimes;
pub mod moves;
pub mod orchestrator;

pub use borrows::BorrowRulesChecker;
pub use builtin_metadata::{BuiltinBorrowRegistry, BuiltinMetadata, ParamEffect};
pub use closure_traits::{analyze_closure_body, analyze_closure_trait};
pub use errors::{BorrowError, BorrowResult};
pub use immutability::ImmutabilityChecker;
pub use lifetimes::LifetimeChecker;
pub use moves::MoveChecker;
pub use orchestrator::BorrowChecker;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_checker_creation() {
        let checker = BorrowChecker::new();
        assert!(checker.immutability.immutable_vars.is_empty());
    }
}
