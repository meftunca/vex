// Type conversion and inference
// Contains ast_type_to_llvm, infer_expression_type and related methods

use super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use std::collections::HashMap;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Convert AST Type to LLVM BasicTypeEnum
    pub(crate) fn ast_type_to_llvm(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::I8 => BasicTypeEnum::IntType(self.context.i8_type()),
            Type::I16 => BasicTypeEnum::IntType(self.context.i16_type()),
            Type::I32 => BasicTypeEnum::IntType(self.context.i32_type()),
            Type::I64 => BasicTypeEnum::IntType(self.context.i64_type()),
            Type::I128 => BasicTypeEnum::IntType(self.context.i128_type()),
            Type::U8 => BasicTypeEnum::IntType(self.context.i8_type()),
            Type::U16 => BasicTypeEnum::IntType(self.context.i16_type()),
            Type::U32 => BasicTypeEnum::IntType(self.context.i32_type()),
            Type::U64 => BasicTypeEnum::IntType(self.context.i64_type()),
            Type::U128 => BasicTypeEnum::IntType(self.context.i128_type()),
            Type::F16 => BasicTypeEnum::FloatType(self.context.f16_type()),
            Type::F32 => BasicTypeEnum::FloatType(self.context.f32_type()),
            Type::F64 => BasicTypeEnum::FloatType(self.context.f64_type()),
            Type::Bool => BasicTypeEnum::IntType(self.context.bool_type()),
            Type::Byte => BasicTypeEnum::IntType(self.context.i8_type()),
            Type::String => {
                // String as ptr (C-style string pointer)
                BasicTypeEnum::PointerType(self.context.ptr_type(inkwell::AddressSpace::default()))
            }
            Type::Any => {
                // Any type as opaque pointer (void* equivalent)
                BasicTypeEnum::PointerType(self.context.ptr_type(inkwell::AddressSpace::default()))
            }
            Type::Nil => {
                // Nil as void/i8 (placeholder)
                // In LLVM, we use i8 as a minimal type
                BasicTypeEnum::IntType(self.context.i8_type())
            }
            Type::Error => {
                // Error type as i32 (error code)
                BasicTypeEnum::IntType(self.context.i32_type())
            }
            Type::Array(elem_ty, size) => {
                let elem_llvm = self.ast_type_to_llvm(elem_ty);
                match elem_llvm {
                    BasicTypeEnum::IntType(it) => {
                        BasicTypeEnum::ArrayType(it.array_type(*size as u32))
                    }
                    BasicTypeEnum::FloatType(ft) => {
                        BasicTypeEnum::ArrayType(ft.array_type(*size as u32))
                    }
                    BasicTypeEnum::ArrayType(at) => {
                        BasicTypeEnum::ArrayType(at.array_type(*size as u32))
                    }
                    _ => BasicTypeEnum::IntType(self.context.i32_type()), // fallback
                }
            }
            Type::Reference(inner_ty, _is_mutable) => {
                // Reference type: &T or &mut T
                // In LLVM, references are just pointers
                let inner_llvm = self.ast_type_to_llvm(inner_ty);
                match inner_llvm {
                    BasicTypeEnum::IntType(it) => {
                        BasicTypeEnum::PointerType(it.ptr_type(inkwell::AddressSpace::default()))
                    }
                    BasicTypeEnum::FloatType(ft) => {
                        BasicTypeEnum::PointerType(ft.ptr_type(inkwell::AddressSpace::default()))
                    }
                    BasicTypeEnum::ArrayType(at) => {
                        BasicTypeEnum::PointerType(at.ptr_type(inkwell::AddressSpace::default()))
                    }
                    BasicTypeEnum::StructType(st) => {
                        BasicTypeEnum::PointerType(st.ptr_type(inkwell::AddressSpace::default()))
                    }
                    BasicTypeEnum::PointerType(pt) => {
                        // Already a pointer, just return it
                        BasicTypeEnum::PointerType(pt)
                    }
                    _ => BasicTypeEnum::PointerType(
                        self.context
                            .i32_type()
                            .ptr_type(inkwell::AddressSpace::default()),
                    ),
                }
            }
            Type::Named(name) => {
                // ⚠️ Check type aliases FIRST before any other lookups
                if let Some(resolved) = self.type_aliases.get(name) {
                    return self.ast_type_to_llvm(resolved);
                }

                // Special case: "void" type (for FFI/C interop)
                if name == "void" {
                    // void is represented as i8 in LLVM (opaque type)
                    return BasicTypeEnum::IntType(self.context.i8_type());
                }

                // ⚠️ CRITICAL FIX: "str" is a special type alias for String
                // The parser treats `: str` as Type::Named("str"), but it should be a pointer
                if name == "str" {
                    return BasicTypeEnum::PointerType(
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                    );
                }

                // Handle custom struct types

                // Check if this struct is registered
                if let Some(struct_def) = self.struct_defs.get(name) {
                    // Build struct type from registry
                    let field_types: Vec<BasicTypeEnum> = struct_def
                        .fields
                        .iter()
                        .map(|(_, field_ty)| self.ast_type_to_llvm(field_ty))
                        .collect();

                    let struct_ty = self.context.struct_type(&field_types, false);

                    // ⚠️ CRITICAL FIX: Return the struct TYPE, not a pointer!
                    // Functions that return struct types will return by value.
                    // Variables that store structs will have pointer type (handled in Let statement).
                    // This allows proper struct return semantics while still using pointers for variables.
                    BasicTypeEnum::StructType(struct_ty)
                } else if let Some(enum_def) = self.enum_ast_defs.get(name) {
                    // Handle enum types
                    // Enums with data are represented as structs: {i32 tag, T data}
                    // Find the largest data type among all variants
                    let has_data = enum_def.variants.iter().any(|v| !v.data.is_empty());
                    eprintln!(
                        "🟠 ast_type_to_llvm for enum {}: has_data={}, variants={}",
                        name,
                        has_data,
                        enum_def.variants.len()
                    );

                    if has_data {
                        // For data-carrying enums, we need to find the largest data type
                        // For simplicity, use the first data type we find
                        // For mixed enums (Some + None), calculate union size

                        // Find the largest data type (for single-value variants, use first type)
                        let largest_data_type = enum_def
                            .variants
                            .iter()
                            .filter(|v| !v.data.is_empty())
                            .map(|v| {
                                // For multi-value tuple variants, create a tuple type
                                if v.data.len() > 1 {
                                    // Multi-value tuple: create struct type
                                    let field_types: Vec<BasicTypeEnum> =
                                        v.data.iter().map(|ty| self.ast_type_to_llvm(ty)).collect();
                                    let tuple_ty = self.context.struct_type(&field_types, false);
                                    (&v.data[0], BasicTypeEnum::StructType(tuple_ty))
                                } else {
                                    // Single-value tuple
                                    let ty = &v.data[0];
                                    let llvm_ty = self.ast_type_to_llvm(ty);
                                    (ty, llvm_ty)
                                }
                            })
                            .max_by_key(|(_, llvm_ty)| {
                                // Get size of LLVM type
                                match llvm_ty {
                                    BasicTypeEnum::IntType(i) => i.get_bit_width() as usize,
                                    BasicTypeEnum::FloatType(f) => match f {
                                        _ if *f == self.context.f32_type() => 32,
                                        _ if *f == self.context.f64_type() => 64,
                                        _ => 32,
                                    },
                                    BasicTypeEnum::PointerType(_) => 64,
                                    BasicTypeEnum::StructType(s) => {
                                        // Approximate struct size
                                        s.count_fields() as usize * 32
                                    }
                                    _ => 32,
                                }
                            })
                            .map(|(_, llvm_ty)| llvm_ty)
                            .unwrap_or(self.context.i8_type().into()); // Default: i8 for unit variants

                        let enum_struct_type = self.context.struct_type(
                            &[self.context.i32_type().into(), largest_data_type],
                            false,
                        );
                        BasicTypeEnum::StructType(enum_struct_type)
                    } else {
                        // Unit-only enum: just an i32 tag
                        BasicTypeEnum::IntType(self.context.i32_type())
                    }
                } else if name == "AnonymousStruct" {
                    // Placeholder for inferred structs
                    BasicTypeEnum::IntType(self.context.i32_type())
                } else if name == "Vec" || name == "Box" || name == "String" || name == "Map" {
                    // Builtin heap types are represented as opaque pointers
                    // The actual struct is defined in C runtime (vex_vec.c, vex_box.c, etc.)
                    BasicTypeEnum::PointerType(
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                    )
                } else if let Some(struct_def) = self.struct_defs.get(name) {
                    // User-defined struct type: Point, Container_Point, etc.
                    let field_types: Vec<BasicTypeEnum> = struct_def
                        .fields
                        .iter()
                        .map(|(_, field_ty)| self.ast_type_to_llvm(field_ty))
                        .collect();

                    BasicTypeEnum::StructType(self.context.struct_type(&field_types, false))
                } else {
                    // Unknown named type, default to i32
                    // TODO: Better error handling
                    eprintln!("⚠️ Unknown type name '{}', defaulting to i32", name);
                    BasicTypeEnum::IntType(self.context.i32_type())
                }
            }
            Type::Function {
                params,
                return_type,
            } => {
                // Function type: fn(T1, T2) -> R
                // In LLVM, this is a function pointer type
                let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = params
                    .iter()
                    .map(|t| self.ast_type_to_llvm(t).into())
                    .collect();

                let ret_llvm = self.ast_type_to_llvm(return_type);

                // Create function type
                let fn_type = match ret_llvm {
                    BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
                    BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
                    _ => self.context.i32_type().fn_type(&param_types, false),
                };

                // Return as pointer to function
                BasicTypeEnum::PointerType(fn_type.ptr_type(inkwell::AddressSpace::default()))
            }
            Type::Generic { name, type_args } => {
                // Generic struct type: Box<T>, Pair<T, U>
                // Need to instantiate and look up monomorphized struct
                // Use const self, so can't call instantiate_generic_struct (needs &mut self)
                // Instead, generate mangled name and look up in struct_defs

                let type_arg_strings: Vec<String> =
                    type_args.iter().map(|t| self.type_to_string(t)).collect();

                let mangled_name = format!("{}_{}", name, type_arg_strings.join("_"));

                // Look up the monomorphized struct
                if let Some(struct_def) = self.struct_defs.get(&mangled_name) {
                    let field_types: Vec<BasicTypeEnum> = struct_def
                        .fields
                        .iter()
                        .map(|(_, field_ty)| self.ast_type_to_llvm(field_ty))
                        .collect();

                    let struct_ty = self.context.struct_type(&field_types, false);

                    // ⭐ FIX: Return struct TYPE for generic structs (same as Named types)
                    // This allows functions to return structs by value
                    BasicTypeEnum::StructType(struct_ty)
                } else {
                    // Struct not yet monomorphized
                    // ⚠️ Instead of returning i32, try to use a forward-declared opaque struct
                    // This prevents incorrect type mapping during early function declaration
                    eprintln!(
                        "⚠️  Generic struct {} not yet instantiated, using opaque pointer",
                        mangled_name
                    );
                    BasicTypeEnum::PointerType(
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                    )
                }
            }
            Type::Union(types) => {
                // Union type: T1 | T2 | T3
                // Implementation: Tagged union with discriminator (like Rust enums)
                // Layout: { i32 tag, <largest_type> data }

                if types.is_empty() {
                    // Empty union, fallback to i32
                    return BasicTypeEnum::IntType(self.context.i32_type());
                }

                // Find the largest type among all union members
                let largest_type = types
                    .iter()
                    .map(|ty| {
                        let llvm_ty = self.ast_type_to_llvm(ty);
                        let size = Self::approximate_type_size(&llvm_ty);
                        (llvm_ty, size)
                    })
                    .max_by_key(|(_, size)| *size)
                    .map(|(ty, _)| ty)
                    .unwrap_or(self.context.i32_type().into());

                // Create tagged union: { i32 tag, <largest_type> data }
                let tag_ty = self.context.i32_type();
                let union_struct = self
                    .context
                    .struct_type(&[tag_ty.into(), largest_type], false);

                BasicTypeEnum::StructType(union_struct)
            }

            Type::Intersection(types) => {
                // Intersection type: T1 & T2 & T3
                // Implementation: Merge all struct fields (for trait composition)
                // For now, use first type as representation
                // TODO: Implement proper intersection semantics
                if let Some(first_type) = types.first() {
                    self.ast_type_to_llvm(first_type)
                } else {
                    BasicTypeEnum::IntType(self.context.i32_type())
                }
            }

            // ===== PHASE 0: BUILTIN TYPES =====
            Type::Vec(_elem_ty) => {
                // Vec<T> layout: { i8*, i64, i64, i64 }
                // Fields: data_ptr, len, capacity, elem_size
                // Match C runtime vex_vec_t struct
                let ptr_ty = self
                    .context
                    .i8_type()
                    .ptr_type(inkwell::AddressSpace::default());
                let size_ty = self.context.i64_type();

                let vec_struct = self.context.struct_type(
                    &[
                        ptr_ty.into(),  // void *data
                        size_ty.into(), // size_t len
                        size_ty.into(), // size_t capacity
                        size_ty.into(), // size_t elem_size
                    ],
                    false,
                );

                BasicTypeEnum::StructType(vec_struct)
            }

            Type::Box(_elem_ty) => {
                // Box<T> layout: { i8*, i64 }
                // Fields: ptr, size
                // Match C runtime vex_box_t struct
                let ptr_ty = self
                    .context
                    .i8_type()
                    .ptr_type(inkwell::AddressSpace::default());
                let size_ty = self.context.i64_type();

                let box_struct = self.context.struct_type(
                    &[
                        ptr_ty.into(),  // void *ptr
                        size_ty.into(), // size_t size
                    ],
                    false,
                );

                BasicTypeEnum::StructType(box_struct)
            }

            Type::Option(inner_ty) => {
                // Option<T> layout: { i32, T }
                // Fields: tag (0=None, 1=Some), value
                // Runtime helpers handle unwrap/is_some checks
                let tag_ty = self.context.i32_type();
                let value_ty = self.ast_type_to_llvm(inner_ty);

                let option_struct = self.context.struct_type(
                    &[
                        tag_ty.into(), // i32 tag (consistent with user enums)
                        value_ty,      // T value
                    ],
                    false,
                );

                BasicTypeEnum::StructType(option_struct)
            }

            Type::Result(ok_ty, err_ty) => {
                // Result<T, E> layout: { i32, union { T, E } }
                // Fields: tag (0=Err, 1=Ok), value
                // For simplicity, use max(sizeof(T), sizeof(E)) for value field
                let tag_ty = self.context.i32_type();
                let ok_llvm = self.ast_type_to_llvm(ok_ty);
                let err_llvm = self.ast_type_to_llvm(err_ty);

                // Calculate which type is larger (approximate)
                let ok_size = Self::approximate_type_size(&ok_llvm);
                let err_size = Self::approximate_type_size(&err_llvm);
                let value_ty = if ok_size >= err_size {
                    ok_llvm
                } else {
                    err_llvm
                };

                let result_struct = self.context.struct_type(
                    &[
                        tag_ty.into(), // u8 tag
                        value_ty,      // union { T ok, E err }
                    ],
                    false,
                );

                BasicTypeEnum::StructType(result_struct)
            }

            Type::SelfType => {
                // Self type - needs to be resolved in context
                // For now, treat as opaque pointer
                // TODO: Resolve to actual type from impl context
                eprintln!("⚠️  Self type encountered, needs resolution");
                BasicTypeEnum::PointerType(self.context.ptr_type(inkwell::AddressSpace::default()))
            }

            Type::AssociatedType { self_type, name } => {
                // Associated type: Self.Item, Self.Output
                // Try to resolve from trait impl context
                match self.resolve_associated_type(self_type, name) {
                    Ok(concrete_type) => {
                        eprintln!(
                            "✅ Resolved associated type {}.{} → {:?}",
                            self.type_to_string(self_type),
                            name,
                            concrete_type
                        );
                        return self.ast_type_to_llvm(&concrete_type);
                    }
                    Err(e) => {
                        eprintln!(
                            "⚠️  Failed to resolve {}.{}: {}",
                            self.type_to_string(self_type),
                            name,
                            e
                        );
                        // Fall back to opaque pointer for now
                        BasicTypeEnum::PointerType(
                            self.context.ptr_type(inkwell::AddressSpace::default()),
                        )
                    }
                }
            }

            Type::Slice(_elem_ty, _is_mutable) => {
                // Slice<T> layout: { i8*, i64, i64 }
                // Fields: data_ptr, len, elem_size
                // Match C runtime VexSlice struct
                let ptr_ty = self.context.ptr_type(inkwell::AddressSpace::default());
                let size_ty = self.context.i64_type();

                let slice_struct = self.context.struct_type(
                    &[
                        ptr_ty.into(),  // void *data
                        size_ty.into(), // size_t len
                        size_ty.into(), // size_t elem_size
                    ],
                    false,
                );

                BasicTypeEnum::StructType(slice_struct)
            }

            Type::Never => {
                // Never type (!) - represents diverging functions (panic, exit, infinite loop)
                // In LLVM, use i8 as a minimal type (should never be instantiated)
                BasicTypeEnum::IntType(self.context.i8_type())
            }

            Type::RawPtr { inner, is_const: _ } => {
                // Raw pointer: *T or *const T
                // Unsafe pointer for FFI/C interop
                // In LLVM, all pointers are opaque (LLVM 15+)
                // We ignore is_const flag as LLVM doesn't have const pointers
                let _inner_llvm = self.ast_type_to_llvm(inner);
                BasicTypeEnum::PointerType(self.context.ptr_type(inkwell::AddressSpace::default()))
            }

            _ => {
                // Default to i32 for unsupported types
                BasicTypeEnum::IntType(self.context.i32_type())
            }
        }
    }

    /// Extract LLVM FunctionType from AST Function type
    /// Used for indirect function calls
    pub(crate) fn ast_function_type_to_llvm(
        &self,
        params: &[Type],
        return_type: &Type,
    ) -> Result<inkwell::types::FunctionType<'ctx>, String> {
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = params
            .iter()
            .map(|t| self.ast_type_to_llvm(t).into())
            .collect();

        let ret_llvm = self.ast_type_to_llvm(return_type);

        // Create function type based on return type
        let fn_type = match ret_llvm {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            _ => return Err("Unsupported return type for function".to_string()),
        };

        Ok(fn_type)
    }

    /// Infer AST type from LLVM type (for type inference)
    pub(crate) fn infer_ast_type_from_llvm(
        &self,
        llvm_type: BasicTypeEnum<'ctx>,
    ) -> Result<Type, String> {
        match llvm_type {
            BasicTypeEnum::IntType(int_ty) => {
                let bit_width = int_ty.get_bit_width();
                match bit_width {
                    8 => Ok(Type::I8),
                    16 => Ok(Type::I16),
                    32 => Ok(Type::I32),
                    64 => Ok(Type::I64),
                    128 => Ok(Type::I128),
                    1 => Ok(Type::Bool),
                    _ => {
                        // Note: Cannot emit diagnostic here due to &self limitation
                        // This is an internal error that shouldn't normally occur
                        Err(format!(
                            "Unsupported integer bit width: {} (internal error)",
                            bit_width
                        ))
                    }
                }
            }
            BasicTypeEnum::FloatType(float_ty) => {
                // Check float type based on LLVM representation
                if float_ty == self.context.f16_type() {
                    Ok(Type::F16)
                } else if float_ty == self.context.f32_type() {
                    Ok(Type::F32)
                } else if float_ty == self.context.f64_type() {
                    Ok(Type::F64)
                } else {
                    // Fallback to f64 if unknown
                    Ok(Type::F64)
                }
            }
            BasicTypeEnum::PointerType(_) => Ok(Type::String), // Assume string for now
            BasicTypeEnum::ArrayType(arr_ty) => {
                let elem_ty = arr_ty.get_element_type();
                let size = arr_ty.len() as usize;

                // Recursively infer element type
                let elem_ast_ty = match elem_ty {
                    BasicTypeEnum::IntType(int_ty) => {
                        let bit_width = int_ty.get_bit_width();
                        match bit_width {
                            1 => Type::Bool,
                            8 => Type::I8,
                            16 => Type::I16,
                            32 => Type::I32,
                            64 => Type::I64,
                            128 => Type::I128,
                            _ => {
                                return Err(format!(
                                    "Unsupported array element int type with bit width {}",
                                    bit_width
                                ))
                            }
                        }
                    }
                    BasicTypeEnum::FloatType(float_ty) => {
                        if float_ty == self.context.f16_type() {
                            Type::F16
                        } else if float_ty == self.context.f32_type() {
                            Type::F32
                        } else if float_ty == self.context.f64_type() {
                            Type::F64
                        } else {
                            Type::F64
                        }
                    }
                    BasicTypeEnum::ArrayType(inner_arr_ty) => {
                        // Nested array: [[i32; 2]; 2]
                        let inner_elem_ty = inner_arr_ty.get_element_type();
                        let inner_size = inner_arr_ty.len() as usize;

                        let inner_ast_ty = match inner_elem_ty {
                            BasicTypeEnum::IntType(int_ty) => {
                                let bit_width = int_ty.get_bit_width();
                                match bit_width {
                                    1 => Type::Bool,
                                    8 => Type::I8,
                                    16 => Type::I16,
                                    32 => Type::I32,
                                    64 => Type::I64,
                                    128 => Type::I128,
                                    _ => return Err(format!(
                                        "Unsupported nested array element type with bit width {}",
                                        bit_width
                                    )),
                                }
                            }
                            _ => return Err("Unsupported nested array element type".to_string()),
                        };
                        Type::Array(Box::new(inner_ast_ty), inner_size)
                    }
                    _ => return Err(format!("Unsupported array element type: {:?}", elem_ty)),
                };
                Ok(Type::Array(Box::new(elem_ast_ty), size))
            }
            BasicTypeEnum::StructType(_) => {
                // For struct types, we can't fully infer without metadata
                // For now, use a placeholder named type
                Ok(Type::Named("AnonymousStruct".to_string()))
            }
            _ => Err("Cannot infer type from LLVM type".to_string()),
        }
    }

    /// Infer expression type for type inference
    pub(crate) fn infer_expression_type(&self, expr: &Expression) -> Result<Type, String> {
        let result = match expr {
            Expression::IntLiteral(_) => Ok(Type::I32),
            Expression::BigIntLiteral(_) => Ok(Type::I128), // Large integers default to i128
            Expression::FloatLiteral(_) => Ok(Type::F64),
            Expression::StringLiteral(_) => Ok(Type::String),
            Expression::FStringLiteral(_) => Ok(Type::String),
            Expression::BoolLiteral(_) => Ok(Type::Bool),
            Expression::MapLiteral(_) => Ok(Type::Named("Map".to_string())),
            Expression::Array(elements) => {
                // Array literal [1, 2, 3] is a Vec<T>
                if elements.is_empty() {
                    return Ok(Type::Generic {
                        name: "Vec".to_string(),
                        type_args: vec![Type::I32], // Default to Vec<i32>
                    });
                }
                // Infer element type from first element
                let elem_type = self.infer_expression_type(&elements[0])?;
                Ok(Type::Generic {
                    name: "Vec".to_string(),
                    type_args: vec![elem_type],
                })
            }
            Expression::Ident(name) => {
                // Check if we have AST type information first (most accurate)
                if let Some(ast_type) = self.variable_ast_types.get(name) {
                    return Ok(ast_type.clone());
                }

                // Check if this is a struct variable
                if let Some(struct_name) = self.variable_struct_names.get(name) {
                    // Handle mangled generic types (e.g., "Vec_i32" -> Vec<i32>)
                    if struct_name.starts_with("Vec_") {
                        let elem_type_str = &struct_name["Vec_".len()..];
                        let elem_type = match elem_type_str {
                            "i32" => Type::I32,
                            "i64" => Type::I64,
                            "f32" => Type::F32,
                            "f64" => Type::F64,
                            _ => Type::I32, // Fallback
                        };
                        return Ok(Type::Generic {
                            name: "Vec".to_string(),
                            type_args: vec![elem_type],
                        });
                    }
                    // Handle other generic types similarly
                    if struct_name.starts_with("Box_") {
                        let inner_type_str = &struct_name["Box_".len()..];
                        let inner_type = match inner_type_str {
                            "i32" => Type::I32,
                            "i64" => Type::I64,
                            _ => Type::I32,
                        };
                        return Ok(Type::Generic {
                            name: "Box".to_string(),
                            type_args: vec![inner_type],
                        });
                    }
                    return Ok(Type::Named(struct_name.clone()));
                }

                // Try to get type from variable
                if let Some(llvm_type) = self.variable_types.get(name) {
                    // Convert LLVM type back to AST type (simplified)
                    match llvm_type {
                        BasicTypeEnum::IntType(_) => Ok(Type::I32),
                        BasicTypeEnum::FloatType(_) => Ok(Type::F64),
                        _ => Ok(Type::I32), // Fallback
                    }
                } else {
                    Ok(Type::I32) // Default fallback
                }
            }
            _ => Ok(Type::I32), // Default for complex expressions
        };
        result
    }

    /// Convert Type to string for mangling
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
    fn approximate_type_size(llvm_ty: &BasicTypeEnum) -> u32 {
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
