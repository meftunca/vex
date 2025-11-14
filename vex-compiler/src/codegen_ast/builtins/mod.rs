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
mod formatting; // Compile-time type-safe formatting
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

        registry.register_all_builtins();

        registry
    }

    /// Register core builtin functions (print, panic, assert)
    fn register_core_builtins(&mut self) {
        self.register("print", core::builtin_print);
        self.register("println", core::builtin_println);
        self.register("panic", core::builtin_panic);
        self.register("assert", core::builtin_assert);
        self.register("unreachable", core::builtin_unreachable);
    }

    /// Register memory management builtins
    fn register_memory_builtins(&mut self) {
        self.register("alloc", memory::builtin_alloc);
        self.register("free", memory::builtin_free);
        self.register("realloc", memory::builtin_realloc);
        self.register("sizeof", memory::builtin_sizeof);
        self.register("alignof", memory::builtin_alignof);
    }

    /// Register LLVM intrinsics
    fn register_llvm_intrinsics(&mut self) {
        // Bit manipulation
        self.register("ctlz", intrinsics::builtin_ctlz);
        self.register("cttz", intrinsics::builtin_cttz);
        self.register("ctpop", intrinsics::builtin_ctpop);
        self.register("bswap", intrinsics::builtin_bswap);
        self.register("bitreverse", intrinsics::builtin_bitreverse);

        // Overflow checking
        self.register("sadd_overflow", intrinsics::builtin_sadd_overflow);
        self.register("ssub_overflow", intrinsics::builtin_ssub_overflow);
        self.register("smul_overflow", intrinsics::builtin_smul_overflow);
    }

    /// Register compiler hints
    fn register_compiler_hints(&mut self) {
        self.register("assume", hints::builtin_assume);
        self.register("likely", hints::builtin_likely);
        self.register("unlikely", hints::builtin_unlikely);
        self.register("prefetch", hints::builtin_prefetch);
    }

    /// Register string functions
    fn register_string_functions(&mut self) {
        self.register("strlen", string::builtin_strlen);
        self.register("strcmp", string::builtin_strcmp);
        self.register("strcpy", string::builtin_strcpy);
        self.register("strcat", string::builtin_strcat);
        self.register("strdup", string::builtin_strdup);
        self.register("vex_string_as_cstr", string::builtin_string_as_cstr);
        self.register("vex_string_len", string::builtin_string_len);
    }

    /// Register formatting functions
    fn register_formatting_functions(&mut self) {
        self.register("i32_to_string", formatting::builtin_i32_to_string);
        self.register("f64_to_string", formatting::builtin_f64_to_string);
        self.register("bool_to_string", formatting::builtin_bool_to_string);
    }

    /// Register memory operations
    fn register_memory_operations(&mut self) {
        self.register("memcpy", memory_ops::builtin_memcpy);
        self.register("memset", memory_ops::builtin_memset);
        self.register("memcmp", memory_ops::builtin_memcmp);
        self.register("memmove", memory_ops::builtin_memmove);
    }

    /// Register UTF-8 functions
    fn register_utf8_functions(&mut self) {
        self.register("utf8_valid", utf8::builtin_utf8_valid);
        self.register("utf8_char_count", utf8::builtin_utf8_char_count);
        self.register("utf8_char_at", utf8::builtin_utf8_char_at);
    }

    /// Register array functions
    fn register_array_functions(&mut self) {
        self.register("array_len", array::builtin_array_len);
        self.register("array_get", array::builtin_array_get);
        self.register("array_set", array::builtin_array_set);
        self.register("array_append", array::builtin_array_append);
    }

    /// Register reflection functions
    fn register_reflection_functions(&mut self) {
        self.register("typeof", reflection::builtin_typeof);
        self.register("type_id", reflection::builtin_type_id);
        self.register("type_size", reflection::builtin_type_size);
        self.register("type_align", reflection::builtin_type_align);
        self.register("is_int_type", reflection::builtin_is_int_type);
        self.register("is_float_type", reflection::builtin_is_float_type);
        self.register("is_pointer_type", reflection::builtin_is_pointer_type);
        self.register("field_metadata", reflection::builtin_field_metadata);
    }

    /// Register HashMap functions
    fn register_hashmap_functions(&mut self) {
        self.register("hashmap_new", hashmap::builtin_hashmap_new);
        self.register("hashmap_insert", hashmap::builtin_hashmap_insert);
        self.register("hashmap_get", hashmap::builtin_hashmap_get);
        self.register("hashmap_len", hashmap::builtin_hashmap_len);
        self.register("hashmap_free", hashmap::builtin_hashmap_free);
        self.register("hashmap_contains", hashmap::builtin_hashmap_contains);
        self.register("hashmap_remove", hashmap::builtin_hashmap_remove);
        self.register("hashmap_clear", hashmap::builtin_hashmap_clear);

        // Map() aliases
        self.register("map_new", hashmap::builtin_hashmap_new);
        self.register("map_insert", hashmap::builtin_hashmap_insert);
        self.register("map_get", hashmap::builtin_hashmap_get);
        self.register("map_len", hashmap::builtin_hashmap_len);
        self.register("map_free", hashmap::builtin_hashmap_free);
    }

    /// Register slice functions
    fn register_slice_functions(&mut self) {
        self.register("slice_from_vec", slice::builtin_slice_from_vec);
        self.register("slice_new", slice::builtin_slice_new);
        self.register("slice_get", slice::builtin_slice_get);
        self.register("slice_len", slice::builtin_slice_len);
    }

    /// Register set functions
    fn register_set_functions(&mut self) {
        self.register("set_new", set::builtin_set_new);
        self.register("set_with_capacity", set::builtin_set_with_capacity);
        self.register("set_insert", set::builtin_set_insert);
        self.register("set_contains", set::builtin_set_contains);
        self.register("set_remove", set::builtin_set_remove);
        self.register("set_len", set::builtin_set_len);
        self.register("set_clear", set::builtin_set_clear);
    }

    /// Register builtin type constructors
    fn register_builtin_types(&mut self) {
        // Vec, Box, String constructors
        self.register("vec_new", builtin_types::builtin_vec_new);
        self.register("Vec.new", builtin_types::builtin_vec_new); // Type constructor syntax
        self.register(
            "vec_with_capacity",
            builtin_types::builtin_vec_with_capacity,
        );
        self.register("vec_free", builtin_types::builtin_vec_free);
        self.register("box_new", builtin_types::builtin_box_new);
        self.register("Box.new", builtin_types::builtin_box_new); // Type constructor syntax
        self.register("box_free", builtin_types::builtin_box_free);
        self.register("string_new", builtin_types::builtin_string_new);
        self.register("String.new", builtin_types::builtin_string_new); // Type constructor syntax
        self.register("string_from", builtin_types::builtin_string_from);
        self.register("string_free", builtin_types::builtin_string_free);
        self.register("channel_new", channel::builtin_channel_new);
        self.register("Channel.new", channel::builtin_channel_new); // Type constructor syntax

        // Option and Result constructors
        self.register("Some", builtin_types::builtin_option_some);
        self.register("None", builtin_types::builtin_option_none);
        self.register("Ok", builtin_types::builtin_result_ok);
        self.register("Err", builtin_types::builtin_result_err);

        // Primitive to string conversions
        self.register(
            "vex_i32_to_string",
            builtin_types::builtin_vex_i32_to_string,
        );
        self.register(
            "vex_i64_to_string",
            builtin_types::builtin_vex_i64_to_string,
        );
        self.register(
            "vex_u32_to_string",
            builtin_types::builtin_vex_u32_to_string,
        );
        self.register(
            "vex_u64_to_string",
            builtin_types::builtin_vex_u64_to_string,
        );
        self.register(
            "vex_f32_to_string",
            builtin_types::builtin_vex_f32_to_string,
        );
        self.register(
            "vex_f64_to_string",
            builtin_types::builtin_vex_f64_to_string,
        );
        self.register(
            "vex_bool_to_string",
            builtin_types::builtin_vex_bool_to_string,
        );
        self.register(
            "vex_string_to_string",
            builtin_types::builtin_vex_string_to_string,
        );
    }

    /// Register async runtime functions
    fn register_async_runtime(&mut self) {
        self.register("runtime_create", ASTCodeGen::builtin_runtime_create);
        self.register("runtime_destroy", ASTCodeGen::builtin_runtime_destroy);
        self.register("runtime_run", ASTCodeGen::builtin_runtime_run);
        self.register("runtime_shutdown", ASTCodeGen::builtin_runtime_shutdown);
        self.register("async_sleep", ASTCodeGen::builtin_async_sleep);
        self.register("spawn_async", ASTCodeGen::builtin_spawn_async);
    }

    /// Register channel functions
    fn register_channel_functions(&mut self) {
        self.register("Channel.new", channel::builtin_channel_new);
        self.register("Channel.send", channel::builtin_channel_send);
        self.register("Channel.recv", channel::builtin_channel_recv);
    }

    /// Register stdlib module functions
    fn register_stdlib_functions(&mut self) {
        // Logger module
        self.register("logger::debug", stdlib_logger::stdlib_logger_debug);
        self.register("logger::info", stdlib_logger::stdlib_logger_info);
        self.register("logger::warn", stdlib_logger::stdlib_logger_warn);
        self.register("logger::error", stdlib_logger::stdlib_logger_error);

        // Time module
        self.register("time::now", stdlib_time::stdlib_time_now);
        self.register("time::high_res", stdlib_time::stdlib_time_high_res);
        self.register("time::sleep_ms", stdlib_time::stdlib_time_sleep_ms);

        // Testing module
        self.register("testing::assert", stdlib_testing::stdlib_testing_assert);
        self.register(
            "testing::assert_eq",
            stdlib_testing::stdlib_testing_assert_eq,
        );
        self.register(
            "testing::assert_ne",
            stdlib_testing::stdlib_testing_assert_ne,
        );
    }

    /// Register all builtin functions by calling category-specific methods
    pub fn register_all_builtins(&mut self) {
        self.register_core_builtins();
        self.register_memory_builtins();
        self.register_llvm_intrinsics();
        self.register_compiler_hints();
        self.register_string_functions();
        self.register_formatting_functions();
        self.register_memory_operations();
        self.register_utf8_functions();
        self.register_array_functions();
        self.register_reflection_functions();
        self.register_hashmap_functions();
        self.register_slice_functions();
        self.register_set_functions();
        self.register_builtin_types();
        self.register_async_runtime();
        self.register_channel_functions();
        self.register_stdlib_functions();
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

    // ===== FMT LIBRARY DECLARATIONS =====

    /// strlen(str: *u8) -> i64
    pub(crate) fn declare_strlen(&mut self) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("strlen") {
            return func;
        }
        let fn_type = self.context.i64_type().fn_type(
            &[self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into()],
            false,
        );
        self.module.add_function("strlen", fn_type, None)
    }

    /// vex_fmt_buffer_new(capacity: i64) -> *FormatBuffer
    pub(crate) fn declare_vex_fmt_buffer_new(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_buffer_new",
            &[self.context.i64_type().into()],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_buffer_free(buf: *FormatBuffer)
    pub(crate) fn declare_vex_fmt_buffer_free(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn_void(
            "vex_fmt_buffer_free",
            &[self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into()],
        )
    }

    /// vex_fmt_buffer_append_str(buf: *FormatBuffer, str: *u8, len: i64)
    pub(crate) fn declare_vex_fmt_buffer_append_str(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn_void(
            "vex_fmt_buffer_append_str",
            &[
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                self.context.i64_type().into(),
            ],
        )
    }

    /// vex_fmt_buffer_to_string(buf: *FormatBuffer) -> *u8
    pub(crate) fn declare_vex_fmt_buffer_to_string(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_buffer_to_string",
            &[self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into()],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_i32(value: i32, spec: *FormatSpec) -> *u8
    pub(crate) fn declare_vex_fmt_i32(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_i32",
            &[
                self.context.i32_type().into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_i64(value: i64, spec: *FormatSpec) -> *u8
    pub(crate) fn declare_vex_fmt_i64(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_i64",
            &[
                self.context.i64_type().into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_u32(value: u32, spec: *FormatSpec) -> *u8
    pub(crate) fn declare_vex_fmt_u32(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_u32",
            &[
                self.context.i32_type().into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_u64(value: u64, spec: *FormatSpec) -> *u8
    pub(crate) fn declare_vex_fmt_u64(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_u64",
            &[
                self.context.i64_type().into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_f32(value: f32, spec: *FormatSpec) -> *u8
    pub(crate) fn declare_vex_fmt_f32(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_f32",
            &[
                self.context.f32_type().into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_f64(value: f64, spec: *FormatSpec) -> *u8
    pub(crate) fn declare_vex_fmt_f64(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_f64",
            &[
                self.context.f64_type().into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_bool(value: bool, spec: *FormatSpec) -> *u8
    pub(crate) fn declare_vex_fmt_bool(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_bool",
            &[
                self.context.bool_type().into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// vex_fmt_string(str: *u8, len: i64, spec: *FormatSpec) -> *u8
    pub(crate) fn declare_vex_fmt_string(&mut self) -> FunctionValue<'ctx> {
        self.declare_runtime_fn(
            "vex_fmt_string",
            &[
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
                self.context.i64_type().into(),
                self.context
                    .ptr_type(inkwell::AddressSpace::default())
                    .into(),
            ],
            self.context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
        )
    }

    /// Get default FormatSpec (NULL pointer for now)
    pub(crate) fn get_default_format_spec(&self) -> inkwell::values::PointerValue<'ctx> {
        self.context
            .ptr_type(inkwell::AddressSpace::default())
            .const_null()
    }
}
