use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_const(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Const, "Expected 'const'")?;

        let name = self.consume_identifier()?;

        self.consume(&Token::Colon, "Expected ':' after const name")?;
        let ty = Some(self.parse_type()?);

        self.consume(&Token::Eq, "Expected '=' after const type")?;
        let value = self.parse_expression()?;

        self.consume(&Token::Semicolon, "Expected ';' after const value")?;

        Ok(Item::Const(Const { name, ty, value }))
    }
}
