// Primary expression parsing
// This module handles literals, identifiers, arrays, tuples, and parenthesized expressions

use super::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
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
        // Closure/Lambda: |x, y| expr or |x: i32, y: i32| { body }
        if self.match_token(&Token::Pipe) {
            return self.parse_closure();
        }

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

            // Method syntax sugar: in method body, identifier followed by ( is a method call
            if self.in_method_body && self.check(&Token::LParen) {
                let args = self.parse_arguments()?;
                return Ok(Expression::MethodCall {
                    receiver: Box::new(Expression::Ident("self".to_string())),
                    method: name,
                    args,
                });
            }

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
}
