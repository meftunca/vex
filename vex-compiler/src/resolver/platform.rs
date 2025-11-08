/**
 * Platform Detection
 * Compile-time platform and architecture detection for stdlib file selection
 */
use std::fmt;

/// Supported operating systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    BSD,
}

/// Supported CPU architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X64,   // x86_64 / AMD64
    Arm64, // ARM64 / AArch64
    Arm32, // ARM32
}

impl Platform {
    /// Get the current platform at compile time
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
        return Platform::BSD;

        #[cfg(not(any(
            target_os = "linux",
            target_os = "macos",
            target_os = "windows",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd"
        )))]
        compile_error!("Unsupported target operating system");
    }

    /// Get platform as lowercase string for file selection
    /// Example: Platform::Linux -> "linux"
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOS => "macos",
            Platform::Windows => "windows",
            Platform::BSD => "bsd",
        }
    }

    /// All supported platforms (for testing/validation)
    pub fn all() -> &'static [Platform] {
        &[
            Platform::Linux,
            Platform::MacOS,
            Platform::Windows,
            Platform::BSD,
        ]
    }
}

impl Arch {
    /// Get the current architecture at compile time
    pub fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        return Arch::X64;

        #[cfg(target_arch = "aarch64")]
        return Arch::Arm64;

        #[cfg(target_arch = "arm")]
        return Arch::Arm32;

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
        compile_error!("Unsupported target architecture");
    }

    /// Get architecture as lowercase string for file selection
    /// Example: Arch::X64 -> "x64"
    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
            Arch::Arm32 => "arm32",
        }
    }

    /// All supported architectures (for testing/validation)
    pub fn all() -> &'static [Arch] {
        &[Arch::X64, Arch::Arm64, Arch::Arm32]
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Platform + Architecture combination for targeted file selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target {
    pub platform: Platform,
    pub arch: Arch,
}

impl Target {
    /// Get the current target (platform + architecture)
    pub fn current() -> Self {
        Self {
            platform: Platform::current(),
            arch: Arch::current(),
        }
    }

    /// Create a custom target (useful for cross-compilation)
    pub fn new(platform: Platform, arch: Arch) -> Self {
        Self { platform, arch }
    }

    /// Get target as string: "linux.x64", "macos.arm64", etc.
    pub fn as_str(&self) -> String {
        format!("{}.{}", self.platform.as_str(), self.arch.as_str())
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();
        println!("Current platform: {}", platform);
        assert!(Platform::all().contains(&platform));
    }

    #[test]
    fn test_arch_current() {
        let arch = Arch::current();
        println!("Current architecture: {}", arch);
        assert!(Arch::all().contains(&arch));
    }

    #[test]
    fn test_target_current() {
        let target = Target::current();
        println!("Current target: {}", target);
        assert_eq!(target.platform, Platform::current());
        assert_eq!(target.arch, Arch::current());
    }

    #[test]
    fn test_platform_string() {
        assert_eq!(Platform::Linux.as_str(), "linux");
        assert_eq!(Platform::MacOS.as_str(), "macos");
        assert_eq!(Platform::Windows.as_str(), "windows");
        assert_eq!(Platform::BSD.as_str(), "bsd");
    }

    #[test]
    fn test_arch_string() {
        assert_eq!(Arch::X64.as_str(), "x64");
        assert_eq!(Arch::Arm64.as_str(), "arm64");
        assert_eq!(Arch::Arm32.as_str(), "arm32");
    }

    #[test]
    fn test_target_string() {
        let target = Target::new(Platform::Linux, Arch::X64);
        assert_eq!(target.as_str(), "linux.x64");

        let target = Target::new(Platform::MacOS, Arch::Arm64);
        assert_eq!(target.as_str(), "macos.arm64");
    }

    #[test]
    fn test_display_traits() {
        let platform = Platform::Linux;
        let arch = Arch::X64;
        let target = Target::new(platform, arch);

        assert_eq!(format!("{}", platform), "linux");
        assert_eq!(format!("{}", arch), "x64");
        assert_eq!(format!("{}", target), "linux.x64");
    }
}
