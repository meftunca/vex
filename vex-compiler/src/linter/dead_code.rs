// Dead code detection linter rule
// Detects:
// - Unused functions (private functions never called)
// - Unused struct definitions
// - Unused enum definitions
// - Unused constants

use vex_ast::{Expression, Function, Item, Program, Statement};
use vex_diagnostics::{Diagnostic, ErrorLevel, Span};
use std::collections::{HashMap, HashSet};

use super::LintRule;



#[derive(Clone, Debug)]
pub struct DeadCodeRule {
    /// Track which functions/structs/enums are used
    used_items: HashSet<String>,
    /// Track all defined items
    defined_items: HashMap<String, ItemInfo>,
}

impl Clone for DeadCodeRule {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct ItemInfo {
    name: String,
    kind: ItemKind,
    is_public: bool, // Public items are never considered dead code
}

#[derive(Clone, PartialEq)]
enum ItemKind {
    Function,
    Struct,
    Enum,
    Const,
}

impl DeadCodeRule {
    pub fn new() -> Self {
        Self {
            used_items: HashSet::new(),
            defined_items: HashMap::new(),
        }
    }

    fn collect_definitions(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Function(func) => {
                    // main() is never dead (no visibility field in AST)
                    let is_public = func.name == "main";
                    self.defined_items.insert(
                        func.name.clone(),
                        ItemInfo {
                            name: func.name.clone(),
                            kind: ItemKind::Function,
                            is_public,
                        },
                    );
                }
                Item::Struct(struct_def) => {
                    let is_public = false; // Assume private for now
                    self.defined_items.insert(
                        struct_def.name.clone(),
                        ItemInfo {
                            name: struct_def.name.clone(),
                            kind: ItemKind::Struct,
                            is_public,
                        },
                    );
                }
                Item::Enum(enum_def) => {
                    let is_public = false;
                    self.defined_items.insert(
                        enum_def.name.clone(),
                        ItemInfo {
                            name: enum_def.name.clone(),
                            kind: ItemKind::Enum,
                            is_public,
                        },
                    );
                }
                Item::Const(const_def) => {
                    let is_public = false;
                    self.defined_items.insert(
                        const_def.name.clone(),
                        ItemInfo {
                            name: const_def.name.clone(),
                            kind: ItemKind::Const,
                            is_public,
                        },
                    );
                }
                _ => {}
            }
        }
    }

    fn collect_usages(&mut self, program: &Program) {
        // Always mark main as used
        self.used_items.insert("main".to_string());

        for item in &program.items {
            if let Item::Function(func) = item {
                self.collect_usages_in_function(func);
            }
        }
    }

    fn collect_usages_in_function(&mut self, func: &Function) {
        for stmt in &func.body.statements {
            self.collect_usages_in_statement(stmt);
        }
    }

    fn collect_usages_in_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expression(expr) => {
                self.collect_usages_in_expression(expr);
            }
            Statement::Let { value, .. } => {ession(value);
            Statement::Return(value) => {
                if let Some(expr) = value {
                    self.collect_usages_in_expression(expr);
                }
            }
            Statement::If { condition, then_block, else_block, .. } => {
                self.collect_usages_in_expression(condition);
                for stmt in &then_block.statements {
                    self.collect_usages_in_statement(stmt);
                }
                if let Some(else_blk) = else_block {
                    for stmt in &else_blk.statements {
                        self.collect_usages_in_statement(stmt);
                    }
                }
            }
            Statement::While { condition, body, .. } => {
                self.collect_usages_in_expression(condition);
                for stmt in &body.statements {
                    self.collect_usages_in_statement(stmt);
                }
            }
            Statement::ForIn { iterable, body, .. } => {
                self.collect_usages_in_expression(iterable);
                for stmt in &body.statements {
                    self.collect_usages_in_statement(stmt);
                }
            }
            _ => {
                    self.collect_usages_in_statement(stmt);
                }
            }
            _ => {}
        }
    }

    fn collect_usages_in_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Call { func, args, .. } => {
                // Function call - mark function as used
                if let Expression::Ident(name) = &**func {
                    self.used_items.insert(name.clone());
                }
                for arg in args {
                    self.collect_usages_in_expression(arg);
                }
            }
            Expression::StructLiteral { name, .. } => {
                // Struct instantiation - mark struct as used
                self.used_items.insert(name.clone());
            }
            Expression::EnumLiteral { enum_name, .. } => {
                // Enum variant - mark enum as used
                self.used_items.insert(enum_name.clone());
            }
            Expression::TypeAnnotation { expr, ty } => {
                self.collect_usages_in_expression(expr);
                // Mark type as used
                if let vex_ast::Type::Named(name) = ty {
                    self.used_items.insert(name.clone());
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_usages_in_expression(left);
                self.collect_usages_in_expression(right);
            }
            Expression::Unary { expr, .. } => {
                self.collect_usages_in_expression(expr);
            }
            Expression::MethodCall { object, args, .. } => {
                self.collect_usages_in_expression(object);
                for arg in args {
                    self.collect_usages_in_expression(arg);
                }
            }
            Expression::Index { object, index, .. } => {
                self.collect_usages_in_expression(object);
                self.collect_usages_in_expression(index);
            }
            Expression::Array(elements) => {
                for elem in elements {
                    self.collect_usages_in_expression(elem);
                }
            }
            Expression::If { condition, then_branch, else_branch, .. } => {
                self.collect_usages_in_expression(condition);
                self.collect_usages_in_expression(then_branch);
                if let Some(else_expr) = else_branch {
                    self.collect_usages_in_expression(else_expr);
                }
            }
            Expression::Block(block) => {
                for stmt in &block.statements {
                    self.collect_usages_in_statement(stmt);
                }
            }
            _ => {}
        }
    }
}

impl LintRule for DeadCodeRule {
    fn check(&self, program: &Program) -> Vec<Diagnostic> {
        let mut self_mut = self.clone();
        self_mut.check_internal(program)
    }

    fn name(&self) -> &str {
        "dead-code"
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}

impl DeadCodeRule {
    fn check_internal(&mut self, program: &Program) -> Vec<Diagnostic> {
        self.used_items.clear();
        self.defined_items.clear();

        // Collect all definitions
        self.collect_definitions(program);
        
        // Collect all usages
        self.collect_usages(program);

        let mut diagnostics = Vec::new();

        // Find dead code
        for (name, info) in &self.defined_items {
            // Skip public items and main
            if info.is_public || name == "main" {
                continue;
            }

            // Check if item is used
            if !self.used_items.contains(name) {
                let (code, message) = match info.kind {
                    ItemKind::Function => (
                        "W0002".to_string(),
                        format!("function `{}` is never used", name),
                    ),
                    ItemKind::Struct => (
                        "W0003".to_string(),
                        format!("struct `{}` is never used", name),
                    ),
                    ItemKind::Enum => (
                        "W0004".to_string(),
                        format!("enum `{}` is never used", name),
                    ),
                    ItemKind::Const => (
                        "W0005".to_string(),
                        format!("constant `{}` is never used", name),
                    ),
                };

                diagnostics.push(Diagnostic {
                    level: ErrorLevel::Warning,
                    code,
                    message,
                    span: Span::unknown(), // TODO: Get actual span from AST
                    help: Some("consider removing this item or making it public".to_string()),
                    notes: vec![],
                });
            }
        }

        diagnostics
    }
}
