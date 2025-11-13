// Phase 4: Lifetime Analysis
// Prevents dangling references by tracking variable lifetimes and reference scopes

use crate::borrow_checker::builtin_metadata::BuiltinBorrowRegistry;
use crate::borrow_checker::builtins_list;
use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use std::collections::{HashMap, HashSet};
use vex_ast::*;

/// Tracks the lifetime (scope) of variables and ensures references don't outlive their referents
pub struct LifetimeChecker {
    /// Maps variable names to their scope depth (higher = inner scope)
    pub(super) variable_scopes: HashMap<String, usize>,

    /// Current scope depth (0 = global, 1 = function body, 2+ = nested blocks)
    pub(super) current_scope: usize,

    /// Maps reference variable names to the variable they reference
    /// Example: `let x = 5; let y = &x;` → references["y"] = "x"
    pub(super) references: HashMap<String, String>,

    /// Tracks which variables are currently in scope (for fast lookup)
    pub(super) in_scope: HashSet<String>,

    /// Global variables (extern functions, constants) - never go out of scope
    pub(crate) global_vars: HashSet<String>,

    /// Builtin function registry for identifying builtin functions
    pub(super) builtin_registry: BuiltinBorrowRegistry,

    /// Current function being checked (for error location tracking)
    pub(super) current_function: Option<String>,

    /// Track if we're inside an unsafe block
    pub(super) in_unsafe_block: bool,
}

impl LifetimeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            variable_scopes: HashMap::new(),
            current_scope: 0,
            references: HashMap::new(),
            in_scope: HashSet::new(),
            global_vars: HashSet::new(),
            builtin_registry: BuiltinBorrowRegistry::new(),
            current_function: None,
            in_unsafe_block: false,
        };

        // Register built-in functions as always in scope (scope 0 = global)
        for name in builtins_list::get_builtin_functions() {
            checker.variable_scopes.insert(name.to_string(), 0);
            checker.in_scope.insert(name.to_string());
        }

        checker
    }

    pub fn check_program(&mut self, program: &Program) -> BorrowResult<()> {
        // Phase 1: Register all top-level items (functions, constants, etc.)
        // This allows forward declarations and mutual recursion
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    self.variable_scopes.insert(func.name.clone(), 0);
                    self.in_scope.insert(func.name.clone());
                }
                Item::Const(const_def) => {
                    self.variable_scopes.insert(const_def.name.clone(), 0);
                    self.in_scope.insert(const_def.name.clone());
                }
                Item::Struct(struct_def) => {
                    // Register struct type name
                    self.variable_scopes.insert(struct_def.name.clone(), 0);
                    self.in_scope.insert(struct_def.name.clone());
                }
                Item::Enum(enum_def) => {
                    // Register enum type name
                    self.variable_scopes.insert(enum_def.name.clone(), 0);
                    self.in_scope.insert(enum_def.name.clone());
                }
                _ => {}
            }
        }

        // Phase 2: Check all items
        for item in &program.items {
            self.check_item(item)?;
        }
        Ok(())
    }

    fn check_item(&mut self, item: &Item) -> BorrowResult<()> {
        match item {
            Item::Function(func) => {
                // Function already registered in phase 1
                self.check_function(func)
            }
            Item::Const(_) => {
                // Constants already registered in phase 1
                Ok(())
            }
            _ => Ok(()), // Other items don't have lifetime concerns
        }
    }

    fn check_function(&mut self, func: &Function) -> BorrowResult<()> {
        // Enter function scope (parameters live here)
        self.enter_scope();

        // ⭐ CRITICAL FIX: Method receiver uses custom name (p, self, this, etc.)
        if let Some(ref receiver) = func.receiver {
            self.declare_variable(&receiver.name);
        }

        // Parameters are in function scope (scope 1)
        for param in &func.params {
            self.declare_variable(&param.name);
        }

        // Enter function body scope (local variables live here - scope 2+)
        self.enter_scope();

        // Check function body
        self.check_block(&func.body)?;

        // Exit function body scope
        self.exit_scope();

        // Exit function scope
        self.exit_scope();

        Ok(())
    }

    pub(super) fn check_block(&mut self, block: &Block) -> BorrowResult<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    /// Enter a new scope (block, function, etc.)
    pub(super) fn enter_scope(&mut self) {
        self.current_scope += 1;
    }

    /// Exit current scope and remove all variables declared in it
    pub(super) fn exit_scope(&mut self) {
        // Remove all variables from this scope
        self.variable_scopes.retain(|name, &mut scope| {
            if scope == self.current_scope {
                self.in_scope.remove(name);
                false // Remove from map
            } else {
                true // Keep
            }
        });

        self.current_scope -= 1;
    }

    /// Declare a new variable in current scope
    pub(super) fn declare_variable(&mut self, name: &str) {
        self.variable_scopes
            .insert(name.to_string(), self.current_scope);
        self.in_scope.insert(name.to_string());
    }

    /// Extract and declare variables from a pattern (for match arms)
    pub(super) fn declare_pattern_bindings(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Ident(name) => {
                self.declare_variable(name);
            }
            Pattern::Enum { data, .. } => {
                // Enum variant with data: Result.Ok(val), Option.Some(x), IpAddr.V4(a,b,c,d)
                for pattern in data {
                    self.declare_pattern_bindings(pattern);
                }
            }
            Pattern::Struct { fields, .. } => {
                // Struct destructuring: Point { x, y }
                for (_, field_pattern) in fields {
                    self.declare_pattern_bindings(field_pattern);
                }
            }
            Pattern::Tuple(patterns) => {
                // Tuple destructuring: (a, b, c)
                for p in patterns {
                    self.declare_pattern_bindings(p);
                }
            }
            Pattern::Wildcard => {
                // _ doesn't bind anything
            }
            Pattern::Literal(_) => {
                // Literals don't bind variables
            }
            Pattern::Or(patterns) => {
                // Or pattern: bindings must be consistent across all alternatives
                for p in patterns {
                    self.declare_pattern_bindings(p);
                }
            }
            Pattern::Array { elements, rest } => {
                // Array/Slice pattern: [a, b, ..rest]
                for elem_pattern in elements {
                    self.declare_pattern_bindings(elem_pattern);
                }
                if let Some(rest_name) = rest {
                    if rest_name != "_" {
                        self.declare_variable(rest_name);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_checker_creation() {
        let checker = LifetimeChecker::new();
        assert_eq!(checker.current_scope, 0);
        assert!(checker.variable_scopes.is_empty());
    }
}
