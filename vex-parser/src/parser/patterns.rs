// Pattern matching parsing
// This module handles match expressions and pattern parsing

use super::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
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

            // Parse body: either a block expression { ... } or a single expression
            let body = if self.check(&Token::LBrace) {
                self.parse_block_expression()?
            } else {
                self.parse_expression()?
            };

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

    /// Parse pattern for match expressions (with Or support)
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let first_pattern = self.parse_single_pattern()?;

        // Check for Or pattern: 1 | 2 | 3 (using Pipe token)
        if self.check(&Token::Pipe) {
            let mut patterns = vec![first_pattern];

            while self.match_token(&Token::Pipe) {
                patterns.push(self.parse_single_pattern()?);
            }

            return Ok(Pattern::Or(patterns));
        }

        Ok(first_pattern)
    }

    /// Parse a single pattern (without Or)
    fn parse_single_pattern(&mut self) -> Result<Pattern, ParseError> {
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
                patterns.push(self.parse_single_pattern()?);
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

            // Check for enum pattern with dot: Result.Ok(x) or Option.None
            if self.match_token(&Token::Dot) {
                let variant = self.consume_identifier()?;

                // Check for data: Variant(pattern)
                let data = if self.match_token(&Token::LParen) {
                    let inner_pattern = if !self.check(&Token::RParen) {
                        Some(Box::new(self.parse_single_pattern()?))
                    } else {
                        None
                    };
                    self.consume(&Token::RParen, "Expected ')' after enum pattern")?;
                    inner_pattern
                } else {
                    None
                };

                return Ok(Pattern::Enum {
                    name,
                    variant,
                    data,
                });
            }

            // Check for struct pattern: Point { x, y }
            if self.check(&Token::LBrace) {
                self.advance();
                let mut fields = Vec::new();

                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let field_name = self.consume_identifier()?;

                    // Check for field: pattern or just field (shorthand)
                    let field_pattern = if self.match_token(&Token::Colon) {
                        self.parse_single_pattern()?
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

            // Check for old-style enum variant (backward compat): Ok(x), Some(value)
            if self.check(&Token::LParen) {
                self.advance();
                let inner_pattern = if !self.check(&Token::RParen) {
                    Some(Box::new(self.parse_single_pattern()?))
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
