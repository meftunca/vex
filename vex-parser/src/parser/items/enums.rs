use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_enum(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Enum, "Expected 'enum'")?;

        // Capture span for the enum name
        let span = self.token_to_diag_span(&self.peek_span().span);
        let span_id = self.span_map.generate_id();
        self.span_map.record(span_id.clone(), span);

        let name = self.consume_identifier()?;

        // Optional type parameters with bounds: enum Option<T: Display>
        let (type_params, _const_params) = self.parse_type_params()?; // Enums don't support const params

        self.consume(&Token::LBrace, "Expected '{'")?;

        let mut variants = Vec::new();
        let mut steps = 0usize;
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.guard_tick(&mut steps, "enum body parse timeout", Self::PARSE_LOOP_DEFAULT_MAX_STEPS) {
                break;
            }
            let variant_name = self.consume_identifier()?;

            // Check for tuple data: None, Some(T), V4(u8, u8, u8, u8)
            let data = if self.match_token(&Token::LParen) {
                let mut types = Vec::new();

                // Parse comma-separated types
                if !self.check(&Token::RParen) {
                    loop {
                        types.push(self.parse_type()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }

                self.consume(&Token::RParen, "Expected ')' after variant types")?;
                types
            } else {
                Vec::new() // Unit variant
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
            is_exported: false, // Default to false
            span_id: Some(span_id),
            name,
            type_params,
            variants,
        }))
    }
}
