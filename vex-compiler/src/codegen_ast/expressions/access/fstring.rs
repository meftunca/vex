// F-string interpolation

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;
use vex_ast::Expression;

impl<'ctx> ASTCodeGen<'ctx> {
    /// Compile F-string with interpolation
    /// Format: f"text {expr} more text {expr2}"
    pub(crate) fn compile_fstring(
        &mut self,
        template: &str,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        // Parse the F-string template to extract text parts and expressions
        // For now, implement a simple version that handles {var_name} placeholders

        enum FStringPart {
            Text(String),
            Expr(String),
        }

        let mut result_parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Check if it's an escaped brace {{
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume second {
                    current_text.push('{');
                    continue;
                }

                // Save current text if any
                if !current_text.is_empty() {
                    result_parts.push(FStringPart::Text(current_text.clone()));
                    current_text.clear();
                }

                // Extract expression until }
                let mut expr_str = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    expr_str.push(ch);
                }

                result_parts.push(FStringPart::Expr(expr_str));
            } else if ch == '}' {
                // Check if it's an escaped brace }}
                if chars.peek() == Some(&'}') {
                    chars.next(); // consume second }
                    current_text.push('}');
                    continue;
                }
                current_text.push(ch);
            } else {
                current_text.push(ch);
            }
        }

        // Add remaining text
        if !current_text.is_empty() {
            result_parts.push(FStringPart::Text(current_text));
        }

        // For now, if there are no interpolations, just return as a regular string
        if result_parts
            .iter()
            .all(|p| matches!(p, FStringPart::Text(_)))
        {
            let full_text: String = result_parts
                .iter()
                .filter_map(|p| {
                    if let FStringPart::Text(s) = p {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            let global_str = self
                .builder
                .build_global_string_ptr(&full_text, "fstr")
                .map_err(|e| format!("Failed to create F-string: {}", e))?;
            return Ok(global_str.as_pointer_value().into());
        }

        // Implement full F-string interpolation
        // Process each part: text → string literal, expr → evaluate + convert to string
        let mut string_parts: Vec<BasicValueEnum<'ctx>> = Vec::new();

        for part in result_parts {
            match part {
                FStringPart::Text(text) => {
                    // Create string literal for text parts
                    let str_ptr = self
                        .builder
                        .build_global_string_ptr(&text, "fstr_text")
                        .map_err(|e| format!("Failed to create F-string text part: {}", e))?;
                    string_parts.push(str_ptr.as_pointer_value().into());
                }
                FStringPart::Expr(expr_str) => {
                    // For now, only support simple variable references
                    // Full expression parsing would require parser integration
                    let trimmed = expr_str.trim();

                    // Try to compile as identifier (variable reference)
                    let value = match self
                        .compile_expression(&Expression::Ident(trimmed.to_string()))
                    {
                        Ok(v) => v,
                        Err(_) => {
                            // If identifier fails, return error message
                            return Err(format!(
                                "F-string interpolation only supports simple variables for now, got: '{}'",
                                trimmed
                            ));
                        }
                    };

                    // Convert value to string based on its type
                    let str_value = self.convert_value_to_string(value)?;
                    string_parts.push(str_value);
                }
            }
        }

        // Concatenate all string parts using vex_strcat_new
        if string_parts.is_empty() {
            // Empty F-string - return empty string
            let empty_str = self
                .builder
                .build_global_string_ptr("", "fstr_empty")
                .map_err(|e| format!("Failed to create empty F-string: {}", e))?;
            return Ok(empty_str.as_pointer_value().into());
        }

        if string_parts.len() == 1 {
            // Single part - no concatenation needed
            return Ok(string_parts[0]);
        }

        // Get vex_strcat_new function (concatenates two strings)
        let strcat_fn = self.get_or_declare_vex_strcat_new()?;

        // Concatenate parts iteratively: result = part0 + part1 + part2 + ...
        let mut result = string_parts[0].into_pointer_value();
        for part in &string_parts[1..] {
            let part_ptr = part.into_pointer_value();
            let concat_result = self
                .builder
                .build_call(strcat_fn, &[result.into(), part_ptr.into()], "fstr_concat")
                .map_err(|e| format!("Failed to concatenate F-string parts: {}", e))?;

            result = concat_result
                .try_as_basic_value()
                .left()
                .ok_or("vex_strcat_new returned void")?
                .into_pointer_value();
        }

        Ok(result.into())
    }
}
