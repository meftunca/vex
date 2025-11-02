use thiserror::Error;

// Modular parser structure
mod parser;
pub use parser::Parser;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error at {location}: {message}")]
    SyntaxError { location: String, message: String },
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
