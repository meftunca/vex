// Builtin type method compilation (Vec, Box, String, Map, etc.)

mod ranges_arrays;
mod string_collections;
mod vec_box;

use crate::builtin_contracts;
use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Try to compile builtin contract methods (i32.to_string(), bool.clone(), etc.)
    /// Returns Some(value) if handled, None if not a builtin contract method
    pub(super) fn try_compile_builtin_contract_method(
        &mut self,
        receiver: &Expression,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        // Get receiver type
        let receiver_type = self.infer_expression_type(receiver)?;
        let type_name = self.type_to_string(&receiver_type);

        // Check all known contracts to find matching method
        let contracts = ["Display", "Clone", "Debug", "Eq"];

        for contract in &contracts {
            if let Some(contract_method) = builtin_contracts::get_builtin_contract_method(contract)
            {
                if contract_method == method
                    && builtin_contracts::has_builtin_contract(&type_name, contract)
                {
                    // Found builtin contract method!
                    let receiver_val = self.compile_expression(receiver)?;

                    // Compile arguments
                    let arg_vals: Vec<BasicValueEnum> = args
                        .iter()
                        .map(|arg| self.compile_expression(arg))
                        .collect::<Result<Vec<_>, _>>()?;

                    // Dispatch to builtin contract codegen
                    if let Some(result) = builtin_contracts::codegen_builtin_contract_method(
                        &type_name,
                        contract,
                        method,
                        receiver_val,
                        &arg_vals,
                    ) {
                        return Ok(Some(result));
                    }
                }
            }
        }

        Ok(None) // Not a builtin contract method
    }

    /// Try to compile builtin type methods (Vec.push, Vec.len, Box.get, etc.)
    /// Returns Some(value) if the method was handled, None if not a builtin method
    pub(super) fn try_compile_builtin_method(
        &mut self,
        receiver: &Expression,
        method: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, String> {
        // Get receiver variable to check its type
        let var_name = match receiver {
            Expression::Ident(name) => name.clone(),
            _ => return Ok(None), // Not a simple identifier, skip
        };

        // Check if this is a builtin type (Vec, Box)
        let struct_name = self.variable_struct_names.get(&var_name).cloned();

        if let Some(type_name) = struct_name {
            // ⚠️ CRITICAL: Check if this is a user-defined struct vs builtin type
            // User-defined structs take precedence over builtin types with same name
            let is_user_defined_struct = self.struct_defs.contains_key(&type_name);

            // Extract base type name (Vec_i32 -> Vec, Box_string -> Box)
            let base_type = type_name.split('_').next().unwrap_or(&type_name);

            // Handle builtin type methods ONLY if not shadowed by user struct
            if !is_user_defined_struct {
                match base_type {
                    "Vec" => return self.compile_vec_method(&var_name, method, args),
                    "Box" => return self.compile_box_method(&var_name, method, args),
                    "String" => return self.compile_string_method(&var_name, method, args),
                    "Map" => return self.compile_map_method(&var_name, method, args),
                    "Set" => return self.compile_set_method(&var_name, method, args),
                    "Range" => return self.compile_range_method(&var_name, method, args, false),
                    "RangeInclusive" => {
                        return self.compile_range_method(&var_name, method, args, true)
                    }
                    "Slice" => return self.compile_slice_method(&var_name, method, args),
                    "Channel" => return self.compile_channel_method(&var_name, method, args),
                    _ => return Ok(None), // Not a builtin type
                }
            }
        }

        // Check if this is an array type (arrays don't have struct names)
        if let Some(var_type) = self.variable_types.get(&var_name) {
            if let inkwell::types::BasicTypeEnum::ArrayType(_) = var_type {
                return self.compile_array_method(&var_name, method, args);
            }
        }

        Ok(None)
    }
}
