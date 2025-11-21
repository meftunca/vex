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
            let lint_warnings = linter.lint(&program, &cached_doc.span_map);
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
                    diagnostics.push(self.borrow_error_to_diagnostic(
                        &error,
                        text,
                        &cached_doc.span_map,
                    ));
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
        span_map: &vex_diagnostics::SpanMap,
    ) -> Diagnostic {
        

        let diag = error.to_diagnostic(span_map);
        // Reuse conversion helper to include relatedInformation
        let mut lsp_diag = vex_to_lsp_diagnostic(&diag);
        lsp_diag.code = Some(NumberOrString::String(diag.code));
        lsp_diag.source = Some("vex-borrow-checker".to_string());
        lsp_diag
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
    // Map related Diagnostic spans to LSP relatedInformation
    let related_information = if !vex_diag.related.is_empty() {
        let mut infos = Vec::new();
        for (span, msg) in &vex_diag.related {
            // Try to resolve a span.file to a URL for LSP: prefer file path, then parse as URL,
            // and fall back to workspace-relative file paths.
            let uri_opt: Option<Url> = match Url::from_file_path(&span.file) {
                Ok(u) => Some(u),
                Err(_) => {
                    // Try parsing as a raw URI string
                    if let Ok(u) = Url::parse(&span.file) {
                        Some(u)
                    } else {
                        // Try to resolve as a relative path from current dir
                        if let Ok(cwd) = std::env::current_dir() {
                            if let Ok(u) = Url::from_file_path(cwd.join(&span.file)) {
                                Some(u)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
            };
            if let Some(uri) = uri_opt {
                let range = Range {
                    start: Position {
                        line: span.line.saturating_sub(1) as u32,
                        character: span.column.saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: span.line.saturating_sub(1) as u32,
                        character: (span.column.saturating_sub(1) + span.length) as u32,
                    },
                };
                infos.push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
                    location: tower_lsp::lsp_types::Location { uri, range },
                    message: msg.clone(),
                });
            }
        }
        Some(infos)
    } else {
        None
    };
    // Add 'help' or 'suggestion' as related information if present and if there are no other related spans.
    let mut related_information = related_information.unwrap_or_default();
    let base_related_len = related_information.len();
    if base_related_len == 0 {
        if let Some(help) = &vex_diag.help {
        // Attach the help message as related information at the primary location
        if let Some(uri) = Url::from_file_path(&vex_diag.span.file).ok() {
            let range = Range {
                start: Position {
                    line: vex_diag.span.line.saturating_sub(1) as u32,
                    character: vex_diag.span.column.saturating_sub(1) as u32,
                },
                end: Position {
                    line: vex_diag.span.line.saturating_sub(1) as u32,
                    character: (vex_diag.span.column.saturating_sub(1) + vex_diag.span.length) as u32,
                },
            };
            related_information.push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
                location: tower_lsp::lsp_types::Location { uri, range },
                message: format!("help: {}", help),
            });
        }
        }
        if let Some(sugg) = &vex_diag.suggestion {
        if let Some(uri) = Url::from_file_path(&vex_diag.span.file).ok() {
            let range = Range {
                start: Position {
                    line: vex_diag.span.line.saturating_sub(1) as u32,
                    character: vex_diag.span.column.saturating_sub(1) as u32,
                },
                end: Position {
                    line: vex_diag.span.line.saturating_sub(1) as u32,
                    character: (vex_diag.span.column.saturating_sub(1) + vex_diag.span.length) as u32,
                },
            };
            related_information.push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
                location: tower_lsp::lsp_types::Location { uri, range },
                message: format!("suggestion: {} -> {}", sugg.message, sugg.replacement),
            });
        }
        }
    }
    let related_information = if related_information.is_empty() {
        None
    } else {
        Some(related_information)
    };
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
        message: if let Some(label) = &vex_diag.primary_label {
            format!("{}: {}", label, vex_diag.message)
        } else {
            vex_diag.message.clone()
        },
        source: Some("vex".to_string()),
        related_information,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vex_diagnostics::{Diagnostic as VexDiagnostic, ErrorLevel, Span};

    #[test]
    fn test_vex_to_lsp_related_information() {
        let mut path = std::env::temp_dir();
        path.push("test.vx");
        let span = Span::new(path.display().to_string(), 5, 1, 1);
        let vex_diag = VexDiagnostic {
            level: ErrorLevel::Error,
            code: "E000".to_string(),
            message: "error".to_string(),
            span: span.clone(),
            primary_label: None,
            notes: Vec::new(),
            help: None,
            suggestion: None,
            related: vec![(span.clone(), "value moved here".to_string())],
        };

        let lsp_diag = vex_to_lsp_diagnostic(&vex_diag);
        assert!(lsp_diag.related_information.is_some());
        let infos = lsp_diag.related_information.unwrap();
        assert_eq!(infos.len(), 1);
        assert!(infos[0].message.contains("value moved here"));
    }

    #[test]
    fn test_mutation_while_borrowed_to_lsp_related_information() {
        use vex_compiler::borrow_checker::errors::BorrowError;
        use vex_diagnostics::SpanMap;

        let mut span_map = SpanMap::new();
        let id = span_map.generate_id();
        let span = Span::new(std::env::temp_dir().join("file_b.vx").display().to_string(), 2, 1, 1);
        span_map.record(id.clone(), span.clone());

        let err = BorrowError::MutationWhileBorrowed {
            variable: "x".to_string(),
            borrowed_at: Some(id.clone()),
        };
        let diag = err.to_diagnostic(&span_map);
        let lsp_diag = vex_to_lsp_diagnostic(&diag);
        assert!(lsp_diag.related_information.is_some());
        let infos = lsp_diag.related_information.unwrap();
        assert_eq!(infos.len(), 1);
        assert!(infos[0].message.contains("borrow occurs"));
    }

    #[test]
    fn test_parse_and_diagnose_end_to_end() {
        // We parse a simple snippet that causes a use-after-move and ensure the LSP conversion
        // includes related information that points at the 'moved here' span.
        let code = r#"
            fn main() {
                let s = "hello";
                let t = foo(s);
                println(s);
            }
        "#;

        // Parse the program and obtain its span map
        let mut tmp_path = std::env::temp_dir();
        tmp_path.push("e2e_test.vx");
        let file_path_str = tmp_path.display().to_string();
        let mut parser = vex_parser::Parser::new_with_file(&file_path_str, code).expect("Parser::new failed");
        let program = parser.parse().expect("Parse failed");
        let span_map = parser.take_span_map();

        // Run borrow checker
        let mut checker = vex_compiler::borrow_checker::BorrowChecker::new();
        let result = checker.check_program(&mut program.clone());

        // Ensure it returns an error (use-after-move should be detected)
        assert!(result.is_err());
        let err = result.err().unwrap();
        // E2E: UseAfterMove error is generated by the checker; ensure moved_at set
        // Inspect the parsed AST: ensure the Call expression has a span_id set
        // Find the Call expression in the program
        let mut found_call_span = false;
        let mut recorded_id: Option<String> = None;
        // Expect UseAfterMove error
        match &err {
            vex_compiler::borrow_checker::errors::BorrowError::UseAfterMove { variable, moved_at, .. } => {
                assert_eq!(variable, "s");
                assert!(moved_at.is_some(), "Expected moved_at to be set on UseAfterMove");
                if let Some(moved_id) = moved_at {
                    // ensure the span_map contains the id
                    assert!(span_map.get(moved_id).is_some(), "SpanMap missing moved_at id: {}", moved_id);
                    // moved_at id exists in span_map
                }
                // If we recorded a call span, ensure the moved_at corresponds to it
                if let Some(call_id) = recorded_id.as_ref() {
                    assert_eq!(moved_at.as_ref(), Some(call_id), "Moved_at should reference the call span id");
                    // Check that the span_map resolves this id
                    assert!(span_map.get(call_id).is_some(), "SpanMap must contain the moved_at id used by the error");
                }
            }
            _ => panic!("Expected UseAfterMove but got {:?}", err),
        }
        // Inspect the parsed AST: ensure the Call expression has a span_id set
        // Find the Call expression in the program
        // reuse found_call_span and recorded_id above for checking AST call spans
        for item in &program.items {
            if let vex_ast::Item::Function(func) = item {
                for stmt in &func.body.statements {
                    if let vex_ast::Statement::Let { value, .. } = stmt {
                        if let vex_ast::Expression::Call { span_id, .. } = value {
                            if let Some(id) = span_id.as_ref() {
                                found_call_span = true;
                                recorded_id = Some(id.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }
        assert!(found_call_span, "Call expression should have a span_id");
        if let Some(id) = recorded_id {
            assert!(span_map.get(&id).is_some(), "SpanMap should contain the call span id");
        }
        let diag = err.to_diagnostic(&span_map);
        let lsp_diag = vex_to_lsp_diagnostic(&diag);

        // Related information (moved here) should be present
        assert!(lsp_diag.related_information.is_some());
        let infos = lsp_diag.related_information.unwrap();
        assert!(!infos.is_empty());
        assert!(infos.iter().any(|i| i.message.to_lowercase().contains("moved here") || i.message.to_lowercase().contains("moved")));
    }

    #[test]
    fn test_help_and_suggestion_to_lsp_related_information() {
        let mut path = std::env::temp_dir();
        path.push("test2.vx");
        let span = Span::new(path.display().to_string(), 10, 2, 3);

        let suggestion = vex_diagnostics::Suggestion {
            message: "rename to print".to_string(),
            replacement: "print".to_string(),
            span: span.clone(),
        };

        let vex_diag = VexDiagnostic {
            level: ErrorLevel::Error,
            code: "E0425".to_string(),
            message: "cannot find function `prinnt`".to_string(),
            span: span.clone(),
            primary_label: Some("undefined function".to_string()),
            notes: Vec::new(),
            help: Some("did you mean `print`?".to_string()),
            suggestion: Some(suggestion),
            related: vec![],
        };

        let lsp_diag = vex_to_lsp_diagnostic(&vex_diag);
        assert!(lsp_diag.related_information.is_some());
        let infos = lsp_diag.related_information.unwrap();
        // Should include both help and suggestion
        assert!(infos.iter().any(|i| i.message.contains("help: did you mean")));
        assert!(infos.iter().any(|i| i.message.contains("suggestion:")));
    }

    #[test]
    fn test_primary_label_in_message_prefix() {
        let span = Span::new("file.vx".to_string(), 1, 1, 1);
        let vex_diag = VexDiagnostic::error(
            "E0308",
            "mismatched types".to_string(),
            span.clone(),
        )
        .with_primary_label("mismatched types".to_string());

        let lsp_diag = vex_to_lsp_diagnostic(&vex_diag);
        assert!(lsp_diag.message.starts_with("mismatched types:"));
    }
}
