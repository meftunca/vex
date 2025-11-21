//! This module contains the helper functions for the language features.

use tower_lsp::lsp_types::*;

// Helper struct for completion context
#[derive(Debug)]
pub struct CompletionContext {
    pub before_cursor: String,
    pub after_dot: bool,
}

pub fn get_word_at_position(text: &str, position: Position) -> String {
    // Backwards-compat: prefer simple alphanumeric-only word detection
    // This preserves callers that expect identifier-only words.
    let token = get_token_at_position(text, position);
    if token.chars().all(|c| c.is_alphanumeric() || c == '_') {
        token
    } else {
        String::new()
    }
}

/// Returns a token at the specified position. Token can be either an identifier (alphanumeric + _)
/// or an operator overload name like `op+`, `op<<`, etc.
pub fn get_token_at_position(text: &str, position: Position) -> String {
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

    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
    let is_op_char = |c: char| match c {
        '+' | '-' | '*' | '/' | '%' | '<' | '>' | '=' | '!' | '&' | '|' | '^' | '~' | '?' | ':' => {
            true
        }
        _ => false,
    };

    // If the character under cursor is a word character, use the existing alphanumeric logic
    let c = line.chars().nth(char_idx).unwrap_or(' ');
    if is_word_char(c) {
        let mut start = char_idx;
        let mut end = char_idx;

        while start > 0 && is_word_char(line.chars().nth(start - 1).unwrap_or(' ')) {
            start -= 1;
        }
        while end < line.len() && is_word_char(line.chars().nth(end).unwrap_or(' ')) {
            end += 1;
        }
        return line[start..end].to_string();
    }

    // Otherwise, expand over combined word/op characters. This will capture `op+`, `op<<` or
    // `opadd`-style method names as a single token when cursor is on an operator char.
    let mut start = char_idx;
    let mut end = char_idx + 1; // include the current char

    while start > 0 {
        let prev = line.chars().nth(start - 1).unwrap_or(' ');
        if is_word_char(prev) || is_op_char(prev) {
            start -= 1;
        } else {
            break;
        }
    }

    while end < line.len() {
        let next = line.chars().nth(end).unwrap_or(' ');
        if is_word_char(next) || is_op_char(next) {
            end += 1;
        } else {
            break;
        }
    }

    let substring = line[start..end].to_string();
    // If substring begins with 'op' followed by operator chars, extract 'op...' token
    if substring.starts_with("op") {
        let mut chars = substring.chars();
        // advance "op"
        chars.next();
        chars.next();
        // collect following op chars
        let mut op_seq = String::from("op");
        while let Some(ch) = chars.next() {
            if is_op_char(ch) {
                op_seq.push(ch);
            } else {
                break;
            }
        }
        if op_seq.len() > 2 {
            // at least 'op' + one op char
            return op_seq;
        }
    }

    // Otherwise return the maximal (alnum/op mixed) substring we found, but prefer alnum-only
    if substring
        .chars()
        .all(|ch| is_word_char(ch) || is_op_char(ch))
    {
        substring
    } else {
        String::new()
    }
}

pub fn is_op_char(c: char) -> bool {
    match c {
        '+' | '-' | '*' | '/' | '%' | '<' | '>' | '=' | '!' | '&' | '|' | '^' | '~' | '?' | ':' => {
            true
        }
        _ => false,
    }
}

pub fn is_operator_token(token: &str) -> bool {
    token.chars().any(is_op_char)
}

/// If the cursor is on a dotted call, return the left-side receiver type name, e.g.
/// for 'Counter.new(3)', when cursor is on 'new' we return Some("Counter").
pub fn get_receiver_at_position(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let line_idx = position.line as usize;
    let char_idx = position.character as usize;

    if line_idx >= lines.len() {
        return None;
    }
    let line = lines[line_idx];
    if char_idx > line.len() {
        return None;
    }

    // Find token start and end for current token
    let mut start = char_idx;
    while start > 0
        && (line.chars().nth(start - 1).unwrap_or(' ').is_alphanumeric()
            || line.chars().nth(start - 1).unwrap_or(' ') == '_')
    {
        start -= 1;
    }
    let mut end = char_idx;
    while end < line.len()
        && (line.chars().nth(end).unwrap_or(' ').is_alphanumeric()
            || line.chars().nth(end).unwrap_or(' ') == '_')
    {
        end += 1;
    }

    // Check the char preceding start - should be dot
    if start == 0 {
        return None;
    }
    let dot_pos = start - 1;
    if line.chars().nth(dot_pos).unwrap_or(' ') != '.' {
        return None;
    }

    // Expand left to find receiver base name (ignore generics for simplicity)
    let recv_end = dot_pos;
    let mut recv_start = dot_pos;
    // Move left while char is word char; this yields 'Counter' from 'Counter.new'
    while recv_start > 0
        && (line
            .chars()
            .nth(recv_start - 1)
            .unwrap_or(' ')
            .is_alphanumeric()
            || line.chars().nth(recv_start - 1).unwrap_or(' ') == '_')
    {
        recv_start -= 1;
    }

    if recv_start < recv_end {
        let recv = line[recv_start..recv_end].to_string();
        if !recv.is_empty() {
            return Some(recv);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::Position;

    #[test]
    fn test_get_token_at_position_alphanumeric() {
        let source = "fn add(x: i32) {\n    add(1, 2);\n}";
        // Position on 'add' in definition
        let pos = Position {
            line: 0,
            character: 3,
        };
        let token = get_token_at_position(source, pos);
        assert_eq!(token, "add");
    }

    #[test]
    fn test_get_token_at_position_operator() {
        let source = "fn op+(other: i32): i32 {\n    op+(1, 2);\n}";
        // Position on '+' in definition
        let pos = Position {
            line: 0,
            character: 3 + 2,
        }; // char of '+' assuming 'fn op+'
        let token = get_token_at_position(source, pos);
        assert_eq!(token, "op+");
    }

    #[test]
    fn test_is_operator_token() {
        assert!(is_operator_token("op+"));
        assert!(!is_operator_token("add"));
    }

    #[test]
    fn test_get_receiver_at_position_counter_new() {
        let source = "let! counter = Counter.new(3);";
        // Find 'new' index
        let line = source;
        let pos = Position {
            line: 0,
            character: line.find("new").unwrap() as u32,
        };
        let recv = get_receiver_at_position(source, pos).unwrap();
        assert_eq!(recv, "Counter");
    }

    #[test]
    fn test_get_receiver_at_position_vec_new() {
        let source = "let v = Vec.new();";
        let line = source;
        let pos = Position {
            line: 0,
            character: line.find("new").unwrap() as u32,
        };
        let recv = get_receiver_at_position(source, pos).unwrap();
        assert_eq!(recv, "Vec");
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
