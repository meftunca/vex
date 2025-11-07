use thiserror::Error;

// Modular parser structure
mod parser;
pub use parser::Parser;

// Re-export diagnostic types for parser use
pub use vex_diagnostics::{error_codes, Diagnostic, DiagnosticEngine, ErrorLevel, Span};

/// Backward compatibility: SourceLocation is now an alias for Span
pub type SourceLocation = Span;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Diagnostic(Diagnostic), // Store the actual diagnostic, not formatted string

    #[error("Lexer error: {0}")]
    LexerError(String),
}

impl ParseError {
    /// Create parse error from diagnostic
    pub fn from_diagnostic(diag: Diagnostic) -> Self {
        Self::Diagnostic(diag)
    }

    /// Create syntax error
    pub fn syntax_error(message: String, span: Span) -> Self {
        let diag = Diagnostic::error(error_codes::SYNTAX_ERROR, message, span);
        Self::Diagnostic(diag)
    }

    /// Create unexpected token error
    pub fn unexpected_token(expected: &str, found: &str, span: Span) -> Self {
        let diag = Diagnostic::error(
            error_codes::UNEXPECTED_TOKEN,
            format!("expected {}, found {}", expected, found),
            span,
        );
        Self::Diagnostic(diag)
    }

    /// Create unexpected EOF error
    pub fn unexpected_eof(expected: &str, span: Span) -> Self {
        let diag = Diagnostic::error(
            error_codes::UNEXPECTED_EOF,
            format!("unexpected end of file, expected {}", expected),
            span,
        );
        Self::Diagnostic(diag)
    }

    /// Get the underlying diagnostic if available
    pub fn as_diagnostic(&self) -> Option<&Diagnostic> {
        match self {
            ParseError::Diagnostic(diag) => Some(diag),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests disabled until parser is fixed
    // #[test]
    // fn test_parse_simple_function() {
    //     let parser = Parser::new();
    //     let input = r#"
    //         fn add(a: int32, b: int32) -> int32 {
    //             return a + b;
    //         }
    //     "#;
    //
    //     let result = parser.parse_file(input);
    //     assert!(result.is_ok());
    //     let file = result.unwrap();
    //     assert_eq!(file.items.len(), 1);
    // }
}
