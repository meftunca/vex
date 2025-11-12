use super::super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use std::collections::HashMap;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    pub(crate) fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::I128 => "i128".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::U128 => "u128".to_string(),
            Type::F16 => "f16".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Byte => "byte".to_string(),
            Type::String => "string".to_string(),
            Type::Any => "any".to_string(),
            Type::Nil => "nil".to_string(),
            Type::Error => "error".to_string(),
            Type::Named(name) => name.clone(),
            Type::Generic { name, type_args } => {
                // Recursive mangling for nested generics: Box<Box<i32>> => Box_Box_i32
                let arg_strs: Vec<String> = type_args
                    .iter()
                    .map(|arg| self.type_to_string(arg))
                    .collect();
                format!("{}_{}", name, arg_strs.join("_"))
            }

            // Phase 0: Builtin types
            Type::Vec(elem_ty) => format!("Vec_{}", self.type_to_string(elem_ty)),
            Type::Box(elem_ty) => format!("Box_{}", self.type_to_string(elem_ty)),
            Type::Option(elem_ty) => format!("Option_{}", self.type_to_string(elem_ty)),
            Type::Result(ok_ty, err_ty) => format!(
                "Result_{}_{}",
                self.type_to_string(ok_ty),
                self.type_to_string(err_ty)
            ),

            _ => "unknown".to_string(),
        }
    }

    /// Substitute type parameters in a type
    pub(crate) fn substitute_type(&self, ty: &Type, type_subst: &HashMap<String, Type>) -> Type {
        match ty {
            Type::Named(name) => {
                // Check if this is a type parameter
                if let Some(concrete_ty) = type_subst.get(name) {
                    concrete_ty.clone()
                } else {
                    ty.clone()
                }
            }
            Type::Reference(inner, is_mut) => {
                Type::Reference(Box::new(self.substitute_type(inner, type_subst)), *is_mut)
            }
            Type::Array(elem_ty, size) => {
                Type::Array(Box::new(self.substitute_type(elem_ty, type_subst)), *size)
            }
            Type::Generic { name, type_args } => {
                // Recursively substitute type arguments (for nested generics)
                // Example: Box<T> where T=Box<i32> becomes Box<Box<i32>>
                let substituted_args: Vec<Type> = type_args
                    .iter()
                    .map(|arg| self.substitute_type(arg, type_subst))
                    .collect();

                // ⭐ NEW: If all type arguments are fully concrete (no Named type params),
                // convert to a mangled Named type for instantiated structs
                // Example: Generic { name: "HashMap", type_args: [Named("str"), I32] }
                //       -> Named("HashMap_str_i32")
                let all_concrete = substituted_args
                    .iter()
                    .all(|arg| !matches!(arg, Type::Named(n) if type_subst.contains_key(n)));

                if all_concrete && !substituted_args.is_empty() {
                    // Build mangled name: HashMap<str, i32> -> HashMap_str_i32
                    let type_names: Vec<String> = substituted_args
                        .iter()
                        .map(|t| self.type_to_string(t))
                        .collect();
                    let mangled_name = format!("{}_{}", name, type_names.join("_"));
                    Type::Named(mangled_name)
                } else {
                    Type::Generic {
                        name: name.clone(),
                        type_args: substituted_args,
                    }
                }
            }

            // Phase 0: Builtin types (substitute inner type parameters)
            Type::Vec(inner) => Type::Vec(Box::new(self.substitute_type(inner, type_subst))),
            Type::Box(inner) => Type::Box(Box::new(self.substitute_type(inner, type_subst))),
            Type::Option(inner) => Type::Option(Box::new(self.substitute_type(inner, type_subst))),
            Type::Result(ok_ty, err_ty) => Type::Result(
                Box::new(self.substitute_type(ok_ty, type_subst)),
                Box::new(self.substitute_type(err_ty, type_subst)),
            ),

            _ => ty.clone(),
        }
    }

    /// Resolve a type, expanding type aliases
    pub(crate) fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Named(name) => {
                // Check if this is a type alias
                if let Some(resolved) = self.type_aliases.get(name) {
                    // Recursively resolve in case the alias points to another alias
                    self.resolve_type(resolved)
                } else {
                    ty.clone()
                }
            }
            // For complex types, recursively resolve components
            Type::Array(inner, size) => Type::Array(Box::new(self.resolve_type(inner)), *size),
            Type::Reference(inner, is_mut) => {
                Type::Reference(Box::new(self.resolve_type(inner)), *is_mut)
            }
            Type::Generic { name, type_args } => Type::Generic {
                name: name.clone(),
                type_args: type_args.iter().map(|t| self.resolve_type(t)).collect(),
            },

            // Phase 0: Builtin types (recursively resolve inner types)
            Type::Vec(inner) => Type::Vec(Box::new(self.resolve_type(inner))),
            Type::Box(inner) => Type::Box(Box::new(self.resolve_type(inner))),
            Type::Option(inner) => Type::Option(Box::new(self.resolve_type(inner))),
            Type::Result(ok_ty, err_ty) => Type::Result(
                Box::new(self.resolve_type(ok_ty)),
                Box::new(self.resolve_type(err_ty)),
            ),

            // Primitive types don't need resolution
            _ => ty.clone(),
        }
    }

    /// Approximate size of LLVM type in bits (for Result<T,E> union layout)
    pub(super) fn approximate_type_size(llvm_ty: &BasicTypeEnum) -> u32 {
        match llvm_ty {
            BasicTypeEnum::IntType(i) => i.get_bit_width(),
            BasicTypeEnum::FloatType(f) => {
                // f32=32, f64=64, f128=128
                if f.get_context().f32_type() == *f {
                    32
                } else if f.get_context().f64_type() == *f {
                    64
                } else if f.get_context().f128_type() == *f {
                    128
                } else {
                    32 // fallback
                }
            }
            BasicTypeEnum::PointerType(_) => 64, // Assume 64-bit pointers
            BasicTypeEnum::StructType(s) => {
                // Approximate: sum of field sizes (ignoring padding)
                let field_count = s.count_fields();
                field_count * 32 // rough estimate
            }
            BasicTypeEnum::ArrayType(a) => {
                // Array size = element size * length
                let elem_ty = a.get_element_type();
                let len = a.len();
                let elem_size = match elem_ty {
                    BasicTypeEnum::IntType(i) => i.get_bit_width(),
                    BasicTypeEnum::FloatType(_) => 32,
                    _ => 32,
                };
                elem_size * len
            }
            _ => 32, // fallback
        }
    }
}
