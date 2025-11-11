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
        for stmt in &block.statements {
            self.compile_statement(stmt)?;

            // Stop compiling statements after a terminator (return/break/continue/branch)
            if let Some(current_block) = self.builder.get_insert_block() {
                if current_block.get_terminator().is_some() {
                    break;
                }
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

            // simple assignment
            Statement::Assign { target, value } => {
                self.compile_assign_statement(target, value)?;
            }

            // compound assignment (+=, -=, *=, /=)
            Statement::CompoundAssign { target, op, value } => {
                self.compile_compound_assign_statement(target, op, value)?;
            }

            // control-flow
            Statement::Return(expr) => {
                self.compile_return_statement(expr.as_ref())?;
            }
            Statement::Break => {
                self.compile_break_statement()?;
            }
            Statement::Continue => {
                self.compile_continue_statement()?;
            }
            Statement::Defer(stmt) => {
                self.compile_defer_statement(stmt.as_ref())?;
            }
            Statement::Go(expr) => {
                self.compile_go_statement(expr)?;
            }

            // Unsafe block - just compile the block (no special handling for now)
            Statement::Unsafe(block) => {
                self.compile_block(block)?;
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
            Statement::Loop { body } => {
                self.compile_loop(body)?;
            }
            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                self.compile_for_in_loop(variable, iterable, body)?;
            }
            Statement::Switch {
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
                    notes: vec![format!("Statement: {}", stmt_str)],
                    help: Some("This feature is planned for a future release".to_string()),
                    suggestion: None,
                });
                return Err(format!("Statement not yet implemented: {:?}", stmt));
            }
        }
        Ok(())
    }
}
