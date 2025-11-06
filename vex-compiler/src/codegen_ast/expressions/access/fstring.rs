// F-string interpolation

use crate::codegen_ast::ASTCodeGen;
use inkwell::values::BasicValueEnum;

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

        // TODO: For now, F-strings with interpolation are not fully supported
        // We would need to:
        // 1. Parse each {expression} as a Vex expression
        // 2. Evaluate each expression
        // 3. Convert each result to string (call to_string methods or format functions)
        // 4. Concatenate all parts

        // For now, just return a placeholder string indicating interpolation is needed
        let placeholder = format!("f\"{}\" (interpolation not yet implemented)", template);
        let global_str = self
            .builder
            .build_global_string_ptr(&placeholder, "fstr_placeholder")
            .map_err(|e| format!("Failed to create F-string placeholder: {}", e))?;
        Ok(global_str.as_pointer_value().into())
    }
}
