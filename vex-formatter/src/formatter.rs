// Main formatter implementation

use crate::config::Config;
use crate::visitor::FormattingVisitor;
use anyhow::Result;
use vex_lexer::Lexer;
use vex_parser::Parser;

/// Code formatter
pub struct Formatter {
    config: Config,
}

impl Formatter {
    /// Create new formatter with configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Format source code
    pub fn format(&self, source: &str) -> Result<String> {
        // Parse source code to AST
        let mut parser = match Parser::new_with_file("formatter.vx", source) {
            Ok(p) => p,
            Err(e) => {
                return Err(anyhow::anyhow!("Lexer error: {}", e));
            }
        };

        let program = match parser.parse_file() {
            Ok(p) => p,
            Err(e) => {
                // Return original source if parse fails - don't format broken code
                return Err(anyhow::anyhow!("Parse error: {}", e));
            }
        };

        // Visit AST and format
        let mut visitor = FormattingVisitor::new(&self.config);
        visitor.visit_program(&program);

        Ok(visitor.output())
    }

    /// Format source code and return if it changed
    pub fn format_check(&self, source: &str) -> Result<bool> {
        let formatted = self.format(source)?;
        Ok(formatted != source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_creation() {
        let config = Config::default();
        let formatter = Formatter::new(config);
        assert_eq!(formatter.config.max_width, 100);
    }
}
