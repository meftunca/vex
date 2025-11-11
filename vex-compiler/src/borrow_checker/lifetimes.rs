// Phase 4: Lifetime Analysis
// Prevents dangling references by tracking variable lifetimes and reference scopes

use crate::borrow_checker::errors::{BorrowError, BorrowResult};
use std::collections::{HashMap, HashSet};
use vex_ast::*;

/// Tracks the lifetime (scope) of variables and ensures references don't outlive their referents
pub struct LifetimeChecker {
    /// Maps variable names to their scope depth (higher = inner scope)
    variable_scopes: HashMap<String, usize>,

    /// Current scope depth (0 = global, 1 = function body, 2+ = nested blocks)
    current_scope: usize,

    /// Maps reference variable names to the variable they reference
    /// Example: `let x = 5; let y = &x;` → references["y"] = "x"
    references: HashMap<String, String>,

    /// Tracks which variables are currently in scope (for fast lookup)
    in_scope: HashSet<String>,

    /// Global variables (extern functions, constants) - never go out of scope
    pub(super) global_vars: HashSet<String>,

    /// Builtin function registry for identifying builtin functions
    builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry,

    /// Current function being checked (for error location tracking)
    current_function: Option<String>,
}

impl LifetimeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            variable_scopes: HashMap::new(),
            current_scope: 0,
            references: HashMap::new(),
            in_scope: HashSet::new(),
            global_vars: HashSet::new(),
            builtin_registry: super::builtin_metadata::BuiltinBorrowRegistry::new(),
            current_function: None,
        };

        // Register built-in functions as always in scope (scope 0 = global)
        let builtins = [
            // Core builtins
            "print",
            "println",
            "panic",
            "assert",
            "unreachable",
            // Memory builtins
            "alloc",
            "free",
            "realloc",
            "sizeof",
            "alignof",
            // Bit manipulation
            "ctlz",
            "cttz",
            "ctpop",
            "bswap",
            "bitreverse",
            // Overflow checking
            "sadd_overflow",
            "ssub_overflow",
            "smul_overflow",
            // Compiler hints
            "assume",
            "likely",
            "unlikely",
            "prefetch",
            // String functions
            "strlen",
            "strcmp",
            "strcpy",
            "strcat",
            "strdup",
            // Memory operations
            "memcpy",
            "memset",
            "memcmp",
            "memmove",
            // UTF-8 functions
            "utf8_valid",
            "utf8_char_count",
            "utf8_char_at",
            // Array functions
            "array_len",
            "array_get",
            "array_set",
            "array_append",
            // Type reflection
            "typeof",
            "type_id",
            "type_size",
            "type_align",
            "is_int_type",
            "is_float_type",
            "is_pointer_type",
            // HashMap functions
            "hashmap_new",
            "hashmap_insert",
            "hashmap_get",
            "hashmap_len",
            "hashmap_free",
            "hashmap_contains",
            "hashmap_remove",
            "hashmap_clear",
            // Phase 0.4b: Builtin type constructors
            "vec_new",
            "vec_free",
            "box_new",
            "box_free",
            // Phase 0.7: Primitive to string conversions
            "vex_i32_to_string",
            "vex_i64_to_string",
            "vex_u32_to_string",
            "vex_u64_to_string",
            "vex_f32_to_string",
            "vex_f64_to_string",
            "vex_bool_to_string",
            "vex_string_to_string",
        ];

        for name in &builtins {
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

        // Method receiver (self) is also a parameter in function scope
        if func.receiver.is_some() {
            self.declare_variable("self");
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

    fn check_block(&mut self, block: &Block) -> BorrowResult<()> {
        for statement in &block.statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> BorrowResult<()> {
        match stmt {
            Statement::Let {
                name,
                value,
                is_mutable: _,
                ty: _,
            } => {
                // Check the value expression first
                self.check_expression(value)?;

                // Check if this is a reference assignment
                if let Expression::Reference { expr: ref_expr, .. } = value {
                    // Track what this reference points to
                    if let Expression::Ident(target) = ref_expr.as_ref() {
                        self.references.insert(name.clone(), target.clone());

                        // Verify the target is still in scope
                        if !self.in_scope.contains(target) {
                            return Err(BorrowError::DanglingReference {
                                reference: name.clone(),
                                referent: target.clone(),
                            });
                        }
                    }
                }

                // Declare the new variable
                self.declare_variable(name);
                Ok(())
            }

            Statement::Assign { target, value } => {
                self.check_expression(target)?;
                self.check_expression(value)?;

                // Track reference assignments (e.g., `ref_var = &local;`)
                if let Expression::Ident(var_name) = target {
                    if let Expression::Reference { expr: ref_expr, .. } = value {
                        if let Expression::Ident(target_name) = ref_expr.as_ref() {
                            // Check if target is still in scope
                            if let Some(&target_scope) = self.variable_scopes.get(target_name) {
                                // If target is in a deeper scope than the reference variable,
                                // this will create a dangling reference when target goes out of scope
                                if let Some(&ref_scope) = self.variable_scopes.get(var_name) {
                                    if target_scope > ref_scope {
                                        return Err(BorrowError::DanglingReference {
                                            reference: var_name.clone(),
                                            referent: target_name.clone(),
                                        });
                                    }
                                }
                            }

                            // Update reference tracking
                            self.references
                                .insert(var_name.clone(), target_name.clone());
                        }
                    }
                }
                Ok(())
            }

            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expression(e)?;

                    // CRITICAL: Check if returning a reference to local variable
                    if let Expression::Reference { expr: ref_expr, .. } = e {
                        if let Expression::Ident(var_name) = ref_expr.as_ref() {
                            // Check if the variable is local (not a parameter)
                            if let Some(&scope) = self.variable_scopes.get(var_name) {
                                // scope 1 = function params (OK to return)
                                // scope 2+ = local variables (ERROR - will be dropped)
                                if scope >= 2 {
                                    return Err(BorrowError::ReturnDanglingReference {
                                        variable: var_name.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(())
            }

            Statement::If {
                span_id: _,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.check_expression(condition)?;

                // Each branch is a new scope
                self.enter_scope();
                self.check_block(then_block)?;
                self.exit_scope();

                for (elif_cond, elif_body) in elif_branches {
                    self.check_expression(elif_cond)?;
                    self.enter_scope();
                    self.check_block(elif_body)?;
                    self.exit_scope();
                }

                if let Some(else_body) = else_block {
                    self.enter_scope();
                    self.check_block(else_body)?;
                    self.exit_scope();
                }

                Ok(())
            }

            Statement::While {
                span_id: _,
                condition,
                body,
            } => {
                self.check_expression(condition)?;
                self.enter_scope();
                self.check_block(body)?;
                self.exit_scope();
                Ok(())
            }

            Statement::For {
                span_id: _,
                init,
                condition,
                post,
                body,
            } => {
                self.enter_scope();

                // Check init statement if present
                if let Some(init_stmt) = init {
                    self.check_statement(init_stmt)?;
                }

                // Check condition if present
                if let Some(cond) = condition {
                    self.check_expression(cond)?;
                }

                // Check body
                self.check_block(body)?;

                // Check post statement if present
                if let Some(post_stmt) = post {
                    self.check_statement(post_stmt)?;
                }

                self.exit_scope();
                Ok(())
            }

            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                self.check_expression(iterable)?;
                self.enter_scope();
                self.declare_variable(variable);
                self.check_block(body)?;
                self.exit_scope();
                Ok(())
            }

            Statement::Switch {
                value,
                cases,
                default_case,
            } => {
                if let Some(val) = value {
                    self.check_expression(val)?;
                }

                for case in cases {
                    // Check all pattern expressions
                    for pattern_expr in &case.patterns {
                        self.check_expression(pattern_expr)?;
                    }

                    self.enter_scope();
                    self.check_block(&case.body)?;
                    self.exit_scope();
                }

                if let Some(default) = default_case {
                    self.enter_scope();
                    self.check_block(default)?;
                    self.exit_scope();
                }

                Ok(())
            }

            Statement::Expression(expr) => self.check_expression(expr),

            Statement::CompoundAssign { target, value, .. } => {
                self.check_expression(target)?;
                self.check_expression(value)
            }

            Statement::Select { .. } => {
                // TODO: Implement select case checking when async is ready
                Ok(())
            }

            Statement::Unsafe(block) => {
                // Check unsafe block content
                self.check_block(block)
            }

            Statement::Defer(_) | Statement::Go(_) | Statement::Break | Statement::Continue => {
                Ok(())
            }
        }
    }

    fn check_expression(&mut self, expr: &Expression) -> BorrowResult<()> {
        match expr {
            Expression::Ident(name) => {
                // Skip checking builtin functions/types as variables
                if self.builtin_registry.is_builtin(name) {
                    return Ok(());
                }

                // Global variables (extern functions) are always in scope
                if self.global_vars.contains(name) {
                    return Ok(());
                }

                // Skip builtin type names (Vec, Box, Map, etc.) - O(1) hash lookup
                // These are used in static method calls like Vec.new()
                if crate::type_registry::is_builtin_type(name) {
                    return Ok(());
                }

                // Verify variable is in scope
                if !self.in_scope.contains(name) {
                    // Collect available names for fuzzy matching
                    let available_names: Vec<String> = self.in_scope.iter().cloned().collect();

                    return Err(BorrowError::UseAfterScopeEnd {
                        variable: name.clone(),
                        available_names,
                    });
                }
                Ok(())
            }

            Expression::Reference { expr: ref_expr, .. } => self.check_expression(ref_expr),

            Expression::Deref(expr) => self.check_expression(expr),

            Expression::Binary {
                span_id: _,
                left,
                right,
                ..
            } => {
                self.check_expression(left)?;
                self.check_expression(right)
            }

            Expression::Unary {
                span_id: _, expr, ..
            } => self.check_expression(expr),

            Expression::Call { func, args, .. } => {
                // Skip checking builtin function names as variables
                if let Expression::Ident(func_name) = func.as_ref() {
                    if !self.builtin_registry.is_builtin(func_name) {
                        self.check_expression(func)?;
                    }
                } else {
                    self.check_expression(func)?;
                }

                // Validate reference arguments
                for arg in args {
                    // If passing a reference to a local variable, ensure it's valid
                    if let Expression::Reference { expr: ref_expr, .. } = arg {
                        if let Expression::Ident(var_name) = ref_expr.as_ref() {
                            // Check if the variable is still in scope
                            if !self.in_scope.contains(var_name) {
                                let available_names: Vec<String> =
                                    self.in_scope.iter().cloned().collect();
                                return Err(BorrowError::UseAfterScopeEnd {
                                    variable: var_name.clone(),
                                    available_names,
                                });
                            }
                        }
                    }
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::MethodCall { receiver, args, .. } => {
                self.check_expression(receiver)?;

                // Validate reference arguments in method calls
                for arg in args {
                    if let Expression::Reference { expr: ref_expr, .. } = arg {
                        if let Expression::Ident(var_name) = ref_expr.as_ref() {
                            if !self.in_scope.contains(var_name) {
                                let available_names: Vec<String> =
                                    self.in_scope.iter().cloned().collect();
                                return Err(BorrowError::UseAfterScopeEnd {
                                    variable: var_name.clone(),
                                    available_names,
                                });
                            }
                        }
                    }
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::FieldAccess { object, .. } => self.check_expression(object),

            Expression::Index { object, index } => {
                self.check_expression(object)?;
                self.check_expression(index)
            }

            Expression::Array(elements) => {
                for elem in elements {
                    self.check_expression(elem)?;
                }
                Ok(())
            }

            Expression::ArrayRepeat(value, count) => {
                self.check_expression(value)?;
                self.check_expression(count)?;
                Ok(())
            }

            Expression::TupleLiteral(elements) => {
                for elem in elements {
                    self.check_expression(elem)?;
                }
                Ok(())
            }

            Expression::StructLiteral { fields, .. } => {
                for (_, expr) in fields {
                    self.check_expression(expr)?;
                }
                Ok(())
            }

            Expression::MapLiteral(entries) => {
                for (key, value) in entries {
                    self.check_expression(key)?;
                    self.check_expression(value)?;
                }
                Ok(())
            }

            Expression::Match { value, arms } => {
                self.check_expression(value)?;

                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.check_expression(guard)?;
                    }

                    // Each match arm is a new scope (for pattern bindings)
                    self.enter_scope();

                    // Extract and declare pattern bindings in this scope
                    self.declare_pattern_bindings(&arm.pattern);

                    self.check_expression(&arm.body)?;
                    self.exit_scope();
                }

                Ok(())
            }

            Expression::Block {
                statements,
                return_expr,
            } => {
                self.enter_scope();

                for stmt in statements {
                    self.check_statement(stmt)?;
                }

                if let Some(expr) = return_expr {
                    self.check_expression(expr)?;
                }

                self.exit_scope();
                Ok(())
            }

            // Literals have no lifetime concerns
            Expression::IntLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::FStringLiteral(_)
            | Expression::BoolLiteral(_)
            | Expression::Nil => Ok(()),

            Expression::EnumLiteral { data, .. } => {
                for expr in data {
                    self.check_expression(expr)?;
                }
                Ok(())
            }

            Expression::Range { start, end } | Expression::RangeInclusive { start, end } => {
                if let Some(s) = start {
                    self.check_expression(s)?;
                }
                if let Some(e) = end {
                    self.check_expression(e)?;
                }
                Ok(())
            }

            Expression::PostfixOp { expr, .. } => self.check_expression(expr),

            Expression::Await(expr)
            | Expression::QuestionMark(expr)
            | Expression::ChannelReceive(expr) => self.check_expression(expr),

            Expression::Launch { args, .. } => {
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::New(expr) => self.check_expression(expr),

            Expression::Make { size, .. } => self.check_expression(size),

            Expression::Cast { expr, .. } => self.check_expression(expr),

            Expression::ErrorNew(expr) => self.check_expression(expr),

            Expression::Typeof(expr) => {
                // typeof is compile-time, just check the inner expression
                self.check_expression(expr)
            }

            Expression::Closure { params, body, .. } => {
                // Enter a new scope for closure parameters
                self.enter_scope();

                // Register closure parameters in scope
                for param in params {
                    self.variable_scopes
                        .insert(param.name.clone(), self.current_scope);
                    self.in_scope.insert(param.name.clone());
                }

                // Check closure body with parameters in scope
                self.check_expression(body)?;

                // Exit closure scope
                self.exit_scope();

                Ok(())
            }

            Expression::TypeConstructor { args, .. } => {
                // Check all constructor arguments
                for arg in args {
                    self.check_expression(arg)?;
                }
                Ok(())
            }

            Expression::BigIntLiteral(_) => Ok(()), // Literals are always valid
        }
    }

    /// Enter a new scope (block, function, etc.)
    fn enter_scope(&mut self) {
        self.current_scope += 1;
    }

    /// Exit current scope and remove all variables declared in it
    fn exit_scope(&mut self) {
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
    fn declare_variable(&mut self, name: &str) {
        self.variable_scopes
            .insert(name.to_string(), self.current_scope);
        self.in_scope.insert(name.to_string());
    }

    /// Extract and declare variables from a pattern (for match arms)
    fn declare_pattern_bindings(&mut self, pattern: &Pattern) {
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
