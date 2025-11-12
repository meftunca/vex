// Builtin contract implementations for primitive types
// This module provides compiler-generated contract methods for i32, f64, bool, string
// 
// Architecture:
// 1. Parser reads `i32 extends Add, Sub;` from stdlib
// 2. Creates Item::BuiltinExtension AST node
// 3. Compiler registers in BuiltinContractRegistry
// 4. Binary op codegen checks registry, generates LLVM IR directly (zero overhead)

use std::collections::{HashMap, HashSet};

/// Registry of builtin type contract extensions
/// Populated from `i32 extends Add, Sub;` declarations in stdlib
pub struct BuiltinContractRegistry {
    /// Type -> Set of contracts (e.g., "i32" -> {"Add", "Sub", "Mul"})
    extensions: HashMap<String, HashSet<String>>,
}

impl BuiltinContractRegistry {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }
    
    /// Register a builtin extension from AST
    /// Called during first compilation pass
    pub fn register(&mut self, type_name: String, contracts: Vec<String>) {
        self.extensions
            .entry(type_name)
            .or_insert_with(HashSet::new)
            .extend(contracts);
    }
    
    /// Check if a type has a contract
    pub fn has_contract(&self, type_name: &str, contract: &str) -> bool {
        self.extensions
            .get(type_name)
            .map(|contracts| contracts.contains(contract))
            .unwrap_or(false)
    }
    
    /// Get all contracts for a type
    pub fn get_contracts(&self, type_name: &str) -> Option<&HashSet<String>> {
        self.extensions.get(type_name)
    }
}

/// Legacy function - kept for compatibility during migration
/// TODO: Remove after migration to BuiltinContractRegistry
pub fn has_builtin_contract(type_name: &str, contract_name: &str) -> bool {
    // ⭐ PHASE 1: Arithmetic operators (i32, i64, f32, f64)
    let is_int = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128");
    let is_float = matches!(type_name, "f32" | "f64");
    let is_numeric = is_int || is_float;
    
    match contract_name {
        // Arithmetic (Phase 1)
        "Add" | "Sub" | "Mul" | "Div" => is_numeric,
        "Mod" => is_int,  // Modulo only for integers
        
        // Bitwise (Phase 1.5) - integers only
        "BitAnd" | "BitOr" | "BitXor" | "BitNot" | "Shl" | "Shr" => is_int,
        
        // Comparison (Phase 2)
        "Eq" | "Ord" => {
            matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "string")
        }
        
        // Non-operator contracts
        "Display" | "Clone" | "Debug" => {
            matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "f32" | "f64" | "bool" | "string")
        }
        
        _ => false,
    }
}

/// Map contract name to method name
/// Used for operator overloading: Add -> "op+", Sub -> "op-"
pub fn contract_to_operator_method(contract_name: &str) -> Option<&'static str> {
    match contract_name {
        // Arithmetic (binary)
        "Add" => Some("op+"),
        "Sub" => Some("op-"),
        "Mul" => Some("op*"),
        "Div" => Some("op/"),
        "Rem" => Some("op%"),
        
        // Arithmetic (unary)
        "Neg" => Some("op-"),  // Unary minus
        
        // Bitwise (binary)
        "BitAnd" => Some("op&"),
        "BitOr" => Some("op|"),
        "BitXor" => Some("op^"),
        "Shl" => Some("op<<"),
        "Shr" => Some("op>>"),
        
        // Bitwise (unary)
        "BitNot" => Some("op~"),
        
        // Comparison
        "Eq" => Some("op=="),  // Also op!=
        "Ord" => Some("op<"),  // Also op>, op<=, op>=
        
        // Logical (unary)
        "Not" => Some("op!"),
        
        // Assignment
        "AddAssign" => Some("op+="),
        "SubAssign" => Some("op-="),
        "MulAssign" => Some("op*="),
        "DivAssign" => Some("op/="),
        "RemAssign" => Some("op%="),
        "BitAndAssign" => Some("op&="),
        "BitOrAssign" => Some("op|="),
        "BitXorAssign" => Some("op^="),
        "ShlAssign" => Some("op<<="),
        "ShrAssign" => Some("op>>="),
        
        // Index
        "Index" => Some("op[]"),
        "IndexMut" => Some("op[]="),
        
        // Advanced
        "Pow" => Some("op**"),
        "PreInc" => Some("op++"),    // ++i
        "PostInc" => Some("op++"),   // i++
        "PreDec" => Some("op--"),    // --i
        "PostDec" => Some("op--"),   // i--
        "Range" => Some("op.."),
        "RangeInclusive" => Some("op..="),
        "NullCoalesce" => Some("op??"),
        
        // Non-operator contracts
        "Display" => Some("to_string"),
        "Debug" => Some("debug"),
        "Clone" => Some("clone"),
        
        _ => None,
    }
}

/// Legacy function for non-operator contracts
/// TODO: Migrate to new architecture
pub fn get_builtin_contract_method(contract_name: &str) -> Option<&'static str> {
    match contract_name {
        "Display" => Some("to_string"),
        "Debug" => Some("debug"),
        "Clone" => Some("clone"),
        "Eq" => Some("eq"),
        _ => None,
    }
}

/// Codegen builtin operator methods - generates LLVM IR directly (zero overhead)
/// Called from binary_ops.rs when operator overloading is detected
pub fn codegen_builtin_operator<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    type_name: &str,
    contract_name: &str,
    method_name: &str,
    left: inkwell::values::BasicValueEnum<'ctx>,
    right: inkwell::values::BasicValueEnum<'ctx>,
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    use inkwell::IntPredicate;
    use inkwell::FloatPredicate;
    
    let is_int = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128");
    let is_float = matches!(type_name, "f32" | "f64");
    
    match (contract_name, method_name) {
        // ⭐ PHASE 1: Arithmetic operators
        ("Add", "op+") => {
            if is_int {
                Some(builder.build_int_add(left.into_int_value(), right.into_int_value(), "add").unwrap().into())
            } else if is_float {
                Some(builder.build_float_add(left.into_float_value(), right.into_float_value(), "fadd").unwrap().into())
            } else {
                None
            }
        }
        
        ("Sub", "op-") => {
            if is_int {
                Some(builder.build_int_sub(left.into_int_value(), right.into_int_value(), "sub").unwrap().into())
            } else if is_float {
                Some(builder.build_float_sub(left.into_float_value(), right.into_float_value(), "fsub").unwrap().into())
            } else {
                None
            }
        }
        
        ("Mul", "op*") => {
            if is_int {
                Some(builder.build_int_mul(left.into_int_value(), right.into_int_value(), "mul").unwrap().into())
            } else if is_float {
                Some(builder.build_float_mul(left.into_float_value(), right.into_float_value(), "fmul").unwrap().into())
            } else {
                None
            }
        }
        
        ("Div", "op/") => {
            if is_int {
                let is_signed = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128");
                if is_signed {
                    Some(builder.build_int_signed_div(left.into_int_value(), right.into_int_value(), "sdiv").unwrap().into())
                } else {
                    Some(builder.build_int_unsigned_div(left.into_int_value(), right.into_int_value(), "udiv").unwrap().into())
                }
            } else if is_float {
                Some(builder.build_float_div(left.into_float_value(), right.into_float_value(), "fdiv").unwrap().into())
            } else {
                None
            }
        }
        
        ("Mod", "op%") => {
            if is_int {
                let is_signed = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128");
                if is_signed {
                    Some(builder.build_int_signed_rem(left.into_int_value(), right.into_int_value(), "srem").unwrap().into())
                } else {
                    Some(builder.build_int_unsigned_rem(left.into_int_value(), right.into_int_value(), "urem").unwrap().into())
                }
            } else {
                None
            }
        }
        
        // ⭐ PHASE 1.5: Bitwise operators
        ("BitAnd", "op&") => {
            if is_int {
                Some(builder.build_and(left.into_int_value(), right.into_int_value(), "and").unwrap().into())
            } else {
                None
            }
        }
        
        ("BitOr", "op|") => {
            if is_int {
                Some(builder.build_or(left.into_int_value(), right.into_int_value(), "or").unwrap().into())
            } else {
                None
            }
        }
        
        ("BitXor", "op^") => {
            if is_int {
                Some(builder.build_xor(left.into_int_value(), right.into_int_value(), "xor").unwrap().into())
            } else {
                None
            }
        }
        
        ("Shl", "op<<") => {
            if is_int {
                Some(builder.build_left_shift(left.into_int_value(), right.into_int_value(), "shl").unwrap().into())
            } else {
                None
            }
        }
        
        ("Shr", "op>>") => {
            if is_int {
                let is_signed = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128");
                Some(builder.build_right_shift(left.into_int_value(), right.into_int_value(), is_signed, "shr").unwrap().into())
            } else {
                None
            }
        }
        
        // ⭐ PHASE 2: Comparison operators
        ("Eq", "op==") => {
            if is_int {
                Some(builder.build_int_compare(IntPredicate::EQ, left.into_int_value(), right.into_int_value(), "eq").unwrap().into())
            } else if is_float {
                Some(builder.build_float_compare(FloatPredicate::OEQ, left.into_float_value(), right.into_float_value(), "feq").unwrap().into())
            } else {
                None
            }
        }
        
        ("Eq", "op!=") => {
            if is_int {
                Some(builder.build_int_compare(IntPredicate::NE, left.into_int_value(), right.into_int_value(), "ne").unwrap().into())
            } else if is_float {
                Some(builder.build_float_compare(FloatPredicate::ONE, left.into_float_value(), right.into_float_value(), "fne").unwrap().into())
            } else {
                None
            }
        }
        
        ("Ord", "op<") => {
            if is_int {
                let is_signed = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128");
                let pred = if is_signed { IntPredicate::SLT } else { IntPredicate::ULT };
                Some(builder.build_int_compare(pred, left.into_int_value(), right.into_int_value(), "lt").unwrap().into())
            } else if is_float {
                Some(builder.build_float_compare(FloatPredicate::OLT, left.into_float_value(), right.into_float_value(), "flt").unwrap().into())
            } else {
                None
            }
        }
        
        ("Ord", "op<=") => {
            if is_int {
                let is_signed = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128");
                let pred = if is_signed { IntPredicate::SLE } else { IntPredicate::ULE };
                Some(builder.build_int_compare(pred, left.into_int_value(), right.into_int_value(), "le").unwrap().into())
            } else if is_float {
                Some(builder.build_float_compare(FloatPredicate::OLE, left.into_float_value(), right.into_float_value(), "fle").unwrap().into())
            } else {
                None
            }
        }
        
        ("Ord", "op>") => {
            if is_int {
                let is_signed = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128");
                let pred = if is_signed { IntPredicate::SGT } else { IntPredicate::UGT };
                Some(builder.build_int_compare(pred, left.into_int_value(), right.into_int_value(), "gt").unwrap().into())
            } else if is_float {
                Some(builder.build_float_compare(FloatPredicate::OGT, left.into_float_value(), right.into_float_value(), "fgt").unwrap().into())
            } else {
                None
            }
        }
        
        ("Ord", "op>=") => {
            if is_int {
                let is_signed = matches!(type_name, "i32" | "i64" | "i8" | "i16" | "i128");
                let pred = if is_signed { IntPredicate::SGE } else { IntPredicate::UGE };
                Some(builder.build_int_compare(pred, left.into_int_value(), right.into_int_value(), "ge").unwrap().into())
            } else if is_float {
                Some(builder.build_float_compare(FloatPredicate::OGE, left.into_float_value(), right.into_float_value(), "fge").unwrap().into())
            } else {
                None
            }
        }
        
        _ => None,
    }
}

/// Legacy codegen function
/// TODO: Remove after migration - operator codegen moved to binary_ops.rs
pub fn codegen_builtin_contract_method<'ctx>(
    _type_name: &str,
    contract_name: &str,
    _method_name: &str,
    receiver: inkwell::values::BasicValueEnum<'ctx>,
    _args: &[inkwell::values::BasicValueEnum<'ctx>],
) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    match contract_name {
        // Clone: return receiver (bitwise copy for primitives)
        "Clone" => Some(receiver),
        
        // Operators: handled in binary_ops.rs now
        _ => None,
    }
}

