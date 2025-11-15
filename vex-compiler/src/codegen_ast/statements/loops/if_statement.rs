// statements/loops/if_statement.rs
// if statement compilation

use super::super::ASTCodeGen;
use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile if statement with elif support
    pub(crate) fn compile_if_statement_dispatch(
        &mut self,
        span_id: &Option<String>,
        condition: &Expression,
        then_block: &Block,
        elif_branches: &[(Expression, Block)],
        else_block: &Option<Block>,
    ) -> Result<(), String> {
        self.compile_if_statement_impl(span_id, condition, then_block, elif_branches, else_block)
    }

    /// Compile if statement with elif support
    fn compile_if_statement_impl(
        &mut self,
        span_id: &Option<String>,
        condition: &Expression,
        then_block: &Block,
        elif_branches: &[(Expression, Block)],
        else_block: &Option<Block>,
    ) -> Result<(), String> {
        let cond_val = self.compile_expression(condition)?;

        // Convert to boolean (i1)
        let bool_val = if let BasicValueEnum::IntValue(iv) = cond_val {
            let zero = iv.get_type().const_int(0, false);
            self.builder
                .build_int_compare(IntPredicate::NE, iv, zero, "ifcond")
                .map_err(|e| format!("Failed to compare: {}", e))?
        } else {
            // ⭐ Get span from span_id
            let span = span_id
                .as_ref()
                .and_then(|id| self.span_map.get(id))
                .cloned()
                .unwrap_or_else(Span::unknown);

            self.diagnostics.emit(Diagnostic {
                level: ErrorLevel::Error,
                code: error_codes::TYPE_MISMATCH.to_string(),
                message: "If condition must be an integer or boolean value".to_string(),
                span,
                notes: vec![format!("Got non-integer type in if condition")],
                help: Some(
                    "Ensure the condition evaluates to a boolean (i1) or integer type".to_string(),
                ),
                suggestion: None,
            });
            return Err("Condition must be integer value".to_string());
        };

        let fn_val = self.current_function.ok_or("No current function")?;

        let then_bb = self.context.append_basic_block(fn_val, "then");
        let merge_bb = self.context.append_basic_block(fn_val, "ifcont");

        // Create blocks for elif branches
        let mut elif_blocks = Vec::new();
        for (i, _) in elif_branches.iter().enumerate() {
            let elif_cond_bb = self
                .context
                .append_basic_block(fn_val, &format!("elif.cond.{}", i));
            let elif_then_bb = self
                .context
                .append_basic_block(fn_val, &format!("elif.then.{}", i));
            elif_blocks.push((elif_cond_bb, elif_then_bb));
        }

        // Else block or final fallthrough
        let else_bb = self.context.append_basic_block(fn_val, "else");

        // Build initial conditional branch
        self.builder
            .build_conditional_branch(
                bool_val,
                then_bb,
                if !elif_blocks.is_empty() {
                    elif_blocks[0].0
                } else {
                    else_bb
                },
            )
            .map_err(|e| format!("Failed to build branch: {}", e))?;

        // Compile then block
        self.builder.position_at_end(then_bb);
        eprintln!(
            "📋 Compiling if then_block with {} statements",
            then_block.statements.len()
        );
        self.compile_block(then_block)?;
        let then_terminated = self
            .builder
            .get_insert_block()
            .ok_or("No active basic block")?
            .get_terminator()
            .is_some();
        eprintln!("🔍 then_terminated = {}", then_terminated);
        if !then_terminated {
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // Compile elif branches
        let mut any_unterminated = !then_terminated;
        for (i, (elif_cond, elif_body)) in elif_branches.iter().enumerate() {
            let (cond_bb, then_bb) = elif_blocks[i];

            // Position at condition block
            self.builder.position_at_end(cond_bb);

            // Evaluate elif condition
            let cond_val = self.compile_expression(elif_cond)?;
            let bool_val = if let BasicValueEnum::IntValue(iv) = cond_val {
                let zero = iv.get_type().const_int(0, false);
                self.builder
                    .build_int_compare(IntPredicate::NE, iv, zero, "elifcond")
                    .map_err(|e| format!("Failed to compare: {}", e))?
            } else {
                return Err("Elif condition must be integer value".to_string());
            };

            // Branch to elif body or next elif/else
            let next_bb = if i + 1 < elif_blocks.len() {
                elif_blocks[i + 1].0
            } else {
                else_bb
            };

            self.builder
                .build_conditional_branch(bool_val, then_bb, next_bb)
                .map_err(|e| format!("Failed to build elif branch: {}", e))?;

            // Compile elif body
            self.builder.position_at_end(then_bb);
            self.compile_block(elif_body)?;
            let elif_terminated = self
                .builder
                .get_insert_block()
                .ok_or("No active basic block")?
                .get_terminator()
                .is_some();
            if !elif_terminated {
                self.builder
                    .build_unconditional_branch(merge_bb)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;
                any_unterminated = true;
            }
        }

        // Compile else block
        self.builder.position_at_end(else_bb);
        if let Some(eb) = else_block {
            self.compile_block(eb)?;
        }
        let else_terminated = self
            .builder
            .get_insert_block()
            .ok_or("No active basic block")?
            .get_terminator()
            .is_some();
        eprintln!("🔍 else_terminated = {}", else_terminated);
        if !else_terminated {
            self.builder
                .build_unconditional_branch(merge_bb)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
            any_unterminated = true;
        }

        // Continue at merge block if at least one branch didn't terminate
        eprintln!("🔍 any_unterminated = {}", any_unterminated);
        if any_unterminated {
            self.builder.position_at_end(merge_bb);
        } else {
            // All branches terminated - merge block is unreachable
            eprintln!("⚠️  All branches terminated - adding unreachable to merge_bb!");
            self.builder.position_at_end(merge_bb);
            self.builder
                .build_unreachable()
                .map_err(|e| format!("Failed to build unreachable: {}", e))?;
        }

        Ok(())
    }
}
