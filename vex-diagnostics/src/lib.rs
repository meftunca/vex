// Comprehensive Error System for Vex Compiler
// Provides Rust-quality error messages with spans, colors, and suggestions

use colored::Colorize;
use std::fmt;
use std::path::Path;

// Span tracking module
pub mod span_map;
pub use span_map::SpanMap;

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

    pub fn from_file_and_span(file: &str, source: &str, span: std::ops::Range<usize>) -> Self {
        let before = &source[..span.start];
        let line = before.chars().filter(|&c| c == '\n').count() + 1;
        let column = before
            .rfind('\n')
            .map_or(before.len() + 1, |pos| before.len() - pos);
        let length = span.end.saturating_sub(span.start).max(1);

        Self {
            file: file.to_string(),
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

    /// Create span from file path
    pub fn from_path(path: &Path) -> Self {
        Self {
            file: path.display().to_string(),
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
    Info,
    Note,
    Help,
}

impl fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorLevel::Error => write!(f, "{}", "error".red().bold()),
            ErrorLevel::Warning => write!(f, "{}", "warning".yellow().bold()),
            ErrorLevel::Info => write!(f, "{}", "info".blue().bold()),
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

    pub fn info(code: &str, message: String, span: Span) -> Self {
        Self::new(ErrorLevel::Info, code, message, span)
    }

    pub fn note(message: String, span: Span) -> Self {
        Self::new(ErrorLevel::Note, "", message, span)
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

    /// Format diagnostic without source code (for Display trait)
    fn format_simple(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "{}[{}]: {}\n",
            self.level,
            self.code,
            self.message.bold()
        ));

        output.push_str(&format!(
            " {} {}:{}:{}\n",
            "-->".cyan().bold(),
            self.span.file,
            self.span.line,
            self.span.column
        ));

        for note in &self.notes {
            output.push_str(&format!(" {} {}\n", "=".cyan().bold(), note.cyan()));
        }

        if let Some(help) = &self.help {
            output.push_str(&format!(" {} {}\n", "help:".green().bold(), help));
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

        // Empty line before source
        snippet.push_str(&format!(" {}\n", " ".repeat(line_num_width + 1).cyan()));

        // Actual source line with line number
        snippet.push_str(&format!(
            " {} {} {}\n",
            format!("{:>width$}", self.span.line, width = line_num_width)
                .cyan()
                .bold(),
            "|".cyan().bold(),
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

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

/// Diagnostic collection and reporting engine
#[derive(Debug, Default)]
pub struct DiagnosticEngine {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
    warning_count: usize,
    info_count: usize,
}

impl DiagnosticEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emit(&mut self, diagnostic: Diagnostic) {
        match diagnostic.level {
            ErrorLevel::Error => self.error_count += 1,
            ErrorLevel::Warning => self.warning_count += 1,
            ErrorLevel::Info => self.info_count += 1,
            _ => {}
        }
        self.diagnostics.push(diagnostic);
    }

    pub fn emit_error(&mut self, code: &str, message: String, span: Span) {
        self.emit(Diagnostic::error(code, message, span));
    }

    pub fn emit_warning(&mut self, code: &str, message: String, span: Span) {
        self.emit(Diagnostic::warning(code, message, span));
    }

    pub fn emit_info(&mut self, code: &str, message: String, span: Span) {
        self.emit(Diagnostic::info(code, message, span));
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn info_count(&self) -> usize {
        self.info_count
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Print all diagnostics to stderr
    pub fn print_all(&self, source_code: &str) {
        for diag in &self.diagnostics {
            eprintln!("{}", diag.format(source_code));
        }
    }

    /// Print summary statistics
    pub fn print_summary(&self) {
        if self.error_count > 0 {
            eprintln!(
                "\n{}: {} error{} emitted",
                "error".red().bold(),
                self.error_count,
                if self.error_count == 1 { "" } else { "s" }
            );
        }

        if self.warning_count > 0 {
            eprintln!(
                "{}: {} warning{} emitted",
                "warning".yellow().bold(),
                self.warning_count,
                if self.warning_count == 1 { "" } else { "s" }
            );
        }

        if self.info_count > 0 {
            eprintln!(
                "{}: {} info message{} emitted",
                "info".blue().bold(),
                self.info_count,
                if self.info_count == 1 { "" } else { "s" }
            );
        }
    }

    /// Export diagnostics as JSON for IDEs/LSP
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\"diagnostics\":[");

        for (i, diag) in self.diagnostics.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }

            let level_str = match diag.level {
                ErrorLevel::Error => "error",
                ErrorLevel::Warning => "warning",
                ErrorLevel::Info => "info",
                ErrorLevel::Note => "note",
                ErrorLevel::Help => "help",
            };

            json.push_str(&format!(
                "{{\"level\":\"{}\",\"code\":\"{}\",\"message\":\"{}\",\"file\":\"{}\",\"line\":{},\"column\":{},\"length\":{}",
                level_str,
                diag.code,
                diag.message.replace('"', "\\\""),
                diag.span.file,
                diag.span.line,
                diag.span.column,
                diag.span.length
            ));

            if !diag.notes.is_empty() {
                json.push_str(",\"notes\":[");
                for (j, note) in diag.notes.iter().enumerate() {
                    if j > 0 {
                        json.push(',');
                    }
                    json.push_str(&format!("\"{}\"", note.replace('"', "\\\"")));
                }
                json.push(']');
            }

            if let Some(help) = &diag.help {
                json.push_str(&format!(",\"help\":\"{}\"", help.replace('"', "\\\"")));
            }

            json.push('}');
        }

        json.push_str("]}");
        json
    }

    /// Clear all diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
        self.warning_count = 0;
        self.info_count = 0;
    }
}

/// Helper functions for common diagnostic patterns
impl DiagnosticEngine {
    /// Create type mismatch error with context
    pub fn type_mismatch(&mut self, expected: &str, found: &str, span: Span) {
        self.emit(
            Diagnostic::error(
                error_codes::TYPE_MISMATCH,
                "mismatched types".to_string(),
                span.clone(),
            )
            .with_note(format!("expected `{}`, found `{}`", expected, found))
            .with_help(format!("try converting `{}` to `{}`", found, expected)),
        );
    }

    /// Create undefined variable error with "did you mean?" suggestion
    pub fn undefined_variable(&mut self, name: &str, span: Span, suggestions: Vec<String>) {
        let mut diag = Diagnostic::error(
            error_codes::UNDEFINED_VARIABLE,
            format!("cannot find value `{}` in this scope", name),
            span,
        );

        if !suggestions.is_empty() {
            diag = diag.with_help(format!("did you mean `{}`?", suggestions.join("`, `")));
        }

        self.emit(diag);
    }

    /// Create argument count error
    pub fn argument_count_mismatch(
        &mut self,
        fn_name: &str,
        expected: usize,
        found: usize,
        span: Span,
    ) {
        self.emit(
            Diagnostic::error(
                error_codes::ARGUMENT_COUNT,
                format!(
                    "this function takes {} argument{} but {} {} supplied",
                    expected,
                    if expected == 1 { "" } else { "s" },
                    found,
                    if found == 1 { "was" } else { "were" }
                ),
                span,
            )
            .with_note(format!("function `{}` defined here", fn_name)),
        );
    }

    /// Create unused variable warning
    pub fn unused_variable(&mut self, name: &str, span: Span) {
        self.emit(
            Diagnostic::warning(
                error_codes::UNUSED_VARIABLE,
                format!("unused variable: `{}`", name),
                span.clone(),
            )
            .with_help(format!("prefix with `_` to silence: `_{}`", name))
            .with_suggestion(
                "if this is intentional, prefix with `_`".to_string(),
                format!("_{}", name),
                span,
            ),
        );
    }

    /// Create type inference info message
    pub fn type_inferred(&mut self, var_name: &str, inferred_type: &str, span: Span) {
        self.emit(Diagnostic::info(
            error_codes::TYPE_INFERENCE,
            format!("type inferred as `{}` for `{}`", inferred_type, var_name),
            span,
        ));
    }
}

/// Common error codes
pub mod error_codes {
    // Syntax errors (E0001-E0099)
    pub const SYNTAX_ERROR: &str = "E0001";
    pub const UNEXPECTED_TOKEN: &str = "E0002";
    pub const UNEXPECTED_EOF: &str = "E0003";
    pub const INVALID_LITERAL: &str = "E0004";
    pub const INVALID_ESCAPE: &str = "E0005";
    pub const NOT_IMPLEMENTED: &str = "E0658"; // Feature not yet implemented

    // Type errors (E0100-E0399)
    pub const TYPE_MISMATCH: &str = "E0308";
    pub const UNDEFINED_TYPE: &str = "E0412";
    pub const RETURN_TYPE: &str = "E0308";
    pub const ARGUMENT_COUNT: &str = "E0061";
    pub const WRONG_ARG_TYPE: &str = "E0308";
    pub const NO_SUCH_FIELD: &str = "E0609";
    pub const NO_SUCH_METHOD: &str = "E0599";
    pub const AMBIGUOUS_TYPE: &str = "E0282";
    pub const CANNOT_INFER: &str = "E0282";
    pub const GENERIC_MISMATCH: &str = "E0308";
    pub const TRAIT_NOT_IMPL: &str = "E0277";
    pub const INVALID_CAST: &str = "E0606";

    // Name resolution errors (E0400-E0499)
    pub const UNDEFINED_VARIABLE: &str = "E0425";
    pub const UNDEFINED_FUNCTION: &str = "E0425";
    pub const DUPLICATE_DEFINITION: &str = "E0428";
    pub const PRIVATE_ACCESS: &str = "E0603";
    pub const AMBIGUOUS_NAME: &str = "E0659";

    // Borrow checker errors (E0500-E0599)
    pub const MOVE_ERROR: &str = "E0382";
    pub const USE_AFTER_MOVE: &str = "E0382";
    pub const BORROW_ERROR: &str = "E0502";
    pub const MULTIPLE_MUTABLE_BORROW: &str = "E0499";
    pub const IMMUTABLE_BORROW: &str = "E0502";
    pub const MUTABLE_BORROW: &str = "E0502";
    pub const IMMUTABLE_ASSIGN: &str = "E0594";
    pub const LIFETIME_ERROR: &str = "E0597";
    pub const DANGLING_REFERENCE: &str = "E0597";
    pub const RETURN_LOCAL_REF: &str = "E0515";

    // Pattern matching errors (E0600-E0699)
    pub const NON_EXHAUSTIVE: &str = "E0004";
    pub const UNREACHABLE_PATTERN: &str = "E0001";
    pub const IRREFUTABLE_PATTERN: &str = "E0005";

    // Module/import errors (E0700-E0799)
    pub const MODULE_NOT_FOUND: &str = "E0583";
    pub const CIRCULAR_DEPENDENCY: &str = "E0391";
    pub const IMPORT_ERROR: &str = "E0432";

    // Trait/impl errors (E0800-E0899)
    pub const MISSING_TRAIT_METHOD: &str = "E0046";
    pub const TRAIT_BOUNDS_NOT_MET: &str = "E0277";
    pub const CONFLICTING_IMPL: &str = "E0119";
    pub const ORPHAN_IMPL: &str = "E0117";

    // Warnings (W0001-W9999)
    pub const UNUSED_VARIABLE: &str = "W0001";
    pub const UNUSED_IMPORT: &str = "W0002";
    pub const UNUSED_FUNCTION: &str = "W0003";
    pub const UNUSED_MUT: &str = "W0004";
    pub const DEAD_CODE: &str = "W0005";
    pub const DEPRECATED: &str = "W0006";
    pub const UNREACHABLE_CODE: &str = "W0007";
    pub const INEFFICIENT_PATTERN: &str = "W0008";

    // Info messages (I0001-I9999)
    pub const TYPE_INFERENCE: &str = "I0001";
    pub const OPTIMIZATION_APPLIED: &str = "I0002";
    pub const GENERIC_INSTANTIATION: &str = "I0003";
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

/// Fuzzy matching utilities for "did you mean?" suggestions
pub mod fuzzy {
    use strsim::jaro_winkler;

    /// Find similar names using fuzzy matching (Jaro-Winkler distance)
    /// Returns up to `max_suggestions` names with similarity > threshold
    pub fn find_similar_names(
        target: &str,
        candidates: &[String],
        threshold: f64,
        max_suggestions: usize,
    ) -> Vec<String> {
        let mut scored: Vec<(String, f64)> = candidates
            .iter()
            .map(|candidate| {
                let similarity = jaro_winkler(target, candidate);
                (candidate.clone(), similarity)
            })
            .filter(|(_, score)| *score > threshold)
            .collect();

        // Sort by similarity (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Take top suggestions
        scored
            .into_iter()
            .take(max_suggestions)
            .map(|(name, _)| name)
            .collect()
    }

    /// Find similar function names with exact prefix matching bonus
    pub fn find_similar_functions(target: &str, candidates: &[String]) -> Vec<String> {
        // Give bonus to functions that start with same prefix
        let prefix_matches: Vec<String> = candidates
            .iter()
            .filter(|c| {
                c.to_lowercase()
                    .starts_with(&target.to_lowercase().chars().take(2).collect::<String>())
            })
            .cloned()
            .collect();

        if !prefix_matches.is_empty() {
            return prefix_matches.into_iter().take(3).collect();
        }

        // Fall back to fuzzy matching
        find_similar_names(target, candidates, 0.7, 3)
    }
}
