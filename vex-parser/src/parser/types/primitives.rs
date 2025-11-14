//! Primitive type parsing (i8, i16, i32, bool, string, etc.)

use super::Parser;
use vex_ast::Type;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    /// Try to parse a primitive type keyword
    pub(super) fn try_parse_primitive_type(&mut self) -> Option<Type> {
        let ty = match self.peek() {
            Token::I8 => {
                self.advance();
                Type::I8
            }
            Token::I16 => {
                self.advance();
                Type::I16
            }
            Token::I32 => {
                self.advance();
                Type::I32
            }
            Token::I64 => {
                self.advance();
                Type::I64
            }
            Token::I128 => {
                self.advance();
                Type::I128
            }
            Token::U8 => {
                self.advance();
                Type::U8
            }
            Token::U16 => {
                self.advance();
                Type::U16
            }
            Token::U32 => {
                self.advance();
                Type::U32
            }
            Token::U64 => {
                self.advance();
                Type::U64
            }
            Token::U128 => {
                self.advance();
                Type::U128
            }
            Token::F16 => {
                self.advance();
                Type::F16
            }
            Token::F32 => {
                self.advance();
                Type::F32
            }
            Token::F64 => {
                self.advance();
                Type::F64
            }
            Token::Bool => {
                self.advance();
                Type::Bool
            }
            Token::String => {
                self.advance();
                Type::String
            }
            Token::Any => {
                self.advance();
                Type::Any
            }
            Token::Byte => {
                self.advance();
                Type::U8 // byte is an alias for u8
            }
            Token::Nil => {
                self.advance();
                Type::Nil
            }
            _ => return None,
        };

        Some(ty)
    }
}
