// Modular parser for Vex language
// This module organizes the parser into logical components

use crate::ParseError;
use vex_ast::*;
use vex_lexer::{Lexer, Token, TokenSpan};

// Sub-modules for different parsing responsibilities
mod expressions;
mod items;
mod statements;
mod types;

// Re-export Parser as the main public interface
pub struct Parser<'a> {
    pub(crate) tokens: Vec<TokenSpan>,
    pub(crate) current: usize,
    pub(crate) source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let lexer = Lexer::new(source);
        let tokens: Result<Vec<_>, _> = lexer.collect();
        let tokens = tokens.map_err(|e| ParseError::LexerError(format!("{:?}", e)))?;

        Ok(Self {
            tokens,
            current: 0,
            source,
        })
    }

    pub fn parse_file(&mut self) -> Result<Program, ParseError> {
        self.parse()
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut imports = Vec::new();
        let mut items = Vec::new();

        while !self.is_at_end() {
            // Parse top-level items
            if self.check(&Token::Import) {
                imports.push(self.parse_import()?);
            } else if self.check(&Token::Export) {
                items.push(self.parse_export()?);
            } else if self.check(&Token::Const) {
                items.push(self.parse_const()?);
            } else if self.check(&Token::Async) {
                // async fn function
                self.advance(); // consume 'async'
                self.consume(&Token::Fn, "Expected 'fn' after 'async'")?;
                let mut func = self.parse_function()?;
                func.is_async = true;
                items.push(Item::Function(func));
            } else if self.check(&Token::Fn) {
                self.advance(); // consume 'fn'
                items.push(Item::Function(self.parse_function()?));
            } else if self.check(&Token::Struct) {
                items.push(self.parse_struct()?);
            } else if self.check(&Token::Type) {
                items.push(self.parse_type_alias()?);
            } else if self.check(&Token::Enum) {
                items.push(self.parse_enum()?);
            } else if self.check(&Token::Interface) || self.check(&Token::Trait) {
                items.push(self.parse_interface_or_trait()?);
            } else if self.check(&Token::Impl) {
                items.push(self.parse_trait_impl()?);
            } else if self.check(&Token::Extern) {
                items.push(self.parse_extern_block()?);
            } else {
                return Err(self.error(
                    "Expected top-level item (import, export, const, fn, struct, type, enum, interface, trait, impl, extern)",
                ));
            }
        }
        Ok(Program { imports, items })
    }

    // ==================== Helper Methods ====================

    pub(crate) fn match_token(&mut self, kind: &Token) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn match_tokens(&mut self, kinds: &[Token]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    pub(crate) fn check(&self, kind: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek()) == std::mem::discriminant(kind)
        }
    }

    pub(crate) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    pub(crate) fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    pub(crate) fn peek_span(&self) -> &TokenSpan {
        &self.tokens[self.current]
    }

    pub(crate) fn peek(&self) -> &Token {
        &self.tokens[self.current].token
    }

    pub(crate) fn previous(&self) -> &Token {
        &self.tokens[self.current - 1].token
    }

    pub(crate) fn consume(&mut self, kind: &Token, message: &str) -> Result<(), ParseError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    pub(crate) fn error(&self, message: &str) -> ParseError {
        let location = if self.is_at_end() {
            "end of file".to_string()
        } else {
            let span = &self.peek_span().span;
            format!("{}..{}", span.start, span.end)
        };

        ParseError::SyntaxError {
            location,
            message: message.to_string(),
        }
    }

    /// Skip tokens until we find a semicolon or closing brace (for unsupported constructs)
    pub(crate) fn skip_until_semicolon_or_brace(&mut self) -> Result<(), ParseError> {
        let mut brace_depth = 0;

        while !self.is_at_end() {
            match self.peek() {
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

        Ok(())
    }

    pub(crate) fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.consume(&Token::LBrace, "Expected '{'")?;
        let mut statements = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Block { statements })
    }

    pub(crate) fn parse_block_until_case_or_brace(&mut self) -> Result<Block, ParseError> {
        let mut statements = Vec::new();

        // Parse statements until we hit "case", "default", or closing brace
        while !self.check(&Token::Case)
            && !self.check(&Token::Default)
            && !self.check(&Token::RBrace)
            && !self.is_at_end()
        {
            statements.push(self.parse_statement()?);
        }

        Ok(Block { statements })
    }
}
