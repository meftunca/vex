use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    // ==================== FFI / Extern Block Parsing ====================

    pub(crate) fn parse_extern_block(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Extern, "Expected 'extern'")?;

        // Parse ABI: extern "C" { ... } or extern "system" { ... }
        let abi = if let Token::StringLiteral(s) = self.peek() {
            let abi = s.clone();
            self.advance();
            abi
        } else {
            "C".to_string() // Default to C ABI
        };

        self.consume(&Token::LBrace, "Expected '{' after extern ABI")?;

        let mut types = Vec::new();
        let mut functions = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check if it's a type declaration or function
            if self.check(&Token::Type) {
                types.push(self.parse_extern_type()?);
            } else if self.check(&Token::Fn) {
                functions.push(self.parse_extern_function()?);
            } else {
                return Err(self.error("Expected 'type' or 'fn' in extern block"));
            }
        }

        self.consume(&Token::RBrace, "Expected '}' after extern block")?;

        Ok(Item::ExternBlock(ExternBlock {
            abi,
            types,
            functions,
        }))
    }

    fn parse_extern_function(&mut self) -> Result<ExternFunction, ParseError> {
        self.consume(&Token::Fn, "Expected 'fn' in extern block")?;

        let name = self.consume_identifier()?;

        self.consume(&Token::LParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        let mut is_variadic = false;

        if !self.check(&Token::RParen) {
            loop {
                // Check for variadic: ...
                if self.match_token(&Token::DotDotDot) {
                    is_variadic = true;
                    break;
                }

                let param_name = self.consume_identifier()?;
                self.consume(&Token::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;

                params.push(Param {
                    name: param_name,
                    ty: param_type,
                    default_value: None,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }

                // Check for variadic after comma: fn printf(fmt: *byte, ...)
                if self.check(&Token::DotDotDot) {
                    self.advance();
                    is_variadic = true;
                    break;
                }
            }
        }

        self.consume(&Token::RParen, "Expected ')' after parameters")?;

        // Optional return type: : Type or -> Type (both allowed)
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&Token::Semicolon, "Expected ';' after extern function")?;

        Ok(ExternFunction {
            name,
            params,
            return_type,
            is_variadic,
            variadic_type: None, // C-style variadic (no type info)
        })
    }

    fn parse_extern_type(&mut self) -> Result<ExternType, ParseError> {
        self.consume(&Token::Type, "Expected 'type'")?;

        let name = self.consume_identifier()?;

        // Check for type alias: type VexDuration = i64;
        let alias = if self.match_token(&Token::Eq) {
            Some(self.parse_type()?)
        } else {
            None // Opaque type
        };

        self.consume(&Token::Semicolon, "Expected ';' after extern type")?;

        Ok(ExternType { name, alias })
    }
}
