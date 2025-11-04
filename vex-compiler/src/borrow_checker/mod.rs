// Vex Borrow Checker Module
// Phase 1: Basic Immutability Check
// Phase 2: Move Semantics
// Phase 3: Borrow Rules
// Phase 4: Lifetime Analysis

pub mod borrows;
pub mod builtin_metadata;
pub mod errors;
pub mod immutability;
pub mod lifetimes;
pub mod moves;

pub use borrows::BorrowRulesChecker;
pub use builtin_metadata::{BuiltinBorrowRegistry, BuiltinMetadata, ParamEffect};
pub use errors::{BorrowError, BorrowResult};
pub use immutability::ImmutabilityChecker;
pub use lifetimes::LifetimeChecker;
pub use moves::MoveChecker;

use vex_ast::Program;

/// Main borrow checker that orchestrates all phases
pub struct BorrowChecker {
    immutability: ImmutabilityChecker,
    moves: MoveChecker,
    borrows: BorrowRulesChecker,
    lifetimes: LifetimeChecker,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            immutability: ImmutabilityChecker::new(),
            moves: MoveChecker::new(),
            borrows: BorrowRulesChecker::new(),
            lifetimes: LifetimeChecker::new(),
        }
    }

    /// Run all borrow checking phases on a program
    pub fn check_program(&mut self, program: &Program) -> BorrowResult<()> {
        // Phase 1: Check immutability violations
        self.immutability.check_program(program)?;

        // Phase 2: Check move semantics (use-after-move)
        self.moves.check_program(program)?;

        // Phase 3: Check borrow rules (1 mutable XOR N immutable)
        self.borrows.check_program(program)?;

        // Phase 4: Lifetime analysis (dangling references)
        self.lifetimes.check_program(program)?;

        Ok(())
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_checker_creation() {
        let checker = BorrowChecker::new();
        assert!(checker.immutability.immutable_vars.is_empty());
    }
}
