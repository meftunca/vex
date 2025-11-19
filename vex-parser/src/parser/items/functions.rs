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
                let receiver_name = self.consume_identifier()?; // 'self', 'p', 'r', etc.
                self.consume(&Token::Colon, "Expected ':' after receiver name")?;
                let receiver_type = self.parse_type()?;
                self.consume(&Token::RParen, "Expected ')' after receiver")?;

                // Check if type is a reference (&T or *T)
                let is_mutable = matches!(receiver_type, Type::Reference(_, true));

                Some(Receiver {
                    name: receiver_name,
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

        // ⭐ NEW: Check for operator method: op+, op-, op*, etc.
        let (is_operator, name) = if let Token::OperatorMethod(op_name) = self.peek() {
            let op_name_owned = op_name.clone();
            self.advance(); // consume operator token
            (true, op_name_owned)
        } else if let Token::New = self.peek() {
            // Allow 'new' as method name
            self.advance();
            (false, "new".to_string())
        } else {
            (false, self.consume_identifier()?)
        };

        // Optional generic type parameters with bounds: fn foo<T: Display, U: Clone>()
        // Note: For static method syntax (fn Vec<T>.new()), generics are already consumed by caller
        let (type_params, const_params) = self.parse_type_params()?;

        self.consume(&Token::LParen, "Expected '('")?;

        // Parse parameters and check for variadic
        let (params, is_variadic, variadic_type) = self.parse_parameters_with_variadic()?;

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
            is_exported: false, // Default to false, set to true by parse_export
            is_async: false,
            is_gpu: false,
            is_mutable,        // ⭐ NEW: Store mutability flag
            is_operator,       // ⭐ NEW: Store operator flag
            is_static: false,  // ⭐ NEW: Set by caller for static methods
            static_type: None, // ⭐ NEW: Set by caller for static methods
            receiver,
            name,
            type_params,
            const_params,
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

            // Check for associated type constraint: T.Item: Display
            if self.match_token(&Token::Dot) {
                let assoc_type = self.consume_identifier()?;

                // Expect ':' after associated type
                self.consume(
                    &Token::Colon,
                    "Expected ':' after associated type in where clause",
                )?;

                // Parse trait bounds for associated type
                let mut bounds = Vec::new();
                loop {
                    let bound_name = self.consume_identifier()?;
                    bounds.push(TraitBound::Simple(bound_name));

                    if !self.match_token(&Token::Plus) {
                        break;
                    }
                }

                predicates.push(WhereClausePredicate::AssociatedTypeBound {
                    type_param,
                    assoc_type,
                    bounds,
                });
            } else {
                // Regular type parameter bound: T: Display
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

                predicates.push(WhereClausePredicate::TypeBound { type_param, bounds });
            }

            // Check for more predicates
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(predicates)
    }

    pub(crate) fn parse_parameters(&mut self) -> Result<Vec<Param>, ParseError> {
        let (params, _, _) = self.parse_parameters_with_variadic()?;
        Ok(params)
    }

    /// Parse parameters with optional variadic support
    /// Returns: (params, is_variadic, variadic_type)
    pub(crate) fn parse_parameters_with_variadic(
        &mut self,
    ) -> Result<(Vec<Param>, bool, Option<Type>), ParseError> {
        let mut params = Vec::new();

        if self.check(&Token::RParen) {
            return Ok((params, false, None));
        }

        loop {
            // Collect parameter names (supports grouping: a, b, c: i32)
            let mut param_names = vec![self.consume_identifier()?];

            // Check for grouped parameters: name1, name2, name3: type
            while self.match_token(&Token::Comma) && !self.check(&Token::RParen) {
                // Peek ahead to see if this is a new parameter or grouped name
                let next_name = self.consume_identifier()?;

                // If followed by colon, it's the type. Otherwise it's another grouped name
                if self.check(&Token::Colon) {
                    // This is the last name in the group, followed by type
                    param_names.push(next_name);
                    break;
                } else {
                    // This could be another grouped name, continue collecting
                    param_names.push(next_name);
                }
            }

            if !self.match_token(&Token::Colon) {
                return Err(ParseError::Other(format!(
                    "Expected ':' after parameter name(s) '{}'",
                    param_names.join(", ")
                )));
            }

            // Check for variadic: ...Type
            if self.match_token(&Token::DotDotDot) {
                let var_type = self.parse_type()?;

                // Grouped parameters cannot be variadic
                if param_names.len() > 1 {
                    return Err(ParseError::Other(
                        "Cannot use parameter grouping with variadic parameters".to_string(),
                    ));
                }

                // Variadic must be the last parameter
                if self.match_token(&Token::Comma) {
                    return Err(ParseError::Other(
                        "Variadic parameter must be last".to_string(),
                    ));
                }
                return Ok((params, true, Some(var_type)));
            }

            // Regular parameter - parse type
            let ty = self.parse_type()?;

            // Check for default value: = expr
            let default_value = if self.match_token(&Token::Eq) {
                // Grouped parameters cannot have default values
                if param_names.len() > 1 {
                    return Err(ParseError::Other(
                        "Cannot use default values with grouped parameters. Each parameter must be declared separately.".to_string(),
                    ));
                }
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            // Expand grouped parameters into individual Param entries
            for param_name in param_names {
                params.push(Param {
                    name: param_name,
                    ty: ty.clone(),
                    default_value: default_value.clone(),
                });
            }

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok((params, false, None))
    }
}
