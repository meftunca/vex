// statements/mod.rs
// Dispatcher + compile_block; submodules export statement-specific codegen.
//
// Layout:
//   - loops.rs           : if / while / for / switch
//   - control_flow.rs    : return / break / continue / defer
//   - assignment.rs      : assign / compound_assign
//   - let_statement.rs   : let + inject_type_args_recursive
//
// Public re-exports provide a flat surface for the parent module.

mod assignment;
mod control_flow;
mod let_statement;
mod loops;

use super::ASTCodeGen;
use crate::diagnostics::{error_codes, Diagnostic, ErrorLevel, Span};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a block of statements
    pub(crate) fn compile_block(&mut self, block: &Block) -> Result<(), String> {
        eprintln!("📋 compile_block: {} statements", block.statements.len());
        for (idx, stmt) in block.statements.iter().enumerate() {
            eprintln!(
                "   [{}/{}] → Compiling statement: {:?}",
                idx + 1,
                block.statements.len(),
                std::mem::discriminant(stmt)
            );

            // Check builder position before compiling
            if let Some(block) = self.builder.get_insert_block() {
                eprintln!(
                    "      Builder at block: {:?}, has_terminator={}",
                    block.get_name().to_str(),
                    block.get_terminator().is_some()
                );
            }

            self.compile_statement(stmt)?;

            // Stop compiling statements after a terminator (return/break/continue/branch)
            if let Some(current_block) = self.builder.get_insert_block() {
                if current_block.get_terminator().is_some() {
                    eprintln!("      🛑 Block has terminator, stopping");
                    break;
                }
            }
        }

        eprintln!("📋 Block compilation complete, checking for implicit terminator");

        // ⭐ CRITICAL FIX: If we're in an async resume block and block has no terminator,
        // the block MUST end somewhere. Add a DONE return if needed.
        if let Some(current_block) = self.builder.get_insert_block() {
            eprintln!(
                "   Builder at: {:?}, has_terminator={}, async_stack_depth={}",
                current_block.get_name().to_str(),
                current_block.get_terminator().is_some(),
                self.async_state_stack.len()
            );

            if current_block.get_terminator().is_none() && !self.async_state_stack.is_empty() {
                eprintln!("   ⭐ Adding implicit DONE return");
                // We're in async context and block has no explicit return/terminator
                // This happens when a resume block reaches end without another await
                let done_status = self.context.i32_type().const_int(2, false); // CORO_STATUS_DONE
                self.builder
                    .build_return(Some(&done_status))
                    .map_err(|e| format!("Failed to add implicit async done return: {}", e))?;
            }
        }

        Ok(())
    }

    /// Statement dispatcher. The actual statement bodies live in submodules.
    pub(crate) fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            // let statement
            Statement::Let {
                is_mutable,
                name,
                ty,
                value,
            } => {
                self.compile_let_statement(*is_mutable, name, ty.as_ref(), value)?;
            }

            // let pattern destructuring: let (a, b) = expr;
            Statement::LetPattern {
                is_mutable,
                pattern,
                ty,
                value,
            } => {
                self.compile_let_pattern_statement(*is_mutable, pattern, ty.as_ref(), value)?;
            }

            // simple assignment
            Statement::Assign { span_id: _, target, value } => {
                self.compile_assign_statement(target, value)?;
            }

            // compound assignment (+=, -=, *=, /=)
            Statement::CompoundAssign { span_id: _, target, op, value } => {
                self.compile_compound_assign_statement(target, op, value)?;
            }

            // control-flow
            Statement::Return { span_id: _, value: expr } => {
                self.compile_return_statement(expr.as_ref())?;
            }
            Statement::Break { span_id: _ } => {
                self.compile_break_statement()?;
            }
            Statement::Continue { span_id: _ } => {
                self.compile_continue_statement()?;
            }
            Statement::Defer(stmt) => {
                self.compile_defer_statement(stmt.as_ref())?;
            }
            Statement::Go { span_id: _, expr } => {
                self.compile_go_statement(expr)?;
            }

            // Unsafe block - enable downcast warnings instead of errors
            Statement::Unsafe { span_id: _, block } => {
                let prev_unsafe = self.is_in_unsafe_block;
                self.is_in_unsafe_block = true;
                let result = self.compile_block(block);
                self.is_in_unsafe_block = prev_unsafe;
                result?;
            }

            // loops & branching
            Statement::If {
                span_id,
                condition,
                then_block,
                elif_branches,
                else_block,
            } => {
                self.compile_if_statement(
                    span_id,
                    condition,
                    then_block,
                    elif_branches,
                    else_block,
                )?;
            }
            Statement::For {
                span_id,
                init,
                condition,
                post,
                body,
            } => {
                self.compile_for_loop(span_id, init, condition, post, body)?;
            }
            Statement::While {
                span_id,
                condition,
                body,
            } => {
                self.compile_while_loop(span_id, condition, body)?;
            }
            Statement::Loop { span_id: _, body } => {
                self.compile_loop(body)?;
            }
            Statement::ForIn {
                span_id: _,
                variable,
                iterable,
                body,
            } => {
                self.compile_for_in_loop(variable, iterable, body)?;
            }
            Statement::Switch {
                span_id: _,
                value,
                cases,
                default_case,
            } => {
                self.compile_switch_statement(value, cases, default_case)?;
            }

            // pure expression statement
            Statement::Expression(expr) => {
                // keep side effects
                let _ = self.compile_expression(expr)?;
            }

            _ => {
                let stmt_str = format!("{:?}", stmt);
                self.diagnostics.emit(Diagnostic {
                    level: ErrorLevel::Error,
                    code: error_codes::NOT_IMPLEMENTED.to_string(),
                    message: "This statement type is not yet implemented".to_string(),
                    span: Span::unknown(),
                    primary_label: Some("feature not implemented".to_string()),
                    notes: vec![format!("Statement: {}", stmt_str)],
                    help: Some("This feature is planned for a future release".to_string()),
                    suggestion: None,
                    related: Vec::new(),
                });
                    return Err(format!("Statement not yet implemented: {:?}", stmt));
            }
        }
        Ok(())
    }
}
