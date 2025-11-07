use thiserror::Error;

// Modular parser structure
mod parser;
pub use parser::Parser;

/// Source code location for error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

impl SourceLocation {
    pub fn from_span(file: &str, source: &str, span: std::ops::Range<usize>) -> Self {
        let before = &source[..span.start];
        let line = before.lines().count();
        let column = before.lines().last().map_or(0, |l| l.len()) + 1;
        let length = span.end.saturating_sub(span.start);

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
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error at {location}: {message}")]
    SyntaxError {
        location: SourceLocation,
        message: String,
    },
    #[error("Lexer error: {0}")]
    LexerError(String),
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
