use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_struct(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Struct, "Expected 'struct'")?;

        let name = self.consume_identifier_or_keyword()?;

        // Optional generic type parameters with bounds: struct Vec<T: Display>
        let type_params = self.parse_type_params()?;

        // Optional trait implementation declaration: struct File impl Reader, Writer
        let impl_traits = if self.match_token(&Token::Impl) {
            let mut traits = Vec::new();
            loop {
                traits.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            traits
        } else {
            Vec::new()
        };

        self.consume(&Token::LBrace, "Expected '{'")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        // Parse fields and methods
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check if this is a method (fn keyword) or field
            if self.check(&Token::Fn) {
                // Parse method
                methods.push(self.parse_struct_method()?);
            } else {
                // Parse field
                let field_name = self.consume_identifier()?;
                self.consume(&Token::Colon, "Expected ':' after field name")?;
                let field_type = self.parse_type()?;

                fields.push(Field {
                    name: field_name,
                    ty: field_type,
                    tag: None,
                });

                if !self.match_token(&Token::Comma) {
                    // Allow optional comma after last field
                    if !self.check(&Token::RBrace) && !self.check(&Token::Fn) {
                        return Err(self.error("Expected ',' or '}' after field"));
                    }
                }
            }
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Item::Struct(Struct {
            name,
            type_params,
            impl_traits,
            fields,
            methods,
        }))
    }

    /// Parse a method inside a struct body
    fn parse_struct_method(&mut self) -> Result<Function, ParseError> {
        self.consume(&Token::Fn, "Expected 'fn'")?;

        // Parse receiver: (self: &StructName) or (self: &StructName!)
        // Syntax: fn (self: &Type) method_name(...)
        let receiver = if self.check(&Token::LParen) {
            self.advance(); // consume '('
            let param_name = self.consume_identifier()?;
            if param_name != "self" {
                return Err(self.error("First parameter of method must be 'self'"));
            }
            self.consume(&Token::Colon, "Expected ':' after 'self'")?;

            let ty = self.parse_type()?;
            self.consume(&Token::RParen, "Expected ')' after receiver")?;

            // Determine mutability from the type
            let is_mutable = if let Type::Reference(_, is_mut) = &ty {
                *is_mut
            } else {
                false
            };

            Some(Receiver { is_mutable, ty })
        } else {
            None
        };

        // Method name comes AFTER receiver: fn (self: &T) method_name(...)
        let name = self.consume_identifier()?;

        // Parameters: (param1: T1, param2: T2)
        self.consume(&Token::LParen, "Expected '('")?;
        let params = self.parse_parameters()?;
        self.consume(&Token::RParen, "Expected ')'")?;

        // Optional return type
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Method body
        let was_in_method = self.in_method_body;
        self.in_method_body = receiver.is_some();
        let body = self.parse_block()?;
        self.in_method_body = was_in_method;

        Ok(Function {
            attributes: Vec::new(),
            is_async: false,
            is_gpu: false,
            receiver,
            name,
            type_params: Vec::new(),
            params,
            return_type,
            body,
        })
    }
}
