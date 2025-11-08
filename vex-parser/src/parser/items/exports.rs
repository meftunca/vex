use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_export(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Export, "Expected 'export'")?;

        // Two patterns:
        // 1. export { io, net };
        // 2. export fn foo() {} or export const X: i32 = 5;

        if self.match_token(&Token::LBrace) {
            // Pattern 1: export { items };
            let mut export_items = Vec::new();

            loop {
                // Accept both identifiers and keywords (like "unsafe")
                let item = self.consume_identifier_or_keyword()?;
                export_items.push(item);

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }

            self.consume(&Token::RBrace, "Expected '}'")?;
            self.consume(&Token::Semicolon, "Expected ';' after export")?;

            Ok(Item::Export(Export {
                items: export_items,
            }))
        } else if self.check(&Token::Fn) {
            // Pattern 2: export fn foo() {}
            self.advance(); // consume 'fn'
            Ok(Item::Function(self.parse_function()?))
        } else if self.check(&Token::Const) {
            // Pattern 2: export const X = 5;
            self.parse_const()
        } else if self.check(&Token::Struct) {
            // Pattern 2: export struct Foo {}
            self.parse_struct()
        } else if self.check(&Token::Trait) {
            // Pattern 2: export trait Foo {}
            self.parse_interface_or_trait()
        } else {
            return Err(self.error("Expected '{', 'fn', 'const', or 'struct' after 'export'"));
        }
    }
}
