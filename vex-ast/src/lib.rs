use serde::{Deserialize, Serialize};

/// Root of the Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
}

/// File is an alias for Program (used in parser)
pub type File = Program;

/// Import kind - how the import is structured
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportKind {
    /// Named imports: import { io, log } from "std";
    Named,
    /// Namespace import: import * as std from "std";
    Namespace(String), // alias name
    /// Module import: import "std/io"; (imports entire module into scope)
    Module,
}

/// Import statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Import {
    pub kind: ImportKind,
    pub items: Vec<String>,    // For Named imports
    pub module: String,        // Module path
    pub alias: Option<String>, // For namespace imports or renaming
}

/// Export statement: export { io, net }; or export fn foo() {}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Export {
    pub items: Vec<String>, // For export { io, net };
}

/// Top-level items (functions, structs, interfaces, type aliases, enums, constants)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Interface(Interface),
    Trait(Trait),
    TraitImpl(TraitImpl),
    TypeAlias(TypeAlias),
    Enum(Enum),
    Const(Const),
    ExternBlock(ExternBlock),
    Export(Export),
}

/// Function definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub attributes: Vec<Attribute>, // #[inline], #[cfg], etc.
    pub is_async: bool,
    pub is_gpu: bool,
    pub receiver: Option<Receiver>, // For methods
    pub name: String,
    pub type_params: Vec<String>, // Generic type parameters: <T, U>
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
}

/// Method receiver: (self: &Vector2) or (self: &mut Vector2)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receiver {
    pub is_mutable: bool,
    pub ty: Type,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

/// Struct definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Struct {
    pub name: String,
    pub type_params: Vec<String>, // Generic type parameters: <T>
    pub fields: Vec<Field>,
}

/// Struct field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub tag: Option<String>, // Go-style tags: `json:"id" db:"pk"`
}

/// Type alias definition: type UserID = u64;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAlias {
    pub name: String,
    pub type_params: Vec<String>, // Generic type parameters
    pub ty: Type,
}

/// Enum definition (Rust/TS-style sum types)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<EnumVariant>,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub data: Option<Type>, // Some(T) or None (unit variant)
}

/// Constant declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Const {
    pub name: String,
    pub ty: Option<Type>,
    pub value: Expression,
}

/// Attribute for declarations (#[inline], #[cfg(target_os = "linux")])
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<AttributeArg>,
}

/// Attribute argument
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeArg {
    Single(String),           // unix, always
    KeyValue(String, String), // target_os = "linux"
    List(Vec<String>),        // [feature1, feature2]
}

/// Extern block for FFI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternBlock {
    pub attributes: Vec<Attribute>, // #[link(name = "c")], #[cfg(unix)]
    pub abi: String,                // "C", "system", etc.
    pub functions: Vec<ExternFunction>,
}

/// Extern function declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternFunction {
    pub attributes: Vec<Attribute>, // #[inline(always)]
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub is_variadic: bool,
}

/// Interface definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    pub type_params: Vec<String>, // Generic type parameters: Cache<K, V>
    pub methods: Vec<InterfaceMethod>,
}

/// Interface method signature
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

/// Trait definition (Rust-style type classes)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trait {
    pub name: String,
    pub type_params: Vec<String>, // Generic type parameters: Converter<T>
    pub methods: Vec<TraitMethod>,
}

/// Trait method signature (no body, just signature)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethod {
    pub name: String,
    pub receiver: Option<Receiver>, // self parameter
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

/// Trait implementation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitImpl {
    pub trait_name: String,
    pub type_params: Vec<String>, // Generic params for the impl
    pub for_type: Type,           // Type implementing the trait
    pub methods: Vec<Function>,   // Method implementations
}

/// Type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Primitive types
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    String,
    Byte,
    Error,
    Nil,

    /// Named type (struct, interface, or custom)
    Named(String),

    /// Generic type with type arguments: Response<T>, Vec<String>
    Generic {
        name: String,
        type_args: Vec<Type>,
    },

    /// Array: [T; N]
    Array(Box<Type>, usize),

    /// Slice: &[T] or &mut [T]
    Slice(Box<Type>, bool), // bool = is_mutable

    /// Reference: &T or &mut T
    Reference(Box<Type>, bool), // bool = is_mutable

    /// Union type: (T1 | T2)
    Union(Vec<Type>),

    /// Intersection type: (T1 & T2)
    Intersection(Vec<Type>),

    /// Tuple: (T1, T2, ...)
    Tuple(Vec<Type>),

    /// Conditional type: T extends U ? X : Y
    Conditional {
        check_type: Box<Type>,
        extends_type: Box<Type>,
        true_type: Box<Type>,
        false_type: Box<Type>,
    },

    /// Infer type (used in conditional types): infer E
    Infer(String),

    /// Unit type (void)
    Unit,
}

/// Block of statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// Statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    /// Variable declaration (let/let mut): let x: i32 = expr;
    Let {
        is_mutable: bool,
        name: String,
        ty: Option<Type>,
        value: Expression,
    },

    /// Variable declaration (old style): x := expr; or int32 x = expr;
    VarDecl {
        is_const: bool,
        name: String,
        ty: Option<Type>,
        value: Expression,
    },

    /// Assignment: x = expr;
    Assign {
        target: Expression,
        value: Expression,
    },

    /// Compound assignment: x += expr;
    CompoundAssign {
        target: Expression,
        op: CompoundOp,
        value: Expression,
    },

    /// Return statement
    Return(Option<Expression>),

    /// If statement
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
    },

    /// For loop: for i in range
    For {
        init: Option<Box<Statement>>,  // for let i = 0; ...
        condition: Option<Expression>, // i < 10
        post: Option<Box<Statement>>,  // i++
        body: Block,
    },

    /// While loop: while condition { body }
    While { condition: Expression, body: Block },

    /// For-in loop: for item in collection
    ForIn {
        variable: String,
        iterable: Expression,
        body: Block,
    },

    /// Switch statement (Go-style)
    Switch {
        value: Option<Expression>, // None for type switch
        cases: Vec<SwitchCase>,
        default_case: Option<Block>,
    },

    /// Select statement (async)
    Select { cases: Vec<SelectCase> },

    /// Unsafe block
    Unsafe(Block),

    /// Expression statement
    Expression(Expression),
}

/// Switch case
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchCase {
    pub patterns: Vec<Expression>, // case 1, 2, 3:
    pub body: Block,
}

/// Select case (async)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectCase {
    pub var: Option<String>, // result variable
    pub expr: Expression,    // await expression
    pub body: Block,
}

/// Match arm: pattern => expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // if guard
    pub body: Expression,
}

/// Pattern for match expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard: _
    Wildcard,
    /// Literal: 42, "hello", true
    Literal(Expression),
    /// Identifier binding: x
    Ident(String),
    /// Tuple pattern: (x, y, z)
    Tuple(Vec<Pattern>),
    /// Struct pattern: Point { x, y }
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    /// Enum variant: Some(x), None
    Enum {
        name: String,
        variant: String,
        data: Option<Box<Pattern>>,
    },
}

/// Compound assignment operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompoundOp {
    Add, // +=
    Sub, // -=
    Mul, // *=
    Div, // /=
}

/// Expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    FStringLiteral(String), // f"..."
    BoolLiteral(bool),
    Nil,

    /// Identifier
    Ident(String),

    /// Binary operation
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },

    /// Unary operation
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },

    /// Function call: foo(a, b)
    Call {
        func: Box<Expression>,
        args: Vec<Expression>,
    },

    /// Method call: obj.method(args)
    MethodCall {
        receiver: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },

    /// Field access: obj.field
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },

    /// Index access: arr[i]
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },

    /// Array literal: [1, 2, 3]
    Array(Vec<Expression>),

    /// Tuple literal: (1, "hello", true)
    TupleLiteral(Vec<Expression>),

    /// Struct literal: Vector2{x: 1.0, y: 2.0} or Box<i32>{value: 10}
    StructLiteral {
        name: String,
        type_args: Vec<Type>, // Generic type arguments: Box<i32>
        fields: Vec<(String, Expression)>,
    },

    /// Range: 0..10
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
    },

    /// Reference: &expr or &mut expr
    Reference {
        is_mutable: bool,
        expr: Box<Expression>,
    },

    /// Dereference: *expr
    Deref(Box<Expression>),

    /// Await: await expr
    Await(Box<Expression>),

    /// Go: go expr
    Go(Box<Expression>),

    /// Try: try expr
    Try(Box<Expression>),

    /// Match expression: match value { pattern => expr, ... }
    Match {
        value: Box<Expression>,
        arms: Vec<MatchArm>,
    },

    /// Launch (HPC): launch func[x, y](args)
    Launch {
        func: String,
        grid: Vec<Expression>,
        args: Vec<Expression>,
    },

    /// New (heap allocation): new(expr)
    New(Box<Expression>),

    /// Make (slice creation): make([T], size)
    Make {
        element_type: Type,
        size: Box<Expression>,
    },

    /// Type cast: expr as Type
    Cast {
        expr: Box<Expression>,
        target_type: Type,
    },

    /// Increment/Decrement: x++ or x--
    PostfixOp {
        expr: Box<Expression>,
        op: PostfixOp,
    },

    /// Error creation: error.new("message")
    ErrorNew(Box<Expression>),
}

/// Binary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,   // -
    Not,   // !
    Ref,   // &
    Deref, // *
}

/// Postfix operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PostfixOp {
    Increment, // ++
    Decrement, // --
}

impl Type {
    /// Check if type is a reference
    pub fn is_reference(&self) -> bool {
        matches!(self, Type::Reference(_, _))
    }

    /// Check if type is mutable
    pub fn is_mutable(&self) -> bool {
        match self {
            Type::Reference(_, m) | Type::Slice(_, m) => *m,
            _ => false,
        }
    }

    /// Get the inner type if this is a reference or slice
    pub fn inner_type(&self) -> Option<&Type> {
        match self {
            Type::Reference(t, _) | Type::Slice(t, _) | Type::Array(t, _) => Some(t),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_constructors() {
        let int_type = Type::I32;
        let ref_type = Type::Reference(Box::new(Type::F64), false);
        let mut_ref_type = Type::Reference(Box::new(Type::String), true);

        assert!(!int_type.is_reference());
        assert!(ref_type.is_reference());
        assert!(!ref_type.is_mutable());
        assert!(mut_ref_type.is_mutable());
    }

    #[test]
    fn test_serialization() {
        let program = Program {
            imports: vec![Import {
                kind: ImportKind::Named,
                items: vec!["io".to_string(), "log".to_string()],
                module: "std".to_string(),
                alias: None,
            }],
            items: vec![],
        };

        let json = serde_json::to_string(&program).unwrap();
        let deserialized: Program = serde_json::from_str(&json).unwrap();
        assert_eq!(program, deserialized);
    }
}
