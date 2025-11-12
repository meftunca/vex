// AST visitor for formatting

use crate::config::Config;
use vex_ast::*;

/// Formatting visitor that traverses AST and generates formatted output
pub struct FormattingVisitor<'a> {
    config: &'a Config,
    output: String,
    indent_level: usize,
}

impl<'a> FormattingVisitor<'a> {
    /// Create new formatting visitor
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Get formatted output
    pub fn output(self) -> String {
        self.output
    }

    /// Visit program (top-level)
    pub fn visit_program(&mut self, program: &Program) {
        // Format imports first
        for import in &program.imports {
            self.visit_import(import);
        }

        if !program.imports.is_empty() && !program.items.is_empty() {
            self.write_line("");
        }

        // Format items
        for (i, item) in program.items.iter().enumerate() {
            self.visit_item(item);

            // Add blank line between top-level items
            if i < program.items.len() - 1 {
                self.write_line("");
            }
        }
    }
    /// Visit import statement
    fn visit_import(&mut self, _import: &Import) {
        // TODO: Implement import formatting
        self.write_line("// import");
    }

    /// Visit top-level item
    fn visit_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.visit_function(func),
            Item::Struct(struct_def) => self.visit_struct(struct_def),
            Item::Enum(enum_def) => self.visit_enum(enum_def),
            Item::Contract(contract_def) => self.visit_trait(contract_def),
            Item::TraitImpl(impl_block) => self.visit_trait_impl(impl_block),
            Item::BuiltinExtension(_) => self.write_line("// builtin extension"),
            Item::Const(const_decl) => self.visit_const(const_decl),
            Item::Export(_) => self.write_line("// export"),
            Item::TypeAlias(_) => self.write_line("// type alias"),
            Item::Policy(policy) => self.visit_policy(policy),
            Item::ExternBlock(_) => self.write_line("// extern"),
        }
    }

    /// Visit function declaration
    fn visit_function(&mut self, func: &Function) {
        self.write_indent();

        // Async modifier
        if func.is_async {
            self.write("async ");
        }

        self.write("fn ");
        self.write(&func.name);

        // Generic type parameters
        if !func.type_params.is_empty() {
            self.write("<");
            for (i, param) in func.type_params.iter().enumerate() {
                self.write(&param.name);
                if i < func.type_params.len() - 1 {
                    self.write(", ");
                }
            }
            self.write(">");
        }

        self.write("(");

        // Parameters
        for (i, param) in func.params.iter().enumerate() {
            self.write(&param.name);
            self.write(": ");
            self.visit_type(&param.ty);

            if i < func.params.len() - 1 {
                self.write(", ");
            }
        }

        self.write(")");

        // Return type
        if let Some(ref ret_type) = func.return_type {
            self.write(": ");
            self.visit_type(ret_type);
        }

        // Mutability suffix for methods
        if func.is_mutable {
            self.write("!");
        }

        // Body
        self.write(" ");
        self.visit_block(&func.body);
        self.write_line("");
    }

    /// Visit struct definition
    fn visit_struct(&mut self, struct_def: &Struct) {
        self.write_indent();
        self.write("struct ");
        self.write(&struct_def.name);

        // Generic type parameters
        if !struct_def.type_params.is_empty() {
            self.write("<");
            for (i, param) in struct_def.type_params.iter().enumerate() {
                self.write(&param.name);
                if i < struct_def.type_params.len() - 1 {
                    self.write(", ");
                }
            }
            self.write(">");
        }

        // Policies (with clause)
        if !struct_def.policies.is_empty() {
            self.write(" with ");
            for (i, policy) in struct_def.policies.iter().enumerate() {
                self.write(policy);
                if i < struct_def.policies.len() - 1 {
                    self.write(", ");
                }
            }
        }

        self.write(" {");
        self.write_line("");

        self.indent_level += 1;
        for field in &struct_def.fields {
            self.write_indent();
            self.write(&field.name);
            self.write(": ");
            self.visit_type(&field.ty);

            // Add metadata if present
            if let Some(ref metadata) = field.metadata {
                self.write(" `");
                self.write(metadata);
                self.write("`");
            }

            self.write_line(",");
        }
        self.indent_level -= 1;

        self.write_indent();
        self.write_line("}");
    }
    /// Visit enum definition
    fn visit_enum(&mut self, enum_def: &Enum) {
        self.write_indent();
        self.write("enum ");
        self.write(&enum_def.name);

        self.write(" {");
        self.write_line("");

        self.indent_level += 1;
        for variant in &enum_def.variants {
            self.write_indent();
            self.write(&variant.name);
            // Enum variants in Vex AST don't have inline data
            self.write_line(",");
        }
        self.indent_level -= 1;

        self.write_indent();
        self.write_line("}");
    }

    /// Visit trait definition
    fn visit_trait(&mut self, trait_def: &Trait) {
        self.write_indent();
        self.write("trait ");
        self.write(&trait_def.name);

        self.write(" {");
        self.write_line("");

        self.indent_level += 1;
        for method in &trait_def.methods {
            self.write_indent();
            self.write("fn ");
            self.write(&method.name);
            self.write("(");
            // TODO: method parameters
            self.write(")");
            if let Some(ref ret_type) = method.return_type {
                self.write(": ");
                self.visit_type(ret_type);
            }
            self.write_line(";");
        }
        self.indent_level -= 1;

        self.write_indent();
        self.write_line("}");
    }

    /// Visit trait impl block
    fn visit_trait_impl(&mut self, impl_block: &TraitImpl) {
        self.write_indent();
        self.write("impl ");
        self.write(&impl_block.trait_name);
        self.write(" for ");
        match &impl_block.for_type {
            Type::Named(name) => self.write(name),
            Type::Generic { name, type_args } => {
                self.write(name);
                self.write("<");
                for (i, arg) in type_args.iter().enumerate() {
                    self.visit_type(arg);
                    if i < type_args.len() - 1 {
                        self.write(", ");
                    }
                }
                self.write(">");
            }
            _ => self.write("/* type */"),
        }

        self.write(" {");
        self.write_line("");

        self.indent_level += 1;
        for method in &impl_block.methods {
            self.visit_function(method);
        }
        self.indent_level -= 1;

        self.write_indent();
        self.write_line("}");
    }

    /// Visit const declaration
    fn visit_const(&mut self, const_decl: &Const) {
        self.write_indent();
        self.write("const ");
        self.write(&const_decl.name);
        if let Some(ref ty) = const_decl.ty {
            self.write(": ");
            self.visit_type(ty);
        }
        self.write(" = ");
        self.visit_expression(&const_decl.value);
        self.write_line(";");
    }

    /// Visit policy declaration
    fn visit_policy(&mut self, policy: &Policy) {
        self.write_indent();
        self.write("policy ");
        self.write(&policy.name);

        // Parent policies (composition)
        if !policy.parent_policies.is_empty() {
            self.write(" with ");
            for (i, parent) in policy.parent_policies.iter().enumerate() {
                self.write(parent);
                if i < policy.parent_policies.len() - 1 {
                    self.write(", ");
                }
            }
        }

        self.write(" {");
        self.write_line("");

        self.indent_level += 1;
        for field in &policy.fields {
            self.write_indent();
            self.write(&field.name);
            self.write(" `");
            self.write(&field.metadata);
            self.write_line("`,");
        }
        self.indent_level -= 1;

        self.write_indent();
        self.write_line("}");
    }

    /// Visit block (statement list)
    fn visit_block(&mut self, block: &Block) {
        self.write_line("{");
        self.indent_level += 1;

        for stmt in &block.statements {
            self.visit_statement(stmt);
        }

        self.indent_level -= 1;
        self.write_indent();
        self.write("}");
    }

    /// Visit statement
    fn visit_statement(&mut self, stmt: &Statement) {
        self.write_indent();

        match stmt {
            Statement::Let {
                is_mutable,
                name,
                ty,
                value,
            } => {
                if *is_mutable {
                    self.write("let! ");
                } else {
                    self.write("let ");
                }
                self.write(name);
                if let Some(ref t) = ty {
                    self.write(": ");
                    self.visit_type(t);
                }
                self.write(" = ");
                self.visit_expression(value);
                self.write_line(";");
            }
            Statement::Assign { target, value } => {
                self.visit_expression(target);
                self.write(" = ");
                self.visit_expression(value);
                self.write_line(";");
            }
            Statement::Return(expr) => {
                self.write("return");
                if let Some(ref e) = expr {
                    self.write(" ");
                    self.visit_expression(e);
                }
                self.write_line(";");
            }
            Statement::Expression(expr) => {
                self.visit_expression(expr);
                self.write_line(";");
            }
            Statement::If {
                condition,
                then_block,
                elif_branches,
                else_block,
                ..
            } => {
                self.write("if ");
                self.visit_expression(condition);
                self.write(" ");
                self.visit_block(then_block);

                for (elif_cond, elif_block) in elif_branches {
                    self.write(" elif ");
                    self.visit_expression(elif_cond);
                    self.write(" ");
                    self.visit_block(elif_block);
                }

                if let Some(ref else_b) = else_block {
                    self.write(" else ");
                    self.visit_block(else_b);
                }

                self.write_line("");
            }
            Statement::Unsafe(block) => {
                self.write("unsafe ");
                self.visit_block(block);
                self.write_line("");
            }
            Statement::Defer(stmt) => {
                self.write("defer ");
                self.visit_statement(&*stmt);
            }
            Statement::Go(expr) => {
                self.write("go ");
                self.visit_expression(expr);
                self.write_line(";");
            }
            Statement::While {
                condition, body, ..
            } => {
                self.write("while ");
                self.visit_expression(condition);
                self.write(" ");
                self.visit_block(body);
                self.write_line("");
            }
            Statement::For {
                init,
                condition,
                post,
                body,
                ..
            } => {
                self.write("for ");
                if let Some(init_stmt) = init {
                    self.visit_statement(&*init_stmt);
                    self.write("; ");
                }
                if let Some(cond) = condition {
                    self.visit_expression(cond);
                }
                self.write("; ");
                if let Some(post_stmt) = post {
                    self.visit_statement(&*post_stmt);
                }
                self.write(" ");
                self.visit_block(body);
                self.write_line("");
            }
            Statement::Break => {
                self.write_line("break;");
            }
            Statement::Continue => {
                self.write_line("continue;");
            }
            _ => {
                self.write_line("// TODO: statement");
            }
        }
    }

    /// Visit expression
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Ident(name) => {
                self.write(name);
            }
            Expression::IntLiteral(value) => {
                self.write(&value.to_string());
            }
            Expression::FloatLiteral(value) => {
                self.write(&value.to_string());
            }
            Expression::StringLiteral(value) => {
                self.write("\"");
                self.write(value);
                self.write("\"");
            }
            Expression::BoolLiteral(value) => {
                self.write(if *value { "true" } else { "false" });
            }
            Expression::Nil => {
                self.write("nil");
            }
            Expression::Binary {
                left, op, right, ..
            } => {
                self.visit_expression(left);
                self.write(" ");
                self.write(self.format_binary_op(op));
                self.write(" ");
                self.visit_expression(right);
            }
            Expression::Unary { op, expr, .. } => {
                self.write(self.format_unary_op(op));
                self.visit_expression(expr);
            }
            Expression::Call {
                func,
                type_args,
                args,
                ..
            } => {
                self.visit_expression(func);

                // Print type arguments if present
                if !type_args.is_empty() {
                    self.write("<");
                    for (i, ty) in type_args.iter().enumerate() {
                        self.visit_type(ty);
                        if i < type_args.len() - 1 {
                            self.write(", ");
                        }
                    }
                    self.write(">");
                }

                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    self.visit_expression(arg);
                    if i < args.len() - 1 {
                        self.write(", ");
                    }
                }
                self.write(")");
            }
            Expression::MethodCall {
                receiver,
                method,
                type_args,
                args,
                is_mutable_call,
            } => {
                self.visit_expression(receiver);
                self.write(".");
                self.write(method);

                // Format generic type arguments: Vec<i32>.new()
                if !type_args.is_empty() {
                    self.write("<");
                    for (i, ty) in type_args.iter().enumerate() {
                        self.visit_type(ty);
                        if i < type_args.len() - 1 {
                            self.write(", ");
                        }
                    }
                    self.write(">");
                }

                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    self.visit_expression(arg);
                    if i < args.len() - 1 {
                        self.write(", ");
                    }
                }
                self.write(")");
                if *is_mutable_call {
                    self.write("!");
                }
            }
            Expression::FieldAccess { object, field } => {
                self.visit_expression(object);
                self.write(".");
                self.write(field);
            }
            Expression::Array(elements) => {
                self.write("[");
                for (i, elem) in elements.iter().enumerate() {
                    self.visit_expression(elem);
                    if i < elements.len() - 1 {
                        self.write(", ");
                    }
                }
                self.write("]");
            }
            Expression::TypeConstructor {
                type_name,
                type_args,
                args,
            } => {
                // Type constructor: Vec(), Point(10, 20), Vec<i32>()
                self.write(type_name);

                // Print type arguments if present
                if !type_args.is_empty() {
                    self.write("<");
                    for (i, ty) in type_args.iter().enumerate() {
                        self.visit_type(ty);
                        if i < type_args.len() - 1 {
                            self.write(", ");
                        }
                    }
                    self.write(">");
                }

                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    self.visit_expression(arg);
                    if i < args.len() - 1 {
                        self.write(", ");
                    }
                }
                self.write(")");
            }
            Expression::ChannelReceive(channel) => {
                // Go-style channel receive: <-ch
                self.write("<-");
                self.visit_expression(channel);
            }
            _ => {
                self.write("/* expr */");
            }
        }
    }

    /// Format binary operator to string
    fn format_binary_op(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::LtEq => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
        }
    }

    /// Format unary operator to string
    fn format_unary_op(&self, op: &UnaryOp) -> &'static str {
        match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::Ref => "&",
            UnaryOp::Deref => "*",
        }
    }

    /// Visit type annotation
    fn visit_type(&mut self, typ: &Type) {
        match typ {
            Type::I8 => self.write("i8"),
            Type::I16 => self.write("i16"),
            Type::I32 => self.write("i32"),
            Type::I64 => self.write("i64"),
            Type::I128 => self.write("i128"),
            Type::U8 => self.write("u8"),
            Type::U16 => self.write("u16"),
            Type::U32 => self.write("u32"),
            Type::U64 => self.write("u64"),
            Type::U128 => self.write("u128"),
            Type::F16 => self.write("f16"),
            Type::F32 => self.write("f32"),
            Type::F64 => self.write("f64"),
            Type::Bool => self.write("bool"),
            Type::String => self.write("string"),
            Type::Byte => self.write("byte"),
            Type::Nil => self.write("nil"),
            Type::Error => self.write("error"),
            Type::Named(name) => {
                self.write(name);
            }
            Type::Reference(inner, mutable) => {
                self.write("&");
                if *mutable {
                    self.write("!");
                }
                self.visit_type(inner);
            }
            Type::Array(inner, _size) => {
                self.write("[");
                self.visit_type(inner);
                self.write("]");
            }
            Type::Generic { name, type_args } => {
                self.write(name);
                self.write("<");
                for (i, arg) in type_args.iter().enumerate() {
                    self.visit_type(arg);
                    if i < type_args.len() - 1 {
                        self.write(", ");
                    }
                }
                self.write(">");
            }
            Type::Function {
                params,
                return_type,
            } => {
                self.write("fn(");
                for (i, param) in params.iter().enumerate() {
                    self.visit_type(param);
                    if i < params.len() - 1 {
                        self.write(", ");
                    }
                }
                self.write("): ");
                self.visit_type(return_type);
            }
            _ => {
                self.write("/* type */");
            }
        }
    }

    // Helper methods

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_line(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        let indent = " ".repeat(self.indent_level * self.config.indent_size);
        self.write(&indent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor_creation() {
        let config = Config::default();
        let visitor = FormattingVisitor::new(&config);
        assert_eq!(visitor.indent_level, 0);
    }
}
