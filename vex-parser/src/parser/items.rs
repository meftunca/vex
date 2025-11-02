// Item parsing (struct, enum, function, interface, const, import, export)

use super::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_import(&mut self) -> Result<Import, ParseError> {
        self.consume(&Token::Import, "Expected 'import'")?;

        // Four patterns:
        // 1. import { io, net } from "std";         - Named imports
        // 2. import * as std from "std";            - Namespace import
        // 3. import "std/io";                       - Module import (entire module)
        // 4. import io from "std/io";               - Default import (future)

        if self.match_token(&Token::Star) {
            // Pattern 2: import * as name from "module";
            self.consume(&Token::As, "Expected 'as' after '*'")?;
            let alias = self.consume_identifier()?;
            self.consume(&Token::From, "Expected 'from' after alias")?;

            let module = if let Token::StringLiteral(s) = self.peek() {
                let m = s.clone();
                self.advance();
                m
            } else {
                return Err(self.error("Expected module string after 'from'"));
            };

            self.consume(&Token::Semicolon, "Expected ';' after import")?;

            return Ok(Import {
                kind: ImportKind::Namespace(alias.clone()),
                items: Vec::new(),
                module,
                alias: Some(alias),
            });
        }

        if self.match_token(&Token::LBrace) {
            // Pattern 1: import { items } from "module"
            let mut import_items = Vec::new();

            loop {
                let item = self.consume_identifier()?;
                import_items.push(item);

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }

            self.consume(&Token::RBrace, "Expected '}'")?;
            self.consume(&Token::From, "Expected 'from' after import items")?;

            let module = if let Token::StringLiteral(s) = self.peek() {
                let m = s.clone();
                self.advance();
                m
            } else {
                return Err(self.error("Expected module string after 'from'"));
            };

            self.consume(&Token::Semicolon, "Expected ';' after import")?;

            return Ok(Import {
                kind: ImportKind::Named,
                items: import_items,
                module,
                alias: None,
            });
        }

        if let Token::StringLiteral(s) = self.peek() {
            // Pattern 3: import "module"
            let module = s.clone();
            self.advance();
            self.consume(&Token::Semicolon, "Expected ';' after import")?;

            return Ok(Import {
                kind: ImportKind::Module,
                items: Vec::new(),
                module,
                alias: None,
            });
        }

        Err(self.error("Expected '{', '*', or string after 'import'"))
    }

    pub(crate) fn parse_export(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Export, "Expected 'export'")?;

        // Two patterns:
        // 1. export { io, net };
        // 2. export fn foo() {} or export const X: i32 = 5;

        if self.match_token(&Token::LBrace) {
            // Pattern 1: export { items };
            let mut export_items = Vec::new();

            loop {
                // Accept both identifiers and keywords (like "unsafe")
                let item = self.consume_identifier_or_keyword()?;
                export_items.push(item);

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }

            self.consume(&Token::RBrace, "Expected '}'")?;
            self.consume(&Token::Semicolon, "Expected ';' after export")?;

            Ok(Item::Export(Export {
                items: export_items,
            }))
        } else if self.check(&Token::Fn) {
            // Pattern 2: export fn foo() {}
            self.advance(); // consume 'fn'
            Ok(Item::Function(self.parse_function()?))
        } else if self.check(&Token::Const) {
            // Pattern 2: export const X = 5;
            self.parse_const()
        } else if self.check(&Token::Struct) {
            // Pattern 2: export struct Foo {}
            self.parse_struct()
        } else {
            return Err(self.error("Expected '{', 'fn', 'const', or 'struct' after 'export'"));
        }
    }

    pub(crate) fn parse_const(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Const, "Expected 'const'")?;

        let name = self.consume_identifier()?;

        self.consume(&Token::Colon, "Expected ':' after const name")?;
        let ty = Some(self.parse_type()?);

        self.consume(&Token::Eq, "Expected '=' after const type")?;
        let value = self.parse_expression()?;

        self.consume(&Token::Semicolon, "Expected ';' after const value")?;

        Ok(Item::Const(Const { name, ty, value }))
    }

    pub(crate) fn parse_struct(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Struct, "Expected 'struct'")?;

        let name = self.consume_identifier_or_keyword()?;

        // Optional generic type parameters: struct Vec<T>
        let type_params = if self.match_token(&Token::Lt) {
            let mut params = Vec::new();
            loop {
                params.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::Gt, "Expected '>' after type parameters")?;
            params
        } else {
            Vec::new()
        };

        self.consume(&Token::LBrace, "Expected '{'")?;

        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let field_name = self.consume_identifier()?;
            self.consume(&Token::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;

            fields.push(Field {
                name: field_name,
                ty: field_type,
                tag: None,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Item::Struct(Struct {
            name,
            type_params,
            fields,
        }))
    }

    pub(crate) fn parse_enum(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Enum, "Expected 'enum'")?;

        let name = self.consume_identifier()?;

        // Optional type parameters: enum Option<T>
        let type_params = if self.match_token(&Token::Lt) {
            let mut params = Vec::new();
            loop {
                params.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::Gt, "Expected '>' after type parameters")?;
            params
        } else {
            Vec::new()
        };

        self.consume(&Token::LBrace, "Expected '{'")?;

        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let variant_name = self.consume_identifier()?;

            // Check for tuple data: Some(T) or unit: None
            let data = if self.match_token(&Token::LParen) {
                let ty = self.parse_type()?;
                self.consume(&Token::RParen, "Expected ')' after variant type")?;
                Some(ty)
            } else {
                None
            };

            variants.push(EnumVariant {
                name: variant_name,
                data,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Item::Enum(Enum {
            name,
            type_params,
            variants,
        }))
    }

    pub(crate) fn parse_type_alias(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Type, "Expected 'type'")?;

        let name = self.consume_identifier()?;

        // Optional type parameters: type Result<T, E>
        let type_params = if self.match_token(&Token::Lt) {
            let mut params = Vec::new();
            loop {
                params.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::Gt, "Expected '>' after type parameters")?;
            params
        } else {
            Vec::new()
        };

        self.consume(&Token::Eq, "Expected '=' after type alias name")?;

        let ty = self.parse_type()?;

        self.consume(&Token::Semicolon, "Expected ';' after type alias")?;

        Ok(Item::TypeAlias(TypeAlias {
            name,
            type_params,
            ty,
        }))
    }

    pub(crate) fn parse_interface_or_trait(&mut self) -> Result<Item, ParseError> {
        // Parse interface or trait keyword
        let is_trait = self.match_token(&Token::Trait);
        if !is_trait {
            self.consume(&Token::Interface, "Expected 'interface' or 'trait'")?;
        }

        let name = self.consume_identifier()?;

        // Parse generic type parameters: interface Cache<K, V> or trait Converter<T>
        let mut type_params = Vec::new();
        if self.match_token(&Token::Lt) {
            loop {
                type_params.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::Gt, "Expected '>' after type parameters")?;
        }

        self.consume(&Token::LBrace, "Expected '{'")?;

        if is_trait {
            // Parse trait methods (signatures only, no body)
            let mut trait_methods = Vec::new();
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                if self.check(&Token::Fn) {
                    self.advance(); // consume 'fn'
                    trait_methods.push(self.parse_trait_method_signature()?);
                } else {
                    return Err(self.error("Expected method in trait"));
                }
            }

            self.consume(&Token::RBrace, "Expected '}'")?;

            Ok(Item::Trait(Trait {
                name,
                type_params,
                methods: trait_methods,
            }))
        } else {
            // Parse interface methods
            let mut methods = Vec::new();
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                if self.check(&Token::Fn) {
                    self.advance(); // consume 'fn'
                    let func = self.parse_function()?;

                    // Convert Function to InterfaceMethod
                    methods.push(InterfaceMethod {
                        name: func.name,
                        params: func.params,
                        return_type: func.return_type,
                    });
                } else {
                    return Err(self.error("Expected method in interface"));
                }
            }

            self.consume(&Token::RBrace, "Expected '}'")?;

            Ok(Item::Interface(Interface {
                name,
                type_params,
                methods,
            }))
        }
    }

    pub(crate) fn parse_trait_impl(&mut self) -> Result<Item, ParseError> {
        // impl TraitName for TypeName { ... }
        // impl<T> TraitName<T> for Vec<T> { ... }
        self.consume(&Token::Impl, "Expected 'impl'")?;

        // Parse generic type parameters: impl<T>
        let mut type_params = Vec::new();
        if self.match_token(&Token::Lt) {
            loop {
                type_params.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::Gt, "Expected '>' after type parameters")?;
        }

        // Parse trait name
        let trait_name = self.consume_identifier()?;

        // Optionally parse trait generic arguments: impl Display<i32> for Point
        // For now, we'll skip this and just store the trait name

        // Consume 'for' keyword (Token::For)
        self.consume(&Token::For, "Expected 'for' after trait name")?;

        // Parse the type being implemented for
        let for_type = self.parse_type()?;

        self.consume(&Token::LBrace, "Expected '{'")?;

        // Parse method implementations
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.check(&Token::Fn) {
                self.advance(); // consume 'fn'
                methods.push(self.parse_function()?);
            } else {
                return Err(self.error("Expected method implementation in impl block"));
            }
        }

        self.consume(&Token::RBrace, "Expected '}'")?;

        Ok(Item::TraitImpl(TraitImpl {
            trait_name,
            type_params,
            for_type,
            methods,
        }))
    }

    pub(crate) fn parse_trait_method_signature(&mut self) -> Result<TraitMethod, ParseError> {
        // Parse trait method signature (no body, just signature)
        // fn method_name(self: &Self, param: Type) -> ReturnType;

        // Check for method receiver: fn (self: Type) method_name()
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
                // This is a receiver!
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

        // Parse optional return type
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Trait methods end with semicolon, not a body
        self.consume(
            &Token::Semicolon,
            "Expected ';' after trait method signature",
        )?;

        Ok(TraitMethod {
            name,
            receiver,
            params,
            return_type,
        })
    }

    pub(crate) fn parse_function(&mut self) -> Result<Function, ParseError> {
        // Note: 'fn' token is already consumed by the caller
        // For async functions, 'async fn' is consumed

        // Check for method receiver: fn (self: Type) method_name()
        let receiver = if self.check(&Token::LParen) {
            // Peek ahead to see if this is a receiver or just parameters
            let checkpoint = self.current;
            self.advance(); // consume '('

            // Check if next token is identifier "self"
            let is_self = if let Token::Ident(name) = self.peek() {
                name == "self"
            } else {
                false
            };

            if is_self {
                // This is a receiver!
                let _self_name = self.consume_identifier()?;
                self.consume(&Token::Colon, "Expected ':' after 'self'")?;
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

        // Optional generic type parameters: fn foo<T, U>()
        let type_params = if self.match_token(&Token::Lt) {
            let mut params = Vec::new();
            loop {
                params.push(self.consume_identifier()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
            self.consume(&Token::Gt, "Expected '>' after type parameters")?;
            params
        } else {
            Vec::new()
        };

        self.consume(&Token::LParen, "Expected '('")?;
        let params = self.parse_parameters()?;
        self.consume(&Token::RParen, "Expected ')'")?;

        // Optional return type with ':' (Vex syntax: fn foo(): i32)
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // parse_block() already consumes { and }
        let body = self.parse_block()?;

        Ok(Function {
            attributes: Vec::new(), // TODO: Parse attributes before function
            is_async: false,
            is_gpu: false,
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

    pub(crate) fn consume_identifier(&mut self) -> Result<String, ParseError> {
        if let Token::Ident(name) = self.peek() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            eprintln!(
                "🔴 Expected identifier but got: {:?} at position {}",
                self.peek(),
                self.current
            );
            Err(self.error("Expected identifier"))
        }
    }

    pub(crate) fn consume_identifier_or_keyword(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            Token::Unsafe => {
                self.advance();
                Ok("unsafe".to_string())
            }
            Token::Mut => {
                self.advance();
                Ok("mut".to_string())
            }
            Token::Error => {
                self.advance();
                Ok("error".to_string())
            }
            Token::Type => {
                self.advance();
                Ok("type".to_string())
            }
            _ => Err(self.error("Expected identifier or keyword")),
        }
    }

    // ==================== FFI / Extern Block Parsing ====================

    pub(crate) fn parse_extern_block(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Extern, "Expected 'extern'")?;

        // Parse ABI: extern "C" { ... } or extern "system" { ... }
        let abi = if let Token::StringLiteral(s) = self.peek() {
            let abi = s.clone();
            self.advance();
            abi
        } else {
            "C".to_string() // Default to C ABI
        };

        self.consume(&Token::LBrace, "Expected '{' after extern ABI")?;

        let mut functions = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            functions.push(self.parse_extern_function()?);
        }

        self.consume(&Token::RBrace, "Expected '}' after extern functions")?;

        Ok(Item::ExternBlock(ExternBlock {
            attributes: Vec::new(), // TODO: Parse attributes
            abi,
            functions,
        }))
    }

    fn parse_extern_function(&mut self) -> Result<ExternFunction, ParseError> {
        self.consume(&Token::Fn, "Expected 'fn' in extern block")?;

        let name = self.consume_identifier()?;

        self.consume(&Token::LParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        let mut is_variadic = false;

        if !self.check(&Token::RParen) {
            loop {
                // Check for variadic: ...
                if self.match_token(&Token::DotDotDot) {
                    is_variadic = true;
                    break;
                }

                let param_name = self.consume_identifier()?;
                self.consume(&Token::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;

                params.push(Param {
                    name: param_name,
                    ty: param_type,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }

                // Check for variadic after comma: fn printf(fmt: *byte, ...)
                if self.check(&Token::DotDotDot) {
                    self.advance();
                    is_variadic = true;
                    break;
                }
            }
        }

        self.consume(&Token::RParen, "Expected ')' after parameters")?;

        // Optional return type: -> Type
        let return_type = if self.match_token(&Token::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume(&Token::Semicolon, "Expected ';' after extern function")?;

        Ok(ExternFunction {
            attributes: Vec::new(), // TODO: Parse attributes
            name,
            params,
            return_type,
            is_variadic,
        })
    }
}
