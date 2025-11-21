// Statement parsing for Vex language

use super::Parser;
use crate::ParseError;
use vex_ast::*;
use vex_lexer::Token;

impl<'a> Parser<'a> {
    pub(crate) fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // Let statement: let x = expr; or let! x = expr; or let (a, b) = expr;
        if self.match_token(&Token::Let) || self.match_token(&Token::LetMut) {
            let is_mutable = *self.previous() == Token::LetMut;

            // Check for tuple destructuring pattern: let (a, b) = ...
            if self.check(&Token::LParen) {
                // Parse tuple pattern
                let pattern = self.parse_pattern()?;

                let ty = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                self.consume(&Token::Eq, "Expected '=' in let statement")?;
                let value = self.parse_expression()?;
                self.consume(&Token::Semicolon, "Expected ';' after let statement")?;

                return Ok(Statement::LetPattern {
                    is_mutable,
                    pattern,
                    ty,
                    value,
                });
            }

            // Regular let binding: let name = expr
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
            let ret_start = self.current - 1;
            let expr = if !self.check(&Token::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.consume(&Token::Semicolon, "Expected ';' after return")?;
            let ret_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[ret_start].span.start..self.tokens[ret_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);
            
            return Ok(Statement::Return {
                span_id: Some(span_id),
                value: expr,
            });
        }

        // Break statement
        if self.match_token(&Token::Break) {
            let break_start = self.current - 1;
            self.consume(&Token::Semicolon, "Expected ';' after break")?;
            let break_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[break_start].span.start..self.tokens[break_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);
            
            return Ok(Statement::Break {
                span_id: Some(span_id),
            });
        }

        // Continue statement
        if self.match_token(&Token::Continue) {
            let cont_start = self.current - 1;
            self.consume(&Token::Semicolon, "Expected ';' after continue")?;
            let cont_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[cont_start].span.start..self.tokens[cont_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);
            
            return Ok(Statement::Continue {
                span_id: Some(span_id),
            });
        }

        // Defer statement (Go-style)
        // defer cleanup();
        // defer { /* block */ };
        if self.match_token(&Token::Defer) {
            let deferred_stmt = if self.check(&Token::LBrace) {
                // defer { block } - parse block and convert to block expression
                let block = self.parse_block()?;
                // Convert Block to Expression::Block
                // Since Block doesn't have return_expr, create one with None
                let block_expr = Expression::Block {
                    statements: block.statements,
                    return_expr: None,
                };
                Box::new(Statement::Expression(block_expr))
            } else {
                // defer function_call();
                let expr = self.parse_expression()?;
                self.consume(&Token::Semicolon, "Expected ';' after defer statement")?;
                Box::new(Statement::Expression(expr))
            };

            return Ok(Statement::Defer(deferred_stmt));
        }

        // Go statement
        if self.match_token(&Token::Go) {
            let go_start = self.current - 1;
            let expr = if self.check(&Token::LBrace) {
                self.parse_block_expression()?
            } else {
                self.parse_expression()?
            };
            self.consume(&Token::Semicolon, "Expected ';' after go statement")?;
            let go_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[go_start].span.start..self.tokens[go_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);
            
            return Ok(Statement::Go {
                span_id: Some(span_id),
                expr,
            });
        }

        // Unsafe block
        if self.match_token(&Token::Unsafe) {
            let unsafe_start = self.current - 1;
            let block = self.parse_block()?;
            let unsafe_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[unsafe_start].span.start..self.tokens[unsafe_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);
            
            return Ok(Statement::Unsafe {
                span_id: Some(span_id),
                block,
            });
        }

        // If statement
        if self.match_token(&Token::If) {
            let if_start = self.current - 1; // 'if' token position
            let cond_start = self.current; // Before parsing condition

            let condition = self.parse_expression()?;
            let cond_end = self.current - 1; // After parsing condition

            let then_block = self.parse_block()?;

            // Parse elif branches
            let mut elif_branches = Vec::new();
            while self.match_token(&Token::Elif) {
                let elif_condition = self.parse_expression()?;
                let elif_block = self.parse_block()?;
                elif_branches.push((elif_condition, elif_block));
            }

            // Parse else block (if present)
            let else_block = if self.match_token(&Token::Else) {
                let block = self.parse_block()?;
                Some(block)
            } else {
                None
            };

            // Generate span ID for the entire if statement
            let if_span_id = self.span_map.generate_id();
            let if_end = self.current - 1;
            let if_span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[if_start].span.start..self.tokens[if_end].span.end,
            );
            self.span_map.record(if_span_id.clone(), if_span);

            // Also record span for condition (for error reporting)
            let cond_span_id = self.span_map.generate_id();
            let cond_span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[cond_start].span.start..self.tokens[cond_end].span.end,
            );
            self.span_map.record(cond_span_id.clone(), cond_span);

            return Ok(Statement::If {
                span_id: Some(cond_span_id), // Use condition span for type errors
                condition,
                then_block,
                elif_branches,
                else_block,
            });
        }

        // While statement
        if self.match_token(&Token::While) {
            let while_start = self.current - 1;
            let cond_start = self.current; // Before condition

            let condition = self.parse_expression()?;
            let cond_end = self.current - 1; // After condition

            let body = self.parse_block()?;

            // Record condition span for error reporting
            let cond_span_id = self.span_map.generate_id();
            let cond_span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[cond_start].span.start..self.tokens[cond_end].span.end,
            );
            self.span_map.record(cond_span_id.clone(), cond_span);

            return Ok(Statement::While {
                span_id: Some(cond_span_id), // Use condition span
                condition,
                body,
            });
        }

        // Loop statement (infinite loop)
        if self.match_token(&Token::Loop) {
            let loop_start = self.current - 1;
            let body = self.parse_block()?;
            let loop_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[loop_start].span.start..self.tokens[loop_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);
            
            return Ok(Statement::Loop {
                span_id: Some(span_id),
                body,
            });
        }

        // Switch statement
        if self.match_token(&Token::Switch) {
            let switch_start = self.current - 1;
            
            // Parse value expression (or none for type switch)
            let value = if !self.check(&Token::LBrace) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(&Token::LBrace, "Expected '{' after switch value")?;

            let mut cases = Vec::new();
            let mut default_case = None;

            let mut steps = 0usize;
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                if self.guard_tick(&mut steps, "switch body parse timeout", Self::PARSE_LOOP_DEFAULT_MAX_STEPS) {
                    break;
                }
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
                    return Err(self.make_syntax_error(
                        "Expected 'case' or 'default' in switch",
                        Some("expected 'case' or 'default'"),
                        Some("Use 'case' to introduce a pattern or 'default' for the fallback case"),
                        Some(("try 'case'", "case 1: { ... }")),
                    ));
                }
            }

            self.consume(&Token::RBrace, "Expected '}' after switch")?;
            let switch_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[switch_start].span.start..self.tokens[switch_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);

            return Ok(Statement::Switch {
                span_id: Some(span_id),
                value,
                cases,
                default_case,
            });
        }

        // For statement
        // For loop
        if self.match_token(&Token::For) {
            let for_start = self.current - 1; // Save 'for' token position

            // Check for for-in loop: for i in range { }
            // Peek ahead to see if this is "ident in ..."
            let checkpoint = self.current;
            if let Ok(var_name) = self.consume_identifier() {
                if self.match_token(&Token::In) {
                    // for-in loop: for i in 0..10 { }
                    let iterable = self.parse_expression()?;
                    let body = self.parse_block()?;
                    let for_end = self.current - 1;
                    
                    let span = crate::Span::from_file_and_span(
                        &self.file_name,
                        self.source,
                        self.tokens[for_start].span.start..self.tokens[for_end].span.end,
                    );
                    let span_id = self.span_map.generate_id();
                    self.span_map.record(span_id.clone(), span);

                    return Ok(Statement::ForIn {
                        span_id: Some(span_id),
                        variable: var_name,
                        iterable,
                        body,
                    });
                } else {
                    // Not a for-in, backtrack for C-style for
                    self.current = checkpoint;
                }
            } else {
                // Not identifier, reset for C-style for
                self.current = checkpoint;
            }

            // C-style for loop: for i := 0; i < 5; i++ { ... }
            // Init
            let init = if !self.check(&Token::Semicolon) {
                Some(Box::new(self.parse_statement()?))
            } else {
                self.advance(); // consume semicolon
                None
            };

            // Condition
            let condition = if !self.check(&Token::Semicolon) {
                let expr = self.parse_expression()?;

                Some(expr)
            } else {
                None
            };
            self.consume(&Token::Semicolon, "Expected ';' after for condition")?;

            // Post (can be assignment or expression, no semicolon before brace)
            let post = if !self.check(&Token::LBrace) {
                // Try to parse as assignment first (i = i + 1)
                let checkpoint = self.current;
                if let Ok(expr) = self.parse_expression() {
                    if self.check(&Token::Eq) {
                        // It's an assignment: i = expr
                        self.current = checkpoint; // Backtrack
                        let target = self.parse_expression()?;
                        self.consume(&Token::Eq, "Expected '='")?;
                        let value = self.parse_expression()?;

                        Some(Box::new(Statement::Assign {
                            span_id: None, // Inside for-loop, not critical
                            target,
                            value,
                        }))
                    } else {
                        Some(Box::new(Statement::Expression(expr)))
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let body = self.parse_block()?;

            let span_id = self.span_map.generate_id();
            let for_end = self.current - 1;
            let for_span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[for_start].span.start..self.tokens[for_end].span.end,
            );
            self.span_map.record(span_id.clone(), for_span);

            return Ok(Statement::For {
                span_id: Some(span_id),
                init,
                condition,
                post,
                body,
            });
        }

        // Assignment or compound assignment
        // Check for identifier followed by = or +=, -=, etc.
        if matches!(self.peek(), Token::Ident(_)) {
            let checkpoint = self.current;
            let name = self.consume_identifier()?;

            // Assignment: x = expr;
            if self.match_token(&Token::Eq) {
                let assign_start = checkpoint;
                let value = self.parse_expression()?;
                self.consume(&Token::Semicolon, "Expected ';'")?;
                let assign_end = self.current - 1;
                
                let span = crate::Span::from_file_and_span(
                    &self.file_name,
                    self.source,
                    self.tokens[assign_start].span.start..self.tokens[assign_end].span.end,
                );
                let span_id = self.span_map.generate_id();
                self.span_map.record(span_id.clone(), span);
                
                return Ok(Statement::Assign {
                    span_id: Some(span_id),
                    target: Expression::Ident(name),
                    value,
                });
            }

            // Compound assignment: x += expr, x -= expr, etc.
            if self.match_tokens(&[
                Token::PlusEq,
                Token::MinusEq,
                Token::StarEq,
                Token::SlashEq,
                Token::PercentEq,
                Token::AmpersandEq,
                Token::PipeEq,
                Token::CaretEq,
                Token::LShiftEq,
                Token::RShiftEq,
            ]) {
                let comp_start = checkpoint;
                let op = match self.previous() {
                    Token::PlusEq => CompoundOp::Add,
                    Token::MinusEq => CompoundOp::Sub,
                    Token::StarEq => CompoundOp::Mul,
                    Token::SlashEq => CompoundOp::Div,
                    Token::PercentEq => CompoundOp::Mod,
                    Token::AmpersandEq => CompoundOp::BitAnd,
                    Token::PipeEq => CompoundOp::BitOr,
                    Token::CaretEq => CompoundOp::BitXor,
                    Token::LShiftEq => CompoundOp::Shl,
                    Token::RShiftEq => CompoundOp::Shr,
                    _ => unreachable!(),
                };
                let value = self.parse_expression()?;
                self.consume(&Token::Semicolon, "Expected ';' after compound assignment")?;
                let comp_end = self.current - 1;
                
                let span = crate::Span::from_file_and_span(
                    &self.file_name,
                    self.source,
                    self.tokens[comp_start].span.start..self.tokens[comp_end].span.end,
                );
                let span_id = self.span_map.generate_id();
                self.span_map.record(span_id.clone(), span);
                
                return Ok(Statement::CompoundAssign {
                    span_id: Some(span_id),
                    target: Expression::Ident(name),
                    op,
                    value,
                });
            }

            // Not a declaration or assignment, backtrack
            self.current = checkpoint;
        }

        // Check for case/default (shouldn't be here, but helps error reporting)
        if self.check(&Token::Case) || self.check(&Token::Default) {
            return Err(self.make_syntax_error(
                "Unexpected case/default outside switch statement",
                Some("unexpected case/default"),
                Some("'case'/'default' are only valid inside switch statements"),
                Some(("wrap in switch", "switch { case 1: ... }")),
            ));
        }

        // Expression statement (or assignment/compound assignment)
        self.parse_expression_statement()
    }

    pub(crate) fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr_start = self.current;
        let expr = self.parse_expression()?;

        // Check for assignment operator (supports field assignment like self.x = value)
        if self.match_token(&Token::Eq) {
            let value = self.parse_expression()?;
            self.consume(&Token::Semicolon, "Expected ';' after assignment")?;
            let expr_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[expr_start].span.start..self.tokens[expr_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);
            
            return Ok(Statement::Assign {
                span_id: Some(span_id),
                target: expr,
                value,
            });
        }

        // Check for compound assignment operators
        if self.match_tokens(&[
            Token::PlusEq,
            Token::MinusEq,
            Token::StarEq,
            Token::SlashEq,
            Token::PercentEq,
            Token::AmpersandEq,
            Token::PipeEq,
            Token::CaretEq,
            Token::LShiftEq,
            Token::RShiftEq,
        ]) {
            let op = match self.previous() {
                Token::PlusEq => CompoundOp::Add,
                Token::MinusEq => CompoundOp::Sub,
                Token::StarEq => CompoundOp::Mul,
                Token::SlashEq => CompoundOp::Div,
                Token::PercentEq => CompoundOp::Mod,
                Token::AmpersandEq => CompoundOp::BitAnd,
                Token::PipeEq => CompoundOp::BitOr,
                Token::CaretEq => CompoundOp::BitXor,
                Token::LShiftEq => CompoundOp::Shl,
                Token::RShiftEq => CompoundOp::Shr,
                _ => unreachable!(),
            };
            let value = self.parse_expression()?;
            self.consume(&Token::Semicolon, "Expected ';' after compound assignment")?;
            let expr_end = self.current - 1;
            
            let span = crate::Span::from_file_and_span(
                &self.file_name,
                self.source,
                self.tokens[expr_start].span.start..self.tokens[expr_end].span.end,
            );
            let span_id = self.span_map.generate_id();
            self.span_map.record(span_id.clone(), span);
            
            return Ok(Statement::CompoundAssign {
                span_id: Some(span_id),
                target: expr,
                op,
                value,
            });
        }

        // Match expressions don't require semicolons when used as statements
        // (they already have braces that make them unambiguous)
        let needs_semicolon = !matches!(expr, Expression::Match { .. });

        if needs_semicolon && !self.check(&Token::LBrace) {
            // Don't consume semicolon if it's for loop post
            self.consume(&Token::Semicolon, "Expected ';' after expression")?;
        }
        Ok(Statement::Expression(expr))
    }
}
