// Spacing rules

/// Add space around binary operators
pub fn format_binary_op(left: &str, op: &str, right: &str) -> String {
    format!("{} {} {}", left, op, right)
}

/// Add space after colon in type annotations
pub fn format_type_annotation(name: &str, typ: &str) -> String {
    format!("{}: {}", name, typ)
}

/// Format function parameters with proper spacing
pub fn format_parameters(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(name, typ)| format_type_annotation(name, typ))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_binary_op() {
        assert_eq!(format_binary_op("x", "+", "y"), "x + y");
        assert_eq!(format_binary_op("a", "==", "b"), "a == b");
    }

    #[test]
    fn test_format_type_annotation() {
        assert_eq!(format_type_annotation("x", "i32"), "x: i32");
        assert_eq!(format_type_annotation("name", "string"), "name: string");
    }

    #[test]
    fn test_format_parameters() {
        let params = vec![
            ("x".to_string(), "i32".to_string()),
            ("y".to_string(), "i32".to_string()),
        ];
        assert_eq!(format_parameters(&params), "x: i32, y: i32");
    }
}
