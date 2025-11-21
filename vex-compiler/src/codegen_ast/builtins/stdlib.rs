/**
 * Vex Stdlib Integration - C Runtime Functions
 * Zero-cost FFI declarations for LLVM codegen
 */
use super::ASTCodeGen;
use inkwell::AddressSpace;

// ============================================================================
// LOGGER MODULE (std.logger)
// ============================================================================

pub fn declare_logger_functions<'ctx>(codegen: &mut ASTCodeGen<'ctx>) {
    let void_type = codegen.context.void_type();
    let i8_ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let i32_type = codegen.context.i32_type();
    let i64_type = codegen.context.i64_type();

    // vex_print(ptr: *const u8, len: u64)
    let print_fn_type = void_type.fn_type(&[i8_ptr_type.into(), i64_type.into()], false);
    codegen
        .module
        .add_function("vex_print", print_fn_type, None);

    // vex_println(ptr: *const u8, len: u64)
    let println_fn_type = void_type.fn_type(&[i8_ptr_type.into(), i64_type.into()], false);
    codegen
        .module
        .add_function("vex_println", println_fn_type, None);

    // vex_eprint(ptr: *const u8, len: u64)
    let eprint_fn_type = void_type.fn_type(&[i8_ptr_type.into(), i64_type.into()], false);
    codegen
        .module
        .add_function("vex_eprint", eprint_fn_type, None);

    // vex_eprintln(ptr: *const u8, len: u64)
    let eprintln_fn_type = void_type.fn_type(&[i8_ptr_type.into(), i64_type.into()], false);
    codegen
        .module
        .add_function("vex_eprintln", eprintln_fn_type, None);

    // vex_printf(fmt: &str, ...) - variadic
    let printf_fn_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    codegen
        .module
        .add_function("vex_printf", printf_fn_type, None);
}

// ============================================================================
// FILESYSTEM MODULE (std.fs)
// ============================================================================

pub fn declare_fs_functions<'ctx>(codegen: &mut ASTCodeGen<'ctx>) {
    let i32_type = codegen.context.i32_type();
    let i64_type = codegen.context.i64_type();
    let bool_type = codegen.context.bool_type();
    let i8_ptr_type = codegen.context.ptr_type(AddressSpace::default());
    let usize_type = codegen.context.i64_type(); // usize = i64 on 64-bit
    let void_type = codegen.context.void_type();

    // File operations
    // vex_file_read(path: &str, content: &&str!, size: &usize!): i32
    let file_read_type = i32_type.fn_type(
        &[
            i8_ptr_type.into(),
            codegen.context.ptr_type(AddressSpace::default()).into(),
            codegen.context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    codegen
        .module
        .add_function("vex_file_read", file_read_type, None);

    // vex_file_write(path: &str, content: &str, size: usize): i32
    let file_write_type = i32_type.fn_type(
        &[i8_ptr_type.into(), i8_ptr_type.into(), usize_type.into()],
        false,
    );
    codegen
        .module
        .add_function("vex_file_write", file_write_type, None);

    // vex_file_append(path: &str, content: &str, size: usize): i32
    let file_append_type = i32_type.fn_type(
        &[i8_ptr_type.into(), i8_ptr_type.into(), usize_type.into()],
        false,
    );
    codegen
        .module
        .add_function("vex_file_append", file_append_type, None);

    // vex_file_exists(path: &str): bool
    let file_exists_type = bool_type.fn_type(&[i8_ptr_type.into()], false);
    codegen
        .module
        .add_function("vex_file_exists", file_exists_type, None);

    // vex_file_delete(path: &str): i32
    let file_delete_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
    codegen
        .module
        .add_function("vex_file_delete", file_delete_type, None);

    // vex_file_size(path: &str): i64
    let file_size_type = i64_type.fn_type(&[i8_ptr_type.into()], false);
    codegen
        .module
        .add_function("vex_file_size", file_size_type, None);

    // Path operations
    // vex_path_join(p1: &str, p2: &str, result: &str!, max_len: usize)
    let path_join_type = void_type.fn_type(
        &[
            i8_ptr_type.into(),
            i8_ptr_type.into(),
            i8_ptr_type.into(),
            usize_type.into(),
        ],
        false,
    );
    codegen
        .module
        .add_function("vex_path_join", path_join_type, None);

    // vex_path_dirname(path: &str, result: &str!, max_len: usize)
    let path_dirname_type = void_type.fn_type(
        &[i8_ptr_type.into(), i8_ptr_type.into(), usize_type.into()],
        false,
    );
    codegen
        .module
        .add_function("vex_path_dirname", path_dirname_type, None);

    // vex_path_basename(path: &str, result: &str!, max_len: usize)
    let path_basename_type = void_type.fn_type(
        &[i8_ptr_type.into(), i8_ptr_type.into(), usize_type.into()],
        false,
    );
    codegen
        .module
        .add_function("vex_path_basename", path_basename_type, None);

    // vex_path_extension(path: &str): &str
    let path_extension_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
    codegen
        .module
        .add_function("vex_path_extension", path_extension_type, None);

    // Directory operations
    // vex_dir_create(path: &str): i32
    let dir_create_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
    codegen
        .module
        .add_function("vex_dir_create", dir_create_type, None);

    // vex_dir_remove(path: &str): i32
    let dir_remove_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
    codegen
        .module
        .add_function("vex_dir_remove", dir_remove_type, None);

    // vex_dir_list(path: &str, files: &&str!, count: &usize!): i32
    let dir_list_type = i32_type.fn_type(
        &[
            i8_ptr_type.into(),
            codegen.context.ptr_type(AddressSpace::default()).into(),
            codegen.context.ptr_type(AddressSpace::default()).into(),
        ],
        false,
    );
    codegen
        .module
        .add_function("vex_dir_list", dir_list_type, None);
}

// ============================================================================
// TIME MODULE (std.time)
// ============================================================================

pub fn declare_time_functions<'ctx>(codegen: &mut ASTCodeGen<'ctx>) {
    let i64_type = codegen.context.i64_type();
    let u64_type = codegen.context.i64_type(); // u64 = i64 in LLVM
    let void_type = codegen.context.void_type();

    // Time functions
    // vex_time_now_sec(): i64
    let time_now_sec_type = i64_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_time_now_sec", time_now_sec_type, None);

    // vex_time_now_ms(): i64
    let time_now_ms_type = i64_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_time_now_ms", time_now_ms_type, None);

    // vex_time_now_us(): i64
    let time_now_us_type = i64_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_time_now_us", time_now_us_type, None);

    // vex_time_now_ns(): i64
    let time_now_ns_type = i64_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_time_now_ns", time_now_ns_type, None);

    // vex_time_high_res(): i64
    let time_high_res_type = i64_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_time_high_res", time_high_res_type, None);

    // Sleep functions
    // vex_sleep_ms(ms: u64)
    let sleep_ms_type = void_type.fn_type(&[u64_type.into()], false);
    codegen
        .module
        .add_function("vex_sleep_ms", sleep_ms_type, None);

    // vex_sleep_us(us: u64)
    let sleep_us_type = void_type.fn_type(&[u64_type.into()], false);
    codegen
        .module
        .add_function("vex_sleep_us", sleep_us_type, None);

    // vex_sleep_ns(ns: u64)
    let sleep_ns_type = void_type.fn_type(&[u64_type.into()], false);
    codegen
        .module
        .add_function("vex_sleep_ns", sleep_ns_type, None);
}

// ============================================================================
// TESTING MODULE (std.testing)
// ============================================================================

pub fn declare_testing_functions<'ctx>(codegen: &mut ASTCodeGen<'ctx>) {
    let void_type = codegen.context.void_type();
    let i32_type = codegen.context.i32_type();
    let i8_ptr_type = codegen.context.ptr_type(AddressSpace::default());

    // vex_test_start(name: &str)
    let test_start_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    codegen
        .module
        .add_function("vex_test_start", test_start_type, None);

    // vex_test_pass()
    let test_pass_type = void_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_test_pass", test_pass_type, None);

    // vex_test_fail(msg: &str)
    let test_fail_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    codegen
        .module
        .add_function("vex_test_fail", test_fail_type, None);

    // vex_test_summary(): i32
    let test_summary_type = i32_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_test_summary", test_summary_type, None);
}

// ============================================================================
// MEMORY / ALLOCATOR BRIDGE (Layer 1 glue)
// ============================================================================

pub fn declare_alloc_functions<'ctx>(codegen: &mut ASTCodeGen<'ctx>) {
    let i64_type = codegen.context.i64_type();
    let void_type = codegen.context.void_type();
    let raw_ptr_type = codegen.context.ptr_type(AddressSpace::default());

    // malloc(size: i64) -> ptr
    let malloc_type = raw_ptr_type.fn_type(&[i64_type.into()], false);
    let malloc_fn = if let Some(existing) = codegen.module.get_function("malloc") {
        existing
    } else {
        codegen.module.add_function("malloc", malloc_type, None)
    };
    codegen.functions.insert("malloc".to_string(), malloc_fn);

    // free(ptr: ptr)
    let free_type = void_type.fn_type(&[raw_ptr_type.into()], false);
    let free_fn = if let Some(existing) = codegen.module.get_function("free") {
        existing
    } else {
        codegen.module.add_function("free", free_type, None)
    };
    codegen.functions.insert("free".to_string(), free_fn);

    // realloc(ptr: ptr, new_size: i64) -> ptr
    let realloc_type = raw_ptr_type.fn_type(&[raw_ptr_type.into(), i64_type.into()], false);
    let realloc_fn = if let Some(existing) = codegen.module.get_function("realloc") {
        existing
    } else {
        codegen.module.add_function("realloc", realloc_type, None)
    };
    codegen.functions.insert("realloc".to_string(), realloc_fn);
}

// ============================================================================
// TIME MODULE (std.time) - Additional declarations
// ============================================================================

pub fn declare_additional_time_functions<'ctx>(codegen: &mut ASTCodeGen<'ctx>) {
    let i64_type = codegen.context.i64_type();
    let void_type = codegen.context.void_type();

    // vex_time_now(): i64 (milliseconds)
    let time_now_type = i64_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_time_now", time_now_type, None);

    // vex_time_monotonic(): i64 (nanoseconds, for high_res)
    let time_monotonic_type = i64_type.fn_type(&[], false);
    codegen
        .module
        .add_function("vex_time_monotonic", time_monotonic_type, None);

    // vex_time_sleep(ms: i64)
    let time_sleep_type = void_type.fn_type(&[i64_type.into()], false);
    codegen
        .module
        .add_function("vex_time_sleep", time_sleep_type, None);
}

// ============================================================================
// REGISTRATION - Called from ASTCodeGen::new()
// ============================================================================

pub fn register_stdlib_runtime<'ctx>(codegen: &mut ASTCodeGen<'ctx>) {
    declare_logger_functions(codegen);
    declare_fs_functions(codegen);
    declare_time_functions(codegen);
    declare_additional_time_functions(codegen);
    declare_testing_functions(codegen);
    declare_alloc_functions(codegen);
}
