// Comprehensive Error System for Vex Compiler
// Provides Rust-quality error messages with spans, colors, and suggestions

use colored::Colorize;
use std::fmt;

/// Source code location (line, column, file)
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize, // Length of the error span
}

impl Span {
    pub fn new(file: String, line: usize, column: usize, length: usize) -> Self {
        Self {
            file,
            line,
            column,
            length,
        }
    }

    pub fn unknown() -> Self {
        Self {
            file: "<unknown>".to_string(),
            line: 0,
            column: 0,
            length: 0,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorLevel {
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorLevel::Error => write!(f, "{}", "error".red().bold()),
            ErrorLevel::Warning => write!(f, "{}", "warning".yellow().bold()),
            ErrorLevel::Note => write!(f, "{}", "note".cyan().bold()),
            ErrorLevel::Help => write!(f, "{}", "help".green().bold()),
        }
    }
}

/// Structured diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: ErrorLevel,
    pub code: String, // e.g., "E0308" for type mismatch
    pub message: String,
    pub span: Span,
    pub notes: Vec<String>,
    pub help: Option<String>,
    pub suggestion: Option<Suggestion>,
}

/// Code suggestion with replacement
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacement: String,
    pub span: Span,
}

impl Diagnostic {
    pub fn new(level: ErrorLevel, code: &str, message: String, span: Span) -> Self {
        Self {
            level,
            code: code.to_string(),
            message,
            span,
            notes: Vec::new(),
            help: None,
            suggestion: None,
        }
    }

    pub fn error(code: &str, message: String, span: Span) -> Self {
        Self::new(ErrorLevel::Error, code, message, span)
    }

    pub fn warning(code: &str, message: String, span: Span) -> Self {
        Self::new(ErrorLevel::Warning, code, message, span)
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_suggestion(mut self, message: String, replacement: String, span: Span) -> Self {
        self.suggestion = Some(Suggestion {
            message,
            replacement,
            span,
        });
        self
    }

    /// Format diagnostic in Rust-style
    pub fn format(&self, source_code: &str) -> String {
        let mut output = String::new();

        // Header: error[E0308]: message
        output.push_str(&format!(
            "{}[{}]: {}\n",
            self.level,
            self.code,
            self.message.bold()
        ));

        // Location: --> file.vx:12:15
        output.push_str(&format!(
            " {} {}:{}:{}\n",
            "-->".cyan().bold(),
            self.span.file,
            self.span.line,
            self.span.column
        ));

        // Source code snippet with highlight
        if let Some(snippet) = self.get_source_snippet(source_code) {
            output.push_str(&snippet);
        }

        // Notes
        for note in &self.notes {
            output.push_str(&format!(" {} {}\n", "=".cyan().bold(), note.cyan()));
        }

        // Help
        if let Some(help) = &self.help {
            output.push_str(&format!(" {} {}\n", "help:".green().bold(), help));
        }

        // Suggestion
        if let Some(suggestion) = &self.suggestion {
            output.push_str(&format!(
                " {} {}\n",
                "help:".green().bold(),
                suggestion.message
            ));
            if let Some(suggested_snippet) = self.get_suggestion_snippet(source_code) {
                output.push_str(&suggested_snippet);
            }
        }

        output
    }

    /// Extract source code snippet with error highlight
    fn get_source_snippet(&self, source_code: &str) -> Option<String> {
        let lines: Vec<&str> = source_code.lines().collect();

        if self.span.line == 0 || self.span.line > lines.len() {
            return None;
        }

        let line_idx = self.span.line - 1;
        let line = lines[line_idx];

        let mut snippet = String::new();

        // Line number with padding
        let line_num_width = self.span.line.to_string().len().max(2);

        // Empty line
        snippet.push_str(&format!(" {}\n", " ".repeat(line_num_width + 1).cyan()));

        // Actual source line
        snippet.push_str(&format!(
            " {} {}\n",
            format!("{:>width$}", self.span.line, width = line_num_width)
                .cyan()
                .bold(),
            "| ".cyan().bold()
        ));
        snippet.push_str(&format!(
            " {} {}\n",
            " ".repeat(line_num_width + 1).cyan(),
            line
        ));

        // Error indicator (^^^)
        let padding = " ".repeat(line_num_width + 3 + self.span.column - 1);
        let underline = "^".repeat(self.span.length.max(1));
        snippet.push_str(&format!(
            " {} {}{}\n",
            " ".repeat(line_num_width + 1).cyan(),
            padding,
            underline.red().bold()
        ));

        Some(snippet)
    }

    /// Get suggestion snippet with replacement
    fn get_suggestion_snippet(&self, source_code: &str) -> Option<String> {
        let suggestion = self.suggestion.as_ref()?;
        let lines: Vec<&str> = source_code.lines().collect();

        if suggestion.span.line == 0 || suggestion.span.line > lines.len() {
            return None;
        }

        let line_idx = suggestion.span.line - 1;
        let line = lines[line_idx];

        let mut snippet = String::new();

        // Line number with padding
        let line_num_width = suggestion.span.line.to_string().len().max(2);

        // Modified line with replacement
        let col = suggestion.span.column - 1;
        let before = &line[..col];
        let after = &line[col + suggestion.span.length..];
        let modified_line = format!("{}{}{}", before, &suggestion.replacement, after);

        snippet.push_str(&format!(
            " {} {}\n",
            format!("{:>width$}", suggestion.span.line, width = line_num_width)
                .cyan()
                .bold(),
            "| ".cyan().bold()
        ));
        snippet.push_str(&format!(
            " {} {}\n",
            " ".repeat(line_num_width + 1).cyan(),
            modified_line
        ));

        // Indicator for added text
        let padding = " ".repeat(line_num_width + 3 + col);
        let indicator = "+".repeat(suggestion.replacement.len());
        snippet.push_str(&format!(
            " {} {}{}\n",
            " ".repeat(line_num_width + 1).cyan(),
            padding,
            indicator.green().bold()
        ));

        Some(snippet)
    }
}

/// Common error codes
pub mod error_codes {
    pub const SYNTAX_ERROR: &str = "E0001";
    pub const TYPE_MISMATCH: &str = "E0308";
    pub const UNDEFINED_VARIABLE: &str = "E0425";
    pub const UNDEFINED_FUNCTION: &str = "E0425";
    pub const UNDEFINED_TYPE: &str = "E0412";
    pub const MOVE_ERROR: &str = "E0382";
    pub const BORROW_ERROR: &str = "E0502";
    pub const IMMUTABLE_ASSIGN: &str = "E0594";
    pub const TRAIT_NOT_IMPL: &str = "E0277";
    pub const ARGUMENT_COUNT: &str = "E0061";
    pub const RETURN_TYPE: &str = "E0308";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_format() {
        let source = "fn main(): i32 {\n    let x = add(42, \"hello\");\n    return 0;\n}";

        let span = Span::new("test.vx".to_string(), 2, 21, 7);
        let diag = Diagnostic::error(
            error_codes::TYPE_MISMATCH,
            "mismatched types".to_string(),
            span.clone(),
        )
        .with_note("expected `i32`, found `string`".to_string())
        .with_help("try converting the string to an integer".to_string())
        .with_suggestion(
            "parse the string".to_string(),
            "\"hello\".parse()?".to_string(),
            span,
        );

        let formatted = diag.format(source);
        println!("{}", formatted);

        assert!(formatted.contains("error[E0308]"));
        assert!(formatted.contains("mismatched types"));
        assert!(formatted.contains("test.vx:2:21"));
    }
}
