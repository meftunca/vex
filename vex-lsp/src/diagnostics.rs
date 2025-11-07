// Convert Vex compiler diagnostics to LSP diagnostics

use tower_lsp::lsp_types::*;
use vex_compiler::{Diagnostic as VexDiagnostic, ErrorLevel, Span as VexSpan};

/// Convert Vex diagnostic to LSP diagnostic
pub fn vex_to_lsp_diagnostic(diag: &VexDiagnostic) -> Diagnostic {
    Diagnostic {
        range: span_to_range(&diag.span),
        severity: Some(error_level_to_severity(diag.level)),
        code: Some(NumberOrString::String(diag.code.clone())),
        source: Some("vex".to_string()),
        message: diag.message.clone(),
        related_information: diag
            .notes
            .iter()
            .map(|note| DiagnosticRelatedInformation {
                location: Location {
                    uri: Url::parse(&format!("file://{}", diag.span.file)).unwrap(),
                    range: span_to_range(&diag.span),
                },
                message: note.clone(),
            })
            .collect::<Vec<_>>()
            .into(),
        ..Default::default()
    }
}

/// Convert Vex span to LSP range
fn span_to_range(span: &VexSpan) -> Range {
    Range {
        start: Position {
            line: (span.line.saturating_sub(1)) as u32,
            character: (span.column.saturating_sub(1)) as u32,
        },
        end: Position {
            line: (span.line.saturating_sub(1)) as u32,
            character: (span.column + span.length).saturating_sub(1) as u32,
        },
    }
}

/// Convert Vex error level to LSP severity
fn error_level_to_severity(level: ErrorLevel) -> DiagnosticSeverity {
    match level {
        ErrorLevel::Error => DiagnosticSeverity::ERROR,
        ErrorLevel::Warning => DiagnosticSeverity::WARNING,
        ErrorLevel::Note => DiagnosticSeverity::INFORMATION,
        ErrorLevel::Help => DiagnosticSeverity::HINT,
    }
}
