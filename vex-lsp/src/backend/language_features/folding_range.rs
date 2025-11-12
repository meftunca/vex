//! This module contains the implementation of the `folding_range` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::VexBackend;

impl VexBackend {
    pub async fn folding_range(
        &self,
        params: FoldingRangeParams,
    ) -> tower_lsp::jsonrpc::Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        let mut folding_ranges = Vec::new();
        self.extract_folding_ranges(&text, &mut folding_ranges);

        Ok(Some(folding_ranges))
    }

    fn extract_folding_ranges(&self, text: &str, folding_ranges: &mut Vec<FoldingRange>) {
        let lines: Vec<&str> = text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32;
            let trimmed = line.trim();

            // Function definitions
            if trimmed.starts_with("fn ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("fn ...")),
                    });
                }
            }
            // Struct definitions
            else if trimmed.starts_with("struct ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("struct ...")),
                    });
                }
            }
            // Enum definitions
            else if trimmed.starts_with("enum ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("enum ...")),
                    });
                }
            }
            // Trait definitions
            else if trimmed.starts_with("trait ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("trait ...")),
                    });
                }
            }
            // Impl blocks
            else if trimmed.starts_with("impl ") {
                // Find the opening brace
                if let Some(opening_brace_pos) = self.find_matching_brace(&lines, i, '{') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: opening_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("impl ...")),
                    });
                }
            }
            // If statements
            else if trimmed.starts_with("if ") && trimmed.ends_with("{") {
                // Find the matching closing brace
                if let Some(closing_brace_pos) = self.find_matching_brace(&lines, i, '}') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: closing_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("if ...")),
                    });
                }
            }
            // While loops
            else if trimmed.starts_with("while ") && trimmed.ends_with("{") {
                // Find the matching closing brace
                if let Some(closing_brace_pos) = self.find_matching_brace(&lines, i, '}') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: closing_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("while ...")),
                    });
                }
            }
            // For loops
            else if trimmed.starts_with("for ") && trimmed.ends_with("{") {
                // Find the matching closing brace
                if let Some(closing_brace_pos) = self.find_matching_brace(&lines, i, '}') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: closing_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("for ...")),
                    });
                }
            }
            // Match expressions
            else if trimmed.starts_with("match ") && trimmed.ends_with("{") {
                // Find the matching closing brace
                if let Some(closing_brace_pos) = self.find_matching_brace(&lines, i, '}') {
                    folding_ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: Some(0),
                        end_line: closing_brace_pos,
                        end_character: Some(0),
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some(format!("match ...")),
                    });
                }
            }
        }
    }

    fn find_matching_brace(
        &self,
        lines: &[&str],
        start_line: usize,
        _target_brace: char,
    ) -> Option<u32> {
        let mut brace_count = 0;
        let mut found_opening = false;

        for (i, line) in lines.iter().enumerate().skip(start_line) {
            for ch in line.chars() {
                if ch == '{' {
                    brace_count += 1;
                    found_opening = true;
                } else if ch == '}' {
                    brace_count -= 1;
                    if found_opening && brace_count == 0 {
                        return Some(i as u32);
                    }
                }
            }
        }

        None
    }
}
