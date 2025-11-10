// LSP Refactorings - Extract Variable and Extract Function

use tower_lsp::lsp_types::*;
use vex_ast::*;

/// Generate refactoring code actions for selected code range
pub fn generate_refactorings(
    uri: &Url,
    range: Range,
    document_text: &str,
    _ast: Option<&Program>,
) -> Vec<CodeAction> {
    let mut actions = Vec::new();

    // Extract Variable: Convert expression to variable
    if let Some(action) = extract_variable(uri, range, document_text) {
        actions.push(action);
    }

    // Extract Function: Convert statements to function
    if let Some(action) = extract_function(uri, range, document_text) {
        actions.push(action);
    }

    actions
}

/// Extract selected expression to a new variable
///
/// Example:
/// ```vex
/// let result = compute() + transform(data);
///              ^^^^^^^^^^^^^^^^^^^^^^^^^ (selected)
/// ```
/// Becomes:
/// ```vex
/// let temp = compute() + transform(data);
/// let result = temp;
/// ```
fn extract_variable(uri: &Url, range: Range, document_text: &str) -> Option<CodeAction> {
    // Get selected text
    let selected_text = get_text_in_range(document_text, range)?;

    // Check if selection is a valid expression (basic heuristic)
    if !is_expression(&selected_text) {
        return None;
    }

    // Generate variable name suggestion
    let var_name = suggest_variable_name(&selected_text);

    // Find the line where this expression appears
    let expr_line = range.start.line;

    // Create text edits
    let mut edits = Vec::new();

    // 1. Insert new variable declaration above
    let indent = get_line_indent(document_text, expr_line as usize);
    let new_var_decl = format!("{}let {} = {};\n", indent, var_name, selected_text);

    edits.push(TextEdit {
        range: Range {
            start: Position {
                line: expr_line,
                character: 0,
            },
            end: Position {
                line: expr_line,
                character: 0,
            },
        },
        new_text: new_var_decl,
    });

    // 2. Replace selected expression with variable name
    edits.push(TextEdit {
        range,
        new_text: var_name.clone(),
    });

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), edits);

    Some(CodeAction {
        title: format!("Extract to variable '{}'", var_name),
        kind: Some(CodeActionKind::REFACTOR_EXTRACT),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
        diagnostics: None,
    })
}

/// Extract selected statements to a new function
///
/// Example:
/// ```vex
/// let x = compute();
/// let y = transform(x);
/// let z = finalize(y);
/// ```
/// Becomes:
/// ```vex
/// fn process(): Type {
///     let x = compute();
///     let y = transform(x);
///     return finalize(y);
/// }
/// let z = process();
/// ```
fn extract_function(uri: &Url, range: Range, document_text: &str) -> Option<CodeAction> {
    // Get selected text
    let selected_text = get_text_in_range(document_text, range)?;

    // Check if selection contains at least one statement
    if !contains_statements(&selected_text) {
        return None;
    }

    // Generate function name suggestion
    let fn_name = "extracted_function"; // TODO: smarter naming

    // Determine return type (analyze last statement)
    let return_type = infer_return_type(&selected_text);

    // Find parameters (variables used but not defined in selection)
    let params = find_required_parameters(&selected_text);

    // Build function signature
    let param_list = if params.is_empty() {
        "".to_string()
    } else {
        params.join(", ")
    };

    let fn_signature = format!("fn {}({}): {} {{", fn_name, param_list, return_type);

    // Build function body (indent selected text)
    let indented_body = indent_text(&selected_text, 4);

    // Add return statement if needed
    let fn_body = if needs_return(&selected_text) {
        format!(
            "{}\n    return {};",
            indented_body.trim_end(),
            extract_last_value(&selected_text)
        )
    } else {
        indented_body
    };

    let new_function = format!("{}\n{}\n}}\n\n", fn_signature, fn_body);

    // Create text edits
    let mut edits = Vec::new();

    // 1. Insert new function above current function
    let function_start = find_current_function_start(document_text, range.start.line as usize);

    edits.push(TextEdit {
        range: Range {
            start: Position {
                line: function_start as u32,
                character: 0,
            },
            end: Position {
                line: function_start as u32,
                character: 0,
            },
        },
        new_text: new_function,
    });

    // 2. Replace selected code with function call
    let indent = get_line_indent(document_text, range.start.line as usize);
    let args = if params.is_empty() {
        "".to_string()
    } else {
        params
            .iter()
            .map(|p| p.split(':').next().unwrap_or(p).trim())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let fn_call = format!("{}let result = {}({});", indent, fn_name, args);

    edits.push(TextEdit {
        range,
        new_text: fn_call,
    });

    let mut changes = std::collections::HashMap::new();
    changes.insert(uri.clone(), edits);

    Some(CodeAction {
        title: format!("Extract to function '{}'", fn_name),
        kind: Some(CodeActionKind::REFACTOR_EXTRACT),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
        diagnostics: None,
    })
}

// ============================================================
// Helper Functions
// ============================================================

/// Get text content within LSP range
fn get_text_in_range(text: &str, range: Range) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();

    if range.start.line == range.end.line {
        // Single line selection
        let line = lines.get(range.start.line as usize)?;
        let start = range.start.character as usize;
        let end = range.end.character as usize;
        Some(line.get(start..end)?.to_string())
    } else {
        // Multi-line selection
        let mut result = String::new();
        for line_idx in range.start.line..=range.end.line {
            if let Some(line) = lines.get(line_idx as usize) {
                if line_idx == range.start.line {
                    result.push_str(&line[range.start.character as usize..]);
                } else if line_idx == range.end.line {
                    result.push_str(&line[..range.end.character as usize]);
                } else {
                    result.push_str(line);
                }
                result.push('\n');
            }
        }
        Some(result.trim_end().to_string())
    }
}

/// Check if text is likely an expression
fn is_expression(text: &str) -> bool {
    let trimmed = text.trim();

    // Basic heuristics:
    // - Contains operators: +, -, *, /, .
    // - Contains function calls: ()
    // - Not a statement keyword: let, return, if, etc.

    !trimmed.starts_with("let ")
        && !trimmed.starts_with("return ")
        && !trimmed.starts_with("if ")
        && !trimmed.starts_with("fn ")
        && (trimmed.contains('(') || trimmed.contains('+') || trimmed.contains('.'))
}

/// Check if text contains statements (has semicolons or let/return)
fn contains_statements(text: &str) -> bool {
    text.contains(';') || text.contains("let ") || text.contains("return ")
}

/// Suggest variable name based on expression
fn suggest_variable_name(expr: &str) -> String {
    // Try to extract meaningful name from expression
    if expr.contains("compute") {
        "computed_value".to_string()
    } else if expr.contains("transform") {
        "transformed_data".to_string()
    } else if expr.contains("parse") {
        "parsed_result".to_string()
    } else {
        "temp".to_string()
    }
}

/// Get indentation of a line
fn get_line_indent(text: &str, line_idx: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if let Some(line) = lines.get(line_idx) {
        let spaces = line.len() - line.trim_start().len();
        " ".repeat(spaces)
    } else {
        "    ".to_string() // Default 4 spaces
    }
}

/// Infer return type from selected code
fn infer_return_type(text: &str) -> String {
    // Look for last statement/expression
    if text.trim_end().ends_with(';') {
        "unit".to_string() // No return value
    } else if text.contains("i32") {
        "i32".to_string()
    } else if text.contains("string") {
        "string".to_string()
    } else {
        "Type".to_string() // Generic placeholder
    }
}

/// Find variables that are used but not defined in selection
fn find_required_parameters(_text: &str) -> Vec<String> {
    // TODO: Proper AST analysis
    // For now, return empty (no parameters)
    Vec::new()
}

/// Check if selection needs explicit return statement
fn needs_return(text: &str) -> bool {
    // If last line doesn't end with semicolon, it's an expression
    !text.trim_end().ends_with(';')
}

/// Extract last value/expression from code
fn extract_last_value(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if let Some(last_line) = lines.last() {
        // Extract variable name from "let x = ..."
        if let Some(var_name) = last_line.split('=').next() {
            let name = var_name
                .replace("let", "")
                .replace("!", "")
                .trim()
                .to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }
    "result".to_string()
}

/// Indent text by specified spaces
fn indent_text(text: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Find the start line of the current function
fn find_current_function_start(text: &str, current_line: usize) -> usize {
    let lines: Vec<&str> = text.lines().collect();

    // Search backwards for "fn " keyword
    for line_idx in (0..=current_line).rev() {
        if let Some(line) = lines.get(line_idx) {
            if line.trim().starts_with("fn ") {
                return line_idx;
            }
        }
    }

    0 // Default to file start
}
