// Unused variable detection - finds variables that are declared but never used
// This helps catch typos and improve code quality

use super::LintRule;
use vex_ast::{Item, Program, Statement, Expression, Function, Pattern};
use vex_diagnostics::Diagnostic;
use std::collections::{HashSet, HashMap};

/// Lint rule for detecting unused variables
pub struct UnusedVariableRule {
    /// Variables that should be ignored (start with _)
    ignore_underscore: bool,
}

impl UnusedVariableRule {
    pub fn new() -> Self {
        Self {
            ignore_underscore: true,
        }
    }
    
    /// Check a function for unused variables
    fn check_function(&self, func: &Function) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Track declared and used variables
        let mut declared: HashMap<String, usize> = HashMap::new(); // name -> declaration count
        let mut used: HashSet<String> = HashSet::new();
        
        // Collect parameter names as declared
        for param in &func.params {
            let name = &param.name;
            // Skip variables starting with _
            if self.ignore_underscore && name.starts_with('_') {
                continue;
            }
            *declared.entry(name.clone()).or_insert(0) += 1;
        }
        
        // Analyze function body
        self.collect_declarations(&func.body.statements, &mut declared);
        self.collect_usages(&func.body.statements, &mut used);
        
        // Find unused variables
        for (var_name, _) in declared {
            if !used.contains(&var_name) {
                // Skip variables starting with _
                if self.ignore_underscore && var_name.starts_with('_') {
                    continue;
                }
                
                diagnostics.push(Diagnostic::warning(
                    "W0001",
                    format!("unused variable: `{}`", var_name),
                    vex_diagnostics::Span::unknown(), // TODO: Get actual span from AST
                ));
            }
        }
        
        diagnostics
    }
    
    /// Extract variable name from pattern
    fn extract_pattern_name(&self, pattern: &Pattern) -> Option<String> {
        match pattern {
            Pattern::Ident(name) => Some(name.clone()),
            _ => None,
        }
    }
    
    /// Collect all variable declarations in statements
    fn collect_declarations(&self, statements: &[Statement], declared: &mut HashMap<String, usize>) {
        for stmt in statements {
            match stmt {
                Statement::Let { name, .. } => {
                    if self.ignore_underscore && name.starts_with('_') {
                        continue;
                    }
                    *declared.entry(name.clone()).or_insert(0) += 1;
                }
                Statement::LetPattern { pattern, .. } => {
                    if let Some(name) = self.extract_pattern_name(pattern) {
                        if self.ignore_underscore && name.starts_with('_') {
                            continue;
                        }
                        *declared.entry(name).or_insert(0) += 1;
                    }
                }
                Statement::For { init, body, .. } => {
                    // Check init statement for variable declarations
                    if let Some(init_stmt) = init {
                        self.collect_declarations(&[*init_stmt.clone()], declared);
                    }
                    self.collect_declarations(&body.statements, declared);
                }
                Statement::While { body, .. } => {
                    self.collect_declarations(&body.statements, declared);
                }
                Statement::If { then_block, elif_branches, else_block, .. } => {
                    self.collect_declarations(&then_block.statements, declared);
                    for (_cond, block) in elif_branches {
                        self.collect_declarations(&block.statements, declared);
                    }
                    if let Some(else_blk) = else_block {
                        self.collect_declarations(&else_blk.statements, declared);
                    }
                }
                _ => {}
            }
        }
    }
    
    /// Collect all variable usages in statements
    fn collect_usages(&self, statements: &[Statement], used: &mut HashSet<String>) {
        for stmt in statements {
            match stmt {
                Statement::Let { value, .. } => {
                    self.collect_usages_expr(value, used);
                }
                Statement::LetPattern { value, .. } => {
                    self.collect_usages_expr(value, used);
                }
                Statement::Expression(expr) | Statement::Return(Some(expr)) => {
                    self.collect_usages_expr(expr, used);
                }
                Statement::Assign { target, value } => {
                    self.collect_usages_expr(target, used);
                    self.collect_usages_expr(value, used);
                }
                Statement::CompoundAssign { target, value, .. } => {
                    self.collect_usages_expr(target, used);
                    self.collect_usages_expr(value, used);
                }
                Statement::If { condition, then_block, elif_branches, else_block, .. } => {
                    self.collect_usages_expr(condition, used);
                    self.collect_usages(&then_block.statements, used);
                    for (elif_cond, elif_block) in elif_branches {
                        self.collect_usages_expr(elif_cond, used);
                        self.collect_usages(&elif_block.statements, used);
                    }
                    if let Some(else_blk) = else_block {
                        self.collect_usages(&else_blk.statements, used);
                    }
                }
                Statement::While { condition, body, .. } => {
                    self.collect_usages_expr(condition, used);
                    self.collect_usages(&body.statements, used);
                }
                Statement::For { init, condition, post, body, .. } => {
                    if let Some(init_stmt) = init {
                        self.collect_usages(&[*init_stmt.clone()], used);
                    }
                    if let Some(cond) = condition {
                        self.collect_usages_expr(cond, used);
                    }
                    if let Some(post_stmt) = post {
                        self.collect_usages(&[*post_stmt.clone()], used);
                    }
                    self.collect_usages(&body.statements, used);
                }
                Statement::Defer(stmt) => {
                    self.collect_usages(&[*stmt.clone()], used);
                }
                _ => {}
            }
        }
    }
    
    /// Collect variable usages in an expression
    fn collect_usages_expr(&self, expr: &Expression, used: &mut HashSet<String>) {
        match expr {
            Expression::Ident(name) => {
                used.insert(name.clone());
            }
            Expression::Binary { left, right, .. } => {
                self.collect_usages_expr(left, used);
                self.collect_usages_expr(right, used);
            }
            Expression::Unary { expr, .. } => {
                self.collect_usages_expr(expr, used);
            }
            Expression::Call { func, args, .. } => {
                self.collect_usages_expr(func, used);
                for arg in args {
                    self.collect_usages_expr(arg, used);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.collect_usages_expr(receiver, used);
                for arg in args {
                    self.collect_usages_expr(arg, used);
                }
            }
            Expression::FieldAccess { object, .. } => {
                self.collect_usages_expr(object, used);
            }
            Expression::Index { object, index } => {
                self.collect_usages_expr(object, used);
                self.collect_usages_expr(index, used);
            }
            Expression::Array(items) => {
                for item in items {
                    self.collect_usages_expr(item, used);
                }
            }
            Expression::ArrayRepeat(value, count) => {
                self.collect_usages_expr(value, used);
                self.collect_usages_expr(count, used);
            }
            Expression::MapLiteral(pairs) => {
                for (key, value) in pairs {
                    self.collect_usages_expr(key, used);
                    self.collect_usages_expr(value, used);
                }
            }
            Expression::TupleLiteral(items) => {
                for item in items {
                    self.collect_usages_expr(item, used);
                }
            }
            Expression::Block { statements, .. } => {
                self.collect_usages(statements, used);
            }
            Expression::AsyncBlock { statements, .. } => {
                self.collect_usages(statements, used);
            }
            Expression::Match { value, arms } => {
                self.collect_usages_expr(value, used);
                for arm in arms {
                    self.collect_usages_expr(&arm.body, used);
                }
            }
            Expression::StructLiteral { fields, .. } => {
                for (_field_name, field_expr) in fields {
                    self.collect_usages_expr(field_expr, used);
                }
            }
            Expression::EnumLiteral { data, .. } => {
                for expr in data {
                    self.collect_usages_expr(expr, used);
                }
            }
            Expression::Range { start, end } | Expression::RangeInclusive { start, end } => {
                if let Some(s) = start {
                    self.collect_usages_expr(s, used);
                }
                if let Some(e) = end {
                    self.collect_usages_expr(e, used);
                }
            }
            Expression::Reference { expr, .. } => {
                self.collect_usages_expr(expr, used);
            }
            Expression::Deref(expr) => {
                self.collect_usages_expr(expr, used);
            }
            Expression::Await(expr) => {
                self.collect_usages_expr(expr, used);
            }
            Expression::Launch { args, grid, .. } => {
                for arg in args {
                    self.collect_usages_expr(arg, used);
                }
                for g in grid {
                    self.collect_usages_expr(g, used);
                }
            }
            Expression::New(expr) => {
                self.collect_usages_expr(expr, used);
            }
            Expression::Make { size, .. } => {
                self.collect_usages_expr(size, used);
            }
            Expression::Cast { expr, .. } => {
                self.collect_usages_expr(expr, used);
            }
            _ => {}
        }
    }
}

impl LintRule for UnusedVariableRule {
    fn check(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    diagnostics.extend(self.check_function(func));
                }
                Item::Struct(s) => {
                    // Check methods in struct
                    for method in &s.methods {
                        diagnostics.extend(self.check_function(method));
                    }
                }
                _ => {}
            }
        }
        
        diagnostics
    }
    
    fn name(&self) -> &str {
        "unused_variables"
    }
}

impl Default for UnusedVariableRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rule_name() {
        let rule = UnusedVariableRule::new();
        assert_eq!(rule.name(), "unused_variables");
    }
}
