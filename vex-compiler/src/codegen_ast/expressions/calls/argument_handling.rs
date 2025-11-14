// Method argument compilation and loading

use crate::codegen_ast::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile method arguments and handle receiver
    pub(crate) fn compile_method_arguments(
        &mut self,
        method_name: &str,
        _receiver: &Expression,
        receiver_val: inkwell::values::PointerValue<'ctx>,
        args: &[Expression],
    ) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, String> {
        // Handle receiver argument
        let receiver_arg = self.compile_receiver_argument(method_name, receiver_val)?;

        // Compile other arguments
        let mut arg_vals: Vec<BasicMetadataValueEnum<'ctx>> = vec![receiver_arg];
        eprintln!(
            "📝 Receiver arg added to arg_vals, receiver_arg={:?}",
            receiver_arg
        );

        for (arg_idx, arg) in args.iter().enumerate() {
            eprintln!("📝 Compiling arg {}: {:?}", arg_idx, arg);
            let mut val = self.compile_expression(arg)?;
            eprintln!("📝 Arg {} compiled: {:?}", arg_idx, val);

            if let Some(expected_ty) = self
                .function_defs
                .get(method_name)
                .and_then(|func_def| func_def.params.get(arg_idx))
                .map(|param| param.ty.clone())
            {
                let source_ty = self.infer_expression_type(arg)?;
                val = self.align_method_arg_numeric_type(val, &source_ty, &expected_ty)?;
            }

            // Handle struct arguments (pass by value)
            let processed_val = self.process_method_argument(method_name, arg_idx, val)?;
            arg_vals.push(processed_val);
        }

        Ok(arg_vals)
    }

    fn align_method_arg_numeric_type(
        &mut self,
        value: BasicValueEnum<'ctx>,
        source_type: &Type,
        expected_type: &Type,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        if !(self.is_integer_type(source_type) && self.is_integer_type(expected_type)) {
            return Ok(value);
        }

        let BasicValueEnum::IntValue(int_val) = value else {
            return Ok(value);
        };

        let target_llvm = self.ast_type_to_llvm(expected_type);
        let BasicTypeEnum::IntType(target_int_type) = target_llvm else {
            return Ok(int_val.into());
        };

        let src_bits = int_val.get_type().get_bit_width();
        let dst_bits = target_int_type.get_bit_width();
        if src_bits == dst_bits {
            return Ok(int_val.into());
        }

        let source_is_unsigned = self.is_unsigned_integer_type(source_type);
        if src_bits < dst_bits {
            let casted = if source_is_unsigned {
                self.builder
                    .build_int_z_extend(int_val, target_int_type, "method_arg_zext")
                    .map_err(|e| format!("Failed to zero-extend method arg: {}", e))?
            } else {
                self.builder
                    .build_int_s_extend(int_val, target_int_type, "method_arg_sext")
                    .map_err(|e| format!("Failed to sign-extend method arg: {}", e))?
            };
            Ok(casted.into())
        } else {
            let truncated = self
                .builder
                .build_int_truncate(int_val, target_int_type, "method_arg_trunc")
                .map_err(|e| format!("Failed to truncate method arg: {}", e))?;
            Ok(truncated.into())
        }
    }

    fn is_integer_type(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::I128
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::U128
                | Type::Bool
        )
    }

    fn is_unsigned_integer_type(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 | Type::Bool
        )
    }

    /// Compile receiver argument with proper loading for external methods
    fn compile_receiver_argument(
        &mut self,
        method_name: &str,
        receiver_val: inkwell::values::PointerValue<'ctx>,
    ) -> Result<BasicMetadataValueEnum<'ctx>, String> {
        // ⚠️ CRITICAL: For external methods, the receiver is also passed BY VALUE
        // receiver_val is already a PointerValue, so we need to load it for struct types
        if let Some(func_def) = self.function_defs.get(method_name) {
            if let Some(receiver_param) = &func_def.receiver {
                // Check if receiver is a reference type (&Vec<T>, &Vec<T>!)
                // References are passed as pointers; non-references are loaded and passed by value
                let is_reference = matches!(receiver_param.ty, Type::Reference(_, _));

                if is_reference {
                    // Reference receiver: pass pointer directly
                    eprintln!("   → Reference receiver: passing pointer directly");
                    Ok(receiver_val.into())
                } else {
                    // Non-reference receiver: check if it's a struct that needs loading
                    let is_struct_receiver = match &receiver_param.ty {
                        Type::Named(type_name) => self.struct_defs.contains_key(type_name),
                        Type::Vec(_) | Type::Box(_) | Type::Option(_) | Type::Result(_, _) => true,
                        Type::Generic { .. } => true,
                        _ => false,
                    };

                    if is_struct_receiver {
                        // Load the struct value from pointer
                        eprintln!("   ⚠️ Loading receiver struct value from pointer");
                        let struct_llvm_type = self.ast_type_to_llvm(&receiver_param.ty);
                        let loaded_val = self
                            .builder
                            .build_load(struct_llvm_type, receiver_val, "receiver_load")
                            .map_err(|e| format!("Failed to load receiver: {}", e))?;
                        Ok(loaded_val.into())
                    } else {
                        Ok(receiver_val.into())
                    }
                }
            } else {
                Ok(receiver_val.into())
            }
        } else {
            Ok(receiver_val.into())
        }
    }

    /// Process method argument, handling struct loading
    fn process_method_argument(
        &mut self,
        method_name: &str,
        arg_idx: usize,
        val: BasicValueEnum<'ctx>,
    ) -> Result<BasicMetadataValueEnum<'ctx>, String> {
        // ⚠️ NEW: Struct arguments are now passed BY VALUE
        // If we have a pointer (from variable), we need to LOAD the value
        // If we already have a struct value (from function return), use it directly
        if let Some(func_def) = self.function_defs.get(method_name) {
            // Account for receiver as first param
            if arg_idx < func_def.params.len() {
                let param_ty = &func_def.params[arg_idx].ty;
                let is_struct_param = match param_ty {
                    Type::Named(type_name) => self.struct_defs.contains_key(type_name),
                    _ => false,
                };

                if is_struct_param {
                    eprintln!(
                        "🔧 Method arg {}: is_struct=true, val.is_pointer={}, val.is_struct={}",
                        arg_idx,
                        val.is_pointer_value(),
                        val.is_struct_value()
                    );

                    if val.is_pointer_value() {
                        // We have a POINTER but need a VALUE - load it
                        eprintln!("   ⚠️ Loading struct value from pointer for method arg");
                        let ptr_val = val.into_pointer_value();
                        let struct_llvm_type = self.ast_type_to_llvm(param_ty);
                        let loaded_val = self
                            .builder
                            .build_load(struct_llvm_type, ptr_val, "arg_load")
                            .map_err(|e| format!("Failed to load struct arg: {}", e))?;
                        return Ok(loaded_val.into());
                    }
                    // else: already a struct value, fall through
                }
            }
        }

        Ok(val.into())
    }
}
