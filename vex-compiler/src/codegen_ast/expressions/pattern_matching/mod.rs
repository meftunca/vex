// Match expression and pattern matching code generation

mod bindings;
mod guards;
mod patterns;

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, PointerValue};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile match expression as a series of if-else comparisons
    pub(crate) fn compile_match_expression(
        &mut self,
        value: &Expression,
        arms: &[MatchArm],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if arms.is_empty() {
            return Err("Match expression must have at least one arm".to_string());
        }

        // Compile the value to match against
        let mut match_value = self.compile_expression(value)?;

        // Special handling for composite types to ensure they are loaded values, not pointers
        match_value = self.load_if_composite_pointer(value, match_value)?;

        // Store match value on stack to avoid consumption issues
        let match_value_ptr = self.build_alloca_and_store(match_value, "match_value_storage")?;

        // Create the merge block where all arms converge
        let merge_block = self
            .context
            .append_basic_block(self.current_function.unwrap(), "match_merge");

        let mut result_ptr: Option<PointerValue> = None;
        let mut result_type: Option<inkwell::types::BasicTypeEnum> = None;

        let mut current_block = self.builder.get_insert_block().unwrap();

        for (i, arm) in arms.iter().enumerate() {
            self.builder.position_at_end(current_block);

            // Load fresh copy of match value for this arm
            let arm_match_value = self.builder.build_load(match_value.get_type(), match_value_ptr, "match_value_load")
                .map_err(|e| format!("Failed to load match value for arm: {}", e))?;

            let is_last_arm = i == arms.len() - 1;

            let then_block = self.context.append_basic_block(self.current_function.unwrap(), &format!("match_arm_{}", i));
            let else_block = if is_last_arm { merge_block } else { self.context.append_basic_block(self.current_function.unwrap(), &format!("match_check_{}", i + 1)) };

            // Handle catch-all patterns without guards separately for efficiency
            if self.is_catch_all_pattern(&arm.pattern) && is_last_arm && arm.guard.is_none() {
                self.builder.build_unconditional_branch(then_block)
                    .map_err(|e| format!("Failed to branch to match arm: {}", e))?;
                self.builder.position_at_end(then_block);
                self.compile_pattern_binding(&arm.pattern, arm_match_value)?;
            } else {
                // For all other patterns, do conditional check
                let matches = self.compile_pattern_check(&arm.pattern, arm_match_value)?;

                // Handle guard condition
                let final_condition = self.compile_guard_condition(matches, &arm.guard, &arm.pattern, arm_match_value, then_block, else_block)?;

                // If compile_guard_condition returns None, it handled the branching itself.
                if let Some(condition) = final_condition {
                     self.builder.build_conditional_branch(condition, then_block, else_block)
                        .map_err(|e| format!("Failed to build match branch: {}", e))?;
                } else {
                    // Branching was handled inside guard compilation for identifier patterns
                    // The body of the arm is also compiled there, so we can continue to the next arm.
                    current_block = else_block;
                    continue;
                }

                self.builder.position_at_end(then_block);
                self.compile_pattern_binding(&arm.pattern, arm_match_value)?;
            }

            let arm_result = self.compile_expression(&arm.body)?;

            // Only store result and branch if block is not terminated by return/break etc.
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                if result_ptr.is_none() {
                    let inferred_type = arm_result.get_type();
                    result_type = Some(inferred_type);
                    result_ptr = Some(self.create_entry_block_alloca_for_type(inferred_type, "match_result")?);
                }

                self.builder.build_store(result_ptr.unwrap(), arm_result)
                    .map_err(|e| format!("Failed to store match result: {}", e))?;
                self.builder.build_unconditional_branch(merge_block)
                    .map_err(|e| format!("Failed to branch to merge: {}", e))?;
            }

            current_block = else_block;
            self.builder.position_at_end(else_block);
        }

        self.builder.position_at_end(merge_block);

        // If all arms did early return, result_ptr will be None.
        if let (Some(res_type), Some(res_ptr)) = (result_type, result_ptr) {
            self.builder.build_load(res_type, res_ptr, "match_result")
                .map_err(|e| format!("Failed to load match result: {}", e))
        } else {
            // All arms returned, so this block should be unreachable.
            // We can build_unreachable or return a dummy value.
            Ok(self.context.i32_type().const_int(0, false).into())
        }
    }

    /// Loads a value if it's a pointer to a composite type (struct, tuple, enum).
    pub(crate) fn load_if_composite_pointer(&mut self, expr: &Expression, value: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>, String> {
        if !value.is_pointer_value() {
            return Ok(value);
        }

        let ptr = value.into_pointer_value();
        let loaded_type = match expr {
            Expression::Ident(name) => self.variable_types.get(name).cloned(),
            Expression::StructLiteral { name, .. } => {
                let struct_def = self.struct_defs.get(name).ok_or_else(|| format!("Struct '{}' not found", name))?.clone();
                let field_types: Vec<_> = struct_def.fields.iter().map(|(_, ty)| self.ast_type_to_llvm(ty)).collect();
                Some(self.context.struct_type(&field_types, false).into())
            }
            Expression::TupleLiteral(elements) => {
                let mut element_types = Vec::new();
                for elem_expr in elements.iter() {
                    let elem_val = self.compile_expression(elem_expr)?;
                    element_types.push(elem_val.get_type());
                }
                Some(self.context.struct_type(&element_types, false).into())
            }
            _ => None,
        };

        if let Some(typ) = loaded_type {
            self.builder.build_load(typ, ptr, "loaded_composite").map_err(|e| e.to_string())
        } else {
            Ok(value)
        }
    }

    /// Checks if a pattern is a catch-all (`_` or a simple identifier not representing an enum variant).
    fn is_catch_all_pattern(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Wildcard => true,
            Pattern::Ident(name) => !self.is_enum_variant(name),
            _ => false,
        }
    }

    /// Checks if an identifier name corresponds to a known enum variant.
    pub(crate) fn is_enum_variant(&self, name: &str) -> bool {
        self.enum_ast_defs.values().any(|e| e.variants.iter().any(|v| v.name == name))
    }

    fn build_alloca_and_store(
        &mut self,
        value: BasicValueEnum<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, String> {
        let ptr = self
            .builder
            .build_alloca(value.get_type(), name)
            .map_err(|e| format!("Failed to allocate '{}': {}", name, e))?;
        self.builder
            .build_store(ptr, value)
            .map_err(|e| format!("Failed to store '{}': {}", name, e))?;
        Ok(ptr)
    }

    fn create_entry_block_alloca_for_type(
        &mut self,
        ty: BasicTypeEnum<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, String> {
        let function = self.current_function.ok_or_else(|| "No current function".to_string())?;
        let entry = function
            .get_first_basic_block()
            .ok_or_else(|| "Function has no entry block".to_string())?;

        let builder = self.context.create_builder();
        if let Some(first_instr) = entry.get_first_instruction() {
            builder.position_before(&first_instr);
        } else {
            builder.position_at_end(entry);
        }

        builder
            .build_alloca(ty, name)
            .map_err(|e| format!("Failed to create entry alloca for '{}': {}", name, e))
    }
}
