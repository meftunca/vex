// Build script for vex-runtime
// Compiles and links the C async runtime + simdutf

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let async_io_dir = PathBuf::from(&manifest_dir).join("c/vex_async_io");

    // Source files
    let sources = vec![
        "src/runtime.c",
        "src/worker_context.c",
        "src/lockfree_queue.c",
        "src/common.c",
    ];

    // Detect platform and add appropriate poller
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let poller_source = match target_os.as_str() {
        "macos" | "ios" | "freebsd" | "openbsd" | "netbsd" | "dragonfly" => "src/poller_kqueue.c",
        "linux" | "android" => {
            // TODO: Detect kernel version for io_uring support
            "src/poller_epoll.c"
        }
        "windows" => "src/poller_iocp.c",
        _ => panic!("Unsupported target OS: {}", target_os),
    };

    println!(
        "cargo:warning=Building async runtime with poller: {}",
        poller_source
    );

    // Build C library
    let mut builder = cc::Build::new();
    builder
        .warnings(true)
        .extra_warnings(true)
        .include(async_io_dir.join("include"))
        .flag("-std=c11")
        .flag("-O2");

    // Add pthread on Unix
    if target_os != "windows" {
        builder.flag("-pthread");
    }

    // Add all source files
    for source in sources {
        builder.file(async_io_dir.join(source));
    }
    builder.file(async_io_dir.join(poller_source));

    // Compile
    builder.compile("vex_async_runtime");

    // Link pthread on Unix
    if target_os != "windows" {
        println!("cargo:rustc-link-lib=pthread");
    }

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
    println!("cargo:rerun-if-changed=c/vex_async_io/src");
    println!("cargo:rerun-if-changed=c/vex_async_io/include");
    println!("cargo:rerun-if-changed=build.rs");
}
