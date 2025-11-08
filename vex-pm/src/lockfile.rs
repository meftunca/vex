// Lock file management (vex.lock)

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::cache::Cache;
use crate::manifest::Manifest;
use crate::resolver::PackageVersion;

/// Lock file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: u32,

    #[serde(rename = "lockTime")]
    pub lock_time: String,

    pub dependencies: HashMap<String, LockedPackage>,
}

/// Locked package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub version: String,
    pub resolved: String,
    pub integrity: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<HashMap<String, String>>,
}

impl LockFile {
    /// Create a new empty lock file
    pub fn new() -> Self {
        Self {
            version: 1,
            lock_time: Utc::now().to_rfc3339(),
            dependencies: HashMap::new(),
        }
    }

    /// Load lock file from disk
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.as_ref().display()))?;

        Self::from_str(&content)
    }

    /// Parse lock file from string
    pub fn from_str(content: &str) -> Result<Self> {
        let lockfile: LockFile =
            serde_json::from_str(content).context("Failed to parse vex.lock")?;

        Ok(lockfile)
    }

    /// Save lock file to disk
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize lock file")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.as_ref().display()))?;

        Ok(())
    }

    /// Generate lock file from manifest and resolved dependencies
    pub fn generate(
        manifest: &Manifest,
        resolved: &[PackageVersion],
        cache: &Cache,
    ) -> Result<Self> {
        let mut lockfile = Self::new();

        for pkg in resolved {
            let git_cache_path = cache.git_dir().join(&pkg.name);

            // Calculate integrity hash
            let integrity = if git_cache_path.exists() {
                format!("sha256:{}", Cache::hash_directory(&git_cache_path)?)
            } else {
                "sha256:unknown".to_string()
            };

            // Build resolved URL
            let resolved_url = format!(
                "https://github.com/{}/archive/{}.tar.gz",
                pkg.name, pkg.version
            );

            let locked = LockedPackage {
                version: pkg.version.clone(),
                resolved: resolved_url,
                integrity,
                dependencies: None,
                platform: None,
            };

            lockfile.dependencies.insert(pkg.name.clone(), locked);
        }

        Ok(lockfile)
    }

    /// Validate lock file integrity
    pub fn validate(&self, cache: &Cache) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        for (name, locked) in &self.dependencies {
            let git_cache_path = cache.git_dir().join(name);

            if !git_cache_path.exists() {
                errors.push(format!("Package not in cache: {}", name));
                continue;
            }

            // Verify integrity hash
            let expected_hash = locked
                .integrity
                .strip_prefix("sha256:")
                .unwrap_or(&locked.integrity);

            match Cache::hash_directory(&git_cache_path) {
                Ok(actual_hash) => {
                    if expected_hash != "unknown" && actual_hash != expected_hash {
                        errors.push(format!(
                            "Integrity mismatch for {}: expected {}, got {}",
                            name, expected_hash, actual_hash
                        ));
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to hash {}: {}", name, e));
                }
            }
        }

        Ok(errors)
    }

    /// Check if lock file exists
    pub fn exists() -> bool {
        Path::new("vex.lock").exists()
    }

    /// Add a locked package
    pub fn add_package(&mut self, name: String, package: LockedPackage) {
        self.dependencies.insert(name, package);
        self.lock_time = Utc::now().to_rfc3339();
    }

    /// Remove a locked package
    pub fn remove_package(&mut self, name: &str) -> Option<LockedPackage> {
        let result = self.dependencies.remove(name);
        if result.is_some() {
            self.lock_time = Utc::now().to_rfc3339();
        }
        result
    }

    /// Get package by name
    pub fn get_package(&self, name: &str) -> Option<&LockedPackage> {
        self.dependencies.get(name)
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_creation() {
        let lockfile = LockFile::new();
        assert_eq!(lockfile.version, 1);
        assert!(lockfile.dependencies.is_empty());
    }

    #[test]
    fn test_lockfile_serialization() {
        let mut lockfile = LockFile::new();
        lockfile.add_package(
            "github.com/user/repo".to_string(),
            LockedPackage {
                version: "v1.0.0".to_string(),
                resolved: "https://github.com/user/repo/archive/v1.0.0.tar.gz".to_string(),
                integrity: "sha256:abc123".to_string(),
                dependencies: None,
                platform: None,
            },
        );

        let json = serde_json::to_string_pretty(&lockfile).unwrap();
        assert!(json.contains("github.com/user/repo"));
        assert!(json.contains("v1.0.0"));
    }
}
