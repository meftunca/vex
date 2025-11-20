// LSP Diagnostics and error handling

use std::sync::Arc;
use tower_lsp::lsp_types::*;
use vex_compiler::borrow_checker::BorrowChecker;
use vex_compiler::linter::Linter;

use super::VexBackend;

impl VexBackend {
    pub async fn parse_and_diagnose(&self, uri: &str, text: &str, version: i32) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Use document cache for incremental parsing
        let cached_doc = self.document_cache.update(uri, text.to_string(), version);

        // Convert parse errors (now Diagnostic objects) to LSP Diagnostics
        if !cached_doc.parse_errors.is_empty() {
            for vex_diag in &cached_doc.parse_errors {
                // Convert vex_diagnostics::Diagnostic to LSP Diagnostic
                let lsp_diag = vex_to_lsp_diagnostic(vex_diag);
                diagnostics.push(lsp_diag);
            }
            return diagnostics;
        }

        // If we have AST, run linter (fast) + borrow checker (slower, optional)
        if let Some(mut program) = cached_doc.ast {
            // ⭐ NEW: Validate imports FIRST (fast, high-value diagnostics)
            self.validate_imports(&program, text, uri, &mut diagnostics)
                .await;

            // Run linter for warnings (unused variables, etc.) - this is fast
            let mut linter = Linter::new();
            let lint_warnings = linter.lint(&program);
            for vex_diag in &lint_warnings {
                let mut lsp_diag = vex_to_lsp_diagnostic(vex_diag);
                lsp_diag.severity = Some(DiagnosticSeverity::WARNING);
                lsp_diag.source = Some("vex-linter".to_string());
                diagnostics.push(lsp_diag);
            }

            // Run borrow checker ONLY if no parse errors (this can be slow)
            // Skip borrow checking if file has syntax errors to reduce CPU usage
            if cached_doc.parse_errors.is_empty() {
                let mut borrow_checker = BorrowChecker::new();
                if let Err(error) = borrow_checker.check_program(&mut program) {
                    diagnostics.push(self.borrow_error_to_diagnostic(&error, text));
                }
            }

            // Store in legacy cache for compatibility (wrap in Arc)
            self.ast_cache.insert(uri.to_string(), Arc::new(program));
        }

        diagnostics
    }

    /// ⭐ NEW: Validate import statements - missing modules, unused imports, circular dependencies
    async fn validate_imports(
        &self,
        program: &vex_ast::Program,
        text: &str,
        uri: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        use std::collections::HashSet;

        // 1. Check for missing/invalid imports
        for (import_idx, import) in program.imports.iter().enumerate() {
            // Skip auto-injected prelude imports (they're synthetic)
            if import_idx < 6 {
                // First 6 imports are core prelude (core/vec, core/box, etc.)
                continue;
            }

            // Resolve import path
            let current_file = std::path::Path::new(uri);
            let resolved = self
                .module_resolver
                .resolve_import(&import.module, Some(current_file));

            if resolved.is_none() {
                // Module not found - create diagnostic
                let range = self.find_import_in_source(text, &import.module);
                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("E0404".to_string())),
                    source: Some("vex-imports".to_string()),
                    message: format!("cannot find module '{}'", import.module),
                    ..Default::default()
                });
            }
        }

        // 2. Detect unused imports (simple heuristic: search for import items in code)
        let used_imports = self.find_used_imports(program, text);
        for (import_idx, import) in program.imports.iter().enumerate() {
            // Skip prelude and module-level imports
            if import_idx < 6 || matches!(import.kind, vex_ast::ImportKind::Module) {
                continue;
            }

            // Check named imports
            if let vex_ast::ImportKind::Named = import.kind {
                for item in &import.items {
                    let import_name = item.alias.as_ref().unwrap_or(&item.name);
                    if !used_imports.contains(import_name) {
                        let range =
                            self.find_import_item_in_source(text, &import.module, &item.name);
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("W0401".to_string())),
                            source: Some("vex-imports".to_string()),
                            message: format!("unused import: '{}'", import_name),
                            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // 3. TODO: Detect circular imports (requires workspace-wide analysis)
        // This would need to traverse import graph across multiple files
    }

    /// Find import statement in source code
    fn find_import_in_source(&self, text: &str, module_path: &str) -> Range {
        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Match: import "module" or import { ... } from "module"
            if line.contains("import") && line.contains(module_path) {
                if let Some(col_idx) = line.find(module_path) {
                    return Range {
                        start: Position {
                            line: line_idx as u32,
                            character: col_idx as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (col_idx + module_path.len()) as u32,
                        },
                    };
                }
            }
        }

        self.default_range()
    }

    /// Find specific import item in source code
    fn find_import_item_in_source(&self, text: &str, _module: &str, item_name: &str) -> Range {
        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Match item name in import statement: import { item, ... }
            if line.contains("import") && line.contains(item_name) {
                if let Some(col_idx) = line.find(item_name) {
                    return Range {
                        start: Position {
                            line: line_idx as u32,
                            character: col_idx as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (col_idx + item_name.len()) as u32,
                        },
                    };
                }
            }
        }

        self.default_range()
    }

    /// Find which imports are actually used in the code
    fn find_used_imports(
        &self,
        program: &vex_ast::Program,
        text: &str,
    ) -> std::collections::HashSet<String> {
        use std::collections::HashSet;
        let mut used = HashSet::new();

        // Scan all items for identifier usage
        for item in &program.items {
            match item {
                vex_ast::Item::Function(func) => {
                    // Check function signature and body for imported types/functions
                    self.scan_function_for_imports(func, &mut used);
                }
                vex_ast::Item::Struct(s) => {
                    // Check struct fields for imported types
                    for field in &s.fields {
                        self.scan_type_for_imports(&field.ty, &mut used);
                    }
                }
                _ => {}
            }
        }

        // Simple text scan for identifiers (fallback)
        // This catches cases we might miss in AST traversal
        for line in text.lines() {
            // Skip import lines themselves
            if line.trim_start().starts_with("import") {
                continue;
            }

            // Extract all words (simple approximation)
            for word in line.split_whitespace() {
                let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !clean.is_empty() {
                    used.insert(clean.to_string());
                }
            }
        }

        used
    }

    /// Scan function for imported identifiers
    fn scan_function_for_imports(
        &self,
        _func: &vex_ast::Function,
        used: &mut std::collections::HashSet<String>,
    ) {
        // TODO: Deep AST traversal for expressions
        // For now, rely on text-based scanning

        // Scan return type
        if let Some(ref ret_ty) = _func.return_type {
            self.scan_type_for_imports(ret_ty, used);
        }

        // Scan parameters
        for param in &_func.params {
            self.scan_type_for_imports(&param.ty, used);
        }
    }

    /// Scan type for imported identifiers
    fn scan_type_for_imports(
        &self,
        ty: &vex_ast::Type,
        used: &mut std::collections::HashSet<String>,
    ) {
        match ty {
            vex_ast::Type::Named(name) => {
                used.insert(name.clone());
            }
            vex_ast::Type::Generic { name, type_args } => {
                used.insert(name.clone());
                for arg in type_args {
                    self.scan_type_for_imports(arg, used);
                }
            }
            vex_ast::Type::Reference(inner, _) | vex_ast::Type::Slice(inner, _) => {
                self.scan_type_for_imports(inner, used);
            }
            vex_ast::Type::Array(inner, _) => {
                self.scan_type_for_imports(inner, used);
            }
            vex_ast::Type::Vec(inner)
            | vex_ast::Type::Box(inner)
            | vex_ast::Type::Option(inner) => {
                self.scan_type_for_imports(inner, used);
            }
            vex_ast::Type::Result(ok, err) => {
                self.scan_type_for_imports(ok, used);
                self.scan_type_for_imports(err, used);
            }
            _ => {}
        }
    }

    /// Convert Vex BorrowError to LSP Diagnostic
    fn borrow_error_to_diagnostic(
        &self,
        error: &vex_compiler::borrow_checker::BorrowError,
        source: &str,
    ) -> Diagnostic {
        use vex_compiler::borrow_checker::BorrowError;

        let (message, code, location_str, variable, field) = match error {
            BorrowError::AssignToImmutable { variable, location } => (
                format!("cannot assign to immutable variable `{}`\nhelp: consider making this binding mutable: `let! {}`", variable, variable),
                "E0101",
                location.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::AssignToImmutableField { variable, field, location } => (
                format!("cannot assign to field `{}` of immutable variable `{}`\nhelp: consider making this binding mutable: `let! {}`", field, variable, variable),
                "E0102",
                location.as_ref(),
                Some(variable.as_str()),
                Some(field.as_str()),
            ),
            BorrowError::UseAfterMove { variable, used_at, .. } => (
                format!("use of moved value: `{}`", variable),
                "E0201",
                used_at.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::MutableBorrowWhileBorrowed { variable, new_borrow, .. } => (
                format!("cannot borrow `{}` as mutable because it is already borrowed as immutable", variable),
                "E0301",
                new_borrow.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::ImmutableBorrowWhileMutableBorrowed { variable, new_borrow, .. } => (
                format!("cannot borrow `{}` as immutable because it is already borrowed as mutable", variable),
                "E0302",
                new_borrow.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::MutationWhileBorrowed { variable, borrowed_at } => (
                format!("cannot assign to `{}` because it is borrowed", variable),
                "E0303",
                borrowed_at.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::MoveWhileBorrowed { variable, borrow_location } => (
                format!("cannot move out of `{}` because it is borrowed", variable),
                "E0304",
                borrow_location.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::ReturnLocalReference { variable } => (
                format!("cannot return reference to local variable `{}`", variable),
                "E0401",
                None,
                Some(variable.as_str()),
                None,
            ),
            BorrowError::DanglingReference { reference, referent } => (
                format!("variable `{}` references `{}` which is out of scope", reference, referent),
                "E0402",
                None,
                Some(reference.as_str()),
                None,
            ),
            BorrowError::UseAfterScopeEnd { variable, .. } => (
                format!("use of `{}` after it went out of scope", variable),
                "E0403",
                None,
                Some(variable.as_str()),
                None,
            ),
            BorrowError::ReturnDanglingReference { variable } => (
                format!("returning reference to local variable `{}` which will be dropped", variable),
                "E0404",
                None,
                Some(variable.as_str()),
                None,
            ),
            BorrowError::UnsafeOperationOutsideUnsafeBlock { operation, location } => (
                format!("unsafe operation `{}` requires unsafe block\nhelp: wrap this in an `unsafe {{ }}` block", operation),
                "E0133",
                location.as_ref(),
                None,
                None,
            ),
        };

        // Choose the appropriate search method based on error type
        let range = if let Some(field_name) = field {
            // For field-specific errors, use field search
            if let Some(var_name) = variable {
                self.find_field_usage_in_source(source, var_name, field_name)
            } else {
                self.default_range()
            }
        } else if let Some(var_name) = variable {
            // For variable errors, use comprehensive variable search
            self.find_variable_usage_in_source(source, var_name, location_str)
        } else {
            self.default_range()
        };

        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(code.to_string())),
            source: Some("vex-borrow-checker".to_string()),
            message: message.to_string(),
            ..Default::default()
        }
    }

    /// Default range for errors without position info
    fn default_range(&self) -> Range {
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        }
    }

    /// Find variable usage in source code - comprehensive pattern matching
    fn find_variable_usage_in_source(
        &self,
        source: &str,
        variable: &str,
        _location: Option<&String>,
    ) -> Range {
        let lines: Vec<&str> = source.lines().collect();

        // Pattern 1: Assignment (most common for immutability errors)
        // "variable = value" or "self.field = value"
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(&format!("{} =", variable)) {
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col_idx + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!("self.{} =", variable)) {
                let start_col = col_idx + 5; // "self.".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
        }

        // Pattern 2: Variable usage (for use-after-move, dangling references)
        // Look for variable name followed by common usage patterns
        for (line_idx, line) in lines.iter().enumerate() {
            // Function calls: "variable.method()"
            if let Some(col_idx) = line.find(&format!("{}.", variable)) {
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col_idx + variable.len()) as u32,
                    },
                };
            }
            // As function argument: "func(variable)"
            if let Some(col_idx) = line.find(&format!("({})", variable)) {
                let start_col = col_idx + 1;
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!("({},", variable)) {
                let start_col = col_idx + 1;
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!(", {})", variable)) {
                let start_col = col_idx + 2;
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!(", {},", variable)) {
                let start_col = col_idx + 2;
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
        }

        // Pattern 3: Return statements (for dangling references)
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(&format!("return {}", variable)) {
                let start_col = col_idx + 7; // "return ".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!("return &{}", variable)) {
                let start_col = col_idx + 8; // "return &".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
        }

        // Pattern 4: Borrow operations (for borrow conflicts)
        for (line_idx, line) in lines.iter().enumerate() {
            // Mutable borrow: "&variable!" or "let x = &variable!"
            if let Some(col_idx) = line.find(&format!("&{}!", variable)) {
                let start_col = col_idx + 1; // "&".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            // Immutable borrow: "&variable"
            if let Some(col_idx) = line.find(&format!("&{}", variable)) {
                let start_col = col_idx + 1; // "&".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
        }

        // Pattern 5: Standalone variable usage (last resort)
        for (line_idx, line) in lines.iter().enumerate() {
            if line.contains(variable) {
                if let Some(col_idx) = line.find(variable) {
                    return Range {
                        start: Position {
                            line: line_idx as u32,
                            character: col_idx as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (col_idx + variable.len()) as u32,
                        },
                    };
                }
            }
        }

        // Default to first line if not found
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        }
    }

    /// Find field usage in source for field-specific errors
    fn find_field_usage_in_source(&self, source: &str, variable: &str, field: &str) -> Range {
        let lines: Vec<&str> = source.lines().collect();

        // Pattern: "variable.field" or "self.field"
        let patterns = vec![format!("{}.{}", variable, field), format!("self.{}", field)];

        for pattern in patterns {
            for (line_idx, line) in lines.iter().enumerate() {
                if let Some(col_idx) = line.find(&pattern) {
                    let start_col = if pattern.starts_with("self.") {
                        col_idx + 5 // "self.".len()
                    } else {
                        col_idx + variable.len() + 1 // "variable.".len()
                    };
                    return Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start_col as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (start_col + field.len()) as u32,
                        },
                    };
                }
            }
        }

        // Fallback: find the field name anywhere
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(field) {
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col_idx + field.len()) as u32,
                    },
                };
            }
        }

        // Default to first line
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        }
    }

    pub async fn publish_diagnostics(
        &self,
        uri: Url,
        diagnostics: Vec<Diagnostic>,
        version: Option<i32>,
    ) {
        self.client
            .publish_diagnostics(uri, diagnostics, version)
            .await;
    }
}

pub fn vex_to_lsp_diagnostic(vex_diag: &vex_diagnostics::Diagnostic) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line: vex_diag.span.line as u32,
                character: vex_diag.span.column as u32,
            },
            end: Position {
                line: vex_diag.span.line as u32,
                character: (vex_diag.span.column + vex_diag.span.length) as u32,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR), // Default to error for now
        code: Some(NumberOrString::String(vex_diag.code.clone())),
        message: vex_diag.message.clone(),
        source: Some("vex".to_string()),
        ..Default::default()
    }
}
