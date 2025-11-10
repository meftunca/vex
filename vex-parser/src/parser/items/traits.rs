use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_trait(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Trait, "Expected 'trait'")?;

        let name = self.consume_identifier()?;

        // Parse generic type parameters with bounds: trait Converter<T: Display>
        let type_params = self.parse_type_params()?;

        self.consume(&Token::LBrace, "Expected '{'")?;

        // Parse optional super traits: trait ReadWriter: Reader, Writer
        let super_traits = if self.match_token(&Token::Colon) {
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

        // Parse trait methods (required or default implementations)
        let mut trait_methods = Vec::new();
        let mut associated_types = Vec::new();
        let mut type_aliases = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.check(&Token::Fn) {
                self.advance(); // consume 'fn'
                trait_methods.push(self.parse_trait_method_signature()?);
            } else if self.check(&Token::Type) {
                // Parse associated type OR type alias
                self.advance(); // consume 'type'
                let type_name = self.consume_identifier()?;

                // Check if this is a type alias (type Iter = Iterator;) or associated type (type Item;)
                if self.match_token(&Token::Eq) {
                    // Type alias: type Iter = Iterator;
                    let aliased_type = self.parse_type()?;
                    self.consume(&Token::Semicolon, "Expected ';' after type alias")?;
                    type_aliases.push(TraitTypeAlias {
                        name: type_name,
                        ty: aliased_type,
                    });
                } else {
                    // Associated type: type Item;
                    self.consume(&Token::Semicolon, "Expected ';' after associated type")?;
                    associated_types.push(type_name);
                }
            } else {
                return Err(self.error("Expected method or associated type in trait"));
            }
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Item::Trait(Trait {
            name,
            type_params,
            super_traits,
            associated_types,
            type_aliases,
            methods: trait_methods,
        }))
    }

    pub(crate) fn parse_trait_impl(&mut self) -> Result<Item, ParseError> {
        // ❌ DEPRECATED SYNTAX: impl TraitName for TypeName { ... }
        // ✅ USE INSTEAD: struct TypeName impl TraitName { ... }
        self.consume(&Token::Impl, "Expected 'impl'")?;

        // Parse generic type parameters (if any)
        let _type_params = self.parse_type_params()?;

        // Parse trait name
        let trait_name = self.consume_identifier()?;

        // Check if this is the deprecated 'impl Trait for Type' syntax
        if self.check(&Token::For) {
            // ❌ REJECT: External trait implementations are not allowed in Vex v0.1+
            return Err(self.error(&format!(
                "External trait implementations are not allowed. Use 'struct <Type> impl {} {{ ... }}' instead of 'impl {} for <Type>'.\n\
                Vex requires trait methods to be defined inside the struct body for clarity.",
                trait_name, trait_name
            )));
        }

        // If we get here without 'for', this is an error (malformed impl)
        return Err(self.error(&format!(
            "Invalid impl syntax. Use 'struct <Type> impl {} {{ ... }}' to implement trait methods.",
            trait_name
        )));
    }

    pub(crate) fn parse_trait_method_signature(&mut self) -> Result<TraitMethod, ParseError> {
        // Parse trait method signature (no body, just signature)
        // Supports both:
        // 1. Golang-style: fn method_name(params): ReturnType;
        // 2. Simplified: fn method_name(params): ReturnType;

        // Check for optional golang-style receiver: fn (self: Type) method_name()
        let receiver = if self.check(&Token::LParen) {
            // Peek ahead to see if this is a receiver
            let checkpoint = self.current;
            self.advance(); // consume '('

            // Check if next token is identifier "self"
            let is_self = if let Token::Ident(name) = self.peek() {
                name == "self"
            } else {
                false
            };

            if is_self {
                // This is a golang-style receiver!
                let _self_name = self.consume_identifier()?;
                self.consume(&Token::Colon, "Expected ':' after 'self'")?;
                let receiver_type = self.parse_type()?;
                self.consume(&Token::RParen, "Expected ')' after receiver")?;

                // Check if type is a reference (&T)
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

        // Parse method name
        let name = self.consume_identifier()?;

        // Parse parameters
        self.consume(&Token::LParen, "Expected '('")?;
        let params = self.parse_parameters()?;
        self.consume(&Token::RParen, "Expected ')'")?;

        // ⭐ NEW: Check for mutability marker (!): fn method()!
        let is_mutable = self.match_token(&Token::Not);

        // Parse optional return type
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Trait methods MUST be signatures only (no body allowed)
        if self.check(&Token::LBrace) {
            return Err(
                self.error("Trait methods cannot have a body. Use only method signature with ';'")
            );
        }

        self.consume(
            &Token::Semicolon,
            "Expected ';' after trait method signature",
        )?;

        Ok(TraitMethod {
            name,
            is_mutable, // ⭐ NEW: Store mutability flag
            receiver,
            params,
            return_type,
            body: None, // Traits never have body
        })
    }
}
