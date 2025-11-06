use crate::parser::Parser;
use crate::ParseError;
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
                return Err(self.error("Expected module string after 'from'"));
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
                let item = self.consume_identifier()?;
                import_items.push(item);

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
                return Err(self.error("Expected module string after 'from'"));
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

        Err(self.error("Expected '{', '*', or string after 'import'"))
    }
}
