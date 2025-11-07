// Expression operator parsing
// This module handles operator precedence and binary/unary operations

use super::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_additive()?;

        while self.match_tokens(&[
            Token::EqEq,
            Token::NotEq,
            Token::Lt,
            Token::LtEq,
            Token::Gt,
            Token::GtEq,
        ]) {
            let op = match self.previous() {
                Token::EqEq => BinaryOp::Eq,
                Token::NotEq => BinaryOp::NotEq,
                Token::Lt => BinaryOp::Lt,
                Token::LtEq => BinaryOp::LtEq,
                Token::Gt => BinaryOp::Gt,
                Token::GtEq => BinaryOp::GtEq,
                _ => unreachable!(),
            };
            let right = self.parse_additive()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_multiplicative()?;

        while self.match_tokens(&[Token::Plus, Token::Minus]) {
            let op = match self.previous() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_multiplicative()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_cast()?;

        while self.match_tokens(&[Token::Star, Token::Slash, Token::Percent]) {
            let op = match self.previous() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_cast()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        // Await expression: await expr
        if self.match_token(&Token::Await) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Await(Box::new(expr)));
        }

        // Reference expression: &expr or &expr! (mutable)
        if self.match_token(&Token::Ampersand) {
            let expr = self.parse_unary()?;

            // Check for mutable reference: &expr!
            let is_mutable = self.match_token(&Token::Not);

            return Ok(Expression::Reference {
                is_mutable,
                expr: Box::new(expr),
            });
        }

        // Dereference: *expr
        if self.match_token(&Token::Star) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Deref(Box::new(expr)));
        }

        if self.match_tokens(&[Token::Not, Token::Minus]) {
            let op = match self.previous() {
                Token::Not => UnaryOp::Not,
                Token::Minus => UnaryOp::Neg,
                _ => unreachable!(),
            };
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op,
                expr: Box::new(expr),
            });
        }

        self.parse_postfix()
    }

    pub(crate) fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            // Track generic type arguments for next operation (call or struct literal)
            let mut pending_type_args: Option<Vec<Type>> = None;

            // Check for generic type arguments: identity<i32>(...) or Box<i32>{...}
            if self.check(&Token::Lt) && matches!(expr, Expression::Ident(_)) {
                // Lookahead: is this a generic call or comparison?
                let checkpoint = self.current;
                self.advance(); // consume <
                eprintln!("🟣 Generic check: after '<', next token={:?}", self.peek());

                // Better heuristic: check if this looks like a type argument list
                // Generic: Foo<Type>, Foo<T>, Foo<i32, i64>
                // Comparison: a < b, x < 10, foo() < bar()
                let first_token = self.peek().clone();
                let looks_like_type = matches!(
                    first_token,
                    Token::I32
                        | Token::I64
                        | Token::I128
                        | Token::I8
                        | Token::I16
                        | Token::U8
                        | Token::U16
                        | Token::U32
                        | Token::U64
                        | Token::U128
                        | Token::F16
                        | Token::F32
                        | Token::F64
                        | Token::String
                        | Token::Bool
                        | Token::LBracket
                        | Token::Ampersand
                );

                // For identifier, check what comes after it (type parameter vs variable)
                let looks_like_generic = if matches!(first_token, Token::Ident(_)) {
                    self.advance(); // consume identifier
                    let next = self.peek().clone();
                    // Generic if followed by: >, comma, another type marker, OR < (nested generic!)
                    matches!(
                        next,
                        Token::Gt | Token::Comma | Token::LBracket | Token::Ampersand | Token::Lt
                    )
                } else {
                    looks_like_type
                };

                eprintln!(
                    "🟣 Generic check: looks_like_generic={}",
                    looks_like_generic
                );

                self.current = checkpoint; // backtrack

                if looks_like_generic {
                    // Parse generic type arguments
                    self.advance(); // consume <
                    let mut type_args = Vec::new();
                    loop {
                        type_args.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::Gt, "Expected '>' after type arguments")?;
                    pending_type_args = Some(type_args);
                }
            }

            if self.match_token(&Token::LParen) {
                // Function call
                let args = self.parse_arguments()?;
                self.consume(&Token::RParen, "Expected ')' after arguments")?;

                // Method syntax sugar: in method body, convert identifier calls to method calls
                // EXCEPT for builtin functions (print, println, panic, etc.)
                if self.in_method_body && matches!(expr, Expression::Ident(_)) {
                    if let Expression::Ident(name) = &expr {
                        // Check if this is a builtin function - don't convert to method call
                        let is_builtin = matches!(
                            name.as_str(),
                            "print"
                                | "println"
                                | "panic"
                                | "assert"
                                | "unreachable"
                                | "alloc"
                                | "free"
                                | "realloc"
                                | "sizeof"
                                | "alignof"
                                | "Some"
                                | "None"
                                | "Ok"
                                | "Err"
                                | "vec_new"
                                | "vec_with_capacity"
                                | "box_new"
                                | "string_new"
                        );

                        if is_builtin {
                            // This is a builtin function - compile as regular function call
                            expr = Expression::Call {
                                func: Box::new(expr),
                                args,
                            };
                        } else {
                            // This is a method call on self
                            expr = Expression::MethodCall {
                                receiver: Box::new(Expression::Ident("self".to_string())),
                                method: name.clone(),
                                args,
                            };
                        }
                    }
                } else {
                    expr = Expression::Call {
                        func: Box::new(expr),
                        args,
                    };
                }
                // Type args for function calls handled in codegen (not stored in AST yet)
            } else if self.check(&Token::LBrace) && matches!(expr, Expression::Ident(_)) {
                // Struct literal: TypeName { field: value, ... } or Box<i32> { field: value }
                // Lookahead to distinguish from block: check if next token after '{' is identifier followed by ':'
                let checkpoint = self.current;
                self.advance(); // consume '{'

                let is_struct_literal = matches!(self.peek(), Token::Ident(_)) && {
                    let temp_checkpoint = self.current;
                    self.advance();
                    let has_colon = self.check(&Token::Colon) || self.check(&Token::RBrace);
                    self.current = temp_checkpoint;
                    has_colon
                };

                if !is_struct_literal {
                    // Not a struct literal, backtrack
                    self.current = checkpoint;
                    break;
                }

                let struct_name = match expr {
                    Expression::Ident(name) => name,
                    _ => unreachable!(),
                };

                let mut fields = Vec::new();

                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field_name = self.consume_identifier()?;
                    self.consume(&Token::Colon, "Expected ':' after field name")?;
                    let field_value = self.parse_expression()?;

                    fields.push((field_name, field_value));

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }

                self.consume(&Token::RBrace, "Expected '}'")?;

                expr = Expression::StructLiteral {
                    name: struct_name,
                    type_args: pending_type_args.unwrap_or_default(),
                    fields,
                };
            } else if self.match_token(&Token::Dot) {
                // Field access, method call, tuple field access, or enum constructor

                // Phase 0.8: Check for tuple field access (numeric field: .0, .1, .2, etc.)
                if let Token::IntLiteral(index_val) = self.peek() {
                    let field_index = index_val.to_string();
                    self.advance(); // consume the number
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field: field_index,
                    };
                    continue;
                }

                let field_or_method = self.consume_identifier()?;

                if self.check(&Token::LParen) {
                    // Could be: method call obj.method(args) OR enum constructor Enum.Variant(data)
                    // Only treat as enum if left side is PascalCase identifier (starts with uppercase)
                    let is_potential_enum = if let Expression::Ident(name) = &expr {
                        name.chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    self.advance(); // consume '('

                    if is_potential_enum && !self.check(&Token::RParen) {
                        // Parse single argument for enum constructor
                        let first_arg = self.parse_expression()?;

                        // If followed by ')', it's enum constructor with single data field
                        // If followed by ',', it's a method call with multiple args
                        if self.check(&Token::RParen) {
                            self.advance(); // consume ')'
                            let enum_name = match expr {
                                Expression::Ident(name) => name,
                                _ => unreachable!(),
                            };
                            expr = Expression::EnumLiteral {
                                enum_name,
                                variant: field_or_method,
                                data: Some(Box::new(first_arg)),
                            };
                            continue; // Skip to next iteration
                        } else {
                            // Multiple args = method call, collect remaining args
                            let mut args = vec![first_arg];
                            while self.match_token(&Token::Comma) {
                                args.push(self.parse_expression()?);
                            }
                            self.consume(&Token::RParen, "Expected ')' after arguments")?;
                            expr = Expression::MethodCall {
                                receiver: Box::new(expr),
                                method: field_or_method,
                                args,
                            };
                        }
                    } else {
                        // Empty parens or not potential enum - parse as method call
                        let args = self.parse_arguments()?;
                        self.consume(&Token::RParen, "Expected ')' after arguments")?;
                        expr = Expression::MethodCall {
                            receiver: Box::new(expr),
                            method: field_or_method,
                            args,
                        };
                    }
                } else {
                    // No parens - could be: field access obj.field OR unit enum Enum.Variant
                    // If left is identifier, could be unit enum variant
                    if matches!(expr, Expression::Ident(_)) {
                        // Heuristic: If it looks like PascalCase (starts with uppercase), treat as enum
                        // Otherwise, treat as field access
                        let enum_name = match &expr {
                            Expression::Ident(name) => name.clone(),
                            _ => unreachable!(),
                        };

                        // Check if first char is uppercase (enum convention)
                        let looks_like_enum = enum_name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false);

                        if looks_like_enum
                            && field_or_method
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false)
                        {
                            // Both PascalCase = likely Enum.Variant
                            expr = Expression::EnumLiteral {
                                enum_name,
                                variant: field_or_method,
                                data: None,
                            };
                        } else {
                            // Field access: obj.field
                            expr = Expression::FieldAccess {
                                object: Box::new(expr),
                                field: field_or_method,
                            };
                        }
                    } else {
                        // Complex expression = definitely field access
                        expr = Expression::FieldAccess {
                            object: Box::new(expr),
                            field: field_or_method,
                        };
                    }
                }
            } else if self.match_token(&Token::LBracket) {
                // Array indexing
                let index = self.parse_expression()?;
                self.consume(&Token::RBracket, "Expected ']'")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&Token::Increment) {
                expr = Expression::PostfixOp {
                    expr: Box::new(expr),
                    op: PostfixOp::Increment,
                };
            } else if self.match_token(&Token::Decrement) {
                expr = Expression::PostfixOp {
                    expr: Box::new(expr),
                    op: PostfixOp::Decrement,
                };
            } else if self.match_token(&Token::Question) {
                // Question mark operator: expr? (Result early return)
                // Desugars to: match expr { Ok(v) => v, Err(e) => return Err(e) }
                expr = Expression::QuestionMark(Box::new(expr));
            } else {
                break;
            }
        }

        Ok(expr)
    }
}
