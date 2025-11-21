use crate::parser::Parser;
use crate::ParseError;
use vex_diagnostics::error_codes;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_import(&mut self) -> Result<Import, ParseError> {
        self.consume(&Token::Import, "Expected 'import'")?;

        // Four patterns:
        // 1. import { io, net } from "std";         - Named imports
        // 2. import * as std from "std";            - Namespace import
        // 3. import "std/io";                       - Module import (entire module)
        // 4. import io from "std/io";               - Default import (future)

        if self.match_token(&Token::Star) {
            // Pattern 2: import * as name from "module";
            self.consume(&Token::As, "Expected 'as' after '*'")?;
            let alias = self.consume_identifier()?;
            self.consume(&Token::From, "Expected 'from' after alias")?;

            let module = if let Token::StringLiteral(s) = self.peek() {
                let m = s.clone();
                self.advance();
                m
            } else {
                // Provide a richer diagnostic with suggestion for module path
                let span = self.token_to_diag_span(&self.peek_span().span);
                let diag = vex_diagnostics::Diagnostic::error(
                    error_codes::SYNTAX_ERROR,
                    "Expected module string after 'from'".to_string(),
                    span.clone(),
                )
                .with_primary_label("expected module string".to_string())
                .with_help("Use a module path string, e.g., from \"./module.vx\"".to_string())
                .with_suggestion(
                    "use module path".to_string(),
                    "\"./module.vx\"".to_string(),
                    span,
                );
                return Err(ParseError::from_diagnostic(diag));
            };

            self.consume(&Token::Semicolon, "Expected ';' after import")?;

            return Ok(Import {
                kind: ImportKind::Namespace(alias.clone()),
                items: Vec::new(),
                module,
                alias: Some(alias),
            });
        }

        if self.match_token(&Token::LBrace) {
            // Pattern 1: import { items } from "module"
            let mut import_items = Vec::new();

            loop {
                let item_name = self.consume_identifier()?;
                
                // Check for alias: import { a as b }
                let alias = if self.match_token(&Token::As) {
                    Some(self.consume_identifier()?)
                } else {
                    None
                };

                import_items.push(ImportItem {
                    name: item_name,
                    alias,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }

            self.consume(&Token::RBrace, "Expected '}'")?;
            self.consume(&Token::From, "Expected 'from' after import items")?;

            let module = if let Token::StringLiteral(s) = self.peek() {
                let m = s.clone();
                self.advance();
                m
            } else {
                // Provide a richer diagnostic with suggestion for module path
                let span = self.token_to_diag_span(&self.peek_span().span);
                let diag = vex_diagnostics::Diagnostic::error(
                    error_codes::SYNTAX_ERROR,
                    "Expected module string after 'from'".to_string(),
                    span.clone(),
                )
                .with_primary_label("expected module string".to_string())
                .with_help("Use a module path string, e.g., from \"./module.vx\"".to_string())
                .with_suggestion(
                    "use module path".to_string(),
                    "\"./module.vx\"".to_string(),
                    span,
                );
                return Err(ParseError::from_diagnostic(diag));
            };

            self.consume(&Token::Semicolon, "Expected ';' after import")?;

            return Ok(Import {
                kind: ImportKind::Named,
                items: import_items,
                module,
                alias: None,
            });
        }

        if let Token::StringLiteral(s) = self.peek() {
            // Pattern 3: import "module"
            let module = s.clone();
            self.advance();
            self.consume(&Token::Semicolon, "Expected ';' after import")?;

            return Ok(Import {
                kind: ImportKind::Module,
                items: Vec::new(),
                module,
                alias: None,
            });
        }

        let span = self.token_to_diag_span(&self.peek_span().span);
        let diag = vex_diagnostics::Diagnostic::error(
            error_codes::SYNTAX_ERROR,
            "Expected '{', '*', or string after 'import'".to_string(),
            span.clone(),
        )
        .with_primary_label("expected import form".to_string())
        .with_help("Valid patterns: `import {name} from \"mod\";`, `import * as ns from \"mod\";`, or `import \"mod\";`".to_string())
        .with_suggestion(
            "try import pattern".to_string(),
            "import { io } from \"std\";".to_string(),
            span,
        );
        Err(ParseError::from_diagnostic(diag))
    }
}
