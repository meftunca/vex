// Builtin functions registry for Vex compiler
// Modular structure for maintainability

use super::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use std::collections::HashMap;

// Submodules
mod array;
mod async_runtime; // Async/await runtime integration
mod builtin_types; // Phase 0: Vec, Option, Result, Box
mod channel;
mod core;
mod hashmap;
mod hints;
mod intrinsics;
mod memory;
mod memory_ops;
mod reflection;
mod set; // Set<T> builtin functions (wraps Map)
mod slice; // Slice<T> builtin functions
mod stdlib; // Zero-cost stdlib C runtime declarations
mod stdlib_logger; // Stdlib: logger module
mod stdlib_testing;
mod stdlib_time; // Stdlib: time module
mod string;
mod utf8; // Stdlib: testing module

// Re-export all builtin implementations
pub use array::*;
 // Async runtime functions
pub use builtin_types::*; // Phase 0 builtin types
pub use channel::*;
pub use core::*;
pub use hashmap::*;
pub use hints::*;
pub use intrinsics::*;
pub use memory::*;
pub use memory_ops::*;
pub use reflection::*;
pub use set::*; // Set<T> operations
pub use slice::*; // Slice<T> operations
pub use stdlib::*; // Stdlib runtime functions
pub use stdlib_logger::*; // Stdlib logger implementations
pub use stdlib_testing::*; // Stdlib testing implementations
pub use stdlib_time::*; // Stdlib time implementations
pub use string::*;
pub use utf8::*;

/// Builtin function generator type
pub type BuiltinGenerator<'ctx> =
    fn(&mut ASTCodeGen<'ctx>, &[BasicValueEnum<'ctx>]) -> Result<BasicValueEnum<'ctx>, String>;

/// Registry of all builtin functions
pub struct BuiltinRegistry<'ctx> {
    functions: HashMap<&'static str, BuiltinGenerator<'ctx>>,
}

impl<'ctx> BuiltinRegistry<'ctx> {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // Register core builtins (print, panic, assert)
        registry.register("print", core::builtin_print);
        registry.register("println", core::builtin_println);
        registry.register("panic", core::builtin_panic);
        registry.register("assert", core::builtin_assert);
        registry.register("unreachable", core::builtin_unreachable);

        // Register memory builtins (alloc, free, sizeof, alignof)
        registry.register("alloc", memory::builtin_alloc);
        registry.register("free", memory::builtin_free);
        registry.register("realloc", memory::builtin_realloc);
        registry.register("sizeof", memory::builtin_sizeof);
        registry.register("alignof", memory::builtin_alignof);

        // Register LLVM intrinsics - bit manipulation
        registry.register("ctlz", intrinsics::builtin_ctlz);
        registry.register("cttz", intrinsics::builtin_cttz);
        registry.register("ctpop", intrinsics::builtin_ctpop);
        registry.register("bswap", intrinsics::builtin_bswap);
        registry.register("bitreverse", intrinsics::builtin_bitreverse);

        // Register LLVM intrinsics - overflow checking
        registry.register("sadd_overflow", intrinsics::builtin_sadd_overflow);
        registry.register("ssub_overflow", intrinsics::builtin_ssub_overflow);
        registry.register("smul_overflow", intrinsics::builtin_smul_overflow);

        // Register compiler hints
        registry.register("assume", hints::builtin_assume);
        registry.register("likely", hints::builtin_likely);
        registry.register("unlikely", hints::builtin_unlikely);
        registry.register("prefetch", hints::builtin_prefetch);

        // Register runtime string functions
        registry.register("strlen", string::builtin_strlen);
        registry.register("strcmp", string::builtin_strcmp);
        registry.register("strcpy", string::builtin_strcpy);
        registry.register("strcat", string::builtin_strcat);
        registry.register("strdup", string::builtin_strdup);
        registry.register("vex_string_as_cstr", string::builtin_string_as_cstr);
        registry.register("vex_string_len", string::builtin_string_len);

        // Register runtime memory operations
        registry.register("memcpy", memory_ops::builtin_memcpy);
        registry.register("memset", memory_ops::builtin_memset);
        registry.register("memcmp", memory_ops::builtin_memcmp);
        registry.register("memmove", memory_ops::builtin_memmove);

        // Register runtime UTF-8 functions
        registry.register("utf8_valid", utf8::builtin_utf8_valid);
        registry.register("utf8_char_count", utf8::builtin_utf8_char_count);
        registry.register("utf8_char_at", utf8::builtin_utf8_char_at);

        // Register runtime array functions
        registry.register("array_len", array::builtin_array_len);
        registry.register("array_get", array::builtin_array_get);
        registry.register("array_set", array::builtin_array_set);
        registry.register("array_append", array::builtin_array_append);

        // Register type reflection functions
        registry.register("typeof", reflection::builtin_typeof);
        registry.register("type_id", reflection::builtin_type_id);
        registry.register("type_size", reflection::builtin_type_size);
        registry.register("type_align", reflection::builtin_type_align);
        registry.register("is_int_type", reflection::builtin_is_int_type);
        registry.register("is_float_type", reflection::builtin_is_float_type);
        registry.register("is_pointer_type", reflection::builtin_is_pointer_type);
        registry.register("field_metadata", reflection::builtin_field_metadata);

        // Register HashMap functions
        registry.register("hashmap_new", hashmap::builtin_hashmap_new);
        registry.register("hashmap_insert", hashmap::builtin_hashmap_insert);
        registry.register("hashmap_get", hashmap::builtin_hashmap_get);
        registry.register("hashmap_len", hashmap::builtin_hashmap_len);
        registry.register("hashmap_free", hashmap::builtin_hashmap_free);
        registry.register("hashmap_contains", hashmap::builtin_hashmap_contains);
        registry.register("hashmap_remove", hashmap::builtin_hashmap_remove);
        registry.register("hashmap_clear", hashmap::builtin_hashmap_clear);

        // Map() type-as-constructor aliases
        registry.register("map_new", hashmap::builtin_hashmap_new);
        registry.register("map_insert", hashmap::builtin_hashmap_insert);
        registry.register("map_get", hashmap::builtin_hashmap_get);
        registry.register("map_len", hashmap::builtin_hashmap_len);
        registry.register("map_free", hashmap::builtin_hashmap_free);

        // Register Slice<T> functions
        registry.register("slice_from_vec", slice::builtin_slice_from_vec);
        registry.register("slice_new", slice::builtin_slice_new);
        registry.register("slice_get", slice::builtin_slice_get);
        registry.register("slice_len", slice::builtin_slice_len);

        // Register Set<T> functions (wraps Map<T, ()>)
        registry.register("set_new", set::builtin_set_new);
        registry.register("set_with_capacity", set::builtin_set_with_capacity);
        registry.register("set_insert", set::builtin_set_insert);
        registry.register("set_contains", set::builtin_set_contains);
        registry.register("set_remove", set::builtin_set_remove);
        registry.register("set_len", set::builtin_set_len);
        registry.register("set_clear", set::builtin_set_clear);

        // Phase 0.4b: Builtin type constructors (free functions)
        registry.register("vec_new", builtin_types::builtin_vec_new);
        registry.register(
            "vec_with_capacity",
            builtin_types::builtin_vec_with_capacity,
        );
        registry.register("vec_free", builtin_types::builtin_vec_free);
        registry.register("box_new", builtin_types::builtin_box_new);
        registry.register("box_free", builtin_types::builtin_box_free);
        registry.register("string_new", builtin_types::builtin_string_new);
        registry.register("string_from", builtin_types::builtin_string_from);
        registry.register("string_free", builtin_types::builtin_string_free);
        registry.register("channel_new", channel::builtin_channel_new);

        // Phase 0.8: Option<T> and Result<T,E> constructors
        registry.register("Some", builtin_types::builtin_option_some);
        registry.register("None", builtin_types::builtin_option_none);
        registry.register("Ok", builtin_types::builtin_result_ok);
        registry.register("Err", builtin_types::builtin_result_err);

        // Async runtime functions
        registry.register("runtime_create", ASTCodeGen::builtin_runtime_create);
        registry.register("runtime_destroy", ASTCodeGen::builtin_runtime_destroy);
        registry.register("runtime_run", ASTCodeGen::builtin_runtime_run);
        registry.register("runtime_shutdown", ASTCodeGen::builtin_runtime_shutdown);
        registry.register("async_sleep", ASTCodeGen::builtin_async_sleep);
        registry.register("spawn_async", ASTCodeGen::builtin_spawn_async);

        // Channel functions
        registry.register("Channel.new", channel::builtin_channel_new);
        registry.register("Channel.send", channel::builtin_channel_send);
        registry.register("Channel.recv", channel::builtin_channel_recv);

        // Phase 0.7: Numeric to string conversions
        registry.register(
            "vex_i32_to_string",
            builtin_types::builtin_vex_i32_to_string,
        );
        registry.register(
            "vex_i64_to_string",
            builtin_types::builtin_vex_i64_to_string,
        );
        registry.register(
            "vex_u32_to_string",
            builtin_types::builtin_vex_u32_to_string,
        );
        registry.register(
            "vex_u64_to_string",
            builtin_types::builtin_vex_u64_to_string,
        );
        registry.register(
            "vex_f32_to_string",
            builtin_types::builtin_vex_f32_to_string,
        );

        // ========================================================================
        // STDLIB MODULE FUNCTIONS
        // ========================================================================

        // Logger module (std.logger)
        registry.register("logger::debug", stdlib_logger::stdlib_logger_debug);
        registry.register("logger::info", stdlib_logger::stdlib_logger_info);
        registry.register("logger::warn", stdlib_logger::stdlib_logger_warn);
        registry.register("logger::error", stdlib_logger::stdlib_logger_error);

        // Time module (std.time)
        registry.register("time::now", stdlib_time::stdlib_time_now);
        registry.register("time::high_res", stdlib_time::stdlib_time_high_res);
        registry.register("time::sleep_ms", stdlib_time::stdlib_time_sleep_ms);

        // Testing module (std.testing)
        registry.register("testing::assert", stdlib_testing::stdlib_testing_assert);
        registry.register(
            "testing::assert_eq",
            stdlib_testing::stdlib_testing_assert_eq,
        );
        registry.register(
            "testing::assert_ne",
            stdlib_testing::stdlib_testing_assert_ne,
        );
        registry.register(
            "vex_f64_to_string",
            builtin_types::builtin_vex_f64_to_string,
        );

        registry
    }

    fn register(&mut self, name: &'static str, generator: BuiltinGenerator<'ctx>) {
        self.functions.insert(name, generator);
    }

    pub fn get(&self, name: &str) -> Option<BuiltinGenerator<'ctx>> {
        self.functions.get(name).copied()
    }

    #[allow(dead_code)]
    pub fn is_builtin(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

impl<'ctx> Default for BuiltinRegistry<'ctx> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HELPER DECLARATIONS FOR ASTCodeGen
// ============================================================================

use inkwell::values::FunctionValue;
use inkwell::AddressSpace;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Declare vex_malloc from runtime
    pub(crate) fn declare_vex_malloc(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("vex_malloc") {
            return func;
        }

        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let size_type = self.context.i64_type(); // size_t

        let fn_type = i8_ptr_type.fn_type(&[size_type.into()], false);
        self.module.add_function("vex_malloc", fn_type, None)
    }

    /// Declare vex_free from runtime
    pub(crate) fn declare_vex_free(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("vex_free") {
            return func;
        }

        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();

        let fn_type = void_type.fn_type(&[i8_ptr_type.into()], false);
        self.module.add_function("vex_free", fn_type, None)
    }

    /// Declare vex_realloc from runtime
    pub(crate) fn declare_vex_realloc(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("vex_realloc") {
            return func;
        }

        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let size_type = self.context.i64_type(); // size_t

        let fn_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), size_type.into()], false);
        self.module.add_function("vex_realloc", fn_type, None)
    }

    /// Declare abort() from libc
    pub(crate) fn declare_abort(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("abort") {
            return func;
        }

        let void_type = self.context.void_type();
        let fn_type = void_type.fn_type(&[], false);
        self.module.add_function("abort", fn_type, None)
    }

    /// Declare LLVM intrinsic function with basic return type
    pub(crate) fn declare_llvm_intrinsic(
        &mut self,
        name: &str,
        param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
        return_type: inkwell::types::BasicMetadataTypeEnum<'ctx>,
    ) -> FunctionValue<'ctx> {
        // Check if already declared
        if let Some(func) = self.module.get_function(name) {
            return func;
        }

        // Create function type
        use inkwell::types::BasicMetadataTypeEnum;

        let fn_type = match return_type {
            BasicMetadataTypeEnum::IntType(int_type) => int_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::FloatType(float_type) => float_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::PointerType(ptr_type) => ptr_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::StructType(struct_type) => {
                struct_type.fn_type(param_types, false)
            }
            BasicMetadataTypeEnum::ArrayType(arr_type) => arr_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::VectorType(vec_type) => vec_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::ScalableVectorType(svec_type) => {
                svec_type.fn_type(param_types, false)
            }
            BasicMetadataTypeEnum::MetadataType(_) => {
                // Metadata type - use i8 as placeholder
                self.context.i8_type().fn_type(param_types, false)
            }
        };

        self.module.add_function(name, fn_type, None)
    }

    /// Declare LLVM intrinsic function with void return type
    pub(crate) fn declare_llvm_intrinsic_void(
        &mut self,
        name: &str,
        param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
    ) -> FunctionValue<'ctx> {
        // Check if already declared
        if let Some(func) = self.module.get_function(name) {
            return func;
        }

        let fn_type = self.context.void_type().fn_type(param_types, false);
        self.module.add_function(name, fn_type, None)
    }

    /// Declare LLVM memset intrinsic: llvm.memset.p0.i64
    /// Signature: void @llvm.memset.p0.i64(i8* ptr, i8 val, i64 len, i1 volatile)
    pub(crate) fn get_or_declare_memset(&mut self) -> FunctionValue<'ctx> {
        let name = "llvm.memset.p0.i64";

        // Check if already declared
        if let Some(func) = self.module.get_function(name) {
            return func;
        }

        // Parameter types: (i8*, i8, i64, i1)
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::default());
        let i64_type = self.context.i64_type();
        let bool_type = self.context.bool_type();

        let param_types = &[
            i8_ptr_type.into(), // ptr
            i8_type.into(),     // val
            i64_type.into(),    // len
            bool_type.into(),   // volatile
        ];

        // Return type: void
        let fn_type = self.context.void_type().fn_type(param_types, false);
        self.module.add_function(name, fn_type, None)
    }

    /// Generic runtime function declaration helper
    pub(crate) fn declare_runtime_fn(
        &mut self,
        name: &str,
        param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
        return_type: inkwell::types::BasicMetadataTypeEnum<'ctx>,
    ) -> FunctionValue<'ctx> {
        // Check if already declared
        if let Some(func) = self.module.get_function(name) {
            return func;
        }

        // Create function type based on return type
        use inkwell::types::BasicMetadataTypeEnum;

        let fn_type = match return_type {
            BasicMetadataTypeEnum::IntType(int_type) => int_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::FloatType(float_type) => float_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::PointerType(ptr_type) => ptr_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::StructType(struct_type) => {
                struct_type.fn_type(param_types, false)
            }
            BasicMetadataTypeEnum::ArrayType(arr_type) => arr_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::VectorType(vec_type) => vec_type.fn_type(param_types, false),
            BasicMetadataTypeEnum::ScalableVectorType(svec_type) => {
                svec_type.fn_type(param_types, false)
            }
            BasicMetadataTypeEnum::MetadataType(_) => {
                // Metadata shouldn't be used as return type
                self.context.i8_type().fn_type(param_types, false)
            }
        };

        self.module.add_function(name, fn_type, None)
    }

    /// Runtime function declaration helper for void return type
    pub(crate) fn declare_runtime_fn_void(
        &mut self,
        name: &str,
        param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
    ) -> FunctionValue<'ctx> {
        // Check if already declared
        if let Some(func) = self.module.get_function(name) {
            return func;
        }

        let fn_type = self.context.void_type().fn_type(param_types, false);
        self.module.add_function(name, fn_type, None)
    }
}
