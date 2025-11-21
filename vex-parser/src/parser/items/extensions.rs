// Parser for builtin type contract extensions
// Syntax: i32 extends Display, Clone, Eq;

use crate::{ParseError, Parser};
use vex_ast::{BuiltinExtension, Item};
use vex_lexer::Token;

impl<'a> Parser<'a> {
    /// Parse builtin extension: i32 extends Display, Clone, Eq;
    pub(crate) fn parse_builtin_extension(&mut self) -> Result<Item, ParseError> {
        // Get type name (i32, f64, bool, string, etc.)
        let type_name = match self.peek() {
            Token::I8 => "i8",
            Token::I16 => "i16",
            Token::I32 => "i32",
            Token::I64 => "i64",
            Token::I128 => "i128",
            Token::U8 => "u8",
            Token::U16 => "u16",
            Token::U32 => "u32",
            Token::U64 => "u64",
            Token::U128 => "u128",
            Token::F16 => "f16",
            Token::F32 => "f32",
            Token::F64 => "f64",
            Token::Bool => "bool",
            Token::String => "string",
            _ => return Err(self.make_syntax_error(
                "Expected builtin type name",
                Some("expected builtin type"),
                Some("Use a builtin type like i32, u64, f32, or string"),
                Some(("try 'i32'", "i32")),
            )),
        }
        .to_string();

        self.advance(); // consume type token

        // Expect 'extends' keyword
        self.consume(
            &Token::Extends,
            "Expected 'extends' after builtin type name",
        )?;

        // Parse contract names: Display, Clone, Eq, Debug
        let mut contracts = Vec::new();

        loop {
            let contract_name = self.consume_identifier()?;
            contracts.push(contract_name);

            // Check for comma (more contracts) or semicolon (end)
            if self.match_token(&Token::Comma) {
                continue; // Parse next contract
            } else if self.match_token(&Token::Semicolon) {
                break; // End of declaration
            } else {
                return Err(self.make_syntax_error(
                    "Expected ',' or ';' after contract name in extends declaration",
                    Some("expected ',' or ';'"),
                    Some("Separate multiple contracts with ',' and terminate the list with ';'"),
                    Some(("try ',' or ';'", ",")),
                ));
            }
        }

        Ok(Item::BuiltinExtension(BuiltinExtension {
            type_name,
            contracts,
        }))
    }
}
