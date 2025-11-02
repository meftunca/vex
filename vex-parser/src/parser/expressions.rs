// Expression parsing for Vex language

use super::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_comparison()
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
        let mut expr = self.parse_unary()?;

        while self.match_tokens(&[Token::Star, Token::Slash, Token::Percent]) {
            let op = match self.previous() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
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

        // Go expression: go expr (spawn goroutine/task)
        if self.match_token(&Token::Go) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Go(Box::new(expr)));
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

                let looks_like_generic = matches!(
                    self.peek(),
                    Token::Ident(_)
                        | Token::I32
                        | Token::I64
                        | Token::I8
                        | Token::I16
                        | Token::U8
                        | Token::U16
                        | Token::U32
                        | Token::U64
                        | Token::F32
                        | Token::F64
                        | Token::String
                        | Token::Bool
                        | Token::LBracket
                        | Token::LParen
                        | Token::Ampersand
                );
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
                expr = Expression::Call {
                    func: Box::new(expr),
                    args,
                };
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
                // Field access or method call
                let field_or_method = self.consume_identifier()?;

                if self.check(&Token::LParen) {
                    // Method call: obj.method(args)
                    self.advance(); // consume '('
                    let args = self.parse_arguments()?;
                    self.consume(&Token::RParen, "Expected ')' after arguments")?;
                    expr = Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: field_or_method,
                        args,
                    };
                } else {
                    // Field access: obj.field
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field: field_or_method,
                    };
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
                // Try operator: expr?
                expr = Expression::Try(Box::new(expr));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    pub(crate) fn parse_arguments(&mut self) -> Result<Vec<Expression>, ParseError> {
        let mut args = Vec::new();

        if self.check(&Token::RParen) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        Ok(args)
    }

    pub(crate) fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        // Integer literal
        if let Token::IntLiteral(n) = self.peek() {
            let n = *n;
            self.advance();
            return Ok(Expression::IntLiteral(n));
        }

        // Float literal
        if let Token::FloatLiteral(f) = self.peek() {
            let f = *f;
            self.advance();
            return Ok(Expression::FloatLiteral(f));
        }

        // String literal
        if let Token::StringLiteral(s) = self.peek() {
            let s = s.clone();
            self.advance();
            return Ok(Expression::StringLiteral(s));
        }

        // F-string literal (formatted string)
        if let Token::FStringLiteral(s) = self.peek() {
            let s = s.clone();
            self.advance();
            return Ok(Expression::FStringLiteral(s));
        }

        // Boolean literals
        if self.match_token(&Token::True) {
            return Ok(Expression::BoolLiteral(true));
        }
        if self.match_token(&Token::False) {
            return Ok(Expression::BoolLiteral(false));
        }

        // Nil literal
        if self.match_token(&Token::Nil) {
            return Ok(Expression::Nil);
        }

        // Match expression
        if self.match_token(&Token::Match) {
            return self.parse_match_expression();
        }

        // Array literal
        if self.match_token(&Token::LBracket) {
            let mut elements = Vec::new();

            if !self.check(&Token::RBracket) {
                loop {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }

            self.consume(&Token::RBracket, "Expected ']'")?;
            return Ok(Expression::Array(elements));
        }

        // Parenthesized expression or tuple literal
        if self.match_token(&Token::LParen) {
            // Empty tuple: ()
            if self.check(&Token::RParen) {
                self.advance();
                return Ok(Expression::TupleLiteral(Vec::new()));
            }

            let first_expr = self.parse_expression()?;

            // If followed by comma, it's a tuple
            if self.match_token(&Token::Comma) {
                let mut elements = vec![first_expr];

                // Parse remaining elements
                if !self.check(&Token::RParen) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                        // Allow trailing comma
                        if self.check(&Token::RParen) {
                            break;
                        }
                    }
                }

                self.consume(&Token::RParen, "Expected ')' after tuple")?;
                return Ok(Expression::TupleLiteral(elements));
            }

            // Just a parenthesized expression
            self.consume(&Token::RParen, "Expected ')'")?;
            return Ok(first_expr);
        }

        // Identifier or keyword as identifier (for struct names like "error")
        if matches!(self.peek(), Token::Ident(_)) {
            let name = self.consume_identifier()?;
            return Ok(Expression::Ident(name));
        }

        // Allow keywords as identifiers in expressions (e.g., "error" struct)
        if matches!(self.peek(), Token::Error) {
            self.advance();
            return Ok(Expression::Ident("error".to_string()));
        }
        if matches!(self.peek(), Token::Type) {
            self.advance();
            return Ok(Expression::Ident("type".to_string()));
        }

        Err(self.error("Expected expression"))
    }

    /// Parse match expression: match value { pattern => expr, ... }
    pub(crate) fn parse_match_expression(&mut self) -> Result<Expression, ParseError> {
        // Parse the value to match on
        let value = Box::new(self.parse_expression()?);

        self.consume(&Token::LBrace, "Expected '{' after match value")?;

        let mut arms = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Parse pattern
            let pattern = self.parse_pattern()?;

            // Optional guard: if condition
            let guard = if self.match_token(&Token::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(&Token::FatArrow, "Expected '=>' after pattern")?;

            // Parse body expression
            let body = self.parse_expression()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            // Optional comma
            self.match_token(&Token::Comma);
        }

        self.consume(&Token::RBrace, "Expected '}' after match arms")?;

        Ok(Expression::Match { value, arms })
    }

    /// Parse pattern for match expressions
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        // Wildcard: _
        if self.match_token(&Token::Underscore) {
            return Ok(Pattern::Wildcard);
        }

        // Tuple pattern: (x, y, z) or (a, _, c)
        if self.check(&Token::LParen) {
            self.advance();
            let mut patterns = Vec::new();

            // Empty tuple
            if self.check(&Token::RParen) {
                self.advance();
                return Ok(Pattern::Tuple(patterns));
            }

            // Parse patterns
            loop {
                patterns.push(self.parse_pattern()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }

            self.consume(&Token::RParen, "Expected ')' after tuple pattern")?;

            // Single element in parens is not a tuple, return the inner pattern
            if patterns.len() == 1 {
                return Ok(patterns.into_iter().next().unwrap());
            }

            return Ok(Pattern::Tuple(patterns));
        }

        // Identifier binding
        if let Token::Ident(name) = self.peek() {
            let name = name.clone();
            self.advance();

            // Check for struct pattern: Point { x, y }
            if self.check(&Token::LBrace) {
                self.advance();
                let mut fields = Vec::new();

                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field_name = self.consume_identifier()?;

                    // Check for field: pattern or just field (shorthand)
                    let field_pattern = if self.match_token(&Token::Colon) {
                        self.parse_pattern()?
                    } else {
                        // Shorthand: { x, y } means { x: x, y: y }
                        Pattern::Ident(field_name.clone())
                    };

                    fields.push((field_name, field_pattern));

                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }

                self.consume(&Token::RBrace, "Expected '}' after struct pattern")?;

                return Ok(Pattern::Struct { name, fields });
            }

            // Check for enum variant: Ok(x), Some(value)
            if self.check(&Token::LParen) {
                self.advance();
                let inner_pattern = if !self.check(&Token::RParen) {
                    Some(Box::new(self.parse_pattern()?))
                } else {
                    None
                };
                self.consume(&Token::RParen, "Expected ')' after enum pattern")?;

                return Ok(Pattern::Enum {
                    name: String::new(), // Will be resolved later
                    variant: name,
                    data: inner_pattern,
                });
            }

            // Simple identifier binding
            return Ok(Pattern::Ident(name));
        }

        // Literal pattern
        let expr = self.parse_primary()?;
        Ok(Pattern::Literal(expr))
    }
}
