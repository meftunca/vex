// Type conversion and inference
// Contains ast_type_to_llvm, infer_expression_type and related methods

use super::super::ASTCodeGen;
use inkwell::types::BasicTypeEnum;
use vex_ast::*;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Convert AST Type to LLVM BasicTypeEnum
    #[allow(deprecated)]
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
                } else if name == "ptr" {
                    // Special case: 'ptr' is a generic pointer type (like void* in C)
                    BasicTypeEnum::PointerType(
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                    )
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
            Type::Vec(elem_ty) => {
                // Vec<T> - Use actual struct definition from stdlib/core/src/vec.vx
                // Convert to mangled name: Vec<i32> -> Vec_i32
                let mangled_name = format!("Vec_{}", self.type_to_string(elem_ty));
                
                // Check if this Vec type has been instantiated
                if let Some(struct_def) = self.struct_defs.get(&mangled_name) {
                    // Build struct from actual field definitions
                    let field_types: Vec<BasicTypeEnum> = struct_def
                        .fields
                        .iter()
                        .map(|(_, field_ty)| self.ast_type_to_llvm(field_ty))
                        .collect();
                    
                    let vec_struct = self.context.struct_type(&field_types, false);
                    BasicTypeEnum::StructType(vec_struct)
                } else {
                    // Vec not yet instantiated - use stdlib layout: { *T, i64, i64 }
                    // This matches stdlib/core/src/vec.vx:
                    //   struct Vec<T> { data: *T, len: i64, cap: i64 }
                    let elem_llvm = self.ast_type_to_llvm(elem_ty);
                    let elem_ptr_type = match elem_llvm {
                        BasicTypeEnum::IntType(it) => it.ptr_type(inkwell::AddressSpace::default()),
                        BasicTypeEnum::FloatType(ft) => ft.ptr_type(inkwell::AddressSpace::default()),
                        BasicTypeEnum::ArrayType(at) => at.ptr_type(inkwell::AddressSpace::default()),
                        BasicTypeEnum::StructType(st) => st.ptr_type(inkwell::AddressSpace::default()),
                        BasicTypeEnum::PointerType(pt) => pt,
                        BasicTypeEnum::VectorType(vt) => vt.ptr_type(inkwell::AddressSpace::default()),
                        BasicTypeEnum::ScalableVectorType(svt) => svt.ptr_type(inkwell::AddressSpace::default()),
                    };
                    
                    let vec_struct = self.context.struct_type(
                        &[
                            elem_ptr_type.into(), // data: *T
                            self.context.i64_type().into(), // len: i64
                            self.context.i64_type().into(), // cap: i64
                        ],
                        false,
                    );
                    BasicTypeEnum::StructType(vec_struct)
                }
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

            Type::Tuple(elements) => {
                // Tuple type: (T, U, V, ...)
                // Represented as anonymous struct: {T, U, V}
                if elements.is_empty() {
                    // Empty tuple () - unit type, represented as zero-sized struct
                    let unit_type = self.context.struct_type(&[], false);
                    BasicTypeEnum::StructType(unit_type)
                } else {
                    let elem_types: Vec<BasicTypeEnum> = elements
                        .iter()
                        .map(|ty| self.ast_type_to_llvm(ty))
                        .collect();
                    let tuple_struct = self.context.struct_type(&elem_types, false);
                    BasicTypeEnum::StructType(tuple_struct)
                }
            }

            Type::Future(_inner_ty) => {
                // Future<T> - Async computation result
                // Represented as opaque pointer (void*) to runtime FutureHandle
                // The runtime owns the future state and result storage
                // Layout (runtime internal): { state: i32, result: T, ... }
                BasicTypeEnum::PointerType(self.context.ptr_type(inkwell::AddressSpace::default()))
            }

            _ => {
                // Default to i32 for unsupported types
                BasicTypeEnum::IntType(self.context.i32_type())
            }
        }
    }
}
