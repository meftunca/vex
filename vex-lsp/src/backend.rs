// LSP Backend - Handles all LSP requests

use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use vex_compiler::borrow_checker::BorrowChecker;

pub struct VexBackend {
    client: Client,
    /// Document cache: URI -> source code
    documents: Arc<DashMap<String, String>>,
    /// Parsed AST cache: URI -> AST
    ast_cache: Arc<DashMap<String, vex_ast::Program>>,
}

impl VexBackend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(DashMap::new()),
            ast_cache: Arc::new(DashMap::new()),
        }
    }

    /// Parse document and return diagnostics
    async fn parse_and_diagnose(&self, uri: &str, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Extract filename from URI
        let filename = uri.split('/').last().unwrap_or("unknown.vx");

        // Create parser with filename for better error reporting
        let parser = match vex_parser::Parser::new_with_file(filename, text) {
            Ok(p) => p,
            Err(e) => {
                // Convert parser creation error to diagnostic
                diagnostics.push(self.parse_error_to_diagnostic(&e, text));
                return diagnostics;
            }
        };

        // Parse the document
        let mut parser = parser;
        match parser.parse_file() {
            Ok(mut program) => {
                // Run borrow checker for semantic errors
                let mut borrow_checker = BorrowChecker::new();
                if let Err(error) = borrow_checker.check_program(&mut program) {
                    // Convert borrow checker error to diagnostic (pass source text for better positioning)
                    diagnostics.push(self.borrow_error_to_diagnostic(&error, text));
                }

                // Store parsed AST (after borrow checking)
                self.ast_cache.insert(uri.to_string(), program);
            }
            Err(parse_error) => {
                // Convert parse error to LSP diagnostic with accurate span
                diagnostics.push(self.parse_error_to_diagnostic(&parse_error, text));
            }
        }

        diagnostics
    }
    /// Convert Vex ParseError to LSP Diagnostic with accurate position
    fn parse_error_to_diagnostic(
        &self,
        error: &vex_parser::ParseError,
        _source: &str,
    ) -> Diagnostic {
        use vex_parser::ParseError;

        match error {
            ParseError::Diagnostic(diag) => {
                // ParseError now contains vex_diagnostics::Diagnostic
                // Convert to LSP Diagnostic
                let range = Range {
                    start: Position {
                        line: (diag.span.line.saturating_sub(1)) as u32,
                        character: (diag.span.column.saturating_sub(1)) as u32,
                    },
                    end: Position {
                        line: (diag.span.line.saturating_sub(1)) as u32,
                        character: (diag.span.column + diag.span.length).saturating_sub(1) as u32,
                    },
                };

                Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(diag.code.clone())),
                    source: Some("vex".to_string()),
                    message: diag.message.clone(),
                    ..Default::default()
                }
            }
            ParseError::LexerError(msg) => Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("E0001".to_string())),
                source: Some("vex".to_string()),
                message: format!("Lexer error: {}", msg),
                ..Default::default()
            },
        }
    }

    /// Convert Vex BorrowError to LSP Diagnostic
    fn borrow_error_to_diagnostic(
        &self,
        error: &vex_compiler::borrow_checker::BorrowError,
        source: &str,
    ) -> Diagnostic {
        use vex_compiler::borrow_checker::BorrowError;

        let (message, code, location_str, variable, field) = match error {
            BorrowError::AssignToImmutable { variable, location } => (
                format!("cannot assign to immutable variable `{}`\nhelp: consider making this binding mutable: `let! {}`", variable, variable),
                "E0101",
                location.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::AssignToImmutableField { variable, field, location } => (
                format!("cannot assign to field `{}` of immutable variable `{}`\nhelp: consider making this binding mutable: `let! {}`", field, variable, variable),
                "E0102",
                location.as_ref(),
                Some(variable.as_str()),
                Some(field.as_str()),
            ),
            BorrowError::UseAfterMove { variable, used_at, .. } => (
                format!("use of moved value: `{}`", variable),
                "E0201",
                used_at.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::MutableBorrowWhileBorrowed { variable, new_borrow, .. } => (
                format!("cannot borrow `{}` as mutable because it is already borrowed as immutable", variable),
                "E0301",
                new_borrow.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::ImmutableBorrowWhileMutableBorrowed { variable, new_borrow, .. } => (
                format!("cannot borrow `{}` as immutable because it is already borrowed as mutable", variable),
                "E0302",
                new_borrow.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::MutationWhileBorrowed { variable, borrowed_at } => (
                format!("cannot assign to `{}` because it is borrowed", variable),
                "E0303",
                borrowed_at.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::MoveWhileBorrowed { variable, borrow_location } => (
                format!("cannot move out of `{}` because it is borrowed", variable),
                "E0304",
                borrow_location.as_ref(),
                Some(variable.as_str()),
                None,
            ),
            BorrowError::ReturnLocalReference { variable } => (
                format!("cannot return reference to local variable `{}`", variable),
                "E0401",
                None,
                Some(variable.as_str()),
                None,
            ),
            BorrowError::DanglingReference { reference, referent } => (
                format!("variable `{}` references `{}` which is out of scope", reference, referent),
                "E0402",
                None,
                Some(reference.as_str()),
                None,
            ),
            BorrowError::UseAfterScopeEnd { variable, .. } => (
                format!("use of `{}` after it went out of scope", variable),
                "E0403",
                None,
                Some(variable.as_str()),
                None,
            ),
            BorrowError::ReturnDanglingReference { variable } => (
                format!("returning reference to local variable `{}` which will be dropped", variable),
                "E0404",
                None,
                Some(variable.as_str()),
                None,
            ),
        };

        // Choose the appropriate search method based on error type
        let range = if let Some(field_name) = field {
            // For field-specific errors, use field search
            if let Some(var_name) = variable {
                self.find_field_usage_in_source(source, var_name, field_name)
            } else {
                self.default_range()
            }
        } else if let Some(var_name) = variable {
            // For variable errors, use comprehensive variable search
            self.find_variable_usage_in_source(source, var_name, location_str)
        } else {
            self.default_range()
        };

        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(code.to_string())),
            source: Some("vex-borrow-checker".to_string()),
            message: message.to_string(),
            ..Default::default()
        }
    }

    /// Default range for errors without position info
    fn default_range(&self) -> Range {
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        }
    }

    /// Find variable usage in source code - comprehensive pattern matching
    fn find_variable_usage_in_source(
        &self,
        source: &str,
        variable: &str,
        _location: Option<&String>,
    ) -> Range {
        let lines: Vec<&str> = source.lines().collect();

        // Pattern 1: Assignment (most common for immutability errors)
        // "variable = value" or "self.field = value"
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(&format!("{} =", variable)) {
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col_idx + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!("self.{} =", variable)) {
                let start_col = col_idx + 5; // "self.".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
        }

        // Pattern 2: Variable usage (for use-after-move, dangling references)
        // Look for variable name followed by common usage patterns
        for (line_idx, line) in lines.iter().enumerate() {
            // Function calls: "variable.method()"
            if let Some(col_idx) = line.find(&format!("{}.", variable)) {
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col_idx + variable.len()) as u32,
                    },
                };
            }
            // As function argument: "func(variable)"
            if let Some(col_idx) = line.find(&format!("({})", variable)) {
                let start_col = col_idx + 1;
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!("({},", variable)) {
                let start_col = col_idx + 1;
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!(", {})", variable)) {
                let start_col = col_idx + 2;
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!(", {},", variable)) {
                let start_col = col_idx + 2;
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
        }

        // Pattern 3: Return statements (for dangling references)
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(&format!("return {}", variable)) {
                let start_col = col_idx + 7; // "return ".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            if let Some(col_idx) = line.find(&format!("return &{}", variable)) {
                let start_col = col_idx + 8; // "return &".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
        }

        // Pattern 4: Borrow operations (for borrow conflicts)
        for (line_idx, line) in lines.iter().enumerate() {
            // Mutable borrow: "&variable!" or "let x = &variable!"
            if let Some(col_idx) = line.find(&format!("&{}!", variable)) {
                let start_col = col_idx + 1; // "&".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
            // Immutable borrow: "&variable"
            if let Some(col_idx) = line.find(&format!("&{}", variable)) {
                let start_col = col_idx + 1; // "&".len()
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: start_col as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (start_col + variable.len()) as u32,
                    },
                };
            }
        }

        // Pattern 5: Standalone variable usage (last resort)
        for (line_idx, line) in lines.iter().enumerate() {
            if line.contains(variable) {
                if let Some(col_idx) = line.find(variable) {
                    return Range {
                        start: Position {
                            line: line_idx as u32,
                            character: col_idx as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (col_idx + variable.len()) as u32,
                        },
                    };
                }
            }
        }

        // Default to first line if not found
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        }
    }

    /// Find field usage in source for field-specific errors
    fn find_field_usage_in_source(&self, source: &str, variable: &str, field: &str) -> Range {
        let lines: Vec<&str> = source.lines().collect();

        // Pattern: "variable.field" or "self.field"
        let patterns = vec![format!("{}.{}", variable, field), format!("self.{}", field)];

        for pattern in patterns {
            for (line_idx, line) in lines.iter().enumerate() {
                if let Some(col_idx) = line.find(&pattern) {
                    let start_col = if pattern.starts_with("self.") {
                        col_idx + 5 // "self.".len()
                    } else {
                        col_idx + variable.len() + 1 // "variable.".len()
                    };
                    return Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start_col as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: (start_col + field.len()) as u32,
                        },
                    };
                }
            }
        }

        // Fallback: find the field name anywhere
        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(field) {
                return Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col_idx as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (col_idx + field.len()) as u32,
                    },
                };
            }
        }

        // Default to first line
        Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        }
    }

    /// Publish diagnostics to client
    async fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// Get word at cursor position
    fn get_word_at_position(&self, text: &str, position: Position) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return String::new();
        }

        let line = lines[position.line as usize];
        let col = position.character as usize;
        if col >= line.len() {
            return String::new();
        }

        // Find word boundaries
        let mut start = col;
        let mut end = col;

        let chars: Vec<char> = line.chars().collect();

        // Go backward to find start
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Go forward to find end
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        chars[start..end].iter().collect()
    }

    /// Find definition location in AST
    fn find_definition_location(
        &self,
        program: &vex_ast::Program,
        symbol: &str,
        source: &str,
    ) -> Option<Range> {
        // Search for function definitions
        for item in &program.items {
            match item {
                vex_ast::Item::Function(func) if func.name == symbol => {
                    // Find "fn <name>" in source
                    return self.find_pattern_in_source(source, &format!("fn {}", symbol));
                }
                vex_ast::Item::Struct(s) if s.name == symbol => {
                    return self.find_pattern_in_source(source, &format!("struct {}", symbol));
                }
                vex_ast::Item::Enum(e) if e.name == symbol => {
                    return self.find_pattern_in_source(source, &format!("enum {}", symbol));
                }
                vex_ast::Item::Trait(t) if t.name == symbol => {
                    return self.find_pattern_in_source(source, &format!("trait {}", symbol));
                }
                vex_ast::Item::Const(c) if c.name == symbol => {
                    return self.find_pattern_in_source(source, &format!("const {}", symbol));
                }
                _ => {}
            }
        }

        None
    }

    /// Find pattern in source and return its range
    fn find_pattern_in_source(&self, source: &str, pattern: &str) -> Option<Range> {
        let lines: Vec<&str> = source.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            if let Some(col_idx) = line.find(pattern) {
                let name_start = col_idx + pattern.len() - pattern.split_whitespace().last()?.len();
                return Some(Range {
                    start: Position {
                        line: line_idx as u32,
                        character: name_start as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: (name_start + pattern.split_whitespace().last()?.len()) as u32,
                    },
                });
            }
        }

        None
    }

    /// Find function call context (function name and current parameter index)
    fn find_function_call_context(
        &self,
        text: &str,
        position: Position,
    ) -> Option<(String, usize)> {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        // Get text before cursor
        let before_cursor = &line[..char_pos.min(line.len())];

        // Find the last opening parenthesis
        let mut paren_depth = 0;
        let mut param_index = 0;
        let mut func_start: Option<usize> = None;

        for (i, ch) in before_cursor.char_indices().rev() {
            match ch {
                ')' => paren_depth += 1,
                '(' => {
                    if paren_depth == 0 {
                        // Found the opening paren of current function call
                        func_start = Some(i);
                        break;
                    }
                    paren_depth -= 1;
                }
                ',' if paren_depth == 0 => {
                    param_index += 1;
                }
                _ => {}
            }
        }

        // Extract function name before the opening paren
        if let Some(start) = func_start {
            let before_paren = &before_cursor[..start].trim_end();

            // Find the function name (word before parenthesis)
            let mut func_name = String::new();
            for ch in before_paren.chars().rev() {
                if ch.is_alphanumeric() || ch == '_' {
                    func_name.insert(0, ch);
                } else {
                    break;
                }
            }

            if !func_name.is_empty() {
                return Some((func_name, param_index));
            }
        }

        None
    }

    /// Get completion context (what's before cursor)
    fn get_completion_context(&self, text: &str, position: Position) -> CompletionContextInfo {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return CompletionContextInfo::default();
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;
        if char_pos == 0 {
            return CompletionContextInfo::default();
        }

        let before_cursor = &line[..char_pos.min(line.len())];

        CompletionContextInfo {
            after_dot: before_cursor.ends_with('.'),
            after_colon: before_cursor.ends_with(':'),
            in_type_position: before_cursor.contains("let") || before_cursor.contains("const"),
        }
    }

    /// Convert Type to string for display
    fn type_to_string(&self, ty: &vex_ast::Type) -> String {
        match ty {
            vex_ast::Type::Unit => "()".to_string(),
            vex_ast::Type::I8 => "i8".to_string(),
            vex_ast::Type::I16 => "i16".to_string(),
            vex_ast::Type::I32 => "i32".to_string(),
            vex_ast::Type::I64 => "i64".to_string(),
            vex_ast::Type::U8 => "u8".to_string(),
            vex_ast::Type::U16 => "u16".to_string(),
            vex_ast::Type::U32 => "u32".to_string(),
            vex_ast::Type::U64 => "u64".to_string(),
            vex_ast::Type::F32 => "f32".to_string(),
            vex_ast::Type::F64 => "f64".to_string(),
            vex_ast::Type::Bool => "bool".to_string(),
            vex_ast::Type::String => "string".to_string(),
            vex_ast::Type::Named(name) => name.clone(),
            vex_ast::Type::Reference(inner, mutable) => {
                if *mutable {
                    format!("&{}!", self.type_to_string(inner))
                } else {
                    format!("&{}", self.type_to_string(inner))
                }
            }
            vex_ast::Type::Generic { name, type_args } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    let args = type_args
                        .iter()
                        .map(|t| self.type_to_string(t))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}<{}>", name, args)
                }
            }
            vex_ast::Type::Array(element_type, size) => {
                format!("[{}; {}]", self.type_to_string(element_type), size)
            }
            vex_ast::Type::Tuple(types) => {
                let inner = types
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", inner)
            }
            vex_ast::Type::Function {
                params,
                return_type,
            } => {
                let params_str = params
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}): {}", params_str, self.type_to_string(return_type))
            }
            vex_ast::Type::Vec(inner) => format!("Vec<{}>", self.type_to_string(inner)),
            vex_ast::Type::Box(inner) => format!("Box<{}>", self.type_to_string(inner)),
            vex_ast::Type::Option(inner) => format!("Option<{}>", self.type_to_string(inner)),
            vex_ast::Type::Result(ok, err) => format!(
                "Result<{}, {}>",
                self.type_to_string(ok),
                self.type_to_string(err)
            ),
            _ => "unknown".to_string(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for VexBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("LSP client connected");

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "vex-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("LSP server initialized");
        self.client
            .log_message(MessageType::INFO, "Vex LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("LSP server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text.clone();

        tracing::info!("Document opened: {}", uri);

        // Store document
        self.documents.insert(uri.clone(), text.clone());

        // Parse and send diagnostics
        let diagnostics = self.parse_and_diagnose(&uri, &text).await;
        self.publish_diagnostics(params.text_document.uri, diagnostics)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();

            tracing::info!("Document changed: {}", uri);

            // Update document
            self.documents.insert(uri.clone(), text.clone());

            // Re-parse and send diagnostics
            let diagnostics = self.parse_and_diagnose(&uri, &text).await;
            self.publish_diagnostics(params.text_document.uri, diagnostics)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        tracing::info!("Document closed: {}", uri);

        // Remove from cache
        self.documents.remove(&uri);
        self.ast_cache.remove(&uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        // Get the AST for this document
        let ast = match self.ast_cache.get(&uri) {
            Some(ast) => ast.clone(),
            None => return Ok(None), // Document not parsed yet
        };

        // Create symbol resolver
        let mut resolver = crate::symbol_resolver::SymbolResolver::new();
        resolver.extract_symbols(&ast);

        // Find symbol at cursor position (LSP uses 0-indexed lines)
        let line = (position.line + 1) as usize;
        let column = (position.character + 1) as usize;

        if let Some(symbol) = resolver.find_symbol_at(line, column) {
            // Format hover content based on symbol kind
            let mut content = String::new();

            // Add kind badge
            let kind_str = match symbol.kind {
                crate::symbol_resolver::SymbolKind::Function => "function",
                crate::symbol_resolver::SymbolKind::Variable => "variable",
                crate::symbol_resolver::SymbolKind::Parameter => "parameter",
                crate::symbol_resolver::SymbolKind::Struct => "struct",
                crate::symbol_resolver::SymbolKind::Enum => "enum",
                crate::symbol_resolver::SymbolKind::Trait => "trait",
                crate::symbol_resolver::SymbolKind::Field => "field",
                crate::symbol_resolver::SymbolKind::Method => "method",
            };

            content.push_str(&format!("**{}** `{}`\n\n", kind_str, symbol.name));

            // Add type information
            if let Some(type_info) = &symbol.type_info {
                content.push_str("```vex\n");
                content.push_str(type_info);
                content.push_str("\n```\n\n");
            }

            // Add documentation if available
            if let Some(doc) = &symbol.documentation {
                content.push_str("---\n\n");
                content.push_str(doc);
                content.push_str("\n");
            }

            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: None,
            }));
        }

        // Fallback: show generic hover info
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "*Vex Language*\n\nNo symbol information available at this position."
                    .to_string(),
            }),
            range: None,
        }))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let mut items = Vec::new();

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get context (what's before cursor)
        let context = self.get_completion_context(&text, position);

        // Keywords
        let keywords = vec![
            ("fn", CompletionItemKind::KEYWORD, "function declaration"),
            ("let", CompletionItemKind::KEYWORD, "immutable variable"),
            ("let!", CompletionItemKind::KEYWORD, "mutable variable"),
            ("const", CompletionItemKind::KEYWORD, "constant"),
            ("struct", CompletionItemKind::KEYWORD, "struct definition"),
            ("enum", CompletionItemKind::KEYWORD, "enum definition"),
            ("trait", CompletionItemKind::KEYWORD, "trait definition"),
            ("impl", CompletionItemKind::KEYWORD, "implementation block"),
            ("if", CompletionItemKind::KEYWORD, "conditional"),
            ("else", CompletionItemKind::KEYWORD, "else clause"),
            ("match", CompletionItemKind::KEYWORD, "pattern matching"),
            ("for", CompletionItemKind::KEYWORD, "for loop"),
            ("while", CompletionItemKind::KEYWORD, "while loop"),
            ("loop", CompletionItemKind::KEYWORD, "infinite loop"),
            ("return", CompletionItemKind::KEYWORD, "return statement"),
            ("break", CompletionItemKind::KEYWORD, "break loop"),
            ("continue", CompletionItemKind::KEYWORD, "continue loop"),
            ("defer", CompletionItemKind::KEYWORD, "defer statement"),
            ("import", CompletionItemKind::KEYWORD, "import module"),
            ("export", CompletionItemKind::KEYWORD, "export symbol"),
            (
                "extern",
                CompletionItemKind::KEYWORD,
                "external declaration",
            ),
            ("async", CompletionItemKind::KEYWORD, "async function"),
            ("await", CompletionItemKind::KEYWORD, "await expression"),
        ];

        // Built-in types
        let builtin_types = vec![
            ("i8", CompletionItemKind::STRUCT, "8-bit integer"),
            ("i16", CompletionItemKind::STRUCT, "16-bit integer"),
            ("i32", CompletionItemKind::STRUCT, "32-bit integer"),
            ("i64", CompletionItemKind::STRUCT, "64-bit integer"),
            ("u8", CompletionItemKind::STRUCT, "unsigned 8-bit integer"),
            ("u16", CompletionItemKind::STRUCT, "unsigned 16-bit integer"),
            ("u32", CompletionItemKind::STRUCT, "unsigned 32-bit integer"),
            ("u64", CompletionItemKind::STRUCT, "unsigned 64-bit integer"),
            ("f32", CompletionItemKind::STRUCT, "32-bit float"),
            ("f64", CompletionItemKind::STRUCT, "64-bit float"),
            ("bool", CompletionItemKind::STRUCT, "boolean"),
            ("string", CompletionItemKind::STRUCT, "string type"),
            ("Vec", CompletionItemKind::STRUCT, "vector collection"),
            ("Box", CompletionItemKind::STRUCT, "heap-allocated value"),
            ("Option", CompletionItemKind::ENUM, "optional value"),
            ("Result", CompletionItemKind::ENUM, "result type"),
            ("HashMap", CompletionItemKind::STRUCT, "hash map"),
        ];

        // Add keywords and types
        for (label, kind, doc) in keywords.iter().chain(builtin_types.iter()) {
            items.push(CompletionItem {
                label: label.to_string(),
                kind: Some(*kind),
                detail: Some(doc.to_string()),
                ..Default::default()
            });
        }

        // Get AST for context-aware suggestions
        if let Some(ast) = self.ast_cache.get(&uri.to_string()) {
            // Add functions from AST
            for item in &ast.items {
                match item {
                    vex_ast::Item::Function(func) => {
                        let params_str = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let return_str = if let Some(ret) = &func.return_type {
                            self.type_to_string(ret)
                        } else {
                            "()".to_string()
                        };

                        items.push(CompletionItem {
                            label: func.name.clone(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!(
                                "fn {}({}): {}",
                                func.name, params_str, return_str
                            )),
                            insert_text: Some(format!("{}()", func.name)),
                            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                            ..Default::default()
                        });
                    }
                    vex_ast::Item::Struct(s) => {
                        items.push(CompletionItem {
                            label: s.name.clone(),
                            kind: Some(CompletionItemKind::STRUCT),
                            detail: Some(format!("struct {}", s.name)),
                            ..Default::default()
                        });

                        // If cursor is after ".", suggest struct fields
                        if context.after_dot {
                            for field in &s.fields {
                                items.push(CompletionItem {
                                    label: field.name.clone(),
                                    kind: Some(CompletionItemKind::FIELD),
                                    detail: Some(format!(
                                        "{}: {}",
                                        field.name,
                                        self.type_to_string(&field.ty)
                                    )),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                    vex_ast::Item::Enum(e) => {
                        items.push(CompletionItem {
                            label: e.name.clone(),
                            kind: Some(CompletionItemKind::ENUM),
                            detail: Some(format!("enum {}", e.name)),
                            ..Default::default()
                        });

                        // Suggest enum variants
                        for variant in &e.variants {
                            items.push(CompletionItem {
                                label: variant.name.clone(),
                                kind: Some(CompletionItemKind::ENUM_MEMBER),
                                detail: Some(format!("{}::{}", e.name, variant.name)),
                                ..Default::default()
                            });
                        }
                    }
                    vex_ast::Item::Trait(t) => {
                        items.push(CompletionItem {
                            label: t.name.clone(),
                            kind: Some(CompletionItemKind::INTERFACE),
                            detail: Some(format!("trait {}", t.name)),
                            ..Default::default()
                        });
                    }
                    vex_ast::Item::Const(c) => {
                        let ty_str = if let Some(ty) = &c.ty {
                            self.type_to_string(ty)
                        } else {
                            "inferred".to_string()
                        };
                        items.push(CompletionItem {
                            label: c.name.clone(),
                            kind: Some(CompletionItemKind::CONSTANT),
                            detail: Some(format!("const {}: {}", c.name, ty_str)),
                            ..Default::default()
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the word at cursor position
        let word = self.get_word_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        // Get the AST
        let ast = match self.ast_cache.get(&uri.to_string()) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Find definition location by searching for the symbol
        if let Some(location) = self.find_definition_location(&ast, &word, &text) {
            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: uri.clone(),
                range: location,
            })));
        }

        Ok(None)
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
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
                            .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");

                        let return_str = if let Some(ret) = &func.return_type {
                            format!(": {}", self.type_to_string(ret))
                        } else {
                            String::new()
                        };

                        let label = format!("{}({}){}", func.name, params_str, return_str);

                        // Build parameter information
                        let parameters: Vec<ParameterInformation> = func
                            .params
                            .iter()
                            .map(|p| {
                                let param_label =
                                    format!("{}: {}", p.name, self.type_to_string(&p.ty));
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

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        // Get the AST
        let ast = match self.ast_cache.get(&uri.to_string()) {
            Some(ast) => ast.clone(),
            None => return Ok(None),
        };

        // Get document text for position calculation
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        let mut symbols = Vec::new();

        // Iterate through all items in the AST
        for item in &ast.items {
            match item {
                vex_ast::Item::Function(func) => {
                    // Find function position in source
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("fn {}", func.name))
                    {
                        let params_str = func
                            .params
                            .iter()
                            .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
                            .collect::<Vec<_>>()
                            .join(", ");

                        let return_str = if let Some(ret) = &func.return_type {
                            format!(": {}", self.type_to_string(ret))
                        } else {
                            String::new()
                        };

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: func.name.clone(),
                            detail: Some(format!("({}){}", params_str, return_str)),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        });
                    }
                }
                vex_ast::Item::Struct(s) => {
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("struct {}", s.name))
                    {
                        let mut children = Vec::new();

                        // Add struct fields as children
                        for field in &s.fields {
                            if let Some(field_range) =
                                self.find_pattern_in_source(&text, &field.name)
                            {
                                #[allow(deprecated)]
                                children.push(DocumentSymbol {
                                    name: field.name.clone(),
                                    detail: Some(self.type_to_string(&field.ty)),
                                    kind: SymbolKind::FIELD,
                                    tags: None,
                                    deprecated: None,
                                    range: field_range,
                                    selection_range: field_range,
                                    children: None,
                                });
                            }
                        }

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: s.name.clone(),
                            detail: Some(format!("struct with {} fields", s.fields.len())),
                            kind: SymbolKind::STRUCT,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: if children.is_empty() {
                                None
                            } else {
                                Some(children)
                            },
                        });
                    }
                }
                vex_ast::Item::Enum(e) => {
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("enum {}", e.name))
                    {
                        let mut children = Vec::new();

                        // Add enum variants as children
                        for variant in &e.variants {
                            if let Some(variant_range) =
                                self.find_pattern_in_source(&text, &variant.name)
                            {
                                #[allow(deprecated)]
                                children.push(DocumentSymbol {
                                    name: variant.name.clone(),
                                    detail: variant.data.as_ref().map(|t| self.type_to_string(t)),
                                    kind: SymbolKind::ENUM_MEMBER,
                                    tags: None,
                                    deprecated: None,
                                    range: variant_range,
                                    selection_range: variant_range,
                                    children: None,
                                });
                            }
                        }

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: e.name.clone(),
                            detail: Some(format!("enum with {} variants", e.variants.len())),
                            kind: SymbolKind::ENUM,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: if children.is_empty() {
                                None
                            } else {
                                Some(children)
                            },
                        });
                    }
                }
                vex_ast::Item::Trait(t) => {
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("trait {}", t.name))
                    {
                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: t.name.clone(),
                            detail: Some("trait".to_string()),
                            kind: SymbolKind::INTERFACE,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        });
                    }
                }
                vex_ast::Item::Const(c) => {
                    if let Some(range) =
                        self.find_pattern_in_source(&text, &format!("const {}", c.name))
                    {
                        let type_str = if let Some(ty) = &c.ty {
                            self.type_to_string(ty)
                        } else {
                            "inferred".to_string()
                        };

                        #[allow(deprecated)]
                        symbols.push(DocumentSymbol {
                            name: c.name.clone(),
                            detail: Some(type_str),
                            kind: SymbolKind::CONSTANT,
                            tags: None,
                            deprecated: None,
                            range,
                            selection_range: range,
                            children: None,
                        });
                    }
                }
                _ => {}
            }
        }

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        }
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the word at cursor position
        let word = self.get_word_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }

        let mut locations = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Search for all occurrences of the word in the document
        for (line_idx, line) in lines.iter().enumerate() {
            let mut start_pos = 0;
            while let Some(col_idx) = line[start_pos..].find(&word) {
                let actual_col = start_pos + col_idx;

                // Check if this is a whole word match (not part of another word)
                let before_ok = actual_col == 0
                    || !line
                        .chars()
                        .nth(actual_col - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let after_pos = actual_col + word.len();
                let after_ok = after_pos >= line.len()
                    || !line.chars().nth(after_pos).unwrap_or(' ').is_alphanumeric();

                if before_ok && after_ok {
                    locations.push(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (actual_col + word.len()) as u32,
                            },
                        },
                    });
                }

                start_pos = actual_col + 1;
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;

        // Get document text
        let text = match self.documents.get(&uri.to_string()) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get the word at cursor position (old name)
        let old_name = self.get_word_at_position(&text, position);
        if old_name.is_empty() {
            return Ok(None);
        }

        let mut edits = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        // Find all occurrences and create text edits
        for (line_idx, line) in lines.iter().enumerate() {
            let mut start_pos = 0;
            while let Some(col_idx) = line[start_pos..].find(&old_name) {
                let actual_col = start_pos + col_idx;

                // Check if this is a whole word match
                let before_ok = actual_col == 0
                    || !line
                        .chars()
                        .nth(actual_col - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let after_pos = actual_col + old_name.len();
                let after_ok = after_pos >= line.len()
                    || !line.chars().nth(after_pos).unwrap_or(' ').is_alphanumeric();

                if before_ok && after_ok {
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (actual_col + old_name.len()) as u32,
                            },
                        },
                        new_text: new_name.clone(),
                    });
                }

                start_pos = actual_col + 1;
            }
        }

        if edits.is_empty() {
            return Ok(None);
        }

        // Create workspace edit
        let mut changes = std::collections::HashMap::new();
        changes.insert(uri, edits);

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();

        // Get document text
        let text = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        // Load config or use defaults
        let config = std::env::current_dir()
            .ok()
            .and_then(|dir| vex_formatter::Config::from_dir(&dir).ok())
            .unwrap_or_default();

        // Format using vex-formatter
        match vex_formatter::format_source(&text, &config) {
            Ok(formatted) => {
                // Return single edit replacing entire document
                let lines: Vec<&str> = text.lines().collect();
                let last_line = lines.len().saturating_sub(1);
                let last_char = lines.last().map(|l| l.len()).unwrap_or(0);

                Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: last_line as u32,
                            character: last_char as u32,
                        },
                    },
                    new_text: formatted,
                }]))
            }
            Err(e) => {
                tracing::error!("Formatting error: {}", e);
                Ok(None)
            }
        }
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        // For now, just format the entire document
        // TODO: Implement proper range formatting
        self.formatting(DocumentFormattingParams {
            text_document: params.text_document,
            options: params.options,
            work_done_progress_params: params.work_done_progress_params,
        })
        .await
    }
}

/// Completion context information
#[derive(Debug, Default)]
struct CompletionContextInfo {
    after_dot: bool,
    after_colon: bool,
    in_type_position: bool,
}
