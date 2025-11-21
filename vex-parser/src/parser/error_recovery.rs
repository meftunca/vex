// Error recovery for parser - continue parsing after errors
// Collects multiple diagnostics instead of stopping at first error

use crate::{ParseError, Parser};
use vex_diagnostics::{Diagnostic, DiagnosticEngine, error_codes};
use vex_lexer::Token;

impl<'a> Parser<'a> {
    /// Parse with error recovery - collect all errors instead of stopping at first
    pub fn parse_with_recovery(&mut self) -> (Option<vex_ast::Program>, Vec<Diagnostic>) {
        let mut diagnostics = DiagnosticEngine::new();
        let mut imports = Vec::new();
        let mut items = Vec::new();

        while !self.is_at_end() {
            // Try to parse top-level item, recover if error
            match self.try_parse_item() {
                Ok(Some(item)) => match item {
                    TopLevelItem::Import(import) => imports.push(import),
                    TopLevelItem::Item(item) => items.push(item),
                },
                Ok(None) => {
                    // Skip unknown token, try to recover
                    diagnostics.emit_error(
                        "E0001",
                        format!("Unexpected token: {:?}", self.peek()),
                        self.current_span(),
                    );
                    self.advance();
                }
                Err(error) => {
                    // Collect error diagnostic
                    if let Some(diag) = error.as_diagnostic() {
                        diagnostics.emit(diag.clone());
                    } else {
                        diagnostics.emit_error("E0001", format!("{}", error), self.current_span());
                    }

                    // Try to recover to next item boundary
                    self.recover_to_next_item(&mut diagnostics);
                }
            }
        }

        // If we collected errors, return them
        if diagnostics.has_errors() {
            return (None, diagnostics.diagnostics().to_vec());
        }

        // Success - return parsed program
        let program = vex_ast::Program { imports, items };
        (Some(program), diagnostics.diagnostics().to_vec())
    }

    /// Try to parse a single top-level item
    fn try_parse_item(&mut self) -> Result<Option<TopLevelItem>, ParseError> {
        use vex_ast::Item;

        if self.check(&Token::Import) {
            Ok(Some(TopLevelItem::Import(self.parse_import()?)))
        } else if self.check(&Token::Export) {
            Ok(Some(TopLevelItem::Item(self.parse_export()?)))
        } else if self.check(&Token::Const) {
            Ok(Some(TopLevelItem::Item(self.parse_const()?)))
        } else if self.check(&Token::Async) {
            self.advance(); // consume 'async'
            self.consume(&Token::Fn, "Expected 'fn' after 'async'")?;
            let mut func = self.parse_function()?;
            func.is_async = true;
            Ok(Some(TopLevelItem::Item(Item::Function(func))))
        } else if self.check(&Token::Fn) {
            self.advance(); // consume 'fn'
            Ok(Some(TopLevelItem::Item(Item::Function(
                self.parse_function()?,
            ))))
        } else if self.check(&Token::Struct) {
            Ok(Some(TopLevelItem::Item(self.parse_struct()?)))
        } else if self.check(&Token::Type) {
            Ok(Some(TopLevelItem::Item(self.parse_type_alias()?)))
        } else if self.check(&Token::Enum) {
            Ok(Some(TopLevelItem::Item(self.parse_enum()?)))
        } else if self.check(&Token::Contract) {
            Ok(Some(TopLevelItem::Item(self.parse_trait()?)))
        } else if self.check(&Token::Impl) {
            Ok(Some(TopLevelItem::Item(self.parse_trait_impl()?)))
        } else if self.check(&Token::Extern) {
            Ok(Some(TopLevelItem::Item(self.parse_extern_block()?)))
        } else if self.check(&Token::Policy) {
            Ok(Some(TopLevelItem::Item(Item::Policy(self.parse_policy()?))))
        } else {
            // Unknown token - return None to signal skip
            Ok(None)
        }
    }

    /// Recover to next item boundary (semicolon, brace, or keyword)
    fn recover_to_next_item(&mut self, diagnostics: &mut DiagnosticEngine) {
        let mut brace_depth = 0;
        let mut steps = 0usize;

        while !self.is_at_end() {
            if self.guard_tick(&mut steps, "parser recovery timeout: possible infinite loop detected", Self::PARSE_LOOP_DEFAULT_MAX_STEPS) {
                // Emit a diagnostic to avoid infinite loop and forcibly advance once
                let span = self.current_span();
                let diag = Diagnostic::error(
                    error_codes::SYNTAX_ERROR,
                    "parser recovery timeout: possible infinite loop detected".to_string(),
                    span,
                )
                .with_primary_label("parser recovery timeout".to_string())
                .with_help("recovery exceeded maximum iterations and forcibly advanced".to_string());
                diagnostics.emit(diag);
                // Force advance and break: don't get stuck
                self.advance();
                break;
            }
            match self.peek() {
                // Item boundaries - stop here
                Token::Fn
                | Token::Struct
                | Token::Enum
                | Token::Contract
                | Token::Type
                | Token::Const
                | Token::Import
                | Token::Export
                | Token::Extern
                | Token::Policy => {
                    if brace_depth == 0 {
                        break;
                    }
                }

                // Track brace depth
                Token::LBrace => {
                    brace_depth += 1;
                    self.advance();
                }
                Token::RBrace => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                        self.advance();
                        if brace_depth == 0 {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Semicolon at top level
                Token::Semicolon => {
                    self.advance();
                    if brace_depth == 0 {
                        break;
                    }
                }

                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Get current span for error reporting
    fn current_span(&self) -> vex_diagnostics::Span {
        if self.is_at_end() {
            vex_diagnostics::Span {
                file: self.file_name.clone(),
                line: self.source.lines().count(),
                column: self.source.lines().last().map_or(0, |l| l.len()),
                length: 0,
            }
        } else {
            let token_span = &self.tokens[self.current].span;
            vex_diagnostics::Span::from_file_and_span(
                &self.file_name,
                self.source,
                token_span.clone(),
            )
        }
    }
}

/// Helper enum for top-level parsing
enum TopLevelItem {
    Import(vex_ast::Import),
    Item(vex_ast::Item),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_multiple_errors() {
        let source = r#"
            fn valid1(): i32 { return 42; }
            fn broken1( { bad syntax }
            fn valid2(): i32 { return 100; }
            fn broken2(): { another error
            fn valid3(): i32 { return 200; }
        "#;

        let mut parser = match Parser::new(source) {
            Ok(p) => p,
            Err(e) => panic!("Failed to create parser: {:?}", e),
        };
        let (_program, diagnostics) = parser.parse_with_recovery();

        // Should collect multiple errors
        assert!(diagnostics.len() >= 2, "Should have at least 2 errors");

        // Should recover and parse valid functions
        // (program may be None if errors are critical, or Some with partial AST)
        println!("Collected {} diagnostics", diagnostics.len());
        for diag in &diagnostics {
            println!("  - {}: {}", diag.code, diag.message);
        }
    }

    #[test]
    fn test_error_recovery_syntax_error() {
        let source = r#"
            fn test1(): i32 { return 1; }
            let x = broken syntax here;
            fn test2(): i32 { return 2; }
        "#;

        let mut parser = match Parser::new(source) {
            Ok(p) => p,
            Err(e) => panic!("Failed to create parser: {:?}", e),
        };
        let (_program, diagnostics) = parser.parse_with_recovery();

        assert!(!diagnostics.is_empty(), "Should have errors");
        println!("Diagnostics: {:?}", diagnostics);
    }

    #[test]
    fn test_error_recovery_timeout_protects() {
        let source = r#"
            fn test1(): i32 { return 1; }
            let x = broken syntax here;
            // Create input that will have lots of tokens and prevent normal recovery
            { { { { { { { { { { { { { { { { { { { { { { { { 
            fn test2(): i32 { return 2; }
        "#;

        let mut parser = match Parser::new(source) {
            Ok(p) => p,
            Err(e) => panic!("Failed to create parser: {:?}", e),
        };

        let (_program, diagnostics) = parser.parse_with_recovery();

        assert!(!diagnostics.is_empty(), "Should have errors");

        // Ensure that the parser returned and produced diagnostics; timeout may or may not be triggered
        let _has_timeout = diagnostics.iter().any(|d| d.message.contains("parser recovery timeout"));
    }

    #[test]
    fn test_error_recovery_json_serialization() {
        let source = r#"
            fn test1(): i32 { return 1; }
            let x = broken syntax here;
            fn test2(): i32 { return 2; }
        "#;

        let mut parser = Parser::new(source).expect("Failed to create parser");
        let (_program, diagnostics) = parser.parse_with_recovery();

        assert!(!diagnostics.is_empty());

        // Build a DiagnosticEngine and serialize to JSON
        let mut engine = DiagnosticEngine::new();
        for d in diagnostics.iter() {
            engine.emit(d.clone());
        }
        let json = engine.to_json();
        assert!(json.contains("diagnostics"));
        assert!(json.contains("E0001"));
    }
}
