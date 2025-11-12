//! This module contains the helper functions for the language features.

use tower_lsp::lsp_types::*;

// Helper struct for completion context
#[derive(Debug)]
pub struct CompletionContext {
    pub before_cursor: String,
    pub after_dot: bool,
}

pub fn get_word_at_position(text: &str, position: Position) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = position.line as usize;
    let char_idx = position.character as usize;

    if line_idx >= lines.len() {
        return String::new();
    }

    let line = lines[line_idx];
    if char_idx >= line.len() {
        return String::new();
    }

    // Find word boundaries
    let mut start = char_idx;
    let mut end = char_idx;

    // Move start backwards to find word start
    while start > 0 && line.chars().nth(start - 1).unwrap_or(' ').is_alphanumeric() {
        start -= 1;
    }

    // Move end forwards to find word end
    while end < line.len() && line.chars().nth(end).unwrap_or(' ').is_alphanumeric() {
        end += 1;
    }

    if start < end {
        line[start..end].to_string()
    } else {
        String::new()
    }
}

pub fn find_pattern_in_source(text: &str, pattern: &str) -> Option<Range> {
    let lines: Vec<&str> = text.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        if let Some(col_idx) = line.find(pattern) {
            return Some(Range {
                start: Position {
                    line: line_idx as u32,
                    character: col_idx as u32,
                },
                end: Position {
                    line: line_idx as u32,
                    character: (col_idx + pattern.len()) as u32,
                },
            });
        }
    }

    None
}

pub fn type_to_string(ty: &vex_ast::Type) -> String {
    match ty {
        vex_ast::Type::I8 => "i8".to_string(),
        vex_ast::Type::I16 => "i16".to_string(),
        vex_ast::Type::I32 => "i32".to_string(),
        vex_ast::Type::I64 => "i64".to_string(),
        vex_ast::Type::I128 => "i128".to_string(),
        vex_ast::Type::U8 => "u8".to_string(),
        vex_ast::Type::U16 => "u16".to_string(),
        vex_ast::Type::U32 => "u32".to_string(),
        vex_ast::Type::U64 => "u64".to_string(),
        vex_ast::Type::F16 => "f16".to_string(),
        vex_ast::Type::F32 => "f32".to_string(),
        vex_ast::Type::F64 => "f64".to_string(),
        vex_ast::Type::Bool => "bool".to_string(),
        vex_ast::Type::String => "string".to_string(),
        vex_ast::Type::Byte => "byte".to_string(),
        vex_ast::Type::Error => "error".to_string(),
        vex_ast::Type::Nil => "nil".to_string(),
        vex_ast::Type::Named(name) => name.clone(),
        vex_ast::Type::Generic { name, type_args } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}<{}>",
                    name,
                    type_args
                        .iter()
                        .map(|t| type_to_string(t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        vex_ast::Type::Array(element, _) => format!("[{}]", type_to_string(element)),
        vex_ast::Type::Slice(element, is_mutable) => {
            if *is_mutable {
                format!("&mut [{}]", type_to_string(element))
            } else {
                format!("&[{}]", type_to_string(element))
            }
        }
        vex_ast::Type::Reference(inner, is_mutable) => {
            if *is_mutable {
                format!("&mut {}", type_to_string(inner))
            } else {
                format!("&{}", type_to_string(inner))
            }
        }
        vex_ast::Type::Tuple(types) => {
            format!(
                "({})",
                types
                    .iter()
                    .map(|t| type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        vex_ast::Type::Function {
            params,
            return_type,
        } => {
            let params_str = params
                .iter()
                .map(|p| type_to_string(p))
                .collect::<Vec<_>>()
                .join(", ");
            format!("fn({}): {}", params_str, type_to_string(return_type))
        }
        vex_ast::Type::Never => "!".to_string(),
        vex_ast::Type::Infer(_) => "_".to_string(),
        vex_ast::Type::Unit => "()".to_string(),
        vex_ast::Type::RawPtr { inner, is_const } => {
            if *is_const {
                format!("*const {}", type_to_string(inner))
            } else {
                format!("*{}", type_to_string(inner))
            }
        }
        vex_ast::Type::Option(inner) => format!("Option<{}>", type_to_string(inner)),
        vex_ast::Type::Result(ok_type, err_type) => format!(
            "Result<{}, {}>",
            type_to_string(ok_type),
            type_to_string(err_type)
        ),
        vex_ast::Type::Vec(inner) => format!("Vec<{}>", type_to_string(inner)),
        vex_ast::Type::Box(inner) => format!("Box<{}>", type_to_string(inner)),
        vex_ast::Type::Channel(inner) => format!("Channel<{}>", type_to_string(inner)),
        // Handle other variants with defaults
        _ => format!("{:?}", ty), // Fallback for unhandled variants
    }
}
