// Symbol Resolution - Find symbols in AST for hover/completion/goto-definition

use vex_ast::*;

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub type_info: Option<String>,
    pub documentation: Option<String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Struct,
    Enum,
    Trait,
    Field,
    Method,
}

pub struct SymbolResolver {
    symbols: Vec<SymbolInfo>,
}

impl SymbolResolver {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Extract all symbols from a program
    pub fn extract_symbols(&mut self, program: &Program) {
        for item in &program.items {
            self.visit_item(item);
        }
    }

    /// Find symbol at a specific position
    pub fn find_symbol_at(&self, line: usize, column: usize) -> Option<&SymbolInfo> {
        self.symbols
            .iter()
            .find(|s| s.line == line && column >= s.column && column < s.column + s.name.len())
    }

    fn visit_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.visit_function(func),
            Item::Struct(s) => self.visit_struct(s),
            Item::Enum(e) => self.visit_enum(e),
            Item::Trait(t) => self.visit_trait(t),
            Item::Const(c) => self.visit_const(c),
            Item::TypeAlias(alias) => self.visit_type_alias(alias),
            _ => {}
        }
    }

    fn visit_function(&mut self, func: &Function) {
        let type_info = self.format_function_signature(func);

        self.symbols.push(SymbolInfo {
            name: func.name.clone(),
            kind: SymbolKind::Function,
            type_info: Some(type_info),
            documentation: None, // TODO: Add doc comment support to AST
            line: 0,             // TODO: Add span tracking to AST
            column: 0,
        });

        // Add parameters as symbols
        for param in &func.params {
            self.symbols.push(SymbolInfo {
                name: param.name.clone(),
                kind: SymbolKind::Parameter,
                type_info: Some(self.format_type(&param.ty)),
                documentation: None,
                line: 0,
                column: 0,
            });
        }

        // Visit function body for local variables
        self.visit_block(&func.body);
    }

    fn visit_struct(&mut self, s: &Struct) {
        let mut type_info = format!("struct {}", s.name);
        if !s.fields.is_empty() {
            type_info.push_str(" {\n");
            for field in &s.fields {
                type_info.push_str(&format!(
                    "    {}: {},\n",
                    field.name,
                    self.format_type(&field.ty)
                ));
            }
            type_info.push_str("}");
        }

        self.symbols.push(SymbolInfo {
            name: s.name.clone(),
            kind: SymbolKind::Struct,
            type_info: Some(type_info),
            documentation: None,
            line: 0,
            column: 0,
        });

        // Add fields as symbols
        for field in &s.fields {
            self.symbols.push(SymbolInfo {
                name: field.name.clone(),
                kind: SymbolKind::Field,
                type_info: Some(self.format_type(&field.ty)),
                documentation: None,
                line: 0,
                column: 0,
            });
        }
    }

    fn visit_enum(&mut self, e: &Enum) {
        let mut type_info = format!("enum {}", e.name);
        if !e.variants.is_empty() {
            type_info.push_str(" {\n");
            for variant in &e.variants {
                type_info.push_str(&format!("    {},\n", variant.name));
            }
            type_info.push_str("}");
        }

        self.symbols.push(SymbolInfo {
            name: e.name.clone(),
            kind: SymbolKind::Enum,
            type_info: Some(type_info),
            documentation: None,
            line: 0,
            column: 0,
        });
    }

    fn visit_trait(&mut self, t: &Trait) {
        let type_info = format!("trait {}", t.name);

        self.symbols.push(SymbolInfo {
            name: t.name.clone(),
            kind: SymbolKind::Trait,
            type_info: Some(type_info),
            documentation: None,
            line: 0,
            column: 0,
        });
    }

    fn visit_const(&mut self, c: &Const) {
        let type_str = if let Some(ty) = &c.ty {
            format!("const {}", self.format_type(ty))
        } else {
            "const".to_string()
        };

        self.symbols.push(SymbolInfo {
            name: c.name.clone(),
            kind: SymbolKind::Variable,
            type_info: Some(type_str),
            documentation: None,
            line: 0,
            column: 0,
        });
    }

    fn visit_type_alias(&mut self, alias: &TypeAlias) {
        self.symbols.push(SymbolInfo {
            name: alias.name.clone(),
            kind: SymbolKind::Struct, // Treat as type
            type_info: Some(format!(
                "type {} = {}",
                alias.name,
                self.format_type(&alias.ty)
            )),
            documentation: None,
            line: 0,
            column: 0,
        });
    }

    fn visit_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, ty, .. } => {
                self.symbols.push(SymbolInfo {
                    name: name.clone(),
                    kind: SymbolKind::Variable,
                    type_info: ty.as_ref().map(|t| self.format_type(t)),
                    documentation: None,
                    line: 0,
                    column: 0,
                });
            }
            Statement::If {
                then_block,
                elif_branches,
                else_block,
                ..
            } => {
                self.visit_block(then_block);
                for (_cond, block) in elif_branches {
                    self.visit_block(block);
                }
                if let Some(else_b) = else_block {
                    self.visit_block(else_b);
                }
            }
            Statement::While { body, .. }
            | Statement::For { body, .. }
            | Statement::ForIn { body, .. } => {
                self.visit_block(body);
            }
            Statement::Switch {
                cases,
                default_case,
                ..
            } => {
                for case in cases {
                    self.visit_block(&case.body);
                }
                if let Some(default) = default_case {
                    self.visit_block(default);
                }
            }
            _ => {}
        }
    }

    fn format_function_signature(&self, func: &Function) -> String {
        let mut sig = format!("fn {}(", func.name);

        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, self.format_type(&p.ty)))
            .collect();

        sig.push_str(&params.join(", "));
        sig.push(')');

        if let Some(ret) = &func.return_type {
            sig.push_str(": ");
            sig.push_str(&self.format_type(ret));
        }

        sig
    }

    fn format_type(&self, ty: &Type) -> String {
        match ty {
            Type::I8 => "i8".to_string(),
            Type::I16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::I128 => "i128".to_string(),
            Type::U8 => "u8".to_string(),
            Type::U16 => "u16".to_string(),
            Type::U32 => "u32".to_string(),
            Type::U64 => "u64".to_string(),
            Type::U128 => "u128".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::F128 => "f128".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Byte => "byte".to_string(),
            Type::Error => "error".to_string(),
            Type::Nil => "nil".to_string(),
            Type::Unit => "()".to_string(),
            Type::Never => "!".to_string(),
            Type::Named(name) => name.clone(),
            Type::Reference(inner, mutable) => {
                if *mutable {
                    format!("&{}!", self.format_type(inner))
                } else {
                    format!("&{}", self.format_type(inner))
                }
            }
            Type::Array(element_type, size) => {
                format!("[{}; {}]", self.format_type(element_type), size)
            }
            Type::Slice(element_type, mutable) => {
                if *mutable {
                    format!("&[{}]!", self.format_type(element_type))
                } else {
                    format!("&[{}]", self.format_type(element_type))
                }
            }
            Type::Generic { name, type_args } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    let args: Vec<String> = type_args.iter().map(|t| self.format_type(t)).collect();
                    format!("{}<{}>", name, args.join(", "))
                }
            }
            Type::Function {
                params,
                return_type,
            } => {
                let param_str: Vec<String> = params.iter().map(|p| self.format_type(p)).collect();
                format!(
                    "fn({}): {}",
                    param_str.join(", "),
                    self.format_type(return_type)
                )
            }
            Type::Tuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.format_type(t)).collect();
                format!("({})", type_strs.join(", "))
            }
            Type::Vec(inner) => format!("Vec<{}>", self.format_type(inner)),
            Type::Box(inner) => format!("Box<{}>", self.format_type(inner)),
            Type::Option(inner) => format!("Option<{}>", self.format_type(inner)),
            Type::Result(ok, err) => format!(
                "Result<{}, {}>",
                self.format_type(ok),
                self.format_type(err)
            ),
            Type::Channel(inner) => format!("Channel<{}>", self.format_type(inner)),
            Type::RawPtr(inner) => format!("*{}", self.format_type(inner)),
            Type::Union(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.format_type(t)).collect();
                format!("({})", type_strs.join(" | "))
            }
            Type::Intersection(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.format_type(t)).collect();
                format!("({})", type_strs.join(" & "))
            }
            _ => "unknown".to_string(),
        }
    }
}
