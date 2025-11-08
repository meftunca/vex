// Platform detection and file selection

use std::env;
use std::path::{Path, PathBuf};

/// Platform information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    pub os: String,
    pub arch: String,
}

impl Platform {
    /// Detect current platform
    pub fn detect() -> Self {
        let os = detect_os();
        let arch = detect_arch();

        Self { os, arch }
    }

    /// Parse platform from target string (e.g., "linux-x64", "wasm")
    pub fn from_target(target: &str) -> Self {
        let parts: Vec<&str> = target.split('-').collect();

        match parts.len() {
            1 => {
                // Single part: could be OS or arch
                let part = parts[0];
                if is_os(part) {
                    Self {
                        os: part.to_string(),
                        arch: detect_arch(),
                    }
                } else {
                    Self {
                        os: detect_os(),
                        arch: part.to_string(),
                    }
                }
            }
            2 => {
                // Two parts: os-arch
                Self {
                    os: parts[0].to_string(),
                    arch: parts[1].to_string(),
                }
            }
            _ => {
                // Invalid format, use current platform
                Self::detect()
            }
        }
    }

    /// Get target triple string
    pub fn to_target_string(&self) -> String {
        format!("{}-{}", self.os, self.arch)
    }
}

/// Select platform-specific file
///
/// Priority:
/// 1. {file}.testing.vx (if is_test = true)
/// 2. {file}.{os}.{arch}.vx
/// 3. {file}.{arch}.vx
/// 4. {file}.{os}.vx
/// 5. {file}.vx (fallback)
pub fn select_platform_file<P: AsRef<Path>>(base_path: P, platform: &Platform) -> PathBuf {
    select_platform_file_internal(base_path, platform, false)
}

/// Select platform-specific file with test mode support
pub fn select_platform_file_for_test<P: AsRef<Path>>(
    base_path: P,
    platform: &Platform,
    is_test: bool,
) -> PathBuf {
    select_platform_file_internal(base_path, platform, is_test)
}

fn select_platform_file_internal<P: AsRef<Path>>(
    base_path: P,
    platform: &Platform,
    is_test: bool,
) -> PathBuf {
    let base = base_path.as_ref();
    let base_str = base.to_string_lossy();

    // Remove .vx extension if present
    let base_without_ext = base_str.strip_suffix(".vx").unwrap_or(&base_str);

    // Build candidate paths in priority order
    let mut candidates = Vec::new();

    // 1. Test variant (highest priority in test mode)
    if is_test {
        candidates.push(format!("{}.testing.vx", base_without_ext));
    }

    // 2. OS + arch specific
    candidates.push(format!(
        "{}.{}.{}.vx",
        base_without_ext, platform.os, platform.arch
    ));

    // 3. Arch only
    candidates.push(format!("{}.{}.vx", base_without_ext, platform.arch));

    // 4. OS only
    candidates.push(format!("{}.{}.vx", base_without_ext, platform.os));

    // 5. Generic fallback
    let fallback = format!("{}.vx", base_without_ext);
    candidates.push(fallback.clone());

    // Return first existing file
    for candidate in &candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return path;
        }
    }

    // Fallback to generic file
    PathBuf::from(fallback)
}
/// Detect operating system
fn detect_os() -> String {
    if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "freebsd") {
        "freebsd".to_string()
    } else if cfg!(target_os = "openbsd") {
        "openbsd".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Detect CPU architecture
fn detect_arch() -> String {
    if cfg!(target_arch = "x86_64") {
        "x64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "arm64".to_string()
    } else if cfg!(target_arch = "wasm32") {
        "wasm".to_string()
    } else if cfg!(target_arch = "riscv64") {
        "riscv64".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Check if string is a known OS
fn is_os(s: &str) -> bool {
    matches!(s, "linux" | "macos" | "windows" | "freebsd" | "openbsd")
}

/// Get platform display name
pub fn platform_display_name(platform: &Platform) -> String {
    let os_name = match platform.os.as_str() {
        "linux" => "Linux",
        "macos" => "macOS",
        "windows" => "Windows",
        "freebsd" => "FreeBSD",
        "openbsd" => "OpenBSD",
        _ => &platform.os,
    };

    let arch_name = match platform.arch.as_str() {
        "x64" => "x86-64",
        "arm64" => "ARM64",
        "wasm" => "WebAssembly",
        "wasi" => "WASI",
        "riscv64" => "RISC-V 64",
        _ => &platform.arch,
    };

    format!("{} {}", os_name, arch_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
    }

    #[test]
    fn test_platform_from_target() {
        let platform = Platform::from_target("linux-x64");
        assert_eq!(platform.os, "linux");
        assert_eq!(platform.arch, "x64");

        let platform = Platform::from_target("wasm");
        assert_eq!(platform.arch, "wasm");
    }

    #[test]
    fn test_file_selection_priority() {
        // Create temporary test files
        let temp_dir = std::env::temp_dir();
        let test_base = temp_dir.join("test_select");

        // Create test files
        let files = vec![
            format!("{}.vx", test_base.display()),
            format!("{}.linux.vx", test_base.display()),
            format!("{}.x64.vx", test_base.display()),
            format!("{}.linux.x64.vx", test_base.display()),
        ];

        for file in &files {
            fs::write(file, "test").ok();
        }

        // Test priority
        let platform = Platform {
            os: "linux".to_string(),
            arch: "x64".to_string(),
        };

        let selected = select_platform_file(&test_base, &platform);
        assert!(selected.to_string_lossy().contains("linux.x64.vx"));

        // Clean up
        for file in &files {
            fs::remove_file(file).ok();
        }
    }
}
