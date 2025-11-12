// Modular parser for Vex language
// This module organizes the parser into logical components

use crate::{ParseError, SourceLocation};
use vex_ast::*;
use vex_lexer::{Lexer, Token, TokenSpan};

// Sub-modules for different parsing responsibilities
mod error_recovery;
mod expressions;
mod items;
mod operators;
mod patterns;
mod primaries;
mod statements;
mod types;

// Re-export Parser as the main public
pub struct Parser<'a> {
    pub tokens: Vec<TokenSpan>, // Make public for debugging
    pub(crate) current: usize,
    pub(crate) source: &'a str,
    pub(crate) file_name: String, // Track filename for error reporting
    pub(crate) in_method_body: bool,
    pub(crate) span_map: vex_diagnostics::SpanMap, // ⭐ NEW: Track spans for AST nodes
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        Self::new_with_file("<input>", source)
    }

    pub fn new_with_file(file_name: &str, source: &'a str) -> Result<Self, ParseError> {
        let lexer = Lexer::new(source);
        let tokens: Result<Vec<_>, _> = lexer.collect();
        let tokens = tokens.map_err(|e| ParseError::LexerError(format!("{:?}", e)))?;

        Ok(Self {
            tokens,
            current: 0,
            source,
            file_name: file_name.to_string(),
            in_method_body: false,
            span_map: vex_diagnostics::SpanMap::new(),
        })
    }

    /// Get the span map (for passing to compiler)
    pub fn span_map(&self) -> &vex_diagnostics::SpanMap {
        &self.span_map
    }

    /// Take ownership of span map
    pub fn take_span_map(self) -> vex_diagnostics::SpanMap {
        self.span_map
    }

    pub fn parse_file(&mut self) -> Result<Program, ParseError> {
        self.parse()
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut imports = Vec::new();
        let mut items = Vec::new();

        println!(
            "🔧 Parser: Starting parse, total tokens: {}",
            self.tokens.len()
        );

        while !self.is_at_end() {
            println!(
                "🔧 Parser: Current token at {}: {:?}",
                self.current,
                self.peek()
            );
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
                                // Note: parse_function() handles receiver syntax: fn (self: Type) name()
                items.push(Item::Function(self.parse_function()?));
            } else if self.check(&Token::Struct) {
                items.push(self.parse_struct()?);
            } else if self.check(&Token::Type) {
                items.push(self.parse_type_alias()?);
            } else if self.check(&Token::Enum) {
                items.push(self.parse_enum()?);
            } else if self.check(&Token::Contract) {
                items.push(self.parse_trait()?);
            } else if self.check(&Token::Impl) {
                items.push(self.parse_trait_impl()?);
            } else if self.check(&Token::Policy) {
                items.push(Item::Policy(self.parse_policy()?));
            } else if self.check(&Token::Extern) {
                eprintln!("🔧 Parser: Found extern keyword");
                items.push(self.parse_extern_block()?);
            } else {
                eprintln!("🔧 Parser: Unknown token: {:?}", self.peek());
                return Err(self.error(
                    "Expected top-level item (import, export, const, fn, struct, type, enum, contract, impl, policy, extern)",
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

    pub(crate) fn consume_generic_close(&mut self, message: &str) -> Result<(), ParseError> {
        if self.match_token(&Token::Gt) {
            return Ok(());
        }

        if self.check(&Token::RShift) {
            let span = self.peek_span().span.clone();
            self.advance();

            let idx = self.current - 1;
            self.tokens[idx].token = Token::Gt;
            self.tokens[idx].span = span.clone();

            self.tokens.insert(
                self.current,
                TokenSpan {
                    token: Token::Gt,
                    span,
                },
            );

            return Ok(());
        }

        Err(self.error(message))
    }

    pub(crate) fn error(&self, message: &str) -> ParseError {
        let location = if self.is_at_end() {
            SourceLocation {
                file: self.file_name.clone(),
                line: self.source.lines().count(),
                column: self.source.lines().last().map_or(0, |l| l.len()),
                length: 0,
            }
        } else {
            let span = &self.peek_span().span;
            crate::Span::from_file_and_span(&self.file_name, self.source, span.clone())
        };

        ParseError::syntax_error(message.to_string(), location)
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

    /// Parse block as expression: { stmt1; stmt2; expr }
    /// Last expression without semicolon becomes the return value
    pub(crate) fn parse_block_expression(&mut self) -> Result<Expression, ParseError> {
        self.consume(&Token::LBrace, "Expected '{'")?;
        let mut statements = Vec::new();
        let mut return_expr = None;

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Try to parse as expression first (peek for semicolon)
            let checkpoint = self.current;

            // If next token is not a statement keyword, might be an expression
            if !self.check(&Token::Let)
                && !self.check(&Token::Return)
                && !self.check(&Token::If)
                && !self.check(&Token::While)
                && !self.check(&Token::For)
                && !self.check(&Token::Break)
                && !self.check(&Token::Continue)
                && !self.check(&Token::Switch)
                && !self.check(&Token::Defer)
            {
                // Try to parse expression
                if let Ok(expr) = self.parse_expression() {
                    // If no semicolon follows and we're at closing brace, this is the return expr
                    if !self.match_token(&Token::Semicolon) && self.check(&Token::RBrace) {
                        return_expr = Some(Box::new(expr));
                        break;
                    } else {
                        // Expression statement
                        statements.push(Statement::Expression(expr));
                        continue;
                    }
                } else {
                    // Failed to parse as expression, restore and try statement
                    self.current = checkpoint;
                }
            }

            // Parse as statement
            statements.push(self.parse_statement()?);
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Expression::Block {
            statements,
            return_expr,
        })
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

    /// Parse generic type parameters with optional trait bounds
    /// Examples: <T>, <T: Display>, <T: Display + Clone, U: Debug>
    /// Closure traits: <F: Callable(i32): i32>, <F: CallableMut(T, U): bool>
    pub(crate) fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, ParseError> {
        if !self.match_token(&Token::Lt) {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();
        loop {
            let name = self.consume_identifier()?;

            // Optional trait bounds: T: Display + Clone or F: Callable(i32): i32
            let mut bounds = Vec::new();
            if self.match_token(&Token::Colon) {
                loop {
                    // Check if this is a closure trait bound (Callable, CallableMut, CallableOnce)
                    let bound_name = self.consume_identifier()?;

                    if (bound_name == "Callable"
                        || bound_name == "CallableMut"
                        || bound_name == "CallableOnce")
                        && self.check(&Token::LParen)
                    {
                        // Parse closure trait: Callable(T, U): ReturnType
                        self.consume(&Token::LParen, "Expected '(' after closure trait name")?;

                        let mut param_types = Vec::new();
                        if !self.check(&Token::RParen) {
                            loop {
                                param_types.push(self.parse_type()?);
                                if !self.match_token(&Token::Comma) {
                                    break;
                                }
                            }
                        }

                        self.consume(&Token::RParen, "Expected ')' after closure parameters")?;
                        self.consume(&Token::Colon, "Expected ':' before closure return type")?;

                        let return_type = Box::new(self.parse_type()?);

                        bounds.push(TraitBound::Callable {
                            trait_name: bound_name,
                            param_types,
                            return_type,
                        });
                    } else {
                        // Simple trait bound
                        bounds.push(TraitBound::Simple(bound_name));
                    }

                    if !self.match_token(&Token::Plus) {
                        break;
                    }
                }
            }

            params.push(TypeParam { name, bounds });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.consume_generic_close("Expected '>' after type parameters")?;
        Ok(params)
    }
}
