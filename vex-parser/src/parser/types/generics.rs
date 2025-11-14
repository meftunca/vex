//! Generic and named type parsing (Vec<T>, Option<T>, custom types)

use super::Parser;
use crate::ParseError;
use vex_ast::Type;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    /// Parse named or generic type (including builtin generic types)
    pub(super) fn parse_named_or_generic_type(&mut self) -> Result<Type, ParseError> {
        // Handle keyword-based types that can be identifiers
        match self.peek() {
            Token::Error => {
                self.advance();
                return Ok(Type::Named("error".to_string()));
            }
            Token::Type => {
                self.advance();
                return Ok(Type::Named("type".to_string()));
            }
            Token::Map => {
                self.advance();
                return self.parse_map_type();
            }
            Token::Set => {
                self.advance();
                return self.parse_set_type();
            }
            Token::Ident(_) => {
                let name = self.consume_identifier()?;
                return self.parse_identifier_type(name);
            }
            _ => {
                eprintln!(
                    "🔴 parse_type failed: token={:?} at position {}",
                    self.peek(),
                    self.current
                );
                return Err(self.error("Expected type"));
            }
        }
    }

    /// Parse type starting with an identifier
    fn parse_identifier_type(&mut self, name: String) -> Result<Type, ParseError> {
        // Check for Self type
        if name == "Self" {
            return self.parse_self_type();
        }

        // Check for builtin generic types
        match name.as_str() {
            "Option" => self.parse_option_type(),
            "Result" => self.parse_result_type(),
            "Vec" => self.parse_vec_type(),
            "Box" => self.parse_box_type(),
            "Channel" => self.parse_channel_type(),
            "Future" => self.parse_future_type(),
            "Map" => self.parse_map_type(),
            _ => self.parse_generic_or_named(name),
        }
    }

    /// Parse Self type and associated types
    fn parse_self_type(&mut self) -> Result<Type, ParseError> {
        // Check for associated type: Self.Item (Vex uses . not ::)
        if self.check(&Token::Dot) {
            self.advance(); // consume .
            let assoc_name = self.consume_identifier()?;

            return Ok(Type::AssociatedType {
                self_type: Box::new(Type::SelfType),
                name: assoc_name,
            });
        }

        Ok(Type::SelfType)
    }

    /// Parse Option<T>
    fn parse_option_type(&mut self) -> Result<Type, ParseError> {
        self.consume(&Token::Lt, "Expected '<' after 'Option'")?;
        let inner_type = Box::new(self.parse_type()?);
        self.consume_generic_close("Expected '>' after Option type argument")?;
        Ok(Type::Option(inner_type))
    }

    /// Parse Result<T, E>
    fn parse_result_type(&mut self) -> Result<Type, ParseError> {
        self.consume(&Token::Lt, "Expected '<' after 'Result'")?;
        let ok_type = Box::new(self.parse_type()?);
        self.consume(&Token::Comma, "Expected ',' in Result type")?;
        let err_type = Box::new(self.parse_type()?);
        self.consume_generic_close("Expected '>' after Result type arguments")?;
        Ok(Type::Result(ok_type, err_type))
    }

    /// Parse Vec<T>
    fn parse_vec_type(&mut self) -> Result<Type, ParseError> {
        self.consume(&Token::Lt, "Expected '<' after 'Vec'")?;
        let elem_type = Box::new(self.parse_type()?);
        self.consume_generic_close("Expected '>' after Vec type argument")?;
        Ok(Type::Vec(elem_type))
    }

    /// Parse Box<T>
    fn parse_box_type(&mut self) -> Result<Type, ParseError> {
        self.consume(&Token::Lt, "Expected '<' after 'Box'")?;
        let inner_type = Box::new(self.parse_type()?);
        self.consume_generic_close("Expected '>' after Box type argument")?;
        Ok(Type::Box(inner_type))
    }

    /// Parse Channel<T>
    fn parse_channel_type(&mut self) -> Result<Type, ParseError> {
        self.consume(&Token::Lt, "Expected '<' after 'Channel'")?;
        let inner_type = Box::new(self.parse_type()?);
        self.consume_generic_close("Expected '>' after Channel type argument")?;
        Ok(Type::Channel(inner_type))
    }

    /// Parse Future<T>
    fn parse_future_type(&mut self) -> Result<Type, ParseError> {
        self.consume(&Token::Lt, "Expected '<' after 'Future'")?;
        let inner_type = Box::new(self.parse_type()?);
        self.consume_generic_close("Expected '>' after Future type argument")?;
        Ok(Type::Future(inner_type))
    }

    /// Parse Map<K, V> or Map (without generics)
    fn parse_map_type(&mut self) -> Result<Type, ParseError> {
        if self.check(&Token::Lt) {
            self.consume(&Token::Lt, "Expected '<' after 'Map'")?;
            let _key_type = self.parse_type()?;
            self.consume(&Token::Comma, "Expected ',' in Map type")?;
            let _value_type = self.parse_type()?;
            self.consume_generic_close("Expected '>' after Map type arguments")?;
        }
        Ok(Type::Named("Map".to_string()))
    }

    /// Parse Set<T> or Set (without generics)
    fn parse_set_type(&mut self) -> Result<Type, ParseError> {
        if self.check(&Token::Lt) {
            self.consume(&Token::Lt, "Expected '<' after 'Set'")?;
            let _elem_type = self.parse_type()?;
            self.consume_generic_close("Expected '>' after Set type argument")?;
        }
        Ok(Type::Named("Set".to_string()))
    }

    /// Parse generic or named type: T<U, V> or T
    fn parse_generic_or_named(&mut self, name: String) -> Result<Type, ParseError> {
        if self.match_token(&Token::Lt) {
            let mut type_args = Vec::new();
            loop {
                type_args.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume_generic_close("Expected '>' after type arguments")?;
            Ok(Type::Generic { name, type_args })
        } else {
            Ok(Type::Named(name))
        }
    }
}
