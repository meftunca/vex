// Statement parsing for Vex language

use super::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // Let statement
        if self.match_token(&Token::Let) {
            let is_mutable = self.match_token(&Token::Mut);
            let name = self.consume_identifier()?;

            let ty = if self.match_token(&Token::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };

            self.consume(&Token::Eq, "Expected '=' in let statement")?;
            let value = self.parse_expression()?;
            self.consume(&Token::Semicolon, "Expected ';' after let statement")?;

            return Ok(Statement::Let {
                is_mutable,
                name,
                ty,
                value,
            });
        }

        // Return statement
        if self.match_token(&Token::Return) {
            let expr = if !self.check(&Token::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.consume(&Token::Semicolon, "Expected ';' after return")?;
            return Ok(Statement::Return(expr));
        }

        // If statement
        if self.match_token(&Token::If) {
            eprintln!("🟡 If: parsing condition, token={:?}", self.peek());
            let condition = self.parse_expression()?;
            eprintln!("🟡 If: condition done, token={:?}", self.peek());
            let then_block = self.parse_block()?;
            eprintln!("🟡 If: then_block done, token={:?}", self.peek());

            let else_block = if self.match_token(&Token::Else) {
                let block = self.parse_block()?;
                Some(block)
            } else {
                None
            };

            return Ok(Statement::If {
                condition,
                then_block,
                else_block,
            });
        }

        // While statement
        if self.match_token(&Token::While) {
            eprintln!("🟠 While: parsing condition, token={:?}", self.peek());
            let condition = self.parse_expression()?;
            eprintln!(
                "🟠 While: condition done, token={:?} at pos {}",
                self.peek(),
                self.current
            );
            let body = self.parse_block()?;
            eprintln!("🟠 While: body done, token={:?}", self.peek());

            return Ok(Statement::While { condition, body });
        }

        // Switch statement
        if self.match_token(&Token::Switch) {
            // Parse value expression (or none for type switch)
            let value = if !self.check(&Token::LBrace) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(&Token::LBrace, "Expected '{' after switch value")?;

            let mut cases = Vec::new();
            let mut default_case = None;

            while !self.check(&Token::RBrace) && !self.is_at_end() {
                if self.match_token(&Token::Default) {
                    self.consume(&Token::Colon, "Expected ':' after default")?;
                    default_case = Some(self.parse_block_until_case_or_brace()?);
                } else if self.match_token(&Token::Case) {
                    // Parse case patterns: case 1, 2, 3:
                    let mut patterns = Vec::new();
                    loop {
                        patterns.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::Colon, "Expected ':' after case pattern")?;

                    let body = self.parse_block_until_case_or_brace()?;

                    cases.push(SwitchCase { patterns, body });
                } else {
                    return Err(self.error("Expected 'case' or 'default' in switch"));
                }
            }

            self.consume(&Token::RBrace, "Expected '}' after switch")?;

            return Ok(Statement::Switch {
                value,
                cases,
                default_case,
            });
        }

        // For statement
        if self.match_token(&Token::For) {
            // For now, we'll parse C-style for loops
            // for i := 0; i < 5; i++ { ... }

            // Init
            let init = if !self.check(&Token::Semicolon) {
                Some(Box::new(self.parse_statement()?))
            } else {
                self.advance(); // consume semicolon
                None
            };

            // Condition
            let condition = if !self.check(&Token::Semicolon) {
                eprintln!(
                    "🟢 For loop: parsing condition, current token: {:?}",
                    self.peek()
                );
                let expr = self.parse_expression()?;
                eprintln!(
                    "🟢 For loop: condition parsed, current token: {:?}",
                    self.peek()
                );
                Some(expr)
            } else {
                None
            };
            self.consume(&Token::Semicolon, "Expected ';' after for condition")?;

            // Post
            let post = if !self.check(&Token::LBrace) {
                Some(Box::new(self.parse_expression_statement()?))
            } else {
                None
            };

            let body = self.parse_block()?;

            return Ok(Statement::For {
                init,
                condition,
                post,
                body,
            });
        }

        // Variable declaration or assignment
        // Check for := or identifier followed by =
        if matches!(self.peek(), Token::Ident(_)) {
            let checkpoint = self.current;
            let name = self.consume_identifier()?;

            // Variable declaration: x := expr;
            if self.match_token(&Token::ColonEq) {
                let value = self.parse_expression()?;
                self.consume(&Token::Semicolon, "Expected ';'")?;
                return Ok(Statement::VarDecl {
                    is_const: false,
                    name,
                    ty: None, // Type inference
                    value,
                });
            }

            // Assignment: x = expr;
            if self.match_token(&Token::Eq) {
                let value = self.parse_expression()?;
                self.consume(&Token::Semicolon, "Expected ';'")?;
                return Ok(Statement::Assign {
                    target: Expression::Ident(name),
                    value,
                });
            }

            // Compound assignment: x += expr, x -= expr, etc.
            if self.match_tokens(&[Token::PlusEq, Token::MinusEq, Token::StarEq, Token::SlashEq]) {
                let op = match self.previous() {
                    Token::PlusEq => CompoundOp::Add,
                    Token::MinusEq => CompoundOp::Sub,
                    Token::StarEq => CompoundOp::Mul,
                    Token::SlashEq => CompoundOp::Div,
                    _ => unreachable!(),
                };
                let value = self.parse_expression()?;
                self.consume(&Token::Semicolon, "Expected ';' after compound assignment")?;
                return Ok(Statement::CompoundAssign {
                    target: Expression::Ident(name),
                    op,
                    value,
                });
            }

            // Not a declaration or assignment, backtrack
            self.current = checkpoint;
        }

        // Type annotation: i32 x = expr;
        if self.is_type_keyword() {
            let ty = self.parse_type()?;
            let name = self.consume_identifier()?;
            self.consume(&Token::Eq, "Expected '=' after variable name")?;
            let value = self.parse_expression()?;
            self.consume(&Token::Semicolon, "Expected ';'")?;
            return Ok(Statement::VarDecl {
                is_const: false,
                name,
                ty: Some(ty),
                value,
            });
        }

        // Check for case/default (shouldn't be here, but helps error reporting)
        if self.check(&Token::Case) || self.check(&Token::Default) {
            return Err(self.error("Unexpected case/default outside switch statement"));
        }

        // Expression statement (or assignment/compound assignment)
        self.parse_expression_statement()
    }

    pub(crate) fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expression()?;

        // Check for compound assignment operators
        if self.match_tokens(&[Token::PlusEq, Token::MinusEq, Token::StarEq, Token::SlashEq]) {
            let op = match self.previous() {
                Token::PlusEq => CompoundOp::Add,
                Token::MinusEq => CompoundOp::Sub,
                Token::StarEq => CompoundOp::Mul,
                Token::SlashEq => CompoundOp::Div,
                _ => unreachable!(),
            };
            let value = self.parse_expression()?;
            self.consume(&Token::Semicolon, "Expected ';' after compound assignment")?;
            return Ok(Statement::CompoundAssign {
                target: expr,
                op,
                value,
            });
        }

        if !self.check(&Token::LBrace) {
            // Don't consume semicolon if it's for loop post
            self.consume(&Token::Semicolon, "Expected ';' after expression")?;
        }
        Ok(Statement::Expression(expr))
    }

    pub(crate) fn is_type_keyword(&self) -> bool {
        matches!(
            self.peek(),
            Token::I8
                | Token::I16
                | Token::I32
                | Token::I64
                | Token::U8
                | Token::U16
                | Token::U32
                | Token::U64
                | Token::F32
                | Token::F64
                | Token::Bool
                | Token::String
        )
    }
}
