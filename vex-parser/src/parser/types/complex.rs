//! Complex type parsing (references, slices, tuples, arrays, function types)

use super::Parser;
use crate::ParseError;
use vex_ast::Type;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    /// Try to parse complex types (reference, slice, tuple, array)
    pub(super) fn try_parse_complex_type(&mut self) -> Result<Option<Type>, ParseError> {
        // Reference type: &T or &T!, or Slice type: &[T] or &[T]!
        if self.check(&Token::Ampersand) {
            return Ok(Some(self.parse_reference_or_slice_type()?));
        }

        // Tuple type: (T1, T2, T3)
        if self.check(&Token::LParen) {
            return Ok(Some(self.parse_tuple_type()?));
        }

        // Array type: [T; N]
        if self.check(&Token::LBracket) {
            return Ok(Some(self.parse_array_type()?));
        }

        Ok(None)
    }

    /// Parse reference or slice type: &T, &T!, &[T], &[T]!
    fn parse_reference_or_slice_type(&mut self) -> Result<Type, ParseError> {
        self.advance(); // consume &

        // Check if this is a slice type: &[T] or &[T]!
        if self.check(&Token::LBracket) {
            self.advance();
            let elem_ty = self.parse_type()?;
            self.consume(&Token::RBracket, "Expected ']' in slice type")?;

            // Check for mutable slice: &[T]!
            let is_mutable = self.match_token(&Token::Not);

            return Ok(Type::Slice(Box::new(elem_ty), is_mutable));
        }

        // Regular reference: &T or &T!
        let inner_ty = self.parse_type()?;

        // Check for mutable reference: &T!
        let is_mutable = self.match_token(&Token::Not);

        Ok(Type::Reference(Box::new(inner_ty), is_mutable))
    }

    /// Parse tuple type: (T1, T2, T3)
    fn parse_tuple_type(&mut self) -> Result<Type, ParseError> {
        self.advance(); // consume (
        let mut types = Vec::new();

        if !self.check(&Token::RParen) {
            loop {
                types.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RParen, "Expected ')' after tuple type")?;

        // A single type in parens is just that type, not a tuple
        if types.len() == 1 {
            return Ok(types.into_iter().next().ok_or_else(|| {
                self.make_syntax_error(
                    "Expected at least one type in tuple",
                    Some("expected tuple type"),
                    Some("A single type in parentheses is just that type - add a trailing comma to make a tuple type"),
                    Some(("add trailing comma to make it a tuple type", "(i32,)")),
                )
            })?);
        }

        Ok(Type::Tuple(types))
    }

    /// Parse array type: [T; N]
    fn parse_array_type(&mut self) -> Result<Type, ParseError> {
        self.advance(); // consume [
        let elem_ty = self.parse_type()?;
        self.consume(&Token::Semicolon, "Expected ';' in array type")?;

        let size = if let Token::IntLiteral(s) = self.peek() {
            let s_val = s.clone();
            self.advance();
            s_val
                .parse::<usize>()
                    .map_err(|_| self.make_syntax_error(
                        &format!("Array size out of range: {}", s_val),
                        Some("array size out of range"),
                        Some("Array size must be a positive integer within range"),
                        Some(("try a small integer", "10")),
                    ))?
        } else {
            return Err(self.make_syntax_error(
                "Expected array size",
                Some("expected array size"),
                Some("Specify the size for the array: [T; N]"),
                Some(("add a size", "[i32; 10]")),
            ));
        };

        self.consume(&Token::RBracket, "Expected ']'")?;
        Ok(Type::Array(Box::new(elem_ty), size))
    }

    /// Parse function type: fn(T1, T2): R
    pub(super) fn parse_function_type(&mut self) -> Result<Type, ParseError> {
        self.advance(); // consume 'fn'
        self.consume(&Token::LParen, "Expected '(' after 'fn'")?;

        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                params.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::RParen, "Expected ')' after function parameters")?;
        self.consume(&Token::Colon, "Expected ':' in function type")?;
        let return_type = Box::new(self.parse_type()?);

        Ok(Type::Function {
            params,
            return_type,
        })
    }
}
