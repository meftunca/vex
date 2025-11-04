// Defer statement handling

use super::ASTCodeGen;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile a defer statement
    /// Defer statements are added to a stack and executed in reverse order
    /// when the function exits or returns
    pub(crate) fn compile_defer_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        // Add statement to defer stack (LIFO)
        // Don't execute now, execute on function exit
        self.deferred_statements.push(stmt.clone());
        Ok(())
    }
}

