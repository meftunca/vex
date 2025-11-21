// Naming convention linter rule
// Enforces Vex naming conventions:
// - Functions/variables: snake_case
// - Structs/Enums/Traits: PascalCase
// - Constants: SCREAMING_SNAKE_CASE

use vex_ast::{Function, Item, Program, Statement};
use vex_diagnostics::{Diagnostic, ErrorLevel, Span};

use super::LintRule;

pub struct NamingConventionRule;

impl NamingConventionRule {
    pub fn new() -> Self {
        Self
    }

    fn is_snake_case(name: &str) -> bool {
        if name.is_empty() || name.starts_with('_') {
            return true; // Ignore underscore-prefixed
        }
        name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
    }

    fn is_pascal_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let mut chars = name.chars();
        let first = chars.next().unwrap();
        first.is_uppercase()
            && chars.all(|c| c.is_alphanumeric())
            && !name.contains('_')
    }

    fn is_screaming_snake_case(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        name.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
    }

    fn suggest_snake_case(name: &str) -> String {
        // Convert PascalCase to snake_case
        let mut result = String::new();
        for (i, c) in name.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }

    fn suggest_pascal_case(name: &str) -> String {
        // Convert snake_case to PascalCase
        let mut result = String::new();
        let mut capitalize_next = true;
        for c in name.chars() {
            if c == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_uppercase().next().unwrap());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
        result
    }

    fn check_function_naming(&self, func: &Function) -> Option<Diagnostic> {
        // main() is always allowed
        if func.name == "main" {
            return None;
        }

        if !Self::is_snake_case(&func.name) {
            let suggestion = Self::suggest_snake_case(&func.name);
            return Some(Diagnostic {
                level: ErrorLevel::Warning,
                code: "W0007".to_string(),
                message: format!(
                    "function `{}` should use snake_case naming",
                    func.name
                ),
                span: Span::unknown(), // TODO: Get actual span
                help: Some(format!("consider renaming to `{}`", suggestion)),
                notes: vec![],
                related: Vec::new(),
                primary_label: Some("naming convention".to_string()),
                suggestion: Some(vex_diagnostics::Suggestion {
                    message: format!("rename to `{}`", suggestion),
                    replacement: suggestion.clone(),
                    span: Span::unknown(),
                }),
            });
        }
        None
    }

    fn check_struct_naming(&self, name: &str) -> Option<Diagnostic> {
        if !Self::is_pascal_case(name) {
            let suggestion = Self::suggest_pascal_case(name);
            return Some(Diagnostic {
                level: ErrorLevel::Warning,
                code: "W0007".to_string(),
                message: format!("struct `{}` should use PascalCase naming", name),
                span: Span::unknown(),
                help: Some(format!("consider renaming to `{}`", suggestion)),
                notes: vec![],
                related: Vec::new(),
                primary_label: Some("naming convention".to_string()),
                suggestion: Some(vex_diagnostics::Suggestion {
                    message: format!("rename to `{}`", suggestion),
                    replacement: suggestion.clone(),
                    span: Span::unknown(),
                }),
            });
        }
        None
    }

    fn check_enum_naming(&self, name: &str) -> Option<Diagnostic> {
        if !Self::is_pascal_case(name) {
            let suggestion = Self::suggest_pascal_case(name);
            return Some(Diagnostic {
                level: ErrorLevel::Warning,
                code: "W0007".to_string(),
                message: format!("enum `{}` should use PascalCase naming", name),
                span: Span::unknown(),
                help: Some(format!("consider renaming to `{}`", suggestion)),
                notes: vec![],
                related: Vec::new(),
                primary_label: Some("naming convention".to_string()),
                suggestion: Some(vex_diagnostics::Suggestion {
                    message: format!("rename to `{}`", suggestion),
                    replacement: suggestion.clone(),
                    span: Span::unknown(),
                }),
            });
        }
        None
    }

    fn check_const_naming(&self, name: &str) -> Option<Diagnostic> {
        if !Self::is_screaming_snake_case(name) {
            return Some(Diagnostic {
                level: ErrorLevel::Warning,
                code: "W0007".to_string(),
                message: format!(
                    "constant `{}` should use SCREAMING_SNAKE_CASE naming",
                    name
                ),
                span: Span::unknown(),
                help: Some(format!("consider renaming to `{}`", name.to_uppercase())),
                notes: vec![],
                related: Vec::new(),
                primary_label: Some("naming convention".to_string()),
                suggestion: Some(vex_diagnostics::Suggestion {
                    message: format!("rename to `{}`", name.to_uppercase()),
                    replacement: name.to_uppercase(),
                    span: Span::unknown(),
                }),
            });
        }
        None
    }

    fn check_variable_naming(&self, stmt: &Statement) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match stmt {
            Statement::Let { name, .. } => {
                // Skip underscore-prefixed
                if name.starts_with('_') {
                    return diagnostics;
                }

                if !Self::is_snake_case(name) {
                    let suggestion = Self::suggest_snake_case(name);
                    diagnostics.push(Diagnostic {
                        level: ErrorLevel::Warning,
                        code: "W0008".to_string(),
                        message: format!("variable `{}` should use snake_case naming", name),
                        span: Span::unknown(),
                        help: Some(format!("consider renaming to `{}`", suggestion)),
                        notes: vec![],
                        related: Vec::new(),
                        primary_label: Some("naming convention".to_string()),
                        suggestion: Some(vex_diagnostics::Suggestion {
                            message: format!("rename to `{}`", suggestion),
                            replacement: suggestion.clone(),
                            span: Span::unknown(),
                        }),
                    });
                }
            }
            Statement::For { variable, body, .. } => {
                if !variable.starts_with('_') && !Self::is_snake_case(variable) {
                    let suggestion = Self::suggest_snake_case(variable);
                    diagnostics.push(Diagnostic {
                        level: ErrorLevel::Warning,
                        code: "W0008".to_string(),
                        message: format!(
                            "loop variable `{}` should use snake_case naming",
                            variable
                        ),
                        span: Span::unknown(),
                        help: Some(format!("consider renaming to `{}`", suggestion)),
                        notes: vec![],
                        related: Vec::new(),
                        primary_label: Some("naming convention".to_string()),
                        suggestion: Some(vex_diagnostics::Suggestion {
                            message: format!("rename to `{}`", suggestion),
                            replacement: suggestion.clone(),
                            span: Span::unknown(),
                        }),
                    });
                }
                
                // Recurse into loop body
                for stmt in body {
                    diagnostics.extend(self.check_variable_naming(stmt));
                }
            }
            Statement::Block(stmts) => {
                for stmt in stmts {
                    diagnostics.extend(self.check_variable_naming(stmt));
                }
            }
            Statement::If { then_block, else_block, .. } => {
                for stmt in then_block {
                    diagnostics.extend(self.check_variable_naming(stmt));
                }
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        diagnostics.extend(self.check_variable_naming(stmt));
                    }
                }
            }
            Statement::While { body, .. } => {
                for stmt in body {
                    diagnostics.extend(self.check_variable_naming(stmt));
                }
            }
            _ => {}
        }

        diagnostics
    }
}

impl LintRule for NamingConventionRule {
    fn check(&mut self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for item in &program.items {
            match item {
                Item::Function(func) => {
                    if let Some(diag) = self.check_function_naming(func) {
                        diagnostics.push(diag);
                    }
                    // Check variables in function body
                    for stmt in &func.body {
                        diagnostics.extend(self.check_variable_naming(stmt));
                    }
                }
                Item::Struct(struct_def) => {
                    if let Some(diag) = self.check_struct_naming(&struct_def.name) {
                        diagnostics.push(diag);
                    }
                }
                Item::Enum(enum_def) => {
                    if let Some(diag) = self.check_enum_naming(&enum_def.name) {
                        diagnostics.push(diag);
                    }
                }
                Item::Const(const_def) => {
                    if let Some(diag) = self.check_const_naming(&const_def.name) {
                        diagnostics.push(diag);
                    }
                }
                _ => {}
            }
        }

        diagnostics
    }

    fn name(&self) -> &str {
        "naming-convention"
    }

    fn enabled_by_default(&self) -> bool {
        false // Disabled by default (style preference)
    }
}
