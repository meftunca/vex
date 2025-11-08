// Indentation rules

/// Calculate indentation for a given nesting level
pub fn indent_string(level: usize, indent_size: usize) -> String {
    " ".repeat(level * indent_size)
}

/// Check if line needs indentation increase
pub fn should_indent(line: &str) -> bool {
    line.trim_end().ends_with('{')
}

/// Check if line needs indentation decrease
pub fn should_dedent(line: &str) -> bool {
    line.trim_start().starts_with('}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_string() {
        assert_eq!(indent_string(0, 4), "");
        assert_eq!(indent_string(1, 4), "    ");
        assert_eq!(indent_string(2, 4), "        ");
    }

    #[test]
    fn test_should_indent() {
        assert!(should_indent("fn main() {"));
        assert!(should_indent("if x > 0 {"));
        assert!(!should_indent("let x = 1;"));
    }

    #[test]
    fn test_should_dedent() {
        assert!(should_dedent("}"));
        assert!(should_dedent("    }"));
        assert!(!should_dedent("let x = 1;"));
    }
}
