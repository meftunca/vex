// Borrow Checker Orchestrator
// Main coordination logic for all borrow checking phases

use crate::borrow_checker::borrows::BorrowRulesChecker;
use crate::borrow_checker::errors::BorrowResult;
use crate::borrow_checker::immutability::ImmutabilityChecker;
use crate::borrow_checker::lifetimes::LifetimeChecker;
use crate::borrow_checker::moves::MoveChecker;
use vex_ast::{ImportKind, Item, Program};

/// Main borrow checker that orchestrates all phases
pub struct BorrowChecker {
    pub(super) immutability: ImmutabilityChecker,
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
    pub fn check_program(&mut self, program: &mut Program) -> BorrowResult<()> {
        // Phase 0.1: Register imported symbols (they're global and always valid)
        eprintln!("🔍 Borrow checker: Registering {} imports", program.imports.len());
        for import in &program.imports {
            match &import.kind {
                ImportKind::Named => {
                    // Register all named imported symbols as global
                    eprintln!("   - Named import: {} items", import.items.len());
                    for name in &import.items {
                        eprintln!("      + {}", name);
                        self.moves.global_vars.insert(name.clone());
                        self.moves.valid_vars.insert(name.clone());
                        self.borrows.valid_vars.insert(name.clone());
                        self.lifetimes.global_vars.insert(name.clone());
                    }
                }
                ImportKind::Namespace(alias) => {
                    // Register the namespace alias itself as a global symbol
                    eprintln!("   - Namespace import: {} (alias)", alias);
                    self.moves.global_vars.insert(alias.clone());
                    self.moves.valid_vars.insert(alias.clone());
                    self.borrows.valid_vars.insert(alias.clone());
                    self.lifetimes.global_vars.insert(alias.clone());
                }
                ImportKind::Module => {
                    // Module import - all imported names are registered individually
                    eprintln!("   - Module import: {} items", import.items.len());
                    for name in &import.items {
                        eprintln!("      + {}", name);
                        self.moves.global_vars.insert(name.clone());
                        self.moves.valid_vars.insert(name.clone());
                        self.borrows.valid_vars.insert(name.clone());
                        self.lifetimes.global_vars.insert(name.clone());
                    }
                }
            }
        }

        // Phase 0.2: Register global symbols (extern functions + top-level functions + constants)
        // These are always valid and never go out of scope
        eprintln!("🔍 Borrow checker: Registering {} top-level items", program.items.len());
        for item in &program.items {
            match item {
                Item::ExternBlock(block) => {
                    // Register extern "C" functions
                    eprintln!("   📦 ExternBlock with {} functions", block.functions.len());
                    for func in &block.functions {
                        eprintln!("      + extern function: {}", func.name);
                        self.moves.global_vars.insert(func.name.clone());
                        self.moves.valid_vars.insert(func.name.clone());
                        self.borrows.valid_vars.insert(func.name.clone());
                        self.lifetimes.global_vars.insert(func.name.clone());
                    }
                }
                Item::Function(func) => {
                    // Register top-level functions (including imported ones)
                    // These are global symbols and never go out of scope
                    eprintln!("   + function: {}", func.name);
                    self.moves.global_vars.insert(func.name.clone());
                    self.moves.valid_vars.insert(func.name.clone());
                    self.borrows.valid_vars.insert(func.name.clone());
                    self.lifetimes.global_vars.insert(func.name.clone());
                }
                Item::Const(const_decl) => {
                    // Register constants (they're immutable globals)
                    eprintln!("   + const: {}", const_decl.name);
                    self.moves.global_vars.insert(const_decl.name.clone());
                    self.moves.valid_vars.insert(const_decl.name.clone());
                    self.borrows.valid_vars.insert(const_decl.name.clone());
                    self.lifetimes.global_vars.insert(const_decl.name.clone());
                }
                _ => {}
            }
        }

        // Phase 1: Check immutability violations
        self.immutability.check_program(program)?;

        // Phase 2: Check move semantics (use-after-move)
        self.moves.check_program(program)?;

        // Phase 3: Check borrow rules (1 mutable XOR N immutable)
        self.borrows.check_program(program)?;

        // Phase 4: Lifetime analysis (dangling references)
        self.lifetimes.check_program(program)?;

        // Phase 5: Analyze closure capture modes (determine Callable/CallableMut/CallableOnce)
        self.analyze_closure_traits(program)?;

        Ok(())
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}
