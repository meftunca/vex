// Type parsing for Vex language

use super::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type(&mut self) -> Result<Type, ParseError> {
        // Infer type: infer E (used in conditional types - must be checked FIRST)
        if self.check(&Token::Infer) {
            self.advance();
            let name = self.consume_identifier()?;
            return Ok(Type::Infer(name));
        }

        // Typeof type: typeof(expr)
        if self.check(&Token::Typeof) {
            self.advance();
            self.consume(&Token::LParen, "Expected '(' after 'typeof'")?;
            let expr = self.parse_expression()?;
            self.consume(&Token::RParen, "Expected ')' after typeof expression")?;
            return Ok(Type::Typeof(Box::new(expr)));
        }

        // Never type: !
        if self.check(&Token::Not) {
            self.advance();
            return Ok(Type::Never);
        }

        // Raw pointer type: *T (immutable) or *T! (mutable)
        if self.check(&Token::Star) {
            self.advance();

            // Parse inner type
            let inner_ty = self.parse_type()?;

            // Check for ! after type for mutable pointer: *T!
            let is_mutable = self.match_token(&Token::Not);

            return Ok(Type::RawPtr {
                inner: Box::new(inner_ty),
                is_const: !is_mutable, // is_const is opposite of is_mutable
            });
        }

        // Function type: fn(T1, T2): R (Vex uses : not ->)
        if self.check(&Token::Fn) {
            self.advance();
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

            return Ok(Type::Function {
                params,
                return_type,
            });
        }

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

        // Tuple type or Union/Intersection in parens: (T1, T2, T3) or (T1 | T2)
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

            let size = if let Token::IntLiteral(s) = self.peek() {
                let s_val = s.clone();
                self.advance();
                s_val
                    .parse::<usize>()
                    .map_err(|_| self.error(&format!("Array size out of range: {}", s_val)))?
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
            Token::I128 => {
                self.advance();
                Type::I128
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
            Token::U128 => {
                self.advance();
                Type::U128
            }
            Token::F16 => {
                self.advance();
                Type::F16
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
            Token::Any => {
                self.advance();
                Type::Any
            }
            Token::Byte => {
                self.advance();
                Type::U8 // byte is an alias for u8
            }
            Token::Nil => {
                self.advance();
                Type::Nil
            }
            Token::Map => {
                // Map or Map<K, V>
                self.advance();
                if self.check(&Token::Lt) {
                    self.consume(&Token::Lt, "Expected '<' after 'Map'")?;
                    let _key_type = self.parse_type()?;
                    self.consume(&Token::Comma, "Expected ',' in Map type")?;
                    let _value_type = self.parse_type()?;
                    self.consume_generic_close("Expected '>' after Map type arguments")?;
                }
                Type::Named("Map".to_string())
            }
            Token::Set => {
                // Set or Set<T>
                self.advance();
                if self.check(&Token::Lt) {
                    self.consume(&Token::Lt, "Expected '<' after 'Set'")?;
                    let _elem_type = self.parse_type()?;
                    self.consume_generic_close("Expected '>' after Set type argument")?;
                }
                Type::Named("Set".to_string())
            }
            Token::Ident(_) => {
                let name = self.consume_identifier()?;

                // ⭐ NEW: Check for Self type
                if name == "Self" {
                    // Check for associated type: Self.Item (Vex uses . not ::)
                    if self.check(&Token::Dot) {
                        self.advance(); // consume .
                        let assoc_name = self.consume_identifier()?;

                        return Ok(Type::AssociatedType {
                            self_type: Box::new(Type::SelfType),
                            name: assoc_name,
                        });
                    }

                    return Ok(Type::SelfType);
                }

                // Check for builtin types (Phase 0: Option, Result, Vec, Box, Map)
                // These are parsed specially to generate Type enum variants
                match name.as_str() {
                    "Option" => {
                        // Option<T>
                        self.consume(&Token::Lt, "Expected '<' after 'Option'")?;
                        let inner_type = Box::new(self.parse_type()?);
                        self.consume_generic_close("Expected '>' after Option type argument")?;
                        Type::Option(inner_type)
                    }
                    "Result" => {
                        // Result<T, E>
                        self.consume(&Token::Lt, "Expected '<' after 'Result'")?;
                        let ok_type = Box::new(self.parse_type()?);
                        self.consume(&Token::Comma, "Expected ',' in Result type")?;
                        let err_type = Box::new(self.parse_type()?);
                        self.consume_generic_close("Expected '>' after Result type arguments")?;
                        Type::Result(ok_type, err_type)
                    }
                    "Vec" => {
                        // Vec<T>
                        self.consume(&Token::Lt, "Expected '<' after 'Vec'")?;
                        let elem_type = Box::new(self.parse_type()?);
                        self.consume_generic_close("Expected '>' after Vec type argument")?;
                        Type::Vec(elem_type)
                    }
                    "Box" => {
                        // Box<T>
                        self.consume(&Token::Lt, "Expected '<' after 'Box'")?;
                        let inner_type = Box::new(self.parse_type()?);
                        self.consume_generic_close("Expected '>' after Box type argument")?;
                        Type::Box(inner_type)
                    }
                    "Channel" => {
                        // Channel<T>
                        self.consume(&Token::Lt, "Expected '<' after 'Channel'")?;
                        let inner_type = Box::new(self.parse_type()?);
                        self.consume_generic_close("Expected '>' after Channel type argument")?;
                        Type::Channel(inner_type)
                    }
                    "Map" => {
                        // Map<K, V> - For now, treat as Named("Map") since we don't have generic maps yet
                        // Just consume the type arguments but don't use them
                        if self.check(&Token::Lt) {
                            self.consume(&Token::Lt, "Expected '<' after 'Map'")?;
                            let _key_type = self.parse_type()?;
                            self.consume(&Token::Comma, "Expected ',' in Map type")?;
                            let _value_type = self.parse_type()?;
                            self.consume_generic_close("Expected '>' after Map type arguments")?;
                        }
                        Type::Named("Map".to_string())
                    }
                    _ => {
                        // Generic type or named type
                        if self.match_token(&Token::Lt) {
                            let mut type_args = Vec::new();
                            loop {
                                type_args.push(self.parse_type()?);
                                if !self.match_token(&Token::Comma) {
                                    break;
                                }
                            }
                            self.consume_generic_close("Expected '>' after type arguments")?;
                            Type::Generic { name, type_args }
                        } else {
                            Type::Named(name)
                        }
                    }
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
            Token::Not => {
                // Never type: !
                self.advance();
                Type::Never
            }
            Token::Star => {
                // Raw pointer: *T or *const T
                self.advance();

                let is_const = if self.check(&Token::Const) {
                    self.advance();
                    true
                } else {
                    false
                };

                let inner_ty = self.parse_type()?;
                Type::RawPtr {
                    inner: Box::new(inner_ty),
                    is_const,
                }
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
    

        // Never type: !
        if self.check(&Token::Not) {
            self.advance();
            return Ok(Type::Never);
        }

        // Raw pointer type: *T (immutable) or *T! (mutable)
        if self.check(&Token::Star) {
            self.advance();

            // Parse inner type first
            let inner_ty = self.parse_type_primary()?;

            // Check for ! after type for mutable pointer: *T!
            let is_mutable = self.match_token(&Token::Not);

            return Ok(Type::RawPtr {
                inner: Box::new(inner_ty),
                is_const: !is_mutable, // is_const is opposite of is_mutable
            });
        }

        // Infer type: infer E (used in conditional types)
        if self.check(&Token::Infer) {
            self.advance();
            let name = self.consume_identifier()?;
            return Ok(Type::Infer(name));
        }

        // Reference type: &T or &T! (v0.1 syntax), or Slice type: &[T] or &[T]!
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

            let size = if let Token::IntLiteral(s) = self.peek() {
                let s_val = s.clone();
                self.advance();
                s_val
                    .parse::<usize>()
                    .map_err(|_| self.error(&format!("Array size out of range: {}", s_val)))?
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
            Token::I128 => {
                self.advance();
                Type::I128
            }
            Token::U128 => {
                self.advance();
                Type::U128
            }
            Token::F16 => {
                self.advance();
                Type::F16
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
            Token::Any => {
                self.advance();
                Type::Any
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
                    self.consume_generic_close("Expected '>' after type arguments")?;

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
