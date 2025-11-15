// statements/loops/switch_statement.rs
// switch statement compilation

use super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile switch statement: switch value { case x: ... default: ... }
    pub(crate) fn compile_switch_statement_dispatch(
        &mut self,
        value: &Option<Expression>,
        cases: &[SwitchCase],
        default_case: &Option<Block>,
    ) -> Result<(), String> {
        self.compile_switch_statement_impl(value, cases, default_case)
    }

    /// Compile switch statement: switch value { case x: ... default: ... }
    fn compile_switch_statement_impl(
        &mut self,
        value: &Option<Expression>,
        cases: &[SwitchCase],
        default_case: &Option<Block>,
    ) -> Result<(), String> {
        // Evaluate the switch value
        let switch_val = if let Some(val_expr) = value {
            self.compile_expression(val_expr)?
        } else {
            return Err("Type switches not yet supported".to_string());
        };

        let switch_int = if let BasicValueEnum::IntValue(iv) = switch_val {
            iv
        } else {
            return Err("Switch value must be an integer".to_string());
        };

        let fn_val = self.current_function.ok_or("No current function")?;

        // Create basic blocks for each case and default
        let mut case_blocks = Vec::new();
        for _ in cases {
            case_blocks.push(self.context.append_basic_block(fn_val, "switch.case"));
        }

        let default_bb = self.context.append_basic_block(fn_val, "switch.default");
        let end_bb = self.context.append_basic_block(fn_val, "switch.end");

        // Build case values for switch instruction
        let mut switch_cases = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            let case_bb = case_blocks[i];

            // Add each pattern as a case
            for pattern in &case.patterns {
                let pattern_val = self.compile_expression(pattern)?;
                if let BasicValueEnum::IntValue(pv) = pattern_val {
                    switch_cases.push((pv, case_bb));
                } else {
                    return Err("Case pattern must be an integer constant".to_string());
                }
            }
        }

        // Build the switch instruction with all cases
        self.builder
            .build_switch(switch_int, default_bb, &switch_cases)
            .map_err(|e| format!("Failed to build switch: {}", e))?;

        // Compile each case body
        for (i, case) in cases.iter().enumerate() {
            self.builder.position_at_end(case_blocks[i]);
            self.compile_block(&case.body)?;

            // Add branch to end if not already terminated
            if self
                .builder
                .get_insert_block()
                .ok_or("No active basic block")?
                .get_terminator()
                .is_none()
            {
                self.builder
                    .build_unconditional_branch(end_bb)
                    .map_err(|e| format!("Failed to build branch: {}", e))?;
            }
        }

        // Compile default case
        self.builder.position_at_end(default_bb);
        if let Some(def_block) = default_case {
            self.compile_block(def_block)?;
        }
        if self
            .builder
            .get_insert_block()
            .ok_or("No active basic block")?
            .get_terminator()
            .is_none()
        {
            self.builder
                .build_unconditional_branch(end_bb)
                .map_err(|e| format!("Failed to build branch: {}", e))?;
        }

        // Always position at end block for subsequent code
        // Even if unreachable, LLVM will optimize it away
        self.builder.position_at_end(end_bb);

        Ok(())
    }
}
