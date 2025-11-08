// Primary expression parsing
// This module handles literals, identifiers, arrays, tuples, and parenthesized expressions

use super::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_arguments(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();

        if self.check(&Token::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(args)
    }

    pub(crate) fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        // Closure/Lambda: |x, y| expr or |x: i32, y: i32| { body }
        if self.check(&Token::Pipe) {
            return self.parse_closure();
        }

        // Integer literal
        if let Token::IntLiteral(n) = self.peek() {
            let n = *n;
            self.advance();
            return Ok(Expression::IntLiteral(n));
        }

        // Float literal
        if let Token::FloatLiteral(f) = self.peek() {
            let f = *f;
            self.advance();
            return Ok(Expression::FloatLiteral(f));
        }

        // String literal
        if let Token::StringLiteral(s) = self.peek() {
            let s = s.clone();
            self.advance();
            return Ok(Expression::StringLiteral(s));
        }

        // F-string literal (formatted string)
        if let Token::FStringLiteral(s) = self.peek() {
            let s = s.clone();
            self.advance();
            return Ok(Expression::FStringLiteral(s));
        }

        // Boolean literals
        if self.match_token(&Token::True) {
            return Ok(Expression::BoolLiteral(true));
        }
        if self.match_token(&Token::False) {
            return Ok(Expression::BoolLiteral(false));
        }

        // Nil literal
        if self.match_token(&Token::Nil) {
            return Ok(Expression::Nil);
        }

        // Map() constructor - keyword handling
        if self.match_token(&Token::Map) {
            return self.parse_map_constructor();
        }

        // Set() constructor - keyword handling
        if self.match_token(&Token::Set) {
            return self.parse_set_constructor();
        }

        // Match expression
        if self.match_token(&Token::Match) {
            return self.parse_match_expression();
        }

        // Array literal
        if self.match_token(&Token::LBracket) {
            let mut elements = Vec::new();

            if !self.check(&Token::RBracket) {
                loop {
                    // Parse element expression (support full expressions except commas)
                    let elem = self.parse_comparison()?;

                    // Check for repeat syntax after first element: [value; count]
                    if elements.is_empty() && self.check(&Token::Semicolon) {
                        self.match_token(&Token::Semicolon); // consume the semicolon
                        let count_expr = self.parse_comparison()?;
                        self.consume(&Token::RBracket, "Expected ']' after array repeat syntax")?;
                        return Ok(Expression::ArrayRepeat(
                            Box::new(elem),
                            Box::new(count_expr),
                        ));
                    }

                    elements.push(elem);

                    // Check for more elements
                    if !self.match_token(&Token::Comma) {
                        break;
                    }

                    // Allow trailing comma
                    if self.check(&Token::RBracket) {
                        break;
                    }
                }
            }

            self.consume(&Token::RBracket, "Expected ']'")?;
            return Ok(Expression::Array(elements));
        }

        // Map literal: {"key": value, "key2": value2}
        if self.match_token(&Token::LBrace) {
            // Check if it's a map literal by looking ahead
            // Map literal: { "key": value, ... } or { key: value, ... }
            // Empty map: {}

            if self.check(&Token::RBrace) {
                self.advance(); // consume '}'
                return Ok(Expression::MapLiteral(Vec::new()));
            }

            // Try to parse as map literal
            // Peek ahead to see if it looks like key: value pattern
            let checkpoint = self.current;

            // Try parsing first key
            let first_key_result = self.parse_expression();

            if first_key_result.is_ok() && self.check(&Token::Colon) {
                // It's a map literal!
                self.advance(); // consume ':'
                let first_value = self.parse_expression()?;

                let mut entries = vec![(first_key_result.unwrap(), first_value)];

                while self.match_token(&Token::Comma) {
                    // Allow trailing comma
                    if self.check(&Token::RBrace) {
                        break;
                    }

                    let key = self.parse_expression()?;
                    self.consume(&Token::Colon, "Expected ':' after map key")?;
                    let value = self.parse_expression()?;

                    entries.push((key, value));
                }

                self.consume(&Token::RBrace, "Expected '}' after map literal")?;
                return Ok(Expression::MapLiteral(entries));
            } else {
                // Not a map literal, restore position and fail
                self.current = checkpoint;
                let span = self.peek_span().span.clone();
                let location = crate::Span::from_file_and_span(&self.file_name, self.source, span);
                return Err(ParseError::syntax_error(
                    "Unexpected '{', map literals need key: value pairs".to_string(),
                    location,
                ));
            }
        }

        // Parenthesized expression or tuple literal
        if self.match_token(&Token::LParen) {
            // Empty tuple: ()
            if self.check(&Token::RParen) {
                self.advance();
                return Ok(Expression::TupleLiteral(Vec::new()));
            }

            let first_expr = self.parse_expression()?;

            // If followed by comma, it's a tuple
            if self.match_token(&Token::Comma) {
                let mut elements = vec![first_expr];

                // Parse remaining elements
                if !self.check(&Token::RParen) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                        // Allow trailing comma
                        if self.check(&Token::RParen) {
                            break;
                        }
                    }
                }

                self.consume(&Token::RParen, "Expected ')' after tuple")?;
                return Ok(Expression::TupleLiteral(elements));
            }

            // Just a parenthesized expression
            self.consume(&Token::RParen, "Expected ')'")?;
            return Ok(first_expr);
        }

        // Identifier or builtin enum constructor
        if matches!(self.peek(), Token::Ident(_)) {
            let name = self.consume_identifier()?;

            // Check for builtin enum constructors: Some, None, Ok, Err
            // Phase 0.4: Special handling for Option/Result constructors
            match name.as_str() {
                "None" => {
                    // None: Option unit variant (no arguments)
                    return Ok(Expression::EnumLiteral {
                        enum_name: "Option".to_string(),
                        variant: "None".to_string(),
                        data: None,
                    });
                }
                "Some" => {
                    // Some(value): Option data variant
                    if self.check(&Token::LParen) {
                        self.advance(); // consume '('
                        let value = self.parse_expression()?;
                        self.consume(&Token::RParen, "Expected ')' after Some argument")?;
                        return Ok(Expression::EnumLiteral {
                            enum_name: "Option".to_string(),
                            variant: "Some".to_string(),
                            data: Some(Box::new(value)),
                        });
                    } else {
                        // No parens - might be used as identifier (error case)
                        return Ok(Expression::Ident(name));
                    }
                }
                "Ok" => {
                    // Ok(value): Result ok variant
                    if self.check(&Token::LParen) {
                        self.advance(); // consume '('
                        let value = self.parse_expression()?;
                        self.consume(&Token::RParen, "Expected ')' after Ok argument")?;
                        return Ok(Expression::EnumLiteral {
                            enum_name: "Result".to_string(),
                            variant: "Ok".to_string(),
                            data: Some(Box::new(value)),
                        });
                    } else {
                        return Ok(Expression::Ident(name));
                    }
                }
                "Err" => {
                    // Err(error): Result error variant
                    if self.check(&Token::LParen) {
                        self.advance(); // consume '('
                        let error = self.parse_expression()?;
                        self.consume(&Token::RParen, "Expected ')' after Err argument")?;
                        return Ok(Expression::EnumLiteral {
                            enum_name: "Result".to_string(),
                            variant: "Err".to_string(),
                            data: Some(Box::new(error)),
                        });
                    } else {
                        return Ok(Expression::Ident(name));
                    }
                }
                "Vec" => {
                    // Vec() or Vec<T>() or Vec(capacity: 100): Create new Vec
                    // Type-as-constructor pattern (like Rust, Swift, Kotlin)
                    if self.check(&Token::LParen) {
                        self.advance(); // consume '('

                        // Parse arguments: empty or named param "capacity: N"
                        let mut args = vec![];
                        if !self.check(&Token::RParen) {
                            // Check for named parameter: capacity
                            if let Token::Ident(param_name) = self.peek() {
                                if param_name == "capacity" {
                                    self.advance(); // consume "capacity"
                                    self.consume(
                                        &Token::Colon,
                                        "Expected ':' after capacity parameter",
                                    )?;
                                    let capacity_expr = self.parse_expression()?;
                                    args.push(capacity_expr);
                                } else {
                                    // Regular expression argument
                                    args.push(self.parse_expression()?);
                                }
                            } else {
                                // Regular expression argument
                                args.push(self.parse_expression()?);
                            }
                        }

                        self.consume(&Token::RParen, "Expected ')' after Vec constructor")?;

                        // Map to vec_new() or vec_with_capacity() builtin call
                        let func_name = if args.is_empty() {
                            "vec_new"
                        } else {
                            "vec_with_capacity"
                        };

                        return Ok(Expression::Call { span_id: None,
                            func: Box::new(Expression::Ident(func_name.to_string())),
                            args,
                        });
                    } else {
                        return Ok(Expression::Ident(name));
                    }
                }
                "Box" => {
                    // Box(value) or Box<T>(value): Create new Box
                    // Type-as-constructor pattern
                    if self.check(&Token::LParen) {
                        self.advance(); // consume '('
                        let value = self.parse_expression()?;
                        self.consume(&Token::RParen, "Expected ')' after Box argument")?;
                        // Map to box_new(value) builtin call
                        return Ok(Expression::Call { span_id: None,
                            func: Box::new(Expression::Ident("box_new".to_string())),
                            args: vec![value],
                        });
                    } else {
                        return Ok(Expression::Ident(name));
                    }
                }
                "Channel" => {
                    // Channel(capacity) or Channel<T>(capacity): Create new Channel
                    // Type-as-constructor pattern
                    if self.check(&Token::LParen) {
                        self.advance(); // consume '('
                        let capacity = self.parse_expression()?;
                        self.consume(&Token::RParen, "Expected ')' after Channel argument")?;
                        // Map to channel_new(capacity) builtin call
                        return Ok(Expression::Call { span_id: None,
                            func: Box::new(Expression::Ident("channel_new".to_string())),
                            args: vec![capacity],
                        });
                    } else {
                        return Ok(Expression::Ident(name));
                    }
                }
                "String" => {
                    // String() or String(str): Create new String
                    // Type-as-constructor pattern - converts string literals to heap strings
                    if self.check(&Token::LParen) {
                        self.advance(); // consume '('

                        // Empty String() or String(literal)
                        let args = if self.check(&Token::RParen) {
                            self.advance(); // consume ')'
                            vec![]
                        } else {
                            let arg = self.parse_expression()?;
                            self.consume(&Token::RParen, "Expected ')' after String argument")?;
                            vec![arg]
                        };

                        // Map to string_new() or string_from() builtin call
                        let func_name = if args.is_empty() {
                            "string_new"
                        } else {
                            "string_from"
                        };

                        return Ok(Expression::Call { span_id: None,
                            func: Box::new(Expression::Ident(func_name.to_string())),
                            args,
                        });
                    } else {
                        return Ok(Expression::Ident(name));
                    }
                }
                "Channel" => {
                    // Channel<T>(capacity): Create new Channel
                    if self.check(&Token::LParen) {
                        self.advance(); // consume '('
                        let capacity = self.parse_expression()?;
                        self.consume(&Token::RParen, "Expected ')' after Channel argument")?;
                        // Map to channel_new(capacity) builtin call
                        return Ok(Expression::Call { span_id: None,
                            func: Box::new(Expression::Ident("channel_new".to_string())),
                            args: vec![capacity],
                        });
                    } else {
                        return Ok(Expression::Ident(name));
                    }
                }
                // Legacy lowercase syntax (deprecated, will be removed)
                "vec" | "box" => {
                    return Err(self.error(&format!(
                        "Deprecated: Use '{}' instead of '{}' (type-as-constructor pattern)",
                        name.chars().next().unwrap().to_uppercase().to_string() + &name[1..],
                        name
                    )));
                }
                _ => {
                    return Ok(Expression::Ident(name));
                }
            }
        }

        // Allow keywords as identifiers in expressions (e.g., "error" struct)
        if matches!(self.peek(), Token::Error) {
            self.advance();
            return Ok(Expression::Ident("error".to_string()));
        }
        if matches!(self.peek(), Token::Type) {
            self.advance();
            return Ok(Expression::Ident("type".to_string()));
        }

        Err(self.error("Expected expression"))
    }

    fn parse_map_constructor(&mut self) -> Result<Expression, ParseError> {
        // Map() or Map(capacity): Create new HashMap
        if self.check(&Token::LParen) {
            self.advance(); // consume '('

            // Empty Map() or Map(capacity)
            let args = if self.check(&Token::RParen) {
                self.advance(); // consume ')'
                vec![]
            } else {
                let arg = self.parse_expression()?;
                self.consume(&Token::RParen, "Expected ')' after Map argument")?;
                vec![arg]
            };

            // Map to map_new() or map_with_capacity() builtin call
            let func_name = if args.is_empty() {
                "map_new"
            } else {
                "map_with_capacity"
            };

            Ok(Expression::Call { span_id: None,
                func: Box::new(Expression::Ident(func_name.to_string())),
                args,
            })
        } else {
            // Just "Map" identifier (for type annotations)
            Ok(Expression::Ident("Map".to_string()))
        }
    }

    fn parse_set_constructor(&mut self) -> Result<Expression, ParseError> {
        // Set() or Set(capacity): Create new HashSet (wraps Map)
        if self.check(&Token::LParen) {
            self.advance(); // consume '('

            // Empty Set() or Set(capacity)
            let args = if self.check(&Token::RParen) {
                self.advance(); // consume ')'
                vec![]
            } else {
                let arg = self.parse_expression()?;
                self.consume(&Token::RParen, "Expected ')' after Set argument")?;
                vec![arg]
            };

            // Map to set_new() or set_with_capacity() builtin call
            let func_name = if args.is_empty() {
                "set_new"
            } else {
                "set_with_capacity"
            };

            Ok(Expression::Call { span_id: None,
                func: Box::new(Expression::Ident(func_name.to_string())),
                args,
            })
        } else {
            // Just "Set" identifier (for type annotations)
            Ok(Expression::Ident("Set".to_string()))
        }
    }
}


