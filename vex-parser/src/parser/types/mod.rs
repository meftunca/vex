//! Type parsing module for Vex language
//!
//! This module is organized into:
//! - `primitives`: Primitive type parsing (i32, bool, etc.)
//! - `complex`: Complex type parsing (tuples, arrays, references)
//! - `generics`: Generic and named type parsing
//! - `special`: Special type parsing (Self, typeof, infer, never)

mod complex;
mod generics;
mod primitives;
mod special;

use super::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    /// Main entry point for type parsing
    pub(crate) fn parse_type(&mut self) -> Result<Type, ParseError> {
        // 1. Check for special types first (infer, typeof, never, raw pointers)
        if let Some(ty) = self.try_parse_special_type()? {
            return self.parse_type_operators(ty);
        }

        // 2. Check for function types: fn(T1, T2): R
        if self.check(&Token::Fn) {
            let ty = self.parse_function_type()?;
            return self.parse_type_operators(ty);
        }

        // 3. Check for complex types (references, slices, tuples, arrays)
        if let Some(ty) = self.try_parse_complex_type()? {
            return self.parse_type_operators(ty);
        }

        // 4. Parse primary type (primitives, named, generics)
        let ty = self.parse_type_base()?;

        // 5. Parse type operators (extends, union, intersection)
        self.parse_type_operators(ty)
    }

    /// Parse a primary type without union/intersection operators
    pub(crate) fn parse_type_primary(&mut self) -> Result<Type, ParseError> {
        // Special types
        if let Some(ty) = self.try_parse_special_type()? {
            return Ok(ty);
        }

        // Complex types
        if let Some(ty) = self.try_parse_complex_type()? {
            return Ok(ty);
        }

        // Base types
        self.parse_type_base()
    }

    /// Parse base type (primitives, named, generics)
    fn parse_type_base(&mut self) -> Result<Type, ParseError> {
        // Try primitive types first
        if let Some(ty) = self.try_parse_primitive_type() {
            return Ok(ty);
        }

        // Try generic/named types
        self.parse_named_or_generic_type()
    }

    /// Parse type operators (extends, union, intersection) after base type
    fn parse_type_operators(&mut self, base_type: Type) -> Result<Type, ParseError> {
        // Conditional type: T extends U ? X : Y
        if self.check(&Token::Extends) {
            return self.parse_conditional_type(base_type);
        }

        // Intersection type: T1 & T2 & T3
        if self.should_parse_intersection(&base_type) {
            return self.parse_intersection_type(base_type);
        }

        // Union type: T1 | T2 | T3
        if self.check(&Token::Pipe) {
            return self.parse_union_type(base_type);
        }

        Ok(base_type)
    }

    /// Check if we should parse intersection type
    fn should_parse_intersection(&self, ty: &Type) -> bool {
        self.check(&Token::Ampersand)
            && !matches!(
                ty,
                Type::I8
                    | Type::I16
                    | Type::I32
                    | Type::I64
                    | Type::U8
                    | Type::U16
                    | Type::U32
                    | Type::U64
                    | Type::F32
                    | Type::F64
                    | Type::Bool
                    | Type::String
            )
    }

    /// Parse conditional type: T extends U ? X : Y
    fn parse_conditional_type(&mut self, check_type: Type) -> Result<Type, ParseError> {
        self.advance(); // consume 'extends'
        let extends_type = Box::new(self.parse_type_primary()?);
        self.consume(&Token::Question, "Expected '?' in conditional type")?;
        let true_type = Box::new(self.parse_type()?);
        self.consume(&Token::Colon, "Expected ':' in conditional type")?;
        let false_type = Box::new(self.parse_type()?);

        Ok(Type::Conditional {
            check_type: Box::new(check_type),
            extends_type,
            true_type,
            false_type,
        })
    }

    /// Parse intersection type: T1 & T2 & T3
    fn parse_intersection_type(&mut self, first_type: Type) -> Result<Type, ParseError> {
        let mut types = vec![first_type];
        while self.match_token(&Token::Ampersand) {
            types.push(self.parse_type_primary()?);
        }
        Ok(Type::Intersection(types))
    }

    /// Parse union type: T1 | T2 | T3
    fn parse_union_type(&mut self, first_type: Type) -> Result<Type, ParseError> {
        let mut types = vec![first_type];
        while self.match_token(&Token::Pipe) {
            types.push(self.parse_type_primary()?);
        }
        Ok(Type::Union(types))
    }
}
