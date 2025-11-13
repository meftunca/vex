// Binary operations (arithmetic, comparison, logical)

use super::super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use inkwell::{FloatPredicate, IntPredicate};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile binary operation
    pub(crate) fn compile_binary_op(
        &mut self,
        left: &Expression,
        op: &BinaryOp,
        right: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // ⭐ NEW: Operator Overloading - Check if left operand has operator contract
        if let Ok(left_type) = self.infer_expression_type(left) {
            // ⭐ BUILTIN: Check for Vec + Vec (if both are Vec)
            if let Type::Generic { ref name, .. } = left_type {
                if name == "Vec" && matches!(op, BinaryOp::Add) {
                    if let Ok(right_type) = self.infer_expression_type(right) {
                        if let Type::Generic {
                            name: right_name, ..
                        } = right_type
                        {
                            if right_name == "Vec" {
                                let left_val = self.compile_expression(left)?;
                                let right_val = self.compile_expression(right)?;

                                let concat_fn = self.get_vex_vec_concat();
                                let result = self
                                    .builder
                                    .build_call(
                                        concat_fn,
                                        &[left_val.into(), right_val.into()],
                                        "vec_concat",
                                    )
                                    .map_err(|e| format!("Failed to call vex_vec_concat: {}", e))?;

                                return result
                                    .try_as_basic_value()
                                    .left()
                                    .ok_or("vex_vec_concat didn't return a value".to_string());
                            }
                        }
                    }
                }
            }

            // ⭐ Phase 1 Day 3-4: Builtin contract operator dispatch
            if let Type::Named(ref type_name) = left_type {
                let (contract_name, method_name) = self.binary_op_to_trait(op);
                if !contract_name.is_empty() {
                    // Check if builtin contract exists (e.g., i32 extends Add)
                    use crate::builtin_contracts;
                    if builtin_contracts::has_builtin_contract(type_name, contract_name) {
                        eprintln!("🎯 Builtin operator contract: {}.{}()", type_name, method_name);
                        
                        // Compile operands
                        let left_val = self.compile_expression(left)?;
                        let right_val = self.compile_expression(right)?;
                        
                        // ⭐ NEW: Dispatch to builtin operator codegen (zero overhead LLVM IR)
                        if let Some(result) = builtin_contracts::codegen_builtin_operator(
                            &self.builder,
                            type_name,
                            contract_name,
                            method_name,
                            left_val,
                            right_val,
                        ) {
                            return Ok(result);
                        }
                    }
                    
                    // Otherwise check for user-defined contract implementation
                    if let Some(_) = self.has_operator_trait(type_name, contract_name) {
                        eprintln!("🎯 User operator contract: {}.{}()", type_name, method_name);
                        eprintln!("   Left: {:?}", left);
                        eprintln!("   Right: {:?}", right);
                        return self.compile_method_call(
                            left,
                            method_name,
                            &[], // No generic type args for operator overloading
                            &vec![right.clone()],
                            false,
                        );
                    }
                }
            }
        }

        // Fallback: Builtin operations (int, float, pointer)
        let lhs = self.compile_expression(left)?;
        let rhs = self.compile_expression(right)?;

        // ⚡ CRITICAL: If operands are pointers to structs/enums, load them first
        // Variables for structs/enums return pointers (zero-copy semantics)
        // But binary ops need actual struct values for field comparison
        let lhs = if lhs.is_pointer_value() {
            let ptr = lhs.into_pointer_value();
            // Check if this is a struct/enum variable that needs loading
            if let Expression::Ident(var_name) = left {
                // Check if it's a builtin enum (Option, Result) tracked in variable_struct_names
                if let Some(type_name) = self.variable_struct_names.get(var_name) {
                    if type_name.starts_with("Option") || type_name.starts_with("Result") {
                        // Builtin enum - use stored StructType from variable_types
                        if let Some(var_type) = self.variable_types.get(var_name) {
                            self.builder.build_load(*var_type, ptr, "lhs_enum_loaded")
                                .map_err(|e| format!("Failed to load left enum: {}", e))?
                        } else {
                            lhs.into()
                        }
                    } else if self.struct_defs.contains_key(type_name) {
                        // User-defined struct - build type from definition
                        let struct_def = self.struct_defs.get(type_name).unwrap().clone();
                        let field_types: Vec<_> = struct_def.fields.iter()
                            .map(|(_, ty)| self.ast_type_to_llvm(ty))
                            .collect();
                        let struct_type = self.context.struct_type(&field_types, false);
                        self.builder.build_load(struct_type, ptr, "lhs_struct_loaded")
                            .map_err(|e| format!("Failed to load left struct: {}", e))?
                    } else {
                        lhs.into()
                    }
                }
                // Check user-defined enums
                else if self.variable_enum_names.contains_key(var_name) {
                    if let Some(var_type) = self.variable_types.get(var_name) {
                        self.builder.build_load(*var_type, ptr, "lhs_enum_loaded")
                            .map_err(|e| format!("Failed to load left enum: {}", e))?
                    } else {
                        lhs.into()
                    }
                } 
                else {
                    lhs.into()
                }
            } else {
                lhs.into()
            }
        } else {
            lhs
        };

        let rhs = if rhs.is_pointer_value() {
            let ptr = rhs.into_pointer_value();
            if let Expression::Ident(var_name) = right {
                // Check builtin enum
                if let Some(type_name) = self.variable_struct_names.get(var_name) {
                    if type_name.starts_with("Option") || type_name.starts_with("Result") {
                        if let Some(var_type) = self.variable_types.get(var_name) {
                            self.builder.build_load(*var_type, ptr, "rhs_enum_loaded")
                                .map_err(|e| format!("Failed to load right enum: {}", e))?
                        } else {
                            rhs.into()
                        }
                    } else if self.struct_defs.contains_key(type_name) {
                        let struct_def = self.struct_defs.get(type_name).unwrap().clone();
                        let field_types: Vec<_> = struct_def.fields.iter()
                            .map(|(_, ty)| self.ast_type_to_llvm(ty))
                            .collect();
                        let struct_type = self.context.struct_type(&field_types, false);
                        self.builder.build_load(struct_type, ptr, "rhs_struct_loaded")
                            .map_err(|e| format!("Failed to load right struct: {}", e))?
                    } else {
                        rhs.into()
                    }
                }
                // Check user-defined enums
                else if self.variable_enum_names.contains_key(var_name) {
                    if let Some(var_type) = self.variable_types.get(var_name) {
                        self.builder.build_load(*var_type, ptr, "rhs_enum_loaded")
                            .map_err(|e| format!("Failed to load right enum: {}", e))?
                    } else {
                        rhs.into()
                    }
                } 
                else {
                    rhs.into()
                }
            } else {
                rhs.into()
            }
        } else {
            rhs
        };

        match (lhs, rhs) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                // If operands have different bit widths, extend the smaller one
                let (l_final, r_final) =
                    if l.get_type().get_bit_width() != r.get_type().get_bit_width() {
                        if l.get_type().get_bit_width() < r.get_type().get_bit_width() {
                            // Extend left to match right
                            let l_ext = self
                                .builder
                                .build_int_s_extend(l, r.get_type(), "sext_l")
                                .map_err(|e| format!("Failed to extend operand: {}", e))?;
                            (l_ext, r)
                        } else {
                            // Extend right to match left
                            let r_ext = self
                                .builder
                                .build_int_s_extend(r, l.get_type(), "sext_r")
                                .map_err(|e| format!("Failed to extend operand: {}", e))?;
                            (l, r_ext)
                        }
                    } else {
                        (l, r)
                    };

                let l = l_final;
                let r = r_final;
                let result = match op {
                    BinaryOp::Add => self.builder.build_int_add(l, r, "add"),
                    BinaryOp::Sub => self.builder.build_int_sub(l, r, "sub"),
                    BinaryOp::Mul => self.builder.build_int_mul(l, r, "mul"),
                    BinaryOp::Div => self.builder.build_int_signed_div(l, r, "div"),
                    BinaryOp::Mod => self.builder.build_int_signed_rem(l, r, "mod"),
                    BinaryOp::Pow => {
                        // Integer power: loop-based exponentiation
                        return self.compile_int_power(l, r);
                    }
                    BinaryOp::Eq => {
                        return Ok(self
                            .builder
                            .build_int_compare(IntPredicate::EQ, l, r, "eq")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::NotEq => {
                        return Ok(self
                            .builder
                            .build_int_compare(IntPredicate::NE, l, r, "ne")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::Lt => {
                        return Ok(self
                            .builder
                            .build_int_compare(IntPredicate::SLT, l, r, "lt")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::LtEq => {
                        return Ok(self
                            .builder
                            .build_int_compare(IntPredicate::SLE, l, r, "le")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::Gt => {
                        return Ok(self
                            .builder
                            .build_int_compare(IntPredicate::SGT, l, r, "gt")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::GtEq => {
                        return Ok(self
                            .builder
                            .build_int_compare(IntPredicate::SGE, l, r, "ge")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::And => self.builder.build_and(l, r, "and"),
                    BinaryOp::Or => self.builder.build_or(l, r, "or"),
                    BinaryOp::BitAnd => self.builder.build_and(l, r, "bitand"),
                    BinaryOp::BitOr => self.builder.build_or(l, r, "bitor"),
                    BinaryOp::BitXor => self.builder.build_xor(l, r, "bitxor"),
                    BinaryOp::Shl => self.builder.build_left_shift(l, r, "shl"),
                    BinaryOp::Shr => self.builder.build_right_shift(l, r, true, "shr"),
                    BinaryOp::Range | BinaryOp::RangeInclusive => {
                        return Err("Range operators not yet implemented".to_string());
                    }
                    BinaryOp::NullCoalesce => {
                        return Err("Null coalesce operator not yet implemented".to_string());
                    }
                }
                .map_err(|e| format!("Failed to build operation: {}", e))?;
                Ok(result.into())
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                let result = match op {
                    BinaryOp::Add => self.builder.build_float_add(l, r, "fadd"),
                    BinaryOp::Sub => self.builder.build_float_sub(l, r, "fsub"),
                    BinaryOp::Mul => self.builder.build_float_mul(l, r, "fmul"),
                    BinaryOp::Div => self.builder.build_float_div(l, r, "fdiv"),
                    BinaryOp::Mod => self.builder.build_float_rem(l, r, "fmod"),
                    BinaryOp::Pow => {
                        // Float power: call llvm.pow intrinsic
                        return self.compile_float_power(l, r);
                    }
                    BinaryOp::Range | BinaryOp::RangeInclusive | BinaryOp::NullCoalesce => {
                        return Err("Range/NullCoalesce operators not implemented for floats".to_string());
                    }
                    BinaryOp::Eq => {
                        return Ok(self
                            .builder
                            .build_float_compare(FloatPredicate::OEQ, l, r, "feq")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::NotEq => {
                        return Ok(self
                            .builder
                            .build_float_compare(FloatPredicate::ONE, l, r, "fne")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::Lt => {
                        return Ok(self
                            .builder
                            .build_float_compare(FloatPredicate::OLT, l, r, "flt")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::LtEq => {
                        return Ok(self
                            .builder
                            .build_float_compare(FloatPredicate::OLE, l, r, "fle")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::Gt => {
                        return Ok(self
                            .builder
                            .build_float_compare(FloatPredicate::OGT, l, r, "fgt")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    BinaryOp::GtEq => {
                        return Ok(self
                            .builder
                            .build_float_compare(FloatPredicate::OGE, l, r, "fge")
                            .map_err(|e| format!("Failed to compare: {}", e))?
                            .into())
                    }
                    _ => return Err("Invalid float operation".to_string()),
                }
                .map_err(|e| format!("Failed to build operation: {}", e))?;
                Ok(result.into())
            }
            (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
                match op {
                    // String concatenation: s1 + s2 → vex_strcat_new
                    BinaryOp::Add => {
                        eprintln!("🔗 String concatenation: calling vex_strcat_new");

                        // Declare vex_strcat_new if not already declared
                        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                        let strcat_fn = self.declare_runtime_fn(
                            "vex_strcat_new",
                            &[ptr_type.into(), ptr_type.into()],
                            ptr_type.into(),
                        );

                        // Call vex_strcat_new(left, right) → returns new string
                        let concat_result = self
                            .builder
                            .build_call(strcat_fn, &[l.into(), r.into()], "strcat_result")
                            .map_err(|e| format!("Failed to call vex_strcat_new: {}", e))?;

                        let result_ptr = concat_result
                            .try_as_basic_value()
                            .left()
                            .ok_or("vex_strcat_new didn't return a value")?;

                        Ok(result_ptr)
                    }

                    // String comparison using vex_strcmp
                    BinaryOp::Eq | BinaryOp::NotEq => {
                        // Declare vex_strcmp if not already declared
                        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                        let strcmp_fn = self.declare_runtime_fn(
                            "vex_strcmp",
                            &[ptr_type.into(), ptr_type.into()],
                            self.context.i32_type().into(),
                        );

                        // Call vex_strcmp(left, right)
                        let cmp_result = self
                            .builder
                            .build_call(strcmp_fn, &[l.into(), r.into()], "strcmp_result")
                            .map_err(|e| format!("Failed to call vex_strcmp: {}", e))?;

                        let cmp_value = cmp_result
                            .try_as_basic_value()
                            .left()
                            .ok_or("vex_strcmp didn't return a value")?
                            .into_int_value();

                        // vex_strcmp returns 0 if equal
                        let zero = self.context.i32_type().const_int(0, false);
                        let result = if matches!(op, BinaryOp::Eq) {
                            self.builder.build_int_compare(
                                IntPredicate::EQ,
                                cmp_value,
                                zero,
                                "streq",
                            )
                        } else {
                            self.builder.build_int_compare(
                                IntPredicate::NE,
                                cmp_value,
                                zero,
                                "strne",
                            )
                        }
                        .map_err(|e| format!("Failed to compare strcmp result: {}", e))?;

                        Ok(result.into())
                    }
                    _ => Err("Only == and != are supported for string comparison".to_string()),
                }
            }
            
            // Struct comparison (field-by-field equality for == and !=)
            (BasicValueEnum::StructValue(l), BasicValueEnum::StructValue(r)) => {
                match op {
                    BinaryOp::Eq | BinaryOp::NotEq => {
                        // Get struct type
                        let struct_type = l.get_type();
                        
                        // Check if this is an enum (has tag field)
                        // Enums are { i32 tag, T data }, so check field count and first field type
                        let field_count = struct_type.count_fields();
                        if field_count >= 2 {
                            if let Some(first_field) = struct_type.get_field_type_at_index(0) {
                                if first_field.is_int_type() {
                                    // This looks like an enum - compare tag first
                                    let l_tag = self.builder
                                        .build_extract_value(l, 0, "l_tag")
                                        .map_err(|e| format!("Failed to extract left tag: {}", e))?
                                        .into_int_value();
                                    let r_tag = self.builder
                                        .build_extract_value(r, 0, "r_tag")
                                        .map_err(|e| format!("Failed to extract right tag: {}", e))?
                                        .into_int_value();
                                    
                                    let tags_equal = self.builder
                                        .build_int_compare(IntPredicate::EQ, l_tag, r_tag, "tags_eq")
                                        .map_err(|e| format!("Failed to compare tags: {}", e))?;
                                    
                                    // If tags are different, enums are not equal
                                    // If tags are same, also compare data field (index 1)
                                    let l_data = self.builder
                                        .build_extract_value(l, 1, "l_data")
                                        .map_err(|e| format!("Failed to extract left data: {}", e))?;
                                    let r_data = self.builder
                                        .build_extract_value(r, 1, "r_data")
                                        .map_err(|e| format!("Failed to extract right data: {}", e))?;
                                    
                                    // Compare data fields based on type
                                    let data_equal = match (l_data, r_data) {
                                        (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => {
                                            self.builder
                                                .build_int_compare(IntPredicate::EQ, li, ri, "data_eq")
                                                .map_err(|e| format!("Failed to compare enum data: {}", e))?
                                        }
                                        (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => {
                                            self.builder
                                                .build_float_compare(FloatPredicate::OEQ, lf, rf, "data_eq")
                                                .map_err(|e| format!("Failed to compare enum data: {}", e))?
                                        }
                                        (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) => {
                                            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                                            let strcmp_fn = self.declare_runtime_fn(
                                                "vex_strcmp",
                                                &[ptr_type.into(), ptr_type.into()],
                                                self.context.i32_type().into(),
                                            );
                                            
                                            let cmp_result = self.builder
                                                .build_call(strcmp_fn, &[lp.into(), rp.into()], "strcmp_result")
                                                .map_err(|e| format!("Failed to call vex_strcmp: {}", e))?;
                                            
                                            let cmp_value = cmp_result
                                                .try_as_basic_value()
                                                .left()
                                                .ok_or("vex_strcmp didn't return a value")?
                                                .into_int_value();
                                            
                                            let zero = self.context.i32_type().const_int(0, false);
                                            self.builder
                                                .build_int_compare(IntPredicate::EQ, cmp_value, zero, "data_eq")
                                                .map_err(|e| format!("Failed to compare string data: {}", e))?
                                        }
                                        (BasicValueEnum::StructValue(ls), BasicValueEnum::StructValue(rs)) => {
                                            // Nested struct comparison (for multi-field enum data)
                                            // Recursively compare all fields
                                            let struct_type = ls.get_type();
                                            let field_count = struct_type.count_fields();
                                            let mut all_equal = self.context.bool_type().const_int(1, false);
                                            
                                            for i in 0..field_count {
                                                let lf = self.builder
                                                    .build_extract_value(ls, i, &format!("ls_f{}", i))
                                                    .map_err(|e| format!("Failed to extract: {}", e))?;
                                                let rf = self.builder
                                                    .build_extract_value(rs, i, &format!("rs_f{}", i))
                                                    .map_err(|e| format!("Failed to extract: {}", e))?;
                                                
                                                let field_eq = match (lf, rf) {
                                                    (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => {
                                                        self.builder.build_int_compare(IntPredicate::EQ, li, ri, "feq")
                                                            .map_err(|e| format!("Failed to compare: {}", e))?
                                                    }
                                                    (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => {
                                                        self.builder.build_float_compare(FloatPredicate::OEQ, lf, rf, "feq")
                                                            .map_err(|e| format!("Failed to compare: {}", e))?
                                                    }
                                                    _ => self.context.bool_type().const_int(1, false),
                                                };
                                                
                                                all_equal = self.builder.build_and(all_equal, field_eq, "and")
                                                    .map_err(|e| format!("Failed to AND: {}", e))?;
                                            }
                                            
                                            all_equal
                                        }
                                        _ => {
                                            // For other types, assume equal if tags are equal
                                            // This handles None case where data is just zero/undef
                                            self.context.bool_type().const_int(1, false)
                                        }
                                    };
                                    
                                    // Both tag and data must be equal
                                    let both_equal = self.builder
                                        .build_and(tags_equal, data_equal, "enum_eq")
                                        .map_err(|e| format!("Failed to AND tag and data: {}", e))?;
                                    
                                    let result = if matches!(op, BinaryOp::Eq) {
                                        both_equal
                                    } else {
                                        self.builder
                                            .build_not(both_equal, "enum_neq")
                                            .map_err(|e| format!("Failed to negate: {}", e))?
                                    };
                                    
                                    return Ok(result.into());
                                }
                            }
                        }
                        
                        // Regular struct - compare all fields
                        let mut all_equal = self.context.bool_type().const_int(1, false); // Start with true
                        
                        for i in 0..field_count {
                            let l_field = self.builder
                                .build_extract_value(l, i, &format!("l_field_{}", i))
                                .map_err(|e| format!("Failed to extract left field {}: {}", i, e))?;
                            let r_field = self.builder
                                .build_extract_value(r, i, &format!("r_field_{}", i))
                                .map_err(|e| format!("Failed to extract right field {}: {}", i, e))?;
                            
                            // Compare fields based on type
                            let field_eq = match (l_field, r_field) {
                                (BasicValueEnum::IntValue(li), BasicValueEnum::IntValue(ri)) => {
                                    self.builder
                                        .build_int_compare(IntPredicate::EQ, li, ri, &format!("field_{}_eq", i))
                                        .map_err(|e| format!("Failed to compare int fields: {}", e))?
                                }
                                (BasicValueEnum::FloatValue(lf), BasicValueEnum::FloatValue(rf)) => {
                                    self.builder
                                        .build_float_compare(FloatPredicate::OEQ, lf, rf, &format!("field_{}_eq", i))
                                        .map_err(|e| format!("Failed to compare float fields: {}", e))?
                                }
                                (BasicValueEnum::PointerValue(lp), BasicValueEnum::PointerValue(rp)) => {
                                    // For pointers (strings), call vex_strcmp
                                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                                    let strcmp_fn = self.declare_runtime_fn(
                                        "vex_strcmp",
                                        &[ptr_type.into(), ptr_type.into()],
                                        self.context.i32_type().into(),
                                    );
                                    
                                    let cmp_result = self.builder
                                        .build_call(strcmp_fn, &[lp.into(), rp.into()], "strcmp_result")
                                        .map_err(|e| format!("Failed to call vex_strcmp: {}", e))?;
                                    
                                    let cmp_value = cmp_result
                                        .try_as_basic_value()
                                        .left()
                                        .ok_or("vex_strcmp didn't return a value")?
                                        .into_int_value();
                                    
                                    let zero = self.context.i32_type().const_int(0, false);
                                    self.builder
                                        .build_int_compare(IntPredicate::EQ, cmp_value, zero, &format!("field_{}_eq", i))
                                        .map_err(|e| format!("Failed to compare string fields: {}", e))?
                                }
                                _ => {
                                    // For other types, assume not equal (TODO: recursive struct comparison)
                                    return Err(format!("Cannot compare struct fields of type: {:?}", l_field.get_type()));
                                }
                            };
                            
                            // AND with accumulated result
                            all_equal = self.builder
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
                    _ => Err("Only == and != are supported for struct comparison".to_string()),
                }
            }
            
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }

    /// Compile integer power: base ** exp using loop
    fn compile_int_power(
        &mut self,
        base: inkwell::values::IntValue<'ctx>,
        exp: inkwell::values::IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Fast path for constant exponents
        if let Some(exp_const) = exp.get_zero_extended_constant() {
            if exp_const == 0 {
                return Ok(self.context.i64_type().const_int(1, false).into());
            }
            if exp_const == 1 {
                return Ok(base.into());
            }
        }

        // result = 1
        let result_alloca = self.builder
            .build_alloca(base.get_type(), "pow_result")
            .map_err(|e| format!("Failed to allocate power result: {}", e))?;
        let one = base.get_type().const_int(1, false);
        self.builder
            .build_store(result_alloca, one)
            .map_err(|e| format!("Failed to store initial result: {}", e))?;

        // counter = exp
        let counter_alloca = self.builder
            .build_alloca(exp.get_type(), "pow_counter")
            .map_err(|e| format!("Failed to allocate counter: {}", e))?;
        self.builder
            .build_store(counter_alloca, exp)
            .map_err(|e| format!("Failed to store counter: {}", e))?;

        // Loop: while counter > 0
        let parent_fn = self.builder.get_insert_block().unwrap().get_parent().unwrap();
        let loop_block = self.context.append_basic_block(parent_fn, "pow_loop");
        let after_block = self.context.append_basic_block(parent_fn, "pow_after");

        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| format!("Failed to branch to loop: {}", e))?;
        self.builder.position_at_end(loop_block);

        // Load counter
        let counter = self.builder
            .build_load(exp.get_type(), counter_alloca, "counter")
            .map_err(|e| format!("Failed to load counter: {}", e))?
            .into_int_value();

        // Check if counter > 0
        let zero = exp.get_type().const_int(0, false);
        let cond = self.builder
            .build_int_compare(inkwell::IntPredicate::SGT, counter, zero, "pow_cond")
            .map_err(|e| format!("Failed to compare: {}", e))?;

        let loop_body = self.context.append_basic_block(parent_fn, "pow_body");
        self.builder
            .build_conditional_branch(cond, loop_body, after_block)
            .map_err(|e| format!("Failed to build conditional branch: {}", e))?;

        // Loop body: result *= base
        self.builder.position_at_end(loop_body);
        let current_result = self.builder
            .build_load(base.get_type(), result_alloca, "current_result")
            .map_err(|e| format!("Failed to load result: {}", e))?
            .into_int_value();
        let new_result = self.builder
            .build_int_mul(current_result, base, "new_result")
            .map_err(|e| format!("Failed to multiply: {}", e))?;
        self.builder
            .build_store(result_alloca, new_result)
            .map_err(|e| format!("Failed to store result: {}", e))?;

        // counter -= 1
        let new_counter = self.builder
            .build_int_sub(counter, one, "new_counter")
            .map_err(|e| format!("Failed to decrement: {}", e))?;
        self.builder
            .build_store(counter_alloca, new_counter)
            .map_err(|e| format!("Failed to store counter: {}", e))?;

        self.builder
            .build_unconditional_branch(loop_block)
            .map_err(|e| format!("Failed to branch back: {}", e))?;

        // After loop
        self.builder.position_at_end(after_block);
        let final_result = self.builder
            .build_load(base.get_type(), result_alloca, "final_result")
            .map_err(|e| format!("Failed to load final result: {}", e))?;

        Ok(final_result)
    }

    /// Compile float power: base ** exp using llvm.pow intrinsic
    fn compile_float_power(
        &mut self,
        base: inkwell::values::FloatValue<'ctx>,
        exp: inkwell::values::FloatValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Declare llvm.pow.f64 intrinsic
        let pow_intrinsic = self.module.get_function("llvm.pow.f64").unwrap_or_else(|| {
            let f64_type = self.context.f64_type();
            let fn_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
            self.module.add_function("llvm.pow.f64", fn_type, None)
        });

        let result = self.builder
            .build_call(pow_intrinsic, &[base.into(), exp.into()], "pow_result")
            .map_err(|e| format!("Failed to call pow intrinsic: {}", e))?
            .try_as_basic_value()
            .left()
            .ok_or("pow intrinsic should return a value")?;


        Ok(result)
    }
}
