//! This module contains the implementation of the `signature_help` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};

impl VexBackend {
    pub async fn signature_help(
        &self,
        params: SignatureHelpParams,
    ) -> tower_lsp::jsonrpc::Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the AST
        let ast = match self.ast_cache.get(&uri.to_string()) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Find function call context at cursor
        if let Some((func_name, param_index)) = self.find_function_call_context(&text, position) {
            // Search for function in AST
            for item in &ast.items {
                if let vex_ast::Item::Function(func) = item {
                    if func.name == func_name {
                        // Build signature label
                        let params_str = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");

                        let return_str = if let Some(ret) = &func.return_type {
                            format!(": {}", type_to_string(ret))
                        } else {
                            String::new()
                        };

                        let label = format!("{}({}){}", func.name, params_str, return_str);

                        // Build parameter information
                        let parameters: Vec<ParameterInformation> = func
                            .params
                            .iter()
                            .map(|p| {
                                let param_label = format!("{}: {}", p.name, type_to_string(&p.ty));
                                ParameterInformation {
                                    label: ParameterLabel::Simple(param_label),
                                    documentation: None,
                                }
                            })
                            .collect();

                        return Ok(Some(SignatureHelp {
                            signatures: vec![SignatureInformation {
                                label,
                                documentation: None,
                                parameters: Some(parameters),
                                active_parameter: Some(param_index as u32),
                            }],
                            active_signature: Some(0),
                            active_parameter: Some(param_index as u32),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn find_function_call_context(
        &self,
        text: &str,
        position: Position,
    ) -> Option<(String, usize)> {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx];
        if char_idx >= line.len() {
            return None;
        }

        // Look backwards from cursor to find function call
        let mut paren_count = 0;
        let mut param_start = char_idx;
        let mut func_name_end = char_idx;

        for i in (0..char_idx).rev() {
            let ch = line.chars().nth(i).unwrap_or(' ');
            match ch {
                ')' => paren_count += 1,
                '(' => {
                    if paren_count == 0 {
                        // Found the opening paren
                        param_start = i + 1;
                        func_name_end = i;
                        break;
                    } else {
                        paren_count -= 1;
                    }
                }
                ',' if paren_count == 0 => {
                    // Parameter separator
                    param_start = i + 1;
                }
                _ => {}
            }
        }

        if func_name_end > 0 {
            // Extract function name
            let mut name_start = func_name_end;
            while name_start > 0
                && line
                    .chars()
                    .nth(name_start - 1)
                    .unwrap_or(' ')
                    .is_alphanumeric()
            {
                name_start -= 1;
            }

            if name_start < func_name_end {
                let func_name = line[name_start..func_name_end].to_string();

                // Count parameters before cursor
                let params_before = &line[param_start..char_idx];
                let param_index = params_before.chars().filter(|&c| c == ',').count();

                return Some((func_name, param_index));
            }
        }

        None
    }
}
