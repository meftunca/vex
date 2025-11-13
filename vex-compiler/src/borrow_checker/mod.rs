// Vex Borrow Checker Module
// Phase 1: Basic Immutability Check
// Phase 2: Move Semantics
// Phase 3: Borrow Rules
// Phase 4: Lifetime Analysis

pub mod borrows;
pub mod builtin_metadata;
pub mod builtins_list;
pub mod closure_traits;
pub mod errors;
pub mod immutability;
pub mod lifetimes;
pub mod moves;

pub use borrows::BorrowRulesChecker;
pub use builtin_metadata::{BuiltinBorrowRegistry, BuiltinMetadata, ParamEffect};
pub use closure_traits::{analyze_closure_body, analyze_closure_trait};
pub use errors::{BorrowError, BorrowResult};
pub use immutability::ImmutabilityChecker;
pub use lifetimes::LifetimeChecker;
pub use moves::MoveChecker;

use vex_ast::{Expression, Item, Program, Statement};

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
    pub fn check_program(&mut self, program: &mut Program) -> BorrowResult<()> {
        // Phase 0.1: Register imported symbols (they're global and always valid)
        for import in &program.imports {
            // Register all imported names as global symbols
            for name in &import.items {
                self.moves.global_vars.insert(name.clone());
                self.moves.valid_vars.insert(name.clone());
                self.borrows.valid_vars.insert(name.clone());
                self.lifetimes.global_vars.insert(name.clone());
            }
        }

        // Phase 0.2: Register global symbols (extern functions + top-level functions + constants)
        // These are always valid and never go out of scope
        for item in &program.items {
            match item {
                Item::ExternBlock(block) => {
                    // Register extern "C" functions
                    for func in &block.functions {
                        self.moves.global_vars.insert(func.name.clone());
                        self.moves.valid_vars.insert(func.name.clone());
                        self.borrows.valid_vars.insert(func.name.clone());
                        self.lifetimes.global_vars.insert(func.name.clone());
                    }
                }
                Item::Function(func) => {
                    // Register top-level functions (including imported ones)
                    // These are global symbols and never go out of scope
                    self.moves.global_vars.insert(func.name.clone());
                    self.moves.valid_vars.insert(func.name.clone());
                    self.borrows.valid_vars.insert(func.name.clone());
                    self.lifetimes.global_vars.insert(func.name.clone());
                }
                Item::Const(const_decl) => {
                    // Register constants (they're immutable globals)
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

    /// Phase 5: Analyze all closures and set their capture modes
    /// This determines which trait each closure implements:
    /// - Callable: Immutable capture (can call multiple times)
    /// - CallableMut: Mutable capture (can call multiple times, mutates environment)
    /// - CallableOnce: Move capture (can only call once, takes ownership)
    fn analyze_closure_traits(&mut self, program: &mut Program) -> BorrowResult<()> {
        for item in &mut program.items {
            self.analyze_item_closures(item)?;
        }
        Ok(())
    }

    /// Recursively analyze closures in an item
    fn analyze_item_closures(&mut self, item: &mut Item) -> BorrowResult<()> {
        match item {
            Item::Function(func) => {
                for stmt in &mut func.body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
                Ok(())
            }
            Item::TraitImpl(trait_impl) => {
                for func in &mut trait_impl.methods {
                    for stmt in &mut func.body.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Recursively analyze closures in a statement
    fn analyze_statement_closures(&mut self, stmt: &mut Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let { value, .. } => {
                self.analyze_expression_closures(value)?;
            }
            Statement::LetPattern { value, .. } => {
                self.analyze_expression_closures(value)?;
            }
            Statement::Assign { target, value } => {
                self.analyze_expression_closures(target)?;
                self.analyze_expression_closures(value)?;
            }
            Statement::Return(Some(expr)) => {
                self.analyze_expression_closures(expr)?;
            }
            Statement::Expression(expr) => {
                self.analyze_expression_closures(expr)?;
            }
            Statement::If {
                span_id: _,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.analyze_expression_closures(condition)?;
                for stmt in &mut then_block.statements {
                    self.analyze_statement_closures(stmt)?;
                }
                for (cond, block) in elif_branches {
                    self.analyze_expression_closures(cond)?;
                    for stmt in &mut block.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
                if let Some(block) = else_block {
                    for stmt in &mut block.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
            }
            Statement::While {
                span_id: _,
                condition,
                body,
            } => {
                self.analyze_expression_closures(condition)?;
                for stmt in &mut body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
            }
            Statement::Loop { body } => {
                for stmt in &mut body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
            }
            Statement::For {
                span_id: _,
                init,
                condition,
                post,
                body,
            } => {
                if let Some(init_stmt) = init {
                    self.analyze_statement_closures(init_stmt)?;
                }
                if let Some(cond) = condition {
                    self.analyze_expression_closures(cond)?;
                }
                if let Some(post_stmt) = post {
                    self.analyze_statement_closures(post_stmt)?;
                }
                for stmt in &mut body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
            }
            Statement::ForIn { iterable, body, .. } => {
                self.analyze_expression_closures(iterable)?;
                for stmt in &mut body.statements {
                    self.analyze_statement_closures(stmt)?;
                }
            }
            Statement::Switch {
                value,
                cases,
                default_case,
            } => {
                if let Some(val) = value {
                    self.analyze_expression_closures(val)?;
                }
                for case in cases {
                    for stmt in &mut case.body.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
                if let Some(default_block) = default_case {
                    for stmt in &mut default_block.statements {
                        self.analyze_statement_closures(stmt)?;
                    }
                }
            }
            Statement::Select { .. } | Statement::Unsafe(_) => {
                // These don't typically contain user closures
            }
            Statement::Defer(stmt) => {
                self.analyze_statement_closures(stmt)?;
            }
            Statement::Go(expr) => {
                self.analyze_expression_closures(expr)?;
            }
            Statement::CompoundAssign { target, value, .. } => {
                self.analyze_expression_closures(target)?;
                self.analyze_expression_closures(value)?;
            }
            Statement::Break | Statement::Continue | Statement::Return(None) => {
                // No expressions to analyze
            }
        }
        Ok(())
    }

    /// Recursively analyze closures in an expression
    fn analyze_expression_closures(&mut self, expr: &mut Expression) -> BorrowResult<()> {
        match expr {
            Expression::Closure {
                params,
                body,
                capture_mode,
                ..
            } => {
                // First, recursively analyze nested closures in the body
                self.analyze_expression_closures(body)?;

                // Then analyze this closure and determine its capture mode
                let analyzed_mode = analyze_closure_body(params, body);

                // Update the capture mode from Infer to the analyzed result
                *capture_mode = analyzed_mode;

                Ok(())
            }
            Expression::Binary {
                span_id: _,
                left,
                right,
                ..
            } => {
                self.analyze_expression_closures(left)?;
                self.analyze_expression_closures(right)?;
                Ok(())
            }
            Expression::Unary {
                span_id: _, expr, ..
            } => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::Call { func, args, .. } => {
                self.analyze_expression_closures(func)?;
                for arg in args {
                    self.analyze_expression_closures(arg)?;
                }
                Ok(())
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.analyze_expression_closures(receiver)?;
                for arg in args {
                    self.analyze_expression_closures(arg)?;
                }
                Ok(())
            }
            Expression::FieldAccess { object, .. } => {
                self.analyze_expression_closures(object)?;
                Ok(())
            }
            Expression::Index { object, index } => {
                self.analyze_expression_closures(object)?;
                self.analyze_expression_closures(index)?;
                Ok(())
            }
            Expression::Array(elements) => {
                for elem in elements {
                    self.analyze_expression_closures(elem)?;
                }
                Ok(())
            }
            Expression::ArrayRepeat(value, count) => {
                self.analyze_expression_closures(value)?;
                self.analyze_expression_closures(count)?;
                Ok(())
            }
            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.analyze_expression_closures(elem)?;
                }
                Ok(())
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.analyze_expression_closures(value)?;
                }
                Ok(())
            }
            Expression::MapLiteral(entries) => {
                for (key, value) in entries {
                    self.analyze_expression_closures(key)?;
                    self.analyze_expression_closures(value)?;
                }
                Ok(())
            }
            Expression::EnumLiteral { data, .. } => {
                for data_expr in data {
                    self.analyze_expression_closures(data_expr)?;
                }
                Ok(())
            }
            Expression::Block {
                statements,
                return_expr,
            } => {
                for stmt in statements {
                    self.analyze_statement_closures(stmt)?;
                }
                if let Some(expr) = return_expr {
                    self.analyze_expression_closures(expr)?;
                }
                Ok(())
            }
            Expression::Match { value, arms } => {
                self.analyze_expression_closures(value)?;
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        self.analyze_expression_closures(guard)?;
                    }
                    self.analyze_expression_closures(&mut arm.body)?;
                }
                Ok(())
            }
            Expression::Reference { expr, .. } => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::Range { start, end } | Expression::RangeInclusive { start, end } => {
                if let Some(s) = start {
                    self.analyze_expression_closures(s)?;
                }
                if let Some(e) = end {
                    self.analyze_expression_closures(e)?;
                }
                Ok(())
            }
            Expression::Await(expr)
            | Expression::QuestionMark(expr)
            | Expression::New(expr)
            | Expression::Deref(expr)
            | Expression::ChannelReceive(expr)
            | Expression::ErrorNew(expr) => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::Cast { expr, .. } => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::PostfixOp { expr, .. } => {
                self.analyze_expression_closures(expr)?;
                Ok(())
            }
            Expression::Make { size, .. } => {
                self.analyze_expression_closures(size)?;
                Ok(())
            }
            Expression::Launch { grid, args, .. } => {
                for expr in grid {
                    self.analyze_expression_closures(expr)?;
                }
                for arg in args {
                    self.analyze_expression_closures(arg)?;
                }
                Ok(())
            }
            // Literals and identifiers have no nested closures
            Expression::IntLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::FStringLiteral(_)
            | Expression::BoolLiteral(_)
            | Expression::Nil
            | Expression::Ident(_) => Ok(()),

            Expression::Typeof(expr) => {
                // Check nested closures in typeof expression
                self.analyze_expression_closures(expr)
            }

            Expression::TypeConstructor { args, .. } => {
                // Check nested closures in constructor arguments
                for arg in args {
                    self.analyze_expression_closures(arg)?;
                }
                Ok(())
            }
        }
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
