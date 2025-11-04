// Control flow expressions (try, await, etc.)

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile try expression (? operator): Unwrap Result, propagate Err
    /// Desugar: let x = expr? => match expr { Ok(v) => v, Err(e) => return Err(e) }
    pub(crate) fn compile_try_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Compile the Result expression
        let result_val = self.compile_expression(expr)?;

        // Check if this is a Result/Option enum (has tag + data struct)
        if !result_val.is_struct_value() {
            return Err("? operator can only be used on Result/Option enums".to_string());
        }

        // Result is a struct value, but we need to work with it on stack
        // Allocate temporary space and store it
        let result_ptr = self
            .builder
            .build_alloca(result_val.get_type(), "result_tmp")
            .map_err(|e| format!("Failed to allocate result temp: {}", e))?;

        self.builder
            .build_store(result_ptr, result_val)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // Extract tag (field 0)
        let tag_ptr = self
            .builder
            .build_struct_gep(result_val.get_type(), result_ptr, 0, "tag_ptr")
            .map_err(|e| format!("Failed to get tag pointer: {}", e))?;

        let tag = self
            .builder
            .build_load(self.context.i32_type(), tag_ptr, "tag")
            .map_err(|e| format!("Failed to load tag: {}", e))?
            .into_int_value();

        // Extract data (field 1)
        let data_ptr = self
            .builder
            .build_struct_gep(result_val.get_type(), result_ptr, 1, "data_ptr")
            .map_err(|e| format!("Failed to get data pointer: {}", e))?;

        // Create blocks for Ok and Err paths
        let ok_block = self.context.append_basic_block(
            self.current_function.ok_or("? operator outside function")?,
            "try_ok",
        );
        let err_block = self.context.append_basic_block(
            self.current_function.ok_or("? operator outside function")?,
            "try_err",
        );
        let merge_block = self.context.append_basic_block(
            self.current_function.ok_or("? operator outside function")?,
            "try_merge",
        );

        // Check if tag == 0 (Ok variant)
        let is_ok = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::EQ,
                tag,
                self.context.i32_type().const_int(0, false),
                "is_ok",
            )
            .map_err(|e| format!("Failed to compare tag: {}", e))?;

        // Branch: if Ok goto ok_block, else goto err_block
        self.builder
            .build_conditional_branch(is_ok, ok_block, err_block)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

        // Ok block: unwrap data and continue
        self.builder.position_at_end(ok_block);
        let data_type = self.context.i32_type(); // TODO: Infer from Result<T, E>
        let ok_value = self
            .builder
            .build_load(data_type, data_ptr, "ok_value")
            .map_err(|e| format!("Failed to load ok value: {}", e))?;
        self.builder
            .build_unconditional_branch(merge_block)
            .map_err(|e| format!("Failed to branch to merge: {}", e))?;

        // Err block: early return with Err
        self.builder.position_at_end(err_block);

        // Execute deferred statements before early return
        self.execute_deferred_statements()?;

        // Return the error Result value
        self.builder
            .build_return(Some(&result_val))
            .map_err(|e| format!("Failed to build error return: {}", e))?;

        // Merge block: continue with unwrapped value
        self.builder.position_at_end(merge_block);

        Ok(ok_value)
    }
}


