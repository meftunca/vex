use crate::parser::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_export(&mut self) -> Result<Item, ParseError> {
        self.consume(&Token::Export, "Expected 'export'")?;

        // Four patterns:
        // 1. export { io, net };
        // 2. export { Arc } from "./arc.vx";
        // 3. export * from "./module.vx";
        // 4. export fn foo() {} or export const X: i32 = 5;

        // Pattern 3: export * from "module"
        if self.match_token(&Token::Star) {
            self.consume(&Token::From, "Expected 'from' after '*'")?;
            
            let module_path = if let Token::StringLiteral(s) = self.peek() {
                let m = s.clone();
                self.advance();
                m
            } else {
                return Err(self.error("Expected module string after 'from'"));
            };
            
            self.consume(&Token::Semicolon, "Expected ';' after export statement")?;
            
            return Ok(Item::Export(Export {
                items: vec![],
                from_module: Some(module_path),
                is_wildcard: true,
            }));
        }

        if self.match_token(&Token::LBrace) {
            // Pattern 1 or 2: export { items }; or export { items } from "module";
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
            
            // Check for 'from' keyword (re-export)
            if self.match_token(&Token::From) {
                let module_path = if let Token::StringLiteral(s) = self.peek() {
                    let m = s.clone();
                    self.advance();
                    m
                } else {
                    return Err(self.error("Expected module string after 'from'"));
                };
                
                self.consume(&Token::Semicolon, "Expected ';' after export statement")?;
                
                return Ok(Item::Export(Export {
                    items: export_items,
                    from_module: Some(module_path),
                    is_wildcard: false,
                }));
            }
            
            // Regular export
            self.consume(&Token::Semicolon, "Expected ';' after export")?;

            Ok(Item::Export(Export {
                items: export_items,
                from_module: None,
                is_wildcard: false,
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
        } else if self.check(&Token::Contract) {
            // Pattern 2: export contract Foo {}
            self.parse_trait()
        } else if self.check(&Token::Enum) {
            // Pattern 2: export enum Foo {}
            self.parse_enum()
        } else {
            return Err(self.error(
                "Expected '{', 'fn', 'const', 'struct', 'contract', or 'enum' after 'export'",
            ));
        }
    }
}
