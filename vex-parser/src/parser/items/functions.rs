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
        self.consume(&Token::RParen, "Expected ')'")?;

        // ⭐ NEW: Check for mutability marker (!): fn method()!
        let is_mutable = self.match_token(&Token::Not);

        // Optional return type with ':' (Vex syntax: fn foo(): i32)
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Set method body flag for method syntax sugar
        let was_in_method = self.in_method_body;
        self.in_method_body = receiver.is_some();

        // parse_block() already consumes { and }
        let body = self.parse_block()?;

        // Restore previous state
        self.in_method_body = was_in_method;

        Ok(Function {
            attributes: Vec::new(), // TODO: Parse attributes before function
            is_async: false,
            is_gpu: false,
            is_mutable, // ⭐ NEW: Store mutability flag
            receiver,
            name,
            type_params,
            params,
            return_type,
            body,
        })
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
