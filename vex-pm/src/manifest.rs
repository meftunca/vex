// Manifest parser - vex.json

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Main manifest structure (vex.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets: Option<TargetConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<HashMap<String, Profile>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vex: Option<VexConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing: Option<TestingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<NativeConfig>,
}

/// Native C/C++ library configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeConfig {
    /// Dynamic libraries to link (e.g., ["ssl", "crypto"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub libraries: Vec<String>,

    /// Library search paths (e.g., ["/usr/local/lib", "/opt/homebrew/lib"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub search_paths: Vec<String>,

    /// Static library files (e.g., ["./libs/libmylib.a"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub static_libs: Vec<String>,

    /// C/C++ source files to compile (e.g., ["./native/helper.c"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,

    /// Compiler flags for C/C++ compilation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cflags: Vec<String>,

    /// Include directories for C/C++ headers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_dirs: Vec<String>,
}

/// Testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingConfig {
    /// Test directory (default: "tests")
    #[serde(default = "default_test_dir")]
    pub dir: String,

    /// Test file pattern (default: "*.test.vx")
    #[serde(default = "default_test_pattern")]
    pub pattern: String,

    /// Test timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    /// Run tests in parallel
    #[serde(default = "default_parallel")]
    pub parallel: bool,
}

fn default_test_dir() -> String {
    "tests".to_string()
}

fn default_test_pattern() -> String {
    "**/*.test.vx".to_string()
}

fn default_parallel() -> bool {
    true
}

impl Default for TestingConfig {
    fn default() -> Self {
        Self {
            dir: default_test_dir(),
            pattern: default_test_pattern(),
            timeout: None,
            parallel: true,
        }
    }
}

/// Dependency can be a simple version string or detailed config
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed {
        version: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },
}

/// Target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub default: String,
    pub supported: Vec<String>,
}

/// Build profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(rename = "optimizationLevel")]
    pub optimization_level: u8,

    #[serde(rename = "debugSymbols")]
    pub debug_symbols: bool,

    #[serde(skip_serializing_if = "Option::is_none", rename = "memProfiling")]
    pub mem_profiling: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "cpuProfiling")]
    pub cpu_profiling: Option<bool>,
}

/// Vex-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexConfig {
    #[serde(rename = "borrowChecker")]
    pub borrow_checker: String,
}

impl Manifest {
    /// Parse vex.json from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;

        Self::from_str(&content)
    }

    /// Parse vex.json from string
    pub fn from_str(content: &str) -> Result<Self> {
        let manifest: Manifest =
            serde_json::from_str(content).context("Failed to parse vex.json")?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Write manifest to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self).context("Failed to serialize manifest")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Validate manifest
    fn validate(&self) -> Result<()> {
        // Validate name
        if self.name.is_empty() {
            anyhow::bail!("Package name cannot be empty");
        }

        // Validate version (basic semver check)
        if !is_valid_semver(&self.version) {
            anyhow::bail!("Invalid version format: {}", self.version);
        }

        // Validate dependencies
        for (name, dep) in &self.dependencies {
            match dep {
                Dependency::Simple(version) => {
                    if !is_valid_version_spec(version) {
                        anyhow::bail!("Invalid version for {}: {}", name, version);
                    }
                }
                Dependency::Detailed { version, .. } => {
                    if !is_valid_version_spec(version) {
                        anyhow::bail!("Invalid version for {}: {}", name, version);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get main entrypoint (default: src/lib.vx)
    pub fn get_main(&self) -> String {
        self.main
            .clone()
            .unwrap_or_else(|| "src/lib.vx".to_string())
    }

    /// Get profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.as_ref()?.get(name)
    }

    /// Get default target
    pub fn get_default_target(&self) -> String {
        self.targets
            .as_ref()
            .map(|t| t.default.clone())
            .unwrap_or_else(|| "x64".to_string())
    }

    /// Get native configuration
    pub fn get_native(&self) -> Option<&NativeConfig> {
        self.native.as_ref()
    }

    /// Get testing configuration
    pub fn get_testing(&self) -> TestingConfig {
        self.testing.clone().unwrap_or_default()
    }
}

/// Check if version is valid semver
fn is_valid_semver(version: &str) -> bool {
    let version = version.strip_prefix('v').unwrap_or(version);
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

/// Check if version spec is valid (semver or "latest")
fn is_valid_version_spec(spec: &str) -> bool {
    if spec == "latest" {
        return true;
    }

    is_valid_semver(spec)
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            name: "my-package".to_string(),
            version: "0.1.0".to_string(),
            description: Some("A Vex project".to_string()),
            authors: Some(vec!["Your Name <email@example.com>".to_string()]),
            license: Some("MIT".to_string()),
            repository: None,
            dependencies: HashMap::new(),
            targets: Some(TargetConfig {
                default: "x64".to_string(),
                supported: vec!["x64".to_string(), "arm64".to_string()],
            }),
            profiles: Some({
                let mut profiles = HashMap::new();
                profiles.insert(
                    "development".to_string(),
                    Profile {
                        optimization_level: 0,
                        debug_symbols: true,
                        mem_profiling: None,
                        cpu_profiling: None,
                    },
                );
                profiles.insert(
                    "production".to_string(),
                    Profile {
                        optimization_level: 3,
                        debug_symbols: false,
                        mem_profiling: None,
                        cpu_profiling: None,
                    },
                );
                profiles
            }),
            vex: Some(VexConfig {
                borrow_checker: "strict".to_string(),
            }),
            main: None,
            bin: None,
            testing: None,
            native: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_manifest() {
        let json = r#"{
            "name": "test-pkg",
            "version": "1.0.0",
            "dependencies": {
                "github.com/user/repo": "v1.2.0"
            }
        }"#;

        let manifest = Manifest::from_str(json).unwrap();
        assert_eq!(manifest.name, "test-pkg");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.dependencies.len(), 1);
    }

    #[test]
    fn test_version_validation() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("v1.0.0"));
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("invalid"));
    }
}
