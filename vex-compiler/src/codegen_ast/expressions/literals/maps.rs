use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile map literal: {"key": value, "key2": value2}
    pub(crate) fn compile_map_literal(
        &mut self,
        entries: &[(Expression, Expression)],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Create a new Map
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let capacity = self.context.i64_type().const_int(
            if entries.is_empty() {
                8
            } else {
                entries.len() as u64 * 2
            },
            false,
        );

        let vex_map_create = self.declare_runtime_fn(
            "vex_map_create",
            &[self.context.i64_type().into()],
            ptr_type.into(),
        );

        let map_ptr = self
            .builder
            .build_call(vex_map_create, &[capacity.into()], "map_create")
            .map_err(|e| format!("Failed to create map: {}", e))?
            .try_as_basic_value()
            .left()
            .ok_or("map_create should return a value")?;

        // Insert each entry
        if !entries.is_empty() {
            let vex_map_insert = self.declare_runtime_fn(
                "vex_map_insert",
                &[ptr_type.into(), ptr_type.into(), ptr_type.into()],
                self.context.bool_type().into(),
            );

            for (key_expr, value_expr) in entries {
                let key = self.compile_expression(key_expr)?;
                let value = self.compile_expression(value_expr)?;

                self.builder
                    .build_call(
                        vex_map_insert,
                        &[map_ptr.into(), key.into(), value.into()],
                        "map_insert",
                    )
                    .map_err(|e| format!("Failed to insert map entry: {}", e))?;
            }
        }

        Ok(map_ptr)
    }
}
