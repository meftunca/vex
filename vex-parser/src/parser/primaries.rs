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
        // Async block: async { stmts; expr }
        if self.check(&Token::Async) {
            self.advance(); // consume 'async'
            self.consume(&Token::LBrace, "Expected '{' after 'async'")?;
            return self.parse_async_block();
        }

        // Typeof expression: typeof(expr)
        if self.check(&Token::Typeof) {
            self.advance();
            self.consume(&Token::LParen, "Expected '(' after 'typeof'")?;
            let expr = self.parse_expression()?;
            self.consume(&Token::RParen, "Expected ')' after typeof expression")?;
            return Ok(Expression::Typeof(Box::new(expr)));
        }

        // Closure/Lambda: |x, y| expr or |x: i32, y: i32| { body } or || expr
        if self.check(&Token::Pipe) || self.check(&Token::Or) {
            return self.parse_closure();
        }

        // Integer literals (decimal, hex, binary, octal)
        // Now stored as String in lexer to support i128/u128 range
        if let Token::IntLiteral(s) = self.peek() {
            let s = s.clone();
            self.advance();
            // Try to parse as i64 first
            if let Ok(n) = s.parse::<i64>() {
                return Ok(Expression::IntLiteral(n));
            }
            // Handle i64::MIN special case: 9223372036854775808 (will be negated)
            if let Ok(u) = s.parse::<u64>() {
                if u == 9223372036854775808 {
                    return Ok(Expression::IntLiteral(i64::MIN));
                }
            }
            // If it doesn't fit in i64, store as BigIntLiteral for i128/u128
            return Ok(Expression::BigIntLiteral(s));
        }
        if let Token::HexLiteral(s) = self.peek() {
            let s = s.clone();
            self.advance();
            if let Ok(n) = i64::from_str_radix(&s[2..], 16) {
                return Ok(Expression::IntLiteral(n));
            }
            // Store as BigIntLiteral if too large
            return Ok(Expression::BigIntLiteral(s));
        }
        if let Token::BinaryLiteral(s) = self.peek() {
            let s = s.clone();
            self.advance();
            if let Ok(n) = i64::from_str_radix(&s[2..], 2) {
                return Ok(Expression::IntLiteral(n));
            }
            return Ok(Expression::BigIntLiteral(s));
        }
        if let Token::OctalLiteral(s) = self.peek() {
            let s = s.clone();
            self.advance();
            if let Ok(n) = i64::from_str_radix(&s[2..], 8) {
                return Ok(Expression::IntLiteral(n));
            }
            return Ok(Expression::BigIntLiteral(s));
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
            let first_key = self.parse_expression()?;

            if self.check(&Token::Colon) {
                // It's a map literal!
                self.advance(); // consume ':'
                let first_value = self.parse_expression()?;

                let mut entries = vec![(first_key, first_value)];

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
                        data: vec![],
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
                            data: vec![value],
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
                            data: vec![value],
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
                            data: vec![error],
                        });
                    } else {
                        return Ok(Expression::Ident(name));
                    }
                }
                // Generic type constructor handling - moved to end of match
                // Removed hardcoded type constructors - now handled generically below
                // Legacy lowercase syntax (deprecated, will be removed)
                "vec" | "box" => {
                    let capitalized = if let Some(first_char) = name.chars().next() {
                        first_char.to_uppercase().to_string() + &name[1..]
                    } else {
                        name.clone()
                    };
                    return Err(self.error(&format!(
                        "Deprecated: Use '{}' instead of '{}' (type-as-constructor pattern)",
                        capitalized, name
                    )));
                }
                _ => {
                    // Generic type constructor: Type() or Type(args)
                    // IMPORTANT: Check if this is actually a type constructor before parsing
                    // Only treat Ident() as type constructor if the identifier starts with uppercase
                    // (following PascalCase convention for types)
                    let first_char = name.chars().next().unwrap_or('_');

                    if self.check(&Token::LParen) && first_char.is_uppercase() {
                        // Type(...) constructor call - PascalCase identifier

                        // Check for generic type arguments: Vec<T>()
                        let type_args = if self.check(&Token::Lt) {
                            self.advance(); // consume <
                            let mut args = Vec::new();
                            loop {
                                args.push(self.parse_type()?);
                                if !self.match_token(&Token::Comma) {
                                    break;
                                }
                            }
                            self.consume_generic_close("Expected '>' after type arguments")?;
                            args
                        } else {
                            vec![]
                        };

                        self.advance(); // consume '('
                        let args = self.parse_arguments()?;
                        self.consume(
                            &Token::RParen,
                            "Expected ')' after type constructor arguments",
                        )?;

                        return Ok(Expression::TypeConstructor {
                            type_name: name,
                            type_args,
                            args,
                        });
                    } else {
                        // Just an identifier (including lowercase function calls)
                        return Ok(Expression::Ident(name));
                    }
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

            Ok(Expression::Call {
                type_args: Vec::new(),
                span_id: None,
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

            Ok(Expression::Call {
                type_args: Vec::new(),
                span_id: None,
                func: Box::new(Expression::Ident(func_name.to_string())),
                args,
            })
        } else {
            // Just "Set" identifier (for type annotations)
            Ok(Expression::Ident("Set".to_string()))
        }
    }

    /// Parse async block: async { stmts; expr }
    /// Returns AsyncBlock expression
    pub(crate) fn parse_async_block(&mut self) -> Result<Expression, ParseError> {
        // LBrace already consumed by caller
        let mut statements = Vec::new();
        let mut return_expr = None;

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Try to parse as expression first (peek for semicolon)
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
                let checkpoint = self.current;
                if let Ok(expr) = self.parse_expression() {
                    // If no semicolon follows and we're at closing brace, this is the return expr
                    if !self.match_token(&Token::Semicolon) && self.check(&Token::RBrace) {
                        return_expr = Some(Box::new(expr));
                        break;
                    }
                    // Otherwise it's a statement
                    statements.push(Statement::Expression(expr));
                    continue;
                } else {
                    // Reset and parse as statement
                    self.current = checkpoint;
                }
            }

            statements.push(self.parse_statement()?);
        }

        self.consume(&Token::RBrace, "Expected '}' at end of async block")?;

        Ok(Expression::AsyncBlock {
            statements,
            return_expr,
        })
    }
}
