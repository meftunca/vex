// Expression operator parsing
// This module handles operator precedence and binary/unary operations

use super::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_null_coalesce()?;

        while self.match_token(&Token::Or) {
            let op_start = self.current - 1;
            let right = self.parse_null_coalesce()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_null_coalesce(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&Token::QuestionQuestion) {
            let op_start = self.current - 1;
            let right = self.parse_logical_and()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::NullCoalesce,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_bitwise_or()?;

        while self.match_token(&Token::And) {
            let op_start = self.current - 1;
            let right = self.parse_bitwise_or()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_op_range(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_bitwise_or()?;

        // Binary range operators for operator overloading: a..b and a..=b
        if self.match_token(&Token::DotDotEq) {
            let op_start = self.current - 1;
            let right = self.parse_bitwise_or()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::RangeInclusive,
                right: Box::new(right),
            };
        } else if self.match_token(&Token::DotDot) {
            let op_start = self.current - 1;
            let right = self.parse_bitwise_or()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::Range,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_bitwise_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.match_token(&Token::Pipe) {
            let op_start = self.current - 1;
            let right = self.parse_bitwise_xor()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::BitOr,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_bitwise_xor(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_bitwise_and()?;

        while self.match_token(&Token::Caret) {
            let op_start = self.current - 1;
            let right = self.parse_bitwise_and()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::BitXor,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_bitwise_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_shift()?;

        while self.match_token(&Token::Ampersand) {
            let op_start = self.current - 1;
            let right = self.parse_shift()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::BitAnd,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_shift(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_comparison()?;

        while self.match_tokens(&[Token::LShift, Token::RShift]) {
            let op_start = self.current - 1;
            let op = match self.previous() {
                Token::LShift => BinaryOp::Shl,
                Token::RShift => BinaryOp::Shr,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

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
            let op_start = self.current - 1;
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
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
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
            let op_start = self.current - 1;
            let op = match self.previous() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_multiplicative()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_power()?;

        while self.match_tokens(&[Token::Star, Token::Slash, Token::Percent]) {
            let op_start = self.current - 1;
            let op = match self.previous() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_power()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_power(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_cast()?;

        // Right-associative: 2 ** 3 ** 2 = 2 ** (3 ** 2) = 512
        if self.match_token(&Token::StarStar) {
            let op_start = self.current - 1;
            let right = self.parse_power()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            expr = Expression::Binary {
                span_id: Some(span_id),
                left: Box::new(expr),
                op: BinaryOp::Pow,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub(crate) fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        // Await expression: await expr
        if self.match_token(&Token::Await) {
            eprintln!("⏸️ Matched AWAIT token, parsing inner expression");
            let expr = self.parse_unary()?;
            eprintln!("⏸️ Await expression parsed: {:?}", expr);
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

        // Channel receive: <-expr (Go-style syntax)
        if self.match_token(&Token::LeftArrow) {
            let expr = self.parse_unary()?;
            return Ok(Expression::ChannelReceive(Box::new(expr)));
        }

        // Pre-increment: ++i
        if self.match_token(&Token::Increment) {
            let op_start = self.current - 1;
            let expr = self.parse_unary()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            return Ok(Expression::Unary {
                span_id: Some(span_id),
                op: UnaryOp::PreInc,
                expr: Box::new(expr),
            });
        }

        // Pre-decrement: --i
        if self.match_token(&Token::Decrement) {
            let op_start = self.current - 1;
            let expr = self.parse_unary()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            return Ok(Expression::Unary {
                span_id: Some(span_id),
                op: UnaryOp::PreDec,
                expr: Box::new(expr),
            });
        }

        if self.match_tokens(&[Token::Not, Token::Minus, Token::Tilde]) {
            let op_start = self.current - 1;
            let op = match self.previous() {
                Token::Not => UnaryOp::Not,
                Token::Minus => UnaryOp::Neg,
                Token::Tilde => UnaryOp::BitNot,
                _ => unreachable!(),
            };
            let expr = self.parse_unary()?;
            let op_end = self.current - 1;

            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[op_start].span.start..self.tokens[op_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            return Ok(Expression::Unary {
                span_id: Some(span_id),
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
                    self.consume_generic_close("Expected '>' after type arguments")?;
                    pending_type_args = Some(type_args);
                }
            }

            if self.match_token(&Token::LParen) {
                // Function call
                let call_start = self.current - 1;
                let args = self.parse_arguments()?;
                self.consume(&Token::RParen, "Expected ')' after arguments")?;
                let call_end = self.current - 1;

                // Extract pending type arguments (if any)
                let type_args = pending_type_args.take().unwrap_or_default();

                // Generate span for call expression
                let span = crate::Span::from_file_and_span(
                    &self.file_name,
                    self.source,
                    self.tokens[call_start].span.start..self.tokens[call_end].span.end,
                );
                let span_id = self.span_map.generate_id();
                self.span_map.record(span_id.clone(), span);

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
                                span_id: Some(span_id),
                                func: Box::new(expr),
                                type_args,
                                args,
                            };
                        } else {
                            // Treat all function calls inside method bodies as regular Call
                            // not MethodCall. The compiler will resolve whether it's:
                            // - A global function (log2(msg))
                            // - A method on self (self.log(msg))
                            expr = Expression::Call {
                                span_id: Some(span_id),
                                func: Box::new(expr),
                                type_args,
                                args,
                            };
                        }
                    }
                } else {
                    expr = Expression::Call {
                        span_id: Some(span_id),
                        func: Box::new(expr),
                        type_args,
                        args,
                    };
                }
            } else if self.check(&Token::LBrace) && matches!(expr, Expression::Ident(_)) {
                // Struct literal: TypeName { field: value, ... } or Box<i32> { field: value }
                // Lookahead to distinguish from block: check if next token after '{' is identifier followed by ':'
                // OR if it's immediately '}'  (empty struct)
                let checkpoint = self.current;
                self.advance(); // consume '{'

                let is_struct_literal = self.check(&Token::RBrace)
                    || (matches!(self.peek(), Token::Ident(_)) && {
                        let temp_checkpoint = self.current;
                        self.advance();
                        let has_colon = self.check(&Token::Colon) || self.check(&Token::RBrace);
                        self.current = temp_checkpoint;
                        has_colon
                    });

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

                let mut steps = 0usize;
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    if self.guard_tick(
                        &mut steps,
                        "struct literal parse timeout",
                        Self::PARSE_LOOP_DEFAULT_MAX_STEPS,
                    ) {
                        break;
                    }
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
                if let Token::IntLiteral(index_str) = self.peek() {
                    let field_index = index_str.clone();
                    self.advance(); // consume the number
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field: field_index,
                    };
                    continue;
                }

                // Allow keywords as method/field names (new, free, etc.)
                let field_or_method = self.consume_identifier_or_keyword()?;

                // ⭐ NEW: Check for generic type arguments after method name: obj.method<T, U>(...)
                // This enables syntax like: HashMap.new<str, i32>()
                // BUT: Only parse as generic if it looks like a type argument list, not a comparison
                let method_type_args = if self.check(&Token::Lt) {
                    // Lookahead: does this look like generics or comparison?
                    let checkpoint = self.current;
                    self.advance(); // consume <

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

                    let looks_like_generic = if matches!(first_token, Token::Ident(_)) {
                        self.advance(); // consume identifier
                        let next = self.peek().clone();
                        // Generic if followed by: >, comma, type markers, or nested <
                        matches!(
                            next,
                            Token::Gt
                                | Token::Comma
                                | Token::LBracket
                                | Token::Ampersand
                                | Token::Lt
                        )
                    } else {
                        looks_like_type
                    };

                    self.current = checkpoint; // backtrack

                    if looks_like_generic {
                        self.advance(); // consume <
                        let mut type_args = Vec::new();
                        loop {
                            type_args.push(self.parse_type()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                        self.consume_generic_close("Expected '>' after type arguments")?;
                        Some(type_args)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if self.check(&Token::LParen) {
                    // Could be: method call obj.method(args) OR enum constructor Enum.Variant(data)
                    // Priority rules:
                    // 1. If method name is lowercase (new, from, etc.) → ALWAYS method call (static methods)
                    // 2. If Type.Variant(...) with both PascalCase → enum constructor
                    // 3. Otherwise → method call

                    let is_potential_enum = if let Expression::Ident(name) = &expr {
                        let type_is_pascal = name
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false);
                        let method_is_pascal = field_or_method
                            .chars()
                            .next()
                            .map(|c| c.is_uppercase())
                            .unwrap_or(false);

                        // Special keywords like 'new' should ALWAYS be method calls, not enums
                        let is_static_method_keyword = matches!(
                            field_or_method.as_str(),
                            "new" | "from" | "with_capacity" | "default"
                        );

                        // It's an enum only if:
                        // - Both are PascalCase AND
                        // - NOT a known static method keyword
                        type_is_pascal && method_is_pascal && !is_static_method_keyword
                    } else {
                        false
                    };

                    self.advance(); // consume '('

                    if is_potential_enum && !self.check(&Token::RParen) {
                        // Parse arguments (may be enum constructor or method call)
                        let mut args = vec![self.parse_expression()?];
                        while self.match_token(&Token::Comma) {
                            args.push(self.parse_expression()?);
                        }
                        self.consume(&Token::RParen, "Expected ')' after arguments")?;

                        // Enum constructor with data (single or multi-value tuple)
                        let enum_name = match expr {
                            Expression::Ident(name) => name,
                            _ => unreachable!(),
                        };
                        expr = Expression::EnumLiteral {
                            enum_name,
                            variant: field_or_method,
                            data: args,
                        };
                        continue; // Skip to next iteration
                    } else {
                        // Empty parens or not potential enum - parse as method call
                        let args = self.parse_arguments()?;
                        self.consume(&Token::RParen, "Expected ')' after arguments")?;

                        // Check for mutable call suffix: method()!
                        let is_mutable_call = self.match_token(&Token::Not);

                        // ⭐ Combine type args: method-level generics take priority over pending ones
                        // Example: HashMap.new<K, V>() → method_type_args = [K, V]
                        // Example: obj<T>.method() → pending_type_args = [T]
                        let final_type_args = method_type_args
                            .unwrap_or_else(|| pending_type_args.take().unwrap_or_default());

                        expr = Expression::MethodCall {
                            receiver: Box::new(expr),
                            method: field_or_method,
                            type_args: final_type_args,
                            args,
                            is_mutable_call,
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
                            // Both PascalCase = likely Enum.Variant (unit variant, no data)
                            expr = Expression::EnumLiteral {
                                enum_name,
                                variant: field_or_method,
                                data: vec![],
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
                    op: PostfixOp::PostInc,
                };
            } else if self.match_token(&Token::Decrement) {
                expr = Expression::PostfixOp {
                    expr: Box::new(expr),
                    op: PostfixOp::PostDec,
                };
            } else if self.match_token(&Token::Question) {
                // Try operator: expr? (unwrap Result or propagate error)
                // Desugars to: match expr { Ok(v) => v, Err(e) => return Err(e) }
                expr = Expression::TryOp {
                    expr: Box::new(expr),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }
}
