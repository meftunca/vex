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

        // If we have AST, run linter + borrow checker
        if let Some(mut program) = cached_doc.ast {
            // Run linter for warnings (unused variables, etc.)
            let mut linter = Linter::new();
            let lint_warnings = linter.lint(&program);
            for vex_diag in &lint_warnings {
                let mut lsp_diag = vex_to_lsp_diagnostic(vex_diag);
                lsp_diag.severity = Some(DiagnosticSeverity::WARNING);
                lsp_diag.source = Some("vex-linter".to_string());
                diagnostics.push(lsp_diag);
            }

            // Run borrow checker
            let mut borrow_checker = BorrowChecker::new();
            if let Err(error) = borrow_checker.check_program(&mut program) {
                diagnostics.push(self.borrow_error_to_diagnostic(&error, text));
            }

            // Store in legacy cache for compatibility (wrap in Arc)
            self.ast_cache.insert(uri.to_string(), Arc::new(program));
        }

        diagnostics
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
