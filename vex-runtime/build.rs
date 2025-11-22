// Build script for vex-runtime
// Compiles and links the C async runtime + simdutf

use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| format!("Failed to get CARGO_MANIFEST_DIR: {}", e))?;
    let c_dir = PathBuf::from(&manifest_dir).join("c");
    let async_io_dir = c_dir.join("async_runtime");

    // ========== Allocator Selection ==========
    let allocator = env::var("VEX_ALLOCATOR").unwrap_or_else(|_| "mimalloc".to_string());
    println!("cargo:warning=Building with allocator: {}", allocator);

    // Source files
    let sources = vec![
        async_io_dir.join("src/runtime.c"),
        async_io_dir.join("src/worker_context.c"),
        async_io_dir.join("src/lockfree_queue.c"),
        async_io_dir.join("src/common.c"),
        async_io_dir.join("src/task_pool.c"),
        async_io_dir.join("src/timer_heap.c"),
        c_dir.join("vex_args.c"), // Command-line arguments
        c_dir.join("vex_channel.c"),
        c_dir.join("vex_async.c"),        // Async runtime integration
        c_dir.join("vex_io.c"),           // Basic I/O functions (vex_eprintln, vex_sprintf)
        c_dir.join("vex_alloc.c"),        // Memory allocation
        c_dir.join("vex_memory.c"),       // Memory operations (vex_memcpy, etc.)
        c_dir.join("vex_error.c"),        // Error handling (vex_panic, etc.)
        c_dir.join("vex_array.c"),        // Array operations (fixed-size)
        c_dir.join("vex_vec.c"),          // Vec<T> dynamic array operations
        c_dir.join("vex_box.c"),          // Box<T> heap allocations
        c_dir.join("swisstable/vex_swisstable.c"), // HashMap<K,V> (Google Swiss Tables V1)
        // TODO: Re-enable v2/v3 when implementation is complete
        // c_dir.join("swisstable/vex_swisstable_v2.c"), // HashMap<K,V> V2 (2-3x faster, SIMD optimized)
        // c_dir.join("swisstable/vex_swisstable_v3.c"), // HashMap<K,V> V3 (experimental ultimate perf)
        c_dir.join("vex_set.c"),           // Set<T> operations
        c_dir.join("vex_string.c"),        // String operations
        c_dir.join("vex_string_type.c"),   // String type implementation
        c_dir.join("vex_strconv.c"),       // String<->Number conversions (to_string, parse)
        c_dir.join("vex_file.c"),          // File system operations
        c_dir.join("vex_display.c"),       // Display trait - type to string conversions
        c_dir.join("vex_value_helpers.c"), // VexValue constructor helpers
        c_dir.join("vex_format.c"),        // Format buffer for type-safe formatting
    ];

    // Detect platform and add appropriate poller
    let target_os = env::var("CARGO_CFG_TARGET_OS")
        .map_err(|e| format!("Failed to get CARGO_CFG_TARGET_OS: {}", e))?;
    let poller_source = match target_os.as_str() {
        "macos" | "ios" | "freebsd" | "openbsd" | "netbsd" | "dragonfly" => {
            async_io_dir.join("src/poller_kqueue.c")
        }
        "linux" | "android" => {
            // TODO: Detect kernel version for io_uring support
            async_io_dir.join("src/poller_epoll.c")
        }
        "windows" => async_io_dir.join("src/poller_iocp.c"),
        _ => return Err(format!("Unsupported target OS: {}", target_os).into()),
    };

    println!(
        "cargo:warning=Building async runtime with poller: {}",
        poller_source.display()
    );

    // Build C library
    let mut builder = cc::Build::new();
    builder
        .warnings(true)
        .extra_warnings(true)
        .include(async_io_dir.join("include"))
        .include(&c_dir) // For vex_channel.h
        .flag("-std=c11")
        .flag("-O2")
        .define("VEX_RUNTIME_INTEGRATED", None); // Enable vex_malloc/vex_free in async runtime

    // Define _GNU_SOURCE on Linux for pthread_barrier_t
    if target_os == "linux" || target_os == "android" {
        builder.define("_GNU_SOURCE", None);
    }

    // ========== Allocator Configuration ==========
    match allocator.as_str() {
        "mimalloc" => {
            let mimalloc_dir = c_dir.join("allocators/mimalloc");
            let mimalloc_src = mimalloc_dir.join("src/static.c");

            builder.define("VEX_USE_MIMALLOC", None);
            builder.include(mimalloc_dir.join("include"));
            builder.file(&mimalloc_src);

            println!(
                "cargo:warning=Using mimalloc from: {}",
                mimalloc_src.display()
            );
        }
        "system" => {
            println!("cargo:warning=Using system allocator (malloc/free)");
        }
        _ => {
            return Err(
                format!("Unknown allocator: {} (valid: mimalloc, system)", allocator).into(),
            );
        }
    }

    // Add pthread on Unix
    if target_os != "windows" {
        builder.flag("-pthread");
    }

    // Add atomic library on Linux (required for C11 stdatomic.h)
    if target_os == "linux" || target_os == "android" {
        println!("cargo:rustc-link-lib=atomic");
    }

    // Add all source files
    for source in sources {
        builder.file(source);
    }
    builder.file(poller_source);

    // Compile into libvex_runtime.a
    builder.compile("vex_runtime");

    // --- Linker Configuration ---

    let out_dir = env::var("OUT_DIR").map_err(|e| format!("Failed to get OUT_DIR: {}", e))?;

    // 1. Instruct Cargo how to link the `vex` binary itself (for `cargo test`, etc.).
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=vex_runtime");

    // 2. Save linker arguments for the CLI (`vex compile` / `vex run`) to use.
    let linker_args_path = PathBuf::from(&out_dir).join("vex_linker_args.txt");
    println!("cargo:warning=OUT_DIR is: {}", out_dir);
    println!(
        "cargo:warning=Will write linker args to: {}",
        linker_args_path.display()
    );

    let mut linker_args = format!("-L{} -lvex_runtime", out_dir);

    // Add platform-specific libraries.
    if target_os != "windows" {
        println!("cargo:rustc-link-lib=pthread");
        linker_args.push_str(" -lpthread");
    }
    // NOTE: Add other libs like -ldl, -lrt for Linux if needed later.

    println!("cargo:warning=Linker args content: {}", linker_args);
    std::fs::write(&linker_args_path, &linker_args).unwrap_or_else(|e| {
        panic!(
            "Failed to write linker args to {}: {}",
            linker_args_path.display(),
            e
        )
    });
    println!("cargo:warning=Successfully wrote linker args file.");

    // 3. Expose the output directory to the `vex-runtime` crate so it can find the args file.
    println!("cargo:rustc-env=VEX_RUNTIME_OUT_DIR={}", out_dir);

    // Link simdutf library if feature enabled
    #[cfg(feature = "simdutf")]
    {
        println!("cargo:rustc-link-lib=simdutf");

        // Add library search paths for different platforms
        if target_os == "linux" {
            println!("cargo:rustc-link-search=/usr/lib");
            println!("cargo:rustc-link-search=/usr/local/lib");
        } else if target_os == "macos" {
            println!("cargo:rustc-link-search=/opt/homebrew/lib");
            println!("cargo:rustc-link-search=/usr/local/lib");
        } else if target_os == "windows" {
            // vcpkg integration
            if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
                println!(
                    "cargo:rustc-link-search={}/installed/x64-windows/lib",
                    vcpkg_root
                );
            }
        }
    }

    // Tell cargo to rerun if sources change
    println!("cargo:rerun-if-changed=c/async_runtime/src");
    println!("cargo:rerun-if-changed=c/async_runtime/include");
    println!("cargo:rerun-if-changed=c/vex_channel.h");
    println!("cargo:rerun-if-changed=c/vex_channel.c");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
