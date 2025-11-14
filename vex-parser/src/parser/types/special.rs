//! Special type parsing (typeof, infer, never, raw pointers)

use super::Parser;
use crate::ParseError;
use vex_ast::Type;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    /// Try to parse special types (infer, typeof, never, raw pointers)
    pub(super) fn try_parse_special_type(&mut self) -> Result<Option<Type>, ParseError> {
        // Infer type: infer E (used in conditional types)
        if self.check(&Token::Infer) {
            return Ok(Some(self.parse_infer_type()?));
        }

        // Typeof type: typeof(expr)
        if self.check(&Token::Typeof) {
            return Ok(Some(self.parse_typeof_type()?));
        }

        // Never type: !
        if self.check(&Token::Not) {
            return Ok(Some(self.parse_never_type()));
        }

        // Raw pointer type: *T or *T!
        if self.check(&Token::Star) {
            return Ok(Some(self.parse_raw_pointer_type()?));
        }

        Ok(None)
    }

    /// Parse infer type: infer E
    fn parse_infer_type(&mut self) -> Result<Type, ParseError> {
        self.advance(); // consume 'infer'
        let name = self.consume_identifier()?;
        Ok(Type::Infer(name))
    }

    /// Parse typeof type: typeof(expr)
    fn parse_typeof_type(&mut self) -> Result<Type, ParseError> {
        self.advance(); // consume 'typeof'
        self.consume(&Token::LParen, "Expected '(' after 'typeof'")?;
        let expr = self.parse_expression()?;
        self.consume(&Token::RParen, "Expected ')' after typeof expression")?;
        Ok(Type::Typeof(Box::new(expr)))
    }

    /// Parse never type: !
    fn parse_never_type(&mut self) -> Type {
        self.advance(); // consume '!'
        Type::Never
    }

    /// Parse raw pointer type: *T (immutable) or *T! (mutable)
    fn parse_raw_pointer_type(&mut self) -> Result<Type, ParseError> {
        self.advance(); // consume '*'

        // Check for old syntax: *const T
        let is_const = if self.check(&Token::Const) {
            self.advance();
            true
        } else {
            false
        };

        // Parse inner type
        let inner_ty = if is_const {
            // Old syntax: *const T uses parse_type_primary to avoid recursion
            self.parse_type_primary()?
        } else {
            // New syntax: *T or *T! can use full type parsing
            self.parse_type()?
        };

        // Check for ! after type for mutable pointer: *T!
        let is_mutable = if !is_const {
            self.match_token(&Token::Not)
        } else {
            false
        };

        Ok(Type::RawPtr {
            inner: Box::new(inner_ty),
            is_const: if is_const { true } else { !is_mutable },
        })
    }
}
