// vex-formatter - Code formatter for Vex language
// Provides auto-formatting with configurable rules

pub mod config;
pub mod formatter;
pub mod rules;
pub mod visitor;

pub use config::{BraceStyle, Config, QuoteStyle, TrailingComma};
pub use formatter::Formatter;

use anyhow::Result;
use std::path::Path;

/// Format a Vex source file
pub fn format_file<P: AsRef<Path>>(path: P, config: &Config) -> Result<String> {
    let source = std::fs::read_to_string(path.as_ref())?;
    format_source(&source, config)
}

/// Format Vex source code string
pub fn format_source(source: &str, config: &Config) -> Result<String> {
    let formatter = Formatter::new(config.clone());
    formatter.format(source)
}

/// Format source code with default configuration
pub fn format_with_defaults(source: &str) -> Result<String> {
    format_source(source, &Config::default())
}
