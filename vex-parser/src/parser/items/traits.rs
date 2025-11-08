use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_interface_or_trait(&mut self) -> Result<Item, ParseError> {
        // Parse interface or trait keyword
        let is_trait = self.match_token(&Token::Trait);
        if !is_trait {
            self.consume(&Token::Interface, "Expected 'interface' or 'trait'")?;
        }

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

        if is_trait {
            // Parse trait methods (required or default implementations)
            let mut trait_methods = Vec::new();
            let mut associated_types = Vec::new();

            while !self.check(&Token::RBrace) && !self.is_at_end() {
                if self.check(&Token::Fn) {
                    self.advance(); // consume 'fn'
                    trait_methods.push(self.parse_trait_method_signature()?);
                } else if self.check(&Token::Type) {
                    // Parse associated type: type Item;
                    self.advance(); // consume 'type'
                    let type_name = self.consume_identifier()?;
                    self.consume(&Token::Semicolon, "Expected ';' after associated type")?;
                    associated_types.push(type_name);
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
                methods: trait_methods,
            }))
        } else {
            // Interface keyword is deprecated - treat as trait
            return Err(self.error(
                "The 'interface' keyword is deprecated. Use 'trait' instead (Vex v1.3 specification)",
            ));
        }
    }

    pub(crate) fn parse_trait_impl(&mut self) -> Result<Item, ParseError> {
        // impl TraitName for TypeName { ... }
        // impl<T> TraitName<T> for Vec<T> { ... }
        self.consume(&Token::Impl, "Expected 'impl'")?;

        // Parse generic type parameters with bounds: impl<T: Display>
        let type_params = self.parse_type_params()?;

        // Parse trait name
        let trait_name = self.consume_identifier()?;

        // Optionally parse trait generic arguments: impl Display<i32> for Point
        // For now, we'll skip this and just store the trait name

        // Consume 'for' keyword (Token::For)
        self.consume(&Token::For, "Expected 'for' after trait name")?;

        // Parse the type being implemented for
        let for_type = self.parse_type()?;

        self.consume(&Token::LBrace, "Expected '{'")?;

        // Parse associated type bindings and method implementations
        let mut associated_type_bindings = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.check(&Token::Fn) {
                self.advance(); // consume 'fn'
                methods.push(self.parse_function()?);
            } else if self.check(&Token::Type) {
                // Parse associated type binding: type Item = i32;
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
                return Err(self.error("Expected method or associated type binding in impl block"));
            }
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Item::TraitImpl(TraitImpl {
            trait_name,
            type_params,
            for_type,
            associated_type_bindings,
            methods,
        }))
    }

    pub(crate) fn parse_trait_method_signature(&mut self) -> Result<TraitMethod, ParseError> {
        // Parse trait method signature (no body, just signature)
        // Supports both:
        // 1. Golang-style: fn (self: &Self!) method_name(params): ReturnType;
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
