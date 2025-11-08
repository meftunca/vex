// Configuration for Vex formatter

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Formatter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum line width
    #[serde(default = "default_max_width")]
    pub max_width: usize,

    /// Number of spaces for indentation
    #[serde(default = "default_indent_size")]
    pub indent_size: usize,

    /// Brace style
    #[serde(default)]
    pub brace_style: BraceStyle,

    /// Trailing comma style
    #[serde(default)]
    pub trailing_comma: TrailingComma,

    /// Sort imports alphabetically
    #[serde(default = "default_true")]
    pub import_sort: bool,

    /// Add space after colon in type annotations
    #[serde(default = "default_true")]
    pub space_after_colon: bool,

    /// Add space around binary operators
    #[serde(default = "default_true")]
    pub space_around_operators: bool,

    /// Format strings with consistent quote style
    #[serde(default)]
    pub quote_style: QuoteStyle,

    /// Keep empty lines (max consecutive)
    #[serde(default = "default_max_blank_lines")]
    pub max_blank_lines: usize,
}

/// Brace placement style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BraceStyle {
    /// Same line: `fn main() {`
    SameLine,
    /// Next line: `fn main()\n{`
    NextLine,
}

/// Trailing comma style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailingComma {
    /// Always add trailing comma
    Always,
    /// Never add trailing comma
    Never,
    /// Add trailing comma only for multiline
    Multiline,
}

/// Quote style for strings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteStyle {
    /// Double quotes (default)
    Double,
    /// Single quotes
    Single,
    /// Preserve original
    Preserve,
}

// Default values
fn default_max_width() -> usize {
    100
}
fn default_indent_size() -> usize {
    4
}
fn default_true() -> bool {
    true
}
fn default_max_blank_lines() -> usize {
    2
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_width: default_max_width(),
            indent_size: default_indent_size(),
            brace_style: BraceStyle::SameLine,
            trailing_comma: TrailingComma::Multiline,
            import_sort: true,
            space_after_colon: true,
            space_around_operators: true,
            quote_style: QuoteStyle::Double,
            max_blank_lines: default_max_blank_lines(),
        }
    }
}

impl Default for BraceStyle {
    fn default() -> Self {
        BraceStyle::SameLine
    }
}

impl Default for TrailingComma {
    fn default() -> Self {
        TrailingComma::Multiline
    }
}

impl Default for QuoteStyle {
    fn default() -> Self {
        QuoteStyle::Double
    }
}

impl Config {
    /// Load configuration from vexfmt.json file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from directory (searches for vexfmt.json)
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let config_path = dir.as_ref().join("vexfmt.json");

        if config_path.exists() {
            Self::from_file(config_path)
        } else {
            // Look in parent directories
            let mut current = dir.as_ref();
            while let Some(parent) = current.parent() {
                let config_path = parent.join("vexfmt.json");
                if config_path.exists() {
                    return Self::from_file(config_path);
                }
                current = parent;
            }

            // No config found, use defaults
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Create example configuration file
    pub fn example() -> String {
        serde_json::to_string_pretty(&Self::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.max_width, 100);
        assert_eq!(config.indent_size, 4);
        assert_eq!(config.brace_style, BraceStyle::SameLine);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.max_width, deserialized.max_width);
        assert_eq!(config.indent_size, deserialized.indent_size);
    }

    #[test]
    fn test_example_config() {
        let example = Config::example();
        assert!(example.contains("max_width"));
        assert!(example.contains("indent_size"));
        assert!(example.contains("brace_style"));
    }
}
