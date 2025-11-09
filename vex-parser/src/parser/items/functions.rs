use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_function(&mut self) -> Result<Function, ParseError> {
        // Note: 'fn' token is already consumed by the caller
        // For async functions, 'async fn' is consumed

        // Check for method receiver: fn (self: Type) method_name() or fn (p: &Type) method_name()
        let receiver = if self.check(&Token::LParen) {
            // Peek ahead to see if this is a receiver or just parameters
            let checkpoint = self.current;
            self.advance(); // consume '('

            // Check if next token is an identifier followed by ':'
            // This distinguishes receiver (p: Type) from parameters (p Type) or (p, q)
            let is_receiver = if let Token::Ident(_) = self.peek() {
                // Look ahead one more token to check for ':'
                let next_checkpoint = self.current;
                self.advance(); // consume identifier
                let has_colon = self.check(&Token::Colon);
                self.current = next_checkpoint; // backtrack
                has_colon
            } else {
                false
            };

            if is_receiver {
                // This is a receiver!
                let _receiver_name = self.consume_identifier()?; // 'self', 'p', 'r', etc.
                self.consume(&Token::Colon, "Expected ':' after receiver name")?;
                let receiver_type = self.parse_type()?;
                self.consume(&Token::RParen, "Expected ')' after receiver")?;

                // Check if type is a reference (&T or *T)
                let is_mutable = matches!(receiver_type, Type::Reference(_, true));

                Some(Receiver {
                    is_mutable,
                    ty: receiver_type,
                })
            } else {
                // Not a receiver, backtrack
                self.current = checkpoint;
                None
            }
        } else {
            None
        };

        let name = self.consume_identifier()?;

        // Optional generic type parameters with bounds: fn foo<T: Display, U: Clone>()
        let type_params = self.parse_type_params()?;

        self.consume(&Token::LParen, "Expected '('")?;
        let params = self.parse_parameters()?;

        // Check for variadic parameter AFTER regular params: , args: ...any
        let (is_variadic, variadic_type) = if self.check(&Token::Comma) {
            let checkpoint = self.current;
            self.advance(); // consume comma

            // Check if this is a variadic parameter (name: ...)
            if let Token::Ident(_) = self.peek() {
                let _var_name = self.consume_identifier()?;
                if self.match_token(&Token::Colon) {
                    // Check for ... (DotDotDot token)
                    if self.match_token(&Token::DotDotDot) {
                        // This is variadic!
                        let var_type = self.parse_type()?;
                        (true, Some(var_type))
                    } else {
                        // Regular param after comma, backtrack
                        self.current = checkpoint;
                        (false, None)
                    }
                } else {
                    self.current = checkpoint;
                    (false, None)
                }
            } else {
                self.current = checkpoint;
                (false, None)
            }
        } else {
            (false, None)
        };

        self.consume(&Token::RParen, "Expected ')'")?;

        // ⭐ NEW: Check for mutability marker (!): fn method()!
        let is_mutable = self.match_token(&Token::Not);

        // Optional return type with ':' (Vex syntax: fn foo(): i32)
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse optional where clause: where T: Display, U: Clone
        let where_clause = if self.match_token(&Token::Where) {
            self.parse_where_clause()?
        } else {
            Vec::new()
        };

        // Set method body flag for method syntax sugar
        let was_in_method = self.in_method_body;
        self.in_method_body = receiver.is_some();

        // parse_block() already consumes { and }
        let body = self.parse_block()?;

        // Restore previous state
        self.in_method_body = was_in_method;

        Ok(Function {
            is_async: false,
            is_gpu: false,
            is_mutable, // ⭐ NEW: Store mutability flag
            receiver,
            name,
            type_params,
            where_clause,
            params,
            return_type,
            body,
            is_variadic,
            variadic_type,
        })
    }

    /// Parse where clause: where T: Display, U: Clone + Debug
    pub(crate) fn parse_where_clause(&mut self) -> Result<Vec<WhereClausePredicate>, ParseError> {
        let mut predicates = Vec::new();

        loop {
            // Parse type parameter name
            let type_param = self.consume_identifier()?;

            // Expect ':' after type parameter
            self.consume(
                &Token::Colon,
                "Expected ':' after type parameter in where clause",
            )?;

            // Parse trait bounds (inline - same logic as parse_type_params)
            let mut bounds = Vec::new();
            loop {
                let bound_name = self.consume_identifier()?;

                // Check for closure trait with signature: Callable(i32): bool
                if self.match_token(&Token::LParen) {
                    // Parse parameter types
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
                    self.consume(&Token::Colon, "Expected ':' after closure parameters")?;
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

            predicates.push(WhereClausePredicate { type_param, bounds });

            // Check for more predicates
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(predicates)
    }

    pub(crate) fn parse_parameters(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();

        if self.check(&Token::RParen) {
            return Ok(params);
        }

        loop {
            let name = self.consume_identifier()?;
            self.consume(&Token::Colon, "Expected ':' after parameter name")?;
            let ty = self.parse_type()?;

            params.push(Param { name, ty });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(params)
    }
}
