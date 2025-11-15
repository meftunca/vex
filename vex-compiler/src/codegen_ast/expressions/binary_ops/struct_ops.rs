//! Struct binary operations
//!
//! Handles field-by-field comparison for struct equality operations

use super::super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::{FloatPredicate, IntPredicate};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operations for struct operands (field-by-field comparison)
    pub(crate) fn compile_struct_binary_op(
        &mut self,
        l: inkwell::values::StructValue<'ctx>,
        r: inkwell::values::StructValue<'ctx>,
        op: &BinaryOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match op {
            BinaryOp::Eq | BinaryOp::NotEq => {
                // Get struct type
                let struct_type = l.get_type();
                let field_count = struct_type.count_fields();

                // Check if this is an enum (has tag field)
                // Enums are { i32 tag, T data }, so check field count and first field type
                if field_count >= 2 {
                    if let Some(first_field) = struct_type.get_field_type_at_index(0) {
                        if first_field.is_int_type() {
                            // This looks like an enum - delegate to enum operations
                            return self.compile_enum_binary_op(l, r, op);
                        }
                    }
                }

                // Regular struct - compare all fields
                let mut all_equal = self.context.bool_type().const_int(1, false); // Start with true

                for i in 0..field_count {
                    let l_field = self
                        .builder
                        .build_extract_value(l, i, &format!("l_field_{}", i))
                        .map_err(|e| format!("Failed to extract left field {}: {}", i, e))?;
                    let r_field = self
                        .builder
                        .build_extract_value(r, i, &format!("r_field_{}", i))
                        .map_err(|e| format!("Failed to extract right field {}: {}", i, e))?;

                    // Compare fields based on type
                    let field_eq = match (l_field, r_field) {
                        (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => self
                            .builder
                            .build_int_compare(IntPredicate::EQ, li, ri, &format!("field_{}_eq", i))
                            .map_err(|e| format!("Failed to compare int fields: {}", e))?,
                        (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => self
                            .builder
                            .build_float_compare(
                                FloatPredicate::OEQ,
                                lf,
                                rf,
                                &format!("field_{}_eq", i),
                            )
                            .map_err(|e| format!("Failed to compare float fields: {}", e))?,
                        (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) => {
                            // For pointers (strings), call vex_strcmp
                            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                            let strcmp_fn = self.declare_runtime_fn(
                                "vex_strcmp",
                                &[ptr_type.into(), ptr_type.into()],
                                self.context.i32_type().into(),
                            );

                            let cmp_result = self
                                .builder
                                .build_call(strcmp_fn, &[lp.into(), rp.into()], "strcmp_result")
                                .map_err(|e| format!("Failed to call vex_strcmp: {}", e))?;

                            let cmp_value = cmp_result
                                .try_as_basic_value()
                                .unwrap_basic()
                                .into_int_value();

                            let zero = self.context.i32_type().const_int(0, false);
                            self.builder
                                .build_int_compare(
                                    IntPredicate::EQ,
                                    cmp_value,
                                    zero,
                                    &format!("field_{}_eq", i),
                                )
                                .map_err(|e| format!("Failed to compare string fields: {}", e))?
                        }
                        (BasicValueEnum::StructValue(ls), BasicValueEnum::StructValue(rs)) => {
                            // Recursive struct comparison
                            let nested_result =
                                self.compile_struct_binary_op(ls, rs, &BinaryOp::Eq)?;
                            nested_result.into_int_value()
                        }
                        _ => {
                            // For other unsupported types, return error
                            return Err(format!(
                                "Cannot compare struct fields of type: {:?}",
                                l_field.get_type()
                            ));
                        }
                    };

                    // AND with accumulated result
                    all_equal = self
                        .builder
                        .build_and(all_equal, field_eq, &format!("and_{}", i))
                        .map_err(|e| format!("Failed to AND field comparisons: {}", e))?;
                }

                // Return final result (negate for !=)
                let result = if matches!(op, BinaryOp::Eq) {
                    all_equal
                } else {
                    self.builder
                        .build_not(all_equal, "struct_neq")
                        .map_err(|e| format!("Failed to negate: {}", e))?
                };

                Ok(result.into())
            }
            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                let op_symbol = match op {
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::LtEq => "<=",
                    BinaryOp::GtEq => ">=",
                    _ => unreachable!(),
                };
                Err(format!(
                    "Comparison operator '{}' requires Ord contract implementation. \
                     Add `impl Ord {{ op{} ... }}` to your struct type.",
                    op_symbol, op_symbol
                ))
            }
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                let (op_symbol, contract) = match op {
                    BinaryOp::Add => ("+", "Add"),
                    BinaryOp::Sub => ("-", "Sub"),
                    BinaryOp::Mul => ("*", "Mul"),
                    BinaryOp::Div => ("/", "Div"),
                    BinaryOp::Mod => ("%", "Mod"),
                    _ => unreachable!(),
                };
                Err(format!(
                    "Arithmetic operator '{}' requires {} contract implementation. \
                     Add `impl {} {{ op{} ... }}` to your struct type.",
                    op_symbol, contract, contract, op_symbol
                ))
            }
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor | BinaryOp::Shl | BinaryOp::Shr => {
                let op_symbol = match op {
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::Shl => "<<",
                    BinaryOp::Shr => ">>",
                    _ => unreachable!(),
                };
                Err(format!(
                    "Bitwise operator '{}' not supported for struct field comparison. \
                     Implement the operator contract explicitly.",
                    op_symbol
                ))
            }
            _ => Err(format!("Operator '{:?}' is not supported for struct types", op)),
        }
    }
}
