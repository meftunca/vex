// Type parsing for Vex language

use super::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type(&mut self) -> Result<Type, ParseError> {
        // Reference type: &T or &T!, or Slice type: &[T] or &[T]!
        if self.check(&Token::Ampersand) {
            self.advance();

            // Check if this is a slice type: &[T] or &[T]!
            if self.check(&Token::LBracket) {
                self.advance();
                let elem_ty = self.parse_type()?;
                self.consume(&Token::RBracket, "Expected ']' in slice type")?;

                // Check for mutable slice: &[T]!
                let is_mutable = self.match_token(&Token::Not);

                return Ok(Type::Slice(Box::new(elem_ty), is_mutable));
            }

            let inner_ty = self.parse_type()?;

            // Check for mutable reference: &T!
            let is_mutable = self.match_token(&Token::Not);

            return Ok(Type::Reference(Box::new(inner_ty), is_mutable));
        }

        // Tuple type: (T1, T2, T3)
        if self.check(&Token::LParen) {
            self.advance();
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
                return Ok(types.into_iter().next().unwrap());
            }

            return Ok(Type::Tuple(types));
        }

        if self.check(&Token::LBracket) {
            // Array type: [T; N]
            self.advance();
            let elem_ty = self.parse_type()?;
            self.consume(&Token::Semicolon, "Expected ';' in array type")?;

            let size = if let Token::IntLiteral(n) = self.peek() {
                let n_val = *n;
                self.advance();
                n_val as usize
            } else {
                return Err(self.error("Expected array size"));
            };

            self.consume(&Token::RBracket, "Expected ']'")?;
            return Ok(Type::Array(Box::new(elem_ty), size));
        }

        // Check for type keywords first (they are tokens, not identifiers)
        let ty = match self.peek() {
            Token::I8 => {
                self.advance();
                Type::I8
            }
            Token::I16 => {
                self.advance();
                Type::I16
            }
            Token::I32 => {
                self.advance();
                Type::I32
            }
            Token::I64 => {
                self.advance();
                Type::I64
            }
            Token::U8 => {
                self.advance();
                Type::U8
            }
            Token::U16 => {
                self.advance();
                Type::U16
            }
            Token::U32 => {
                self.advance();
                Type::U32
            }
            Token::U64 => {
                self.advance();
                Type::U64
            }
            Token::F32 => {
                self.advance();
                Type::F32
            }
            Token::F64 => {
                self.advance();
                Type::F64
            }
            Token::Bool => {
                self.advance();
                Type::Bool
            }
            Token::String => {
                self.advance();
                Type::String
            }
            Token::Byte => {
                self.advance();
                Type::U8 // byte is an alias for u8
            }
            Token::Nil => {
                self.advance();
                Type::Nil
            }
            Token::Ident(_) => {
                let name = self.consume_identifier()?;

                // Check for generic type arguments: Vec<T>, Option<String>
                if self.match_token(&Token::Lt) {
                    let mut type_args = Vec::new();
                    loop {
                        type_args.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::Gt, "Expected '>' after type arguments")?;

                    Type::Generic { name, type_args }
                } else {
                    Type::Named(name)
                }
            }
            // Allow keywords as custom type names (e.g., "error" struct)
            Token::Error => {
                self.advance();
                Type::Named("error".to_string())
            }
            Token::Type => {
                self.advance();
                Type::Named("type".to_string())
            }
            _ => {
                eprintln!(
                    "🔴 parse_type failed: token={:?} at position {}",
                    self.peek(),
                    self.current
                );
                return Err(self.error("Expected type"));
            }
        };

        // Check for conditional type: T extends U ? X : Y
        if self.check(&Token::Extends) {
            self.advance();
            let extends_type = Box::new(self.parse_type_primary()?);
            self.consume(&Token::Question, "Expected '?' in conditional type")?;
            let true_type = Box::new(self.parse_type()?);
            self.consume(&Token::Colon, "Expected ':' in conditional type")?;
            let false_type = Box::new(self.parse_type()?);
            return Ok(Type::Conditional {
                check_type: Box::new(ty),
                extends_type,
                true_type,
                false_type,
            });
        }

        // Check for intersection type: T1 & T2 & T3
        // Note: We only parse & as intersection if it's not a reference context
        // Reference: &T (leading ampersand)
        // Intersection: T1 & T2 (ampersand between types)
        if self.check(&Token::Ampersand)
            && !matches!(
                ty,
                Type::I8
                    | Type::I16
                    | Type::I32
                    | Type::I64
                    | Type::U8
                    | Type::U16
                    | Type::U32
                    | Type::U64
                    | Type::F32
                    | Type::F64
                    | Type::Bool
                    | Type::String
            )
        {
            // For named types and complex types, allow intersection
            let mut types = vec![ty];
            while self.match_token(&Token::Ampersand) {
                types.push(self.parse_type_primary()?);
            }
            return Ok(Type::Intersection(types));
        }

        // Check for union type: T1 | T2 | T3
        if self.check(&Token::Pipe) {
            let mut types = vec![ty];
            while self.match_token(&Token::Pipe) {
                types.push(self.parse_type_primary()?);
            }
            return Ok(Type::Union(types));
        }

        Ok(ty)
    }

    /// Parse a primary type (without union operator)
    /// This is used internally to avoid infinite recursion in union type parsing
    pub(crate) fn parse_type_primary(&mut self) -> Result<Type, ParseError> {
        // Infer type: infer E (used in conditional types)
        if self.check(&Token::Infer) {
            self.advance();
            let name = self.consume_identifier()?;
            return Ok(Type::Infer(name));
        }

        // Reference type: &T or &T! (v0.9 syntax), or Slice type: &[T] or &[T]!
        if self.check(&Token::Ampersand) {
            self.advance();

            // Check if this is a slice type: &[T] or &[T]!
            if self.check(&Token::LBracket) {
                self.advance();
                let elem_ty = self.parse_type_primary()?;
                self.consume(&Token::RBracket, "Expected ']' in slice type")?;
                // Check for ! after ] for mutable slice: &[T]!
                let is_mutable = self.match_token(&Token::Not);
                return Ok(Type::Slice(Box::new(elem_ty), is_mutable));
            }

            let inner_ty = self.parse_type_primary()?;
            // Check for ! after type for mutable reference: &T!
            let is_mutable = self.match_token(&Token::Not);
            return Ok(Type::Reference(Box::new(inner_ty), is_mutable));
        }

        // Tuple type: (T1, T2, T3)
        if self.check(&Token::LParen) {
            self.advance();
            let mut types = Vec::new();

            if !self.check(&Token::RParen) {
                loop {
                    types.push(self.parse_type_primary()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }

            self.consume(&Token::RParen, "Expected ')' after tuple type")?;

            // A single type in parens is just that type, not a tuple
            if types.len() == 1 {
                return Ok(types.into_iter().next().unwrap());
            }

            return Ok(Type::Tuple(types));
        }

        // Array type: [T; N]
        if self.check(&Token::LBracket) {
            self.advance();
            let elem_ty = self.parse_type_primary()?;
            self.consume(&Token::Semicolon, "Expected ';' in array type")?;

            let size = if let Token::IntLiteral(n) = self.peek() {
                let n_val = *n;
                self.advance();
                n_val as usize
            } else {
                return Err(self.error("Expected array size"));
            };

            self.consume(&Token::RBracket, "Expected ']'")?;
            return Ok(Type::Array(Box::new(elem_ty), size));
        }

        // Primitive and named types
        let ty = match self.peek() {
            Token::I8 => {
                self.advance();
                Type::I8
            }
            Token::I16 => {
                self.advance();
                Type::I16
            }
            Token::I32 => {
                self.advance();
                Type::I32
            }
            Token::I64 => {
                self.advance();
                Type::I64
            }
            Token::U8 => {
                self.advance();
                Type::U8
            }
            Token::U16 => {
                self.advance();
                Type::U16
            }
            Token::U32 => {
                self.advance();
                Type::U32
            }
            Token::U64 => {
                self.advance();
                Type::U64
            }
            Token::F32 => {
                self.advance();
                Type::F32
            }
            Token::F64 => {
                self.advance();
                Type::F64
            }
            Token::Bool => {
                self.advance();
                Type::Bool
            }
            Token::String => {
                self.advance();
                Type::String
            }
            Token::Byte => {
                self.advance();
                Type::U8 // byte is an alias for u8
            }
            Token::Nil => {
                self.advance();
                Type::Nil
            }
            Token::Ident(_) => {
                let name = self.consume_identifier()?;

                // Check for generic type arguments: Vec<T>, Option<String>
                if self.match_token(&Token::Lt) {
                    let mut type_args = Vec::new();
                    loop {
                        type_args.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::Gt, "Expected '>' after type arguments")?;

                    Type::Generic { name, type_args }
                } else {
                    Type::Named(name)
                }
            }
            Token::Error => {
                self.advance();
                Type::Named("error".to_string())
            }
            Token::Type => {
                self.advance();
                Type::Named("type".to_string())
            }
            _ => return Err(self.error("Expected type")),
        };

        Ok(ty)
    }
}
