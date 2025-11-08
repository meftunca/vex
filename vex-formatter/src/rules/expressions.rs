// Expression formatting rules

/// Check if expression should be split across multiple lines
pub fn should_split_expression(expr: &str, max_width: usize) -> bool {
    expr.len() > max_width
}

/// Format chained method calls
pub fn format_method_chain(chain: &[String], indent: usize) -> String {
    if chain.is_empty() {
        return String::new();
    }

    let mut result = chain[0].clone();

    for method in &chain[1..] {
        result.push('\n');
        result.push_str(&" ".repeat(indent));
        result.push('.');
        result.push_str(method);
    }

    result
}

/// Format array/slice literals
pub fn format_array(elements: &[String], max_width: usize, indent: usize) -> String {
    let single_line = format!("[{}]", elements.join(", "));

    if single_line.len() <= max_width {
        return single_line;
    }

    // Multi-line format
    let mut result = String::from("[\n");
    for (i, elem) in elements.iter().enumerate() {
        result.push_str(&" ".repeat(indent + 4));
        result.push_str(elem);
        if i < elements.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }
    result.push_str(&" ".repeat(indent));
    result.push(']');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_split_expression() {
        assert!(!should_split_expression("x + y", 100));
        assert!(should_split_expression(&"x".repeat(150), 100));
    }

    #[test]
    fn test_format_method_chain() {
        let chain = vec![
            "obj".to_string(),
            "method1()".to_string(),
            "method2()".to_string(),
        ];
        let result = format_method_chain(&chain, 4);
        assert!(result.contains(".method1()"));
        assert!(result.contains(".method2()"));
    }

    #[test]
    fn test_format_array_single_line() {
        let elements = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let result = format_array(&elements, 100, 0);
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_format_array_multi_line() {
        let elements = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let result = format_array(&elements, 5, 0);
        assert!(result.contains("[\n"));
        assert!(result.contains("\n]"));
    }
}
