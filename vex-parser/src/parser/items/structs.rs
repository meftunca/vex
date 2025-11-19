use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_struct(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Struct, "Expected 'struct'")?;

        let name = self.consume_identifier_or_keyword()?;

        // Optional generic type parameters with bounds: struct Vec<T: Display>
        let (type_params, const_params) = self.parse_type_params()?;

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
        // Or with type args: struct Vector impl Add<i32>, Add<f64>
        let impl_traits = if self.match_token(&Token::Impl) {
            let mut traits = Vec::new();
            loop {
                let trait_name = self.consume_identifier()?;

                // Check for generic type arguments: Add<i32>
                let type_args = if self.match_token(&Token::Lt) {
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::Gt, "Expected '>' after type arguments")?;
                    args
                } else {
                    Vec::new()
                };

                traits.push(TraitImpl {
                    name: trait_name,
                    type_args,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            traits
        } else {
            Vec::new()
        };

        // Optional where clause for conditional trait impl
        let where_clause = if self.match_token(&Token::Where) {
            self.parse_where_clause()?
        } else {
            Vec::new()
        };

        self.consume(&Token::LBrace, "Expected '{'")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut associated_type_bindings = Vec::new(); // ⭐ NEW: Store associated types

        // Parse fields and methods
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check if this is a method (fn keyword - DEPRECATED), associated type (type keyword), or field
            if self.check(&Token::Fn) {
                // ⚠️ DEPRECATED: Inline struct methods are deprecated!
                // Emit warning but still parse for now
                eprintln!(
                    "⚠️  WARNING: Inline struct methods are deprecated in struct '{}'",
                    name
                );
                eprintln!("   → Use Go-style external methods instead:");
                eprintln!("   → fn (self: &{}) method_name() {{ }}", name);
                eprintln!("   → See VEX_IDENTITY.md for migration guide");

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
            } else if matches!(self.peek(), Token::LParen) {
                // ⭐ Emit deprecation warning for inline methods with receiver
                let span = self.token_to_diag_span(&self.peek_span().span);
                let warning = vex_diagnostics::Diagnostic::warning(
                    "W0001",
                    format!(
                        "Inline struct methods are deprecated",
                    ),
                    span,
                )
                .with_help(format!(
                    "Define methods outside the struct using Go-style syntax: fn (self: &{}) method_name(...) {{ }}",
                    name
                ))
                .with_note("See docs/REFERENCE.md (Method Definitions & Calls) for migration guide".to_string());

                self.diagnostics.push(warning);
                methods.push(self.parse_struct_method()?);
            } else if matches!(self.peek(), Token::OperatorMethod(_)) {
                /* Lines 124-141 omitted */
                methods.push(self.parse_struct_method()?);
            } else if matches!(self.peek(), Token::Ident(s) if s == "op") {
                // ⭐ NEW: Bare "op" identifier for constructor operator method
                let span = self.token_to_diag_span(&self.peek_span().span);
                let warning = vex_diagnostics::Diagnostic::warning(
                    "W0001",
                    format!("Inline struct methods are deprecated",),
                    span,
                )
                .with_help(format!(
                    "Define constructor outside the struct: fn (self: &{}) op(...) {{ }}",
                    name
                ))
                .with_note(
                    "See docs/REFERENCE.md (Method Definitions & Calls) for migration guide"
                        .to_string(),
                );

                self.diagnostics.push(warning);
                methods.push(self.parse_struct_method()?);
            } else if matches!(self.peek(), Token::Ident(_)) {
                // Could be a method without 'fn' or a field - need to disambiguate
                // Look ahead: if next token is '(' it's a method, if ':' it's a field
                let checkpoint = self.current;
                let field_or_method_name = self.consume_identifier()?;

                if self.check(&Token::LParen) {
                    // ⚠️ Emit deprecation warning for inline methods
                    self.current = checkpoint; // Backtrack first to get correct span
                    let span = self.token_to_diag_span(&self.peek_span().span);
                    let warning = vex_diagnostics::Diagnostic::warning(
                        "W0001",
                        format!("Inline struct methods are deprecated",),
                        span,
                    )
                    .with_help(format!(
                        "Define '{}' outside the struct: fn (self: &{}) {}(...) {{ }}",
                        field_or_method_name, name, field_or_method_name
                    ))
                    .with_note(
                        "See docs/REFERENCE.md (Method Definitions & Calls) for migration guide"
                            .to_string(),
                    );

                    self.diagnostics.push(warning);
                    methods.push(self.parse_struct_method()?);
                } else if self.check(&Token::Colon) {
                    // It's a field - continue with field parsing
                    self.advance(); // consume ':'
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
                        name: field_or_method_name,
                        ty: field_type,
                        tag: None,
                        metadata, // Inline backtick metadata
                    });

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                } else {
                    return Err(self.error("Expected '(' for method or ':' for field"));
                }
            } else {
                return Err(self.error("Expected field, method, or '}'"));
            }
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Item::Struct(Struct {
            is_exported: false, // Default to false
            name,
            type_params,
            const_params,
            where_clause,
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
        // ⭐ NEW: 'fn' keyword is now OPTIONAL in struct methods
        let _has_fn_keyword = self.match_token(&Token::Fn);

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

                // Determine mutability from the type.
                // For external (Golang-style) methods, mutability is determined by the receiver's type.
                // e.g., `fn (self: &MyType!)` is a mutable reference.
                // The parser correctly sets the `is_mutable` flag on the `Type::Reference`.
                // If the type is not a reference, it's considered immutable in this context.
                let is_mutable = if let Type::Reference(_, is_mut) = &ty {
                    *is_mut
                } else {
                    // According to the contract, external methods use `&MyType!`.
                    // A non-reference receiver `(self: MyType)` is not mutable.
                    false
                };

                Some(Receiver {
                    name: "self".to_string(), // Inline methods always use 'self'
                    is_mutable,
                    ty,
                })
            } else {
                // Not a receiver, backtrack - this is method name
                self.current = checkpoint;
                None
            }
        } else {
            None
        };

        // ⭐ NEW: Check for operator method AFTER receiver: op+, op-, op*, etc.
        let (is_operator, name) = if let Token::OperatorMethod(op_name) = self.peek() {
            let op_name_owned = op_name.clone();
            self.advance(); // consume operator token
            (true, op_name_owned)
        } else if let Token::Ident(ident_name) = self.peek() {
            // ⭐ NEW: Check if Ident is "op" (constructor)
            if ident_name == "op" {
                self.advance(); // consume 'op'
                (true, "op".to_string())
            } else {
                (false, self.consume_identifier()?)
            }
        } else {
            (false, self.consume_identifier()?)
        };

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
            is_exported: false, // Struct methods are not exported individually
            is_async: false,
            is_gpu: false,
            is_mutable,  // ⭐ NEW: Store mutability flag
            is_operator, // ⭐ NEW: Store operator flag
            is_static: false, // ⭐ Struct methods are never static (use external static methods)
            static_type: None,
            receiver,
            name,
            type_params: Vec::new(),
            const_params: vec![],     // ⭐ TODO: Parse const params
            where_clause: Vec::new(), // Struct inline methods don't support where clauses yet
            params,
            return_type,
            body,
            is_variadic: false,
            variadic_type: None,
        })
    }
}
