//! This module contains the implementation of the `hover` language feature.
use tower_lsp::lsp_types::*;

use crate::backend::{language_features::helpers::*, VexBackend};
use std::fs;

impl VexBackend {
    pub async fn hover(&self, params: HoverParams) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
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

        // Get document text for word extraction
        let text = match self.documents.get(&uri) {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        // Get token at cursor position (supports operator overload "op+" style names)
        let word = get_token_at_position(&text, position);
        if word.is_empty() {
            return Ok(None);
        }
        // Get receiver for dotted calls like `Counter.new()` to provide method hover
        let receiver = get_receiver_at_position(&text, position);

        // Try to find detailed information about the symbol
        if let Some(hover_info) = self.find_symbol_hover_info(&ast, &word) {
            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_info,
                }),
                range: None,
            }))
        } else {
            // Try workspace ASTs for imported symbols
            if let Some(hover_info) = self.find_symbol_hover_info_workspace(&word) {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: hover_info }),
                    range: None,
                }));
            }

            // If still not found, try resolving current file imports and parse the referenced module
            for import in &ast.imports {
                // module strings can be: "time" or "core/vec" or others
                let module = import.module.clone();
                // Try to locate in vendored stdlib under repo root: vex-libs/std/<module>/src/lib.vx
                // Also allow module with path sep
                let cwd = std::env::current_dir().ok();
                if let Some(cwd) = cwd {
                    let module_path = cwd.join("vex-libs").join("std");
                    let mut module_dir = module_path.clone();
                    for part in module.split('/') {
                        module_dir = module_dir.join(part);
                    }
                    let lib_vx = module_dir.join("src").join("lib.vx");
                    if lib_vx.exists() {
                        if let Ok(text) = fs::read_to_string(&lib_vx) {
                            if let Ok(mut parser) = vex_parser::Parser::new(&text) {
                                if let Ok(parsed) = parser.parse() {
                                    // Cache AST for future lookups
                                    let uri = match lib_vx.canonicalize() {
                                        Ok(abs) => format!("file://{}", abs.to_string_lossy()),
                                        Err(_) => format!("file://{}", lib_vx.to_string_lossy()),
                                    };
                                    self.ast_cache.insert(uri.clone(), parsed.clone());
                                    if let Some(hover_info) = self.find_symbol_hover_info(&parsed, &word) {
                                        return Ok(Some(Hover { contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: hover_info }), range: None }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Try receiver-based method hover (Counter.new)
            if let Some(recv) = receiver {
                if let Some(hover_info) = self.find_method_hover_info(&ast, &recv, &word) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_info,
                        }),
                        range: None,
                    }));
                }
                // If method not found inline, check free function like `fn counter_new`
                let free_fn = format!("fn {}_{}", recv.to_lowercase(), word);
                if let Some(range) = find_pattern_in_source(&text, &free_fn) {
                    // Format simple hover for this free function
                    let hover_info = format!(
                        "```vex\nfn {}_{}(...)\n```\n\n*Vex function*",
                        recv.to_lowercase(),
                        word
                    );
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_info,
                        }),
                        range: Some(range),
                    }));
                }
            }
            // Fallback: show the word that was found
            Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Symbol**: `{}`\n\n*Vex Language*", word),
                }),
                range: None,
            }))
        }
    }

    fn find_symbol_hover_info(&self, ast: &vex_ast::Program, symbol: &str) -> Option<String> {
        for item in &ast.items {
            match item {
                vex_ast::Item::Function(func) if func.name == symbol => {
                    return Some(self.format_function_hover(func));
                }
                vex_ast::Item::Struct(s) if s.name == symbol => {
                    return Some(self.format_struct_hover(s));
                }
                vex_ast::Item::Enum(e) if e.name == symbol => {
                    return Some(self.format_enum_hover(e));
                }
                vex_ast::Item::Const(c) if c.name == symbol => {
                    return Some(self.format_const_hover(c));
                }
                vex_ast::Item::Contract(trait_) if trait_.name == symbol => {
                    return Some(self.format_trait_hover(trait_));
                }
                _ => {}
            }
        }
        None
    }

    fn find_symbol_hover_info_workspace(&self, symbol: &str) -> Option<String> {
        // Search in the ast_cache for the symbol across other modules
        for entry in self.ast_cache.iter() {
            let ast = entry.value();
            for item in &ast.items {
                match item {
                    vex_ast::Item::Function(func) if func.name == symbol => {
                        return Some(self.format_function_hover(func));
                    }
                    vex_ast::Item::Struct(s) if s.name == symbol => {
                        return Some(self.format_struct_hover(s));
                    }
                    vex_ast::Item::Enum(e) if e.name == symbol => {
                        return Some(self.format_enum_hover(e));
                    }
                    vex_ast::Item::Const(c) if c.name == symbol => {
                        return Some(self.format_const_hover(c));
                    }
                    vex_ast::Item::Contract(trait_) if trait_.name == symbol => {
                        return Some(self.format_trait_hover(trait_));
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn find_method_hover_info(
        &self,
        ast: &vex_ast::Program,
        recv: &str,
        method: &str,
    ) -> Option<String> {
        for item in &ast.items {
            if let vex_ast::Item::Struct(s) = item {
                if s.name == recv {
                    for m in &s.methods {
                        if m.name == method {
                            return Some(self.format_function_hover(m));
                        }
                    }
                }
            }
            // Check external trait impls as well
            if let vex_ast::Item::TraitImpl(impl_) = item {
                if let vex_ast::Type::Named(ref name) = impl_.for_type {
                    if name == recv {
                        for m in &impl_.methods {
                            if m.name == method {
                                return Some(self.format_function_hover(m));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn format_function_hover(&self, func: &vex_ast::Function) -> String {
        let async_str = if func.is_async { "async " } else { "" };
        let params_str = func
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        let return_str = func
            .return_type
            .as_ref()
            .map(|t| format!(": {}", self.type_to_string(t)))
            .unwrap_or_else(|| "".to_string());

        format!(
            "```vex\n{}fn {}({}){}\n```\n\n*Vex function*",
            async_str, func.name, params_str, return_str
        )
    }

    fn format_struct_hover(&self, s: &vex_ast::Struct) -> String {
        let fields_str = s
            .fields
            .iter()
            .map(|f| format!("  {}: {}", f.name, self.type_to_string(&f.ty)))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "```vex\nstruct {} {{\n{}\n}}\n```\n\n*Vex struct*",
            s.name, fields_str
        )
    }

    fn format_enum_hover(&self, e: &vex_ast::Enum) -> String {
        let variants_str = e
            .variants
            .iter()
            .map(|v| {
                if v.data.is_empty() {
                    format!("  {}", v.name)
                } else {
                    let data_str = v
                        .data
                        .iter()
                        .map(|ty| self.type_to_string(ty))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("  {}({})", v.name, data_str)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "```vex\nenum {} {{\n{}\n}}\n```\n\n*Vex enum*",
            e.name, variants_str
        )
    }

    fn format_const_hover(&self, c: &vex_ast::Const) -> String {
        let type_str =
            c.ty.as_ref()
                .map(|t| self.type_to_string(t))
                .unwrap_or("unknown".to_string());
        format!(
            "```vex\nconst {}: {} = ...\n```\n\n*Vex constant*",
            c.name, type_str
        )
    }

    fn format_trait_hover(&self, trait_: &vex_ast::Trait) -> String {
        let methods_str = trait_
            .methods
            .iter()
            .map(|m| {
                let params_str = m
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.name, self.type_to_string(&p.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let return_str = m
                    .return_type
                    .as_ref()
                    .map(|t| format!(": {}", self.type_to_string(t)))
                    .unwrap_or_else(|| "".to_string());
                format!("  fn {}({}){}", m.name, params_str, return_str)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "```vex\ncontract {} {{\n{}\n}}\n```\n\n*Vex contract*",
            trait_.name, methods_str
        )
    }

    fn type_to_string(&self, ty: &vex_ast::Type) -> String {
        // Simple type to string conversion - can be enhanced
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
            vex_ast::Type::U128 => "u128".to_string(),
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
                let args_str = type_args
                    .iter()
                    .map(|arg| self.type_to_string(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args_str)
            }
            vex_ast::Type::Array(elem, size) => {
                format!("[{}; {}]", self.type_to_string(elem), size)
            }
            vex_ast::Type::ConstArray {
                elem_type,
                size_param,
            } => {
                format!("[{}; {}]", self.type_to_string(elem_type), size_param)
            }
            vex_ast::Type::Slice(elem, is_mutable) => {
                let mut_str = if *is_mutable { "mut " } else { "" };
                format!("&{}{}[]", mut_str, self.type_to_string(elem))
            }
            vex_ast::Type::Reference(elem, is_mutable) => {
                let mut_str = if *is_mutable { "mut " } else { "" };
                format!("&{}{}", mut_str, self.type_to_string(elem))
            }
            vex_ast::Type::Union(types) => {
                let types_str = types
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("({})", types_str)
            }
            vex_ast::Type::Tuple(types) => {
                let types_str = types
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", types_str)
            }
            vex_ast::Type::Function {
                params,
                return_type,
            } => {
                let params_str = params
                    .iter()
                    .map(|p| self.type_to_string(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                let return_str = format!(": {}", self.type_to_string(return_type));
                format!("fn({}){}", params_str, return_str)
            }
            vex_ast::Type::Conditional {
                check_type,
                extends_type,
                true_type,
                false_type,
            } => {
                format!(
                    "{} extends {} ? {} : {}",
                    self.type_to_string(check_type),
                    self.type_to_string(extends_type),
                    self.type_to_string(true_type),
                    self.type_to_string(false_type)
                )
            }
            vex_ast::Type::Infer(name) => format!("infer {}", name),
            vex_ast::Type::Typeof(_) => "typeof(...)".to_string(),
            vex_ast::Type::Unit => "()".to_string(),
            vex_ast::Type::Never => "!".to_string(),
            vex_ast::Type::Any => "any".to_string(),
            vex_ast::Type::SelfType => "Self".to_string(),
            vex_ast::Type::RawPtr { inner, is_const } => {
                let const_str = if *is_const { "const " } else { "" };
                format!("*{}{}", const_str, self.type_to_string(inner))
            }
            vex_ast::Type::AssociatedType { self_type, name } => {
                format!("{}::{}", self.type_to_string(self_type), name)
            }
            vex_ast::Type::Option(inner) => format!("Option<{}>", self.type_to_string(inner)),
            vex_ast::Type::Result(ok, err) => format!(
                "Result<{}, {}>",
                self.type_to_string(ok),
                self.type_to_string(err)
            ),
            vex_ast::Type::Vec(inner) => format!("Vec<{}>", self.type_to_string(inner)),
            vex_ast::Type::Box(inner) => format!("Box<{}>", self.type_to_string(inner)),
            vex_ast::Type::Channel(inner) => format!("Channel<{}>", self.type_to_string(inner)),
            vex_ast::Type::Future(inner) => format!("Future<{}>", self.type_to_string(inner)),
            vex_ast::Type::Unknown => "unknown".to_string(),
            vex_ast::Type::Intersection(types) => {
                let types_str = types
                    .iter()
                    .map(|t| self.type_to_string(t))
                    .collect::<Vec<_>>()
                    .join(" & ");
                format!("({})", types_str)
            }
        }
    }
}
