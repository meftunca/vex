//! Pattern matching: guard logic
use crate::codegen_ast::ASTCodeGen;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, IntValue};
use vex_ast::{Expression, Pattern};

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compiles the guard condition for a match arm.
    /// Returns `Some(final_condition)` if branching should be handled by the caller.
    /// Returns `None` if branching is handled internally (for identifier patterns).
    pub(crate) fn compile_guard_condition(
        &mut self,
        pattern_matches: IntValue<'ctx>,
        guard: &Option<Expression>,
        pattern: &Pattern,
        match_value: BasicValueEnum<'ctx>,
        then_block: BasicBlock<'ctx>,
        else_block: BasicBlock<'ctx>,
    ) -> Result<Option<IntValue<'ctx>>, String> {
        let guard_expr = match guard {
            Some(expr) => expr,
            None => return Ok(Some(pattern_matches)),
        };

        // Special handling for identifier patterns: bind before evaluating the guard.
        if let Pattern::Ident(name) = pattern {
            if !self.is_enum_variant(name) {
                // This is a binding. We need to branch to a temporary block,
                // perform the binding, evaluate the guard, and then branch.
                let guard_check_block = self
                    .context
                    .append_basic_block(self.current_function.unwrap(), "guard_check");
                self.builder
                    .build_conditional_branch(pattern_matches, guard_check_block, else_block)
                    .map_err(|e| format!("Failed to build initial guard branch: {}", e))?;

                self.builder.position_at_end(guard_check_block);
                self.compile_pattern_binding(pattern, match_value)?;

                let guard_val = self.compile_expression(guard_expr)?;
                let guard_bool = guard_val.into_int_value();

                self.builder
                    .build_conditional_branch(guard_bool, then_block, else_block)
                    .map_err(|e| format!("Failed to build guard result branch: {}", e))?;

                // The caller should not add any more branches for this arm.
                return Ok(None);
            }
        }

        // For all other pattern types, the guard is evaluated only if the pattern matches.
        // We can combine the conditions with an AND operation.
        let guard_val = self.compile_expression(guard_expr)?;
        let guard_bool = guard_val.into_int_value();

        let final_condition = self
            .builder
            .build_and(pattern_matches, guard_bool, "match_and_guard")
            .map_err(|e| format!("Failed to build guard AND: {}", e))?;

        Ok(Some(final_condition))
    }
}
