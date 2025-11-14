// Method argument compilation and loading

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile method arguments and handle receiver
    pub(crate) fn compile_method_arguments(
        &mut self,
        method_name: &str,
        receiver: &Expression,
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
            let val = self.compile_expression(arg)?;
            eprintln!("📝 Arg {} compiled: {:?}", arg_idx, val);

            // Handle struct arguments (pass by value)
            let processed_val = self.process_method_argument(method_name, arg_idx, val)?;
            arg_vals.push(processed_val);
        }

        Ok(arg_vals)
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
                // Check if receiver type is a struct (not a reference)
                let is_struct_receiver = match &receiver_param.ty {
                    Type::Named(type_name) => self.struct_defs.contains_key(type_name),
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