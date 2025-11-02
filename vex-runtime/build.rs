// Build script for vex-runtime
// Links external C libraries (simdutf)

fn main() {
    // Link simdutf library
    #[cfg(feature = "simdutf")]
    {
        println!("cargo:rustc-link-lib=simdutf");

        // Add library search paths for different platforms
        if cfg!(target_os = "linux") {
            println!("cargo:rustc-link-search=/usr/lib");
            println!("cargo:rustc-link-search=/usr/local/lib");
        } else if cfg!(target_os = "macos") {
            println!("cargo:rustc-link-search=/opt/homebrew/lib");
            println!("cargo:rustc-link-search=/usr/local/lib");
        } else if cfg!(target_os = "windows") {
            // vcpkg integration
            if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
                println!(
                    "cargo:rustc-link-search={}/installed/x64-windows/lib",
                    vcpkg_root
                );
            }
        }

        println!("cargo:rerun-if-changed=build.rs");
    }
}
