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

        // Optional policy application: struct User with APIModel, ValidationRules
        let policies = if self.match_token(&Token::With) {
            let mut policies_list = Vec::new();
            loop {
                policies_list.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            policies_list
        } else {
            Vec::new()
        };

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
        let mut associated_type_bindings = Vec::new(); // ⭐ NEW: Store associated types

        // Parse fields and methods
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check if this is a method (fn keyword), associated type (type keyword), or field
            if self.check(&Token::Fn) {
                // Parse method
                methods.push(self.parse_struct_method()?);
            } else if self.check(&Token::Type) {
                // ⭐ NEW: Parse associated type binding: type Item = i32;
                self.advance(); // consume 'type'
                let type_name = self.consume_identifier()?;
                self.consume(&Token::Eq, "Expected '=' after associated type name")?;
                let bound_type = self.parse_type()?;
                self.consume(
                    &Token::Semicolon,
                    "Expected ';' after associated type binding",
                )?;

                associated_type_bindings.push((type_name, bound_type));
            } else {
                // Parse field
                let field_name = self.consume_identifier()?;
                self.consume(&Token::Colon, "Expected ':' after field name")?;
                let field_type = self.parse_type()?;

                // Check for inline metadata (backtick)
                let metadata = if matches!(self.peek(), Token::Tag(_)) {
                    if let Token::Tag(tag_str) = self.advance() {
                        Some(tag_str.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };

                fields.push(Field {
                    name: field_name,
                    ty: field_type,
                    tag: None,
                    metadata, // Inline backtick metadata
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
            policies,
            impl_traits,
            associated_type_bindings, // ⭐ NEW: Include associated types
            fields,
            methods,
        }))
    }

    /// Parse a method inside a struct body
    ///
    /// Supports two syntaxes:
    /// 1. Golang-style: fn (self: &Type) method_name(...)
    /// 2. Simplified: fn method_name(...) - receiver auto-detected from body
    fn parse_struct_method(&mut self) -> Result<Function, ParseError> {
        self.consume(&Token::Fn, "Expected 'fn'")?;

        // Check for Golang-style receiver: fn (self: &Type) method_name(...)
        let receiver = if self.check(&Token::LParen) {
            // Peek to see if this is a receiver or method name
            let checkpoint = self.current;
            self.advance(); // consume '('

            // Check if this looks like a receiver: (self: Type) or (self!: Type)
            let next_is_self = if let Token::Ident(name) = self.peek() {
                name == "self"
            } else {
                false
            };

            if next_is_self {
                // Golang-style: fn (self: &Type) method_name(...)
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
                // Not a receiver, backtrack - this is method name
                self.current = checkpoint;
                None
            }
        } else {
            None
        };

        // Method name (comes after receiver in golang-style, or directly after 'fn' in simplified)
        let name = self.consume_identifier()?;

        // Parameters: (param1: T1, param2: T2)
        self.consume(&Token::LParen, "Expected '('")?;
        let params = self.parse_parameters()?;
        self.consume(&Token::RParen, "Expected ')'")?;

        // ⭐ NEW: Check for mutability marker (!): fn method()!
        let is_mutable = self.match_token(&Token::Not);

        // Optional return type
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // For simplified syntax (no explicit receiver), we still allow 'self' usage
        // in_method_body flag will be true for ANY method inside struct
        // Method body
        let was_in_method = self.in_method_body;
        self.in_method_body = true; // Always true for struct methods
        let body = self.parse_block()?;
        self.in_method_body = was_in_method;

        Ok(Function {
            is_async: false,
            is_gpu: false,
            is_mutable, // ⭐ NEW: Store mutability flag
            receiver,
            name,
            type_params: Vec::new(),
            where_clause: Vec::new(), // Struct inline methods don't support where clauses yet
            params,
            return_type,
            body,
            is_variadic: false,
            variadic_type: None,
        })
    }
}
