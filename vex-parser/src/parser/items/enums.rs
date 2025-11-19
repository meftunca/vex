use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_enum(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Enum, "Expected 'enum'")?;

        let name = self.consume_identifier()?;

        // Optional type parameters with bounds: enum Option<T: Display>
        let (type_params, _const_params) = self.parse_type_params()?; // Enums don't support const params

        self.consume(&Token::LBrace, "Expected '{'")?;

        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
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
            name,
            type_params,
            variants,
        }))
    }
}
