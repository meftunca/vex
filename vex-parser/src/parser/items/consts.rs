use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_const(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Const, "Expected 'const'")?;

        // Capture span for the const name
        let span = self.token_to_diag_span(&self.peek_span().span);
        let span_id = self.span_map.generate_id();
        self.span_map.record(span_id.clone(), span);

        let name = self.consume_identifier()?;

        self.consume(&Token::Colon, "Expected ':' after const name")?;
        let ty = Some(self.parse_type()?);

        self.consume(&Token::Eq, "Expected '=' after const type")?;
        let value = self.parse_expression()?;

        self.consume(&Token::Semicolon, "Expected ';' after const value")?;

        Ok(Item::Const(Const {
            is_exported: false, // Default to false
            span_id: Some(span_id),
            name,
            ty,
            value,
        }))
    }
}
