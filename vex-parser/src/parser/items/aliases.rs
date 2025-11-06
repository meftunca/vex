use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_type_alias(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Type, "Expected 'type'")?;

        let name = self.consume_identifier()?;

        // Optional type parameters with bounds: type Result<T: Display, E> = ...
        let type_params = self.parse_type_params()?;

        self.consume(&Token::Eq, "Expected '=' after type alias name")?;

        let ty = self.parse_type()?;

        self.consume(&Token::Semicolon, "Expected ';' after type alias")?;

        Ok(Item::TypeAlias(TypeAlias {
            name,
            type_params,
            ty,
        }))
    }
}
