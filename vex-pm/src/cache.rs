// Global cache management

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Global cache manager
pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    /// Create a new cache instance
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

        Ok(Self { cache_dir })
    }

    /// Get cache directory path
    fn get_cache_dir() -> Result<PathBuf> {
        if let Ok(vex_cache) = std::env::var("VEX_CACHE") {
            return Ok(PathBuf::from(vex_cache));
        }

        if let Some(home) = dirs::home_dir() {
            return Ok(home.join(".vex").join("cache"));
        }

        anyhow::bail!("Cannot determine home directory")
    }

    /// Get cache directory for packages
    pub fn packages_dir(&self) -> PathBuf {
        self.cache_dir.join("packages")
    }

    /// Get cache directory for git repositories
    pub fn git_dir(&self) -> PathBuf {
        self.cache_dir.join("git")
    }

    /// Calculate SHA-256 hash of a file
    pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
        let content = fs::read(&path)
            .with_context(|| format!("Failed to read file: {}", path.as_ref().display()))?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Calculate SHA-256 hash of a directory (all files)
    pub fn hash_directory<P: AsRef<Path>>(path: P) -> Result<String> {
        let mut hasher = Sha256::new();

        // Walk directory in sorted order for reproducibility
        let mut entries: Vec<_> = fs::read_dir(&path)
            .with_context(|| format!("Failed to read directory: {}", path.as_ref().display()))?
            .collect::<Result<Vec<_>, _>>()?;

        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                let content = fs::read(&path)?;
                hasher.update(&content);
            } else if path.is_dir() {
                let dir_hash = Self::hash_directory(&path)?;
                hasher.update(dir_hash.as_bytes());
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Check if package exists in cache
    pub fn has_package(&self, name: &str, version: &str) -> bool {
        let package_dir = self.packages_dir().join(name).join(version);
        package_dir.exists()
    }

    /// Get package path in cache
    pub fn package_path(&self, name: &str, version: &str) -> PathBuf {
        self.packages_dir().join(name).join(version)
    }

    /// Clean cache (remove all cached packages)
    pub fn clean(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).context("Failed to remove cache directory")?;
        }

        fs::create_dir_all(&self.cache_dir).context("Failed to recreate cache directory")?;

        Ok(())
    }

    /// Get cache size in bytes
    pub fn size(&self) -> Result<u64> {
        let mut total = 0u64;

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let metadata = entry.metadata()?;

                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += Self::dir_size(&entry.path())?;
                }
            }
        }

        Ok(total)
    }

    /// Calculate directory size recursively
    fn dir_size(path: &Path) -> Result<u64> {
        let mut total = 0u64;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                total += metadata.len();
            } else if metadata.is_dir() {
                total += Self::dir_size(&entry.path())?;
            }
        }

        Ok(total)
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new().expect("Failed to create cache")
    }
}

/// Format bytes as human-readable size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = Cache::new().unwrap();
        assert!(cache.cache_dir.exists());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(2 * 1024 * 1024), "2.00 MB");
        assert_eq!(format_size(3 * 1024 * 1024 * 1024), "3.00 GB");
    }
}
