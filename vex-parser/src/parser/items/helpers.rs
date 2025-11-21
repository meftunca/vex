use crate::parser::Parser;
use crate::ParseError;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn consume_identifier(&mut self) -> Result<String, ParseError> {
        if let Token::Ident(name) = self.peek() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            eprintln!(
                "🔴 Expected identifier but got: {:?} at position {}",
                self.peek(),
                self.current
            );
            Err(self.make_syntax_error(
                "Expected identifier",
                Some("expected identifier"),
                Some("Identifiers look like: foo, _bar, or CamelCaseName"),
                Some(("try identifier", "foo")),
            ))
        }
    }

    pub(crate) fn consume_identifier_or_keyword(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            Token::Unsafe => {
                self.advance();
                Ok("unsafe".to_string())
            }
            Token::Error => {
                self.advance();
                Ok("error".to_string())
            }
            Token::Type => {
                self.advance();
                Ok("type".to_string())
            }
            Token::New => {
                // Allow 'new' as identifier in method/field names (for Type.new() pattern)
                self.advance();
                Ok("new".to_string())
            }
            Token::From => {
                // Allow 'from' as identifier in function/field names
                self.advance();
                Ok("from".to_string())
            }
            _ => Err(self.make_syntax_error(
                "Expected identifier or keyword",
                Some("expected identifier or keyword"),
                Some("Identifiers or keywords (e.g., 'unsafe', 'new') are allowed here"),
                Some(("try identifier or keyword", "foo")),
            )),
        }
    }
}
