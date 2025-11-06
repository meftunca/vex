use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_enum(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Enum, "Expected 'enum'")?;

        let name = self.consume_identifier()?;

        // Optional type parameters with bounds: enum Option<T: Display>
        let type_params = self.parse_type_params()?;

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
}
