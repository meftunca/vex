// LSP Code Actions - Quick fixes for common issues

use tower_lsp::lsp_types::*;
use vex_diagnostics::{Diagnostic, Span};

/// Generate code actions for a diagnostic
pub fn generate_code_actions(
    uri: &Url,
    diagnostic: &Diagnostic,
    document_text: &str,
) -> Vec<CodeAction> {
    let mut actions = Vec::new();

    // Match diagnostic by error code
    match diagnostic.code.as_str() {
        "E0101" => {
            // AssignToImmutable - suggest let!
            if let Some(action) = fix_immutable_variable(uri, diagnostic, document_text) {
                actions.push(action);
            }
        }
        "E0102" => {
            // AssignToImmutableField - suggest let!
            if let Some(action) = fix_immutable_variable(uri, diagnostic, document_text) {
                actions.push(action);
            }
        }
        _ => {}
    }

    // Check for "not found" errors - suggest import
    if diagnostic.message.contains("not found") || diagnostic.message.contains("undefined") {
        if let Some(action) = suggest_import(uri, diagnostic, document_text) {
            actions.push(action);
        }
    }

    // Check for mutable method call errors
    if diagnostic.message.contains("requires mutable receiver")
        || diagnostic.message.contains("mutable method")
    {
        if let Some(action) = fix_mutable_method_call(uri, diagnostic, document_text) {
            actions.push(action);
        }
    }

    actions
}

/// Convert vex_diagnostics::Diagnostic to LSP Diagnostic
fn vex_diag_to_lsp(diag: &Diagnostic) -> tower_lsp::lsp_types::Diagnostic {
    tower_lsp::lsp_types::Diagnostic {
        range: span_to_range(&diag.span),
        severity: Some(match diag.level {
            vex_diagnostics::ErrorLevel::Error => DiagnosticSeverity::ERROR,
            vex_diagnostics::ErrorLevel::Warning => DiagnosticSeverity::WARNING,
            _ => DiagnosticSeverity::INFORMATION,
        }),
        code: Some(NumberOrString::String(diag.code.clone())),
        code_description: None,
        source: Some("vex".to_string()),
        message: diag.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Convert vex_diagnostics::Span to LSP Range
fn span_to_range(span: &Span) -> Range {
    Range {
        start: Position {
            line: span.line.saturating_sub(1) as u32, // LSP is 0-indexed
            character: span.column.saturating_sub(1) as u32,
        },
        end: Position {
            line: span.line.saturating_sub(1) as u32,
            character: (span.column.saturating_sub(1) + span.length) as u32,
        },
    }
}

/// Fix immutable variable by changing `let` to `let!`
fn fix_immutable_variable(
    uri: &Url,
    diagnostic: &Diagnostic,
    document_text: &str,
) -> Option<CodeAction> {
    // Extract variable name from diagnostic message
    // Format: "cannot assign to immutable variable `x`"
    let variable_name = extract_variable_name(&diagnostic.message)?;

    // Find the line with "let variable_name"
    let lines: Vec<&str> = document_text.lines().collect();
    let decl_line = find_variable_declaration(&lines, &variable_name)?;

    // Check if it's already mutable
    if lines.get(decl_line)?.contains("let!") {
        return None; // Already mutable
    }

    // Create text edit to replace "let" with "let!"
    let line_text = lines.get(decl_line)?;
    let let_pos = line_text.find("let")?;

    let range = Range {
        start: Position {
            line: decl_line as u32,
            character: let_pos as u32,
        },
        end: Position {
            line: decl_line as u32,
            character: (let_pos + 3) as u32, // "let".len()
        },
    };

    let edit = TextEdit {
        range,
        new_text: "let!".to_string(),
    };

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), vec![edit]);

    let workspace_edit = WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    };

    Some(CodeAction {
        title: format!("Change to 'let! {} = ...'", variable_name),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![vex_diag_to_lsp(diagnostic)]),
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

/// Suggest import for undefined type/function
fn suggest_import(uri: &Url, diagnostic: &Diagnostic, _document_text: &str) -> Option<CodeAction> {
    // Extract symbol name from error message
    // Format: "Type 'HashMap' not found" or "Function 'parse_json' not found"
    let symbol_name = extract_symbol_from_not_found(&diagnostic.message)?;

    // Known stdlib imports (hardcoded for now, should query index)
    let import_suggestion = match symbol_name.as_str() {
        "HashMap" | "Map" => "import std.collections.{HashMap};",
        "Vec" | "Vector" => "import std.collections.{Vec};",
        "Result" => "import std.result.{Result};",
        "Option" => "import std.option.{Option};",
        "File" => "import std.fs.{File};",
        "read_file" | "write_file" => "import std.fs.{read_file, write_file};",
        "parse_json" | "stringify_json" => "import std.json.{parse_json, stringify_json};",
        _ => return None, // Unknown symbol
    };

    // Insert at top of file (after any existing imports)
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    };

    let edit = TextEdit {
        range,
        new_text: format!("{}\n", import_suggestion),
    };

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), vec![edit]);

    let workspace_edit = WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    };

    Some(CodeAction {
        title: format!("Import {} from stdlib", symbol_name),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![vex_diag_to_lsp(diagnostic)]),
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

/// Fix mutable method call by adding `!` suffix
fn fix_mutable_method_call(
    uri: &Url,
    diagnostic: &Diagnostic,
    document_text: &str,
) -> Option<CodeAction> {
    // Extract method name from error
    // Format: "method 'push' requires mutable receiver"
    let _method_name = extract_method_name(&diagnostic.message)?;

    // Find the method call location from diagnostic span
    let lines: Vec<&str> = document_text.lines().collect();
    let line = lines.get((diagnostic.span.line.saturating_sub(1)) as usize)?;

    // Find method call pattern: obj.method(
    let call_start = diagnostic.span.column.saturating_sub(1);
    let call_end = line[call_start..].find('(').map(|i| call_start + i)?;

    // Insert `!` before `(`
    let range = Range {
        start: Position {
            line: diagnostic.span.line.saturating_sub(1) as u32,
            character: call_end as u32,
        },
        end: Position {
            line: diagnostic.span.line.saturating_sub(1) as u32,
            character: call_end as u32,
        },
    };

    let edit = TextEdit {
        range,
        new_text: "!".to_string(),
    };

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), vec![edit]);

    let workspace_edit = WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    };

    Some(CodeAction {
        title: "Add '!' suffix for mutable method call".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![vex_diag_to_lsp(diagnostic)]),
        edit: Some(workspace_edit),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

// ============================================================
// Helper Functions
// ============================================================

/// Extract variable name from diagnostic message
/// "cannot assign to immutable variable `x`" → Some("x")
fn extract_variable_name(message: &str) -> Option<String> {
    let start = message.find('`')?;
    let end = message[start + 1..].find('`')?;
    Some(message[start + 1..start + 1 + end].to_string())
}

/// Extract symbol name from "not found" error
/// "Type 'HashMap' not found" → Some("HashMap")
fn extract_symbol_from_not_found(message: &str) -> Option<String> {
    let start = message.find('\'')?;
    let end = message[start + 1..].find('\'')?;
    Some(message[start + 1..start + 1 + end].to_string())
}

/// Extract method name from mutable method error
/// "method 'push' requires mutable receiver" → Some("push")
fn extract_method_name(message: &str) -> Option<String> {
    let start = message.find('\'')?;
    let end = message[start + 1..].find('\'')?;
    Some(message[start + 1..start + 1 + end].to_string())
}

/// Find line number where variable is declared
fn find_variable_declaration(lines: &[&str], variable_name: &str) -> Option<usize> {
    for (i, line) in lines.iter().enumerate() {
        if line.contains("let ") && line.contains(variable_name) {
            // Basic check - should be more robust (parse AST)
            return Some(i);
        }
    }
    None
}
