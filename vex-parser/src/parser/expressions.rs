// Expression parsing for Vex language

use super::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_range()
    }

    /// Parse range expressions: 0..10 or 0..=10
    /// Lowest precedence (below comparison)
    pub(crate) fn parse_range(&mut self) -> Result<Expression, ParseError> {
        let expr = self.parse_comparison()?;

        if self.match_token(&Token::DotDotEq) {
            // Inclusive range: 0..=10
            let end = self.parse_comparison()?;
            return Ok(Expression::RangeInclusive {
                start: Box::new(expr),
                end: Box::new(end),
            });
        } else if self.match_token(&Token::DotDot) {
            // Exclusive range: 0..10
            let end = self.parse_comparison()?;
            return Ok(Expression::Range {
                start: Box::new(expr),
                end: Box::new(end),
            });
        }

        Ok(expr)
    }

    /// Parse type cast: expr as TargetType
    /// Priority: Higher than unary, lower than multiplicative
    pub(crate) fn parse_cast(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_unary()?;

        while self.check(&Token::As) {
            self.advance(); // Consume 'as'
            let target_type = self.parse_type()?;
            expr = Expression::Cast {
                expr: Box::new(expr),
                target_type,
            };
        }

        Ok(expr)
    }

    /// Parse closure/lambda: |x, y| expr or |x: i32, y: i32| { body }
    pub(crate) fn parse_closure(&mut self) -> Result<Expression, ParseError> {
        // Consume opening pipe: |
        self.consume(&Token::Pipe, "Expected '|' at start of closure")?;

        // Parse parameters: |x, y| or |x: i32, y: i32|
        let mut params = Vec::new();

        if !self.check(&Token::Pipe) {
            loop {
                let param_name = self.consume_identifier()?;

                // Optional type annotation: x: i32
                let param_type = if self.match_token(&Token::Colon) {
                    // Use parse_type_primary to avoid parsing | as union type operator
                    self.parse_type_primary()?
                } else {
                    // TODO: Type inference for closures
                    return Err(self.error("Closure parameters require type annotations for now"));
                };

                params.push(vex_ast::Param {
                    name: param_name,
                    ty: param_type,
                });

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.consume(&Token::Pipe, "Expected '|' after closure parameters")?;

        // Optional return type: |x: i32|: i32
        let return_type = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse body: either expression or block
        let body = if self.check(&Token::LBrace) {
            // Block body: |x| { x + 1 }
            self.parse_block_expression()?
        } else {
            // Expression body: |x| x + 1
            self.parse_expression()?
        };

        Ok(Expression::Closure {
            params,
            return_type,
            body: Box::new(body),
            capture_mode: CaptureMode::Infer, // Will be determined by borrow checker
        })
    }
}
