use serde::{Deserialize, Serialize};

/// Root of the Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
}

/// File is an alias for Program (used in parser)
pub type File = Program;

/// Generic type parameter with optional trait bounds
/// Examples: T, T: Display, T: Display + Clone, F: Callable(i32): i32
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeParam {
    pub name: String,
    pub bounds: Vec<TraitBound>, // Trait bounds: Display, Callable(i32): i32, etc.
}

/// Trait bound - can be a simple trait or a closure trait with signature
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraitBound {
    /// Simple trait: Display, Clone, etc.
    Simple(String),
    /// Closure trait: Callable(T): U, CallableMut(i32, i32): i32, etc.
    Callable {
        trait_name: String,     // Callable, CallableMut, CallableOnce
        param_types: Vec<Type>, // Input parameter types
        return_type: Box<Type>, // Return type
    },
}

// Manual Eq and Hash implementations for TraitBound (Type doesn't implement these)
impl Eq for TraitBound {}

impl std::hash::Hash for TraitBound {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TraitBound::Simple(name) => {
                0u8.hash(state);
                name.hash(state);
            }
            TraitBound::Callable { trait_name, .. } => {
                1u8.hash(state);
                trait_name.hash(state);
                // Skip param_types and return_type as Type doesn't implement Hash
                // This is acceptable as trait_name is usually unique enough
            }
        }
    }
}

impl std::fmt::Display for TraitBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraitBound::Simple(name) => write!(f, "{}", name),
            TraitBound::Callable {
                trait_name,
                param_types,
                return_type,
            } => {
                write!(f, "{}(", trait_name)?;
                for (i, ty) in param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", ty)?; // Use Debug as Type doesn't have Display
                }
                write!(f, "): {:?}", return_type)
            }
        }
    }
}

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

/// Top-level items (functions, structs, traits, type aliases, enums, constants)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Item {
    Function(Function),
    Struct(Struct),
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
    pub type_params: Vec<TypeParam>, // Generic type parameters with bounds: <T: Display, U: Clone>
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

/// Struct definition (v1.3: Inline trait implementation)
/// Example: struct File impl Reader, Writer { fd: i32, fn read() {...} }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Struct {
    pub name: String,
    pub type_params: Vec<TypeParam>, // Generic type parameters with bounds: <T: Display>
    pub impl_traits: Vec<String>,    // Traits this struct implements (inline declaration)
    pub fields: Vec<Field>,
    pub methods: Vec<Function>, // Methods defined inline (including trait implementations)
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
    pub type_params: Vec<TypeParam>, // Generic type parameters with bounds
    pub ty: Type,
}

/// Enum definition (Rust/TS-style sum types)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub type_params: Vec<TypeParam>, // Generic type parameters with bounds
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

/// Interface definition (DEPRECATED in v1.3 - Use Trait instead)
/// This is kept for backward compatibility during migration.
/// Will be removed in future versions.
#[deprecated(
    since = "0.9.0",
    note = "Use Trait instead. Interface keyword removed in v1.3"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    pub type_params: Vec<String>,
    pub methods: Vec<InterfaceMethod>,
}

/// Interface method signature (DEPRECATED - Use TraitMethod)
#[deprecated(since = "0.9.0", note = "Use TraitMethod instead")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}

/// Trait definition (Vex v1.3: Required + default methods)
/// Example: trait Logger { fn log(&Self!, msg); fn info(&Self!, msg) {...} }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trait {
    pub name: String,
    pub type_params: Vec<TypeParam>, // Generic type parameters with bounds: Converter<T: Display>
    pub super_traits: Vec<String>,   // Trait inheritance: trait A: B, C
    pub methods: Vec<TraitMethod>,
}

/// Trait method (required or default)
/// Example: fn log(self: &Self!, msg: string); // required
///          fn info(self: &Self!, msg: string) { self.log("INFO", msg); } // default
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethod {
    pub name: String,
    pub receiver: Option<Receiver>, // self parameter (must use Self type)
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Option<Block>, // Some(...) = default impl, None = required
}

/// Trait implementation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitImpl {
    pub trait_name: String,
    pub type_params: Vec<TypeParam>, // Generic params with bounds for the impl
    pub for_type: Type,              // Type implementing the trait
    pub methods: Vec<Function>,      // Method implementations
}

/// Type system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    /// Primitive types
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    // F16,
    F32,
    F64,
    F128,
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

    /// Function type: fn(T1, T2) -> R
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

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

    /// Never type (!) - for diverging functions (panic, exit, infinite loop)
    Never,

    /// Raw pointer: *T (unsafe, for FFI/C interop)
    RawPtr(Box<Type>),

    // ============================================================
    // Builtin Types - Phase 0 (No imports needed, zero-overhead)
    // ============================================================
    /// Option<T> - Nullable type (Some(T) or None)
    Option(Box<Type>),

    /// Result<T, E> - Error handling (Ok(T) or Err(E))
    Result(Box<Type>, Box<Type>), // (Ok type, Err type)

    /// Vec<T> - Dynamic array (growable, heap-allocated)
    Vec(Box<Type>),

    /// Box<T> - Heap allocation (enables recursive types)
    Box(Box<Type>),

    /// Channel<T> - MPSC channel for concurrency
    Channel(Box<Type>),
}

/// Block of statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// Statements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    /// Variable declaration: let x: i32 = expr; or let! x: i32 = expr;
    Let {
        is_mutable: bool, // false = let, true = let!
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

    /// Break statement
    Break,

    /// Continue statement
    Continue,

    /// Defer statement (Go-style): defer cleanup();
    /// Executes when function exits (LIFO order)
    Defer(Box<Statement>),

    /// If statement with elif support
    If {
        condition: Expression,
        then_block: Block,
        elif_branches: Vec<(Expression, Block)>, // (condition, block) pairs
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

    /// Go statement (async)
    Go(Expression),

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
    /// Or pattern: 1 | 2 | 3 (for SIMD-optimized matching)
    Or(Vec<Pattern>),
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

    /// Array repeat literal: [value; count]
    ArrayRepeat(Box<Expression>, Box<Expression>),

    /// Map literal: {"key": value, "key2": value2}
    MapLiteral(Vec<(Expression, Expression)>),

    /// Tuple literal: (1, "hello", true)
    TupleLiteral(Vec<Expression>),

    /// Struct literal: Vector2{x: 1.0, y: 2.0} or Box<i32>{value: 10}
    StructLiteral {
        name: String,
        type_args: Vec<Type>, // Generic type arguments: Box<i32>
        fields: Vec<(String, Expression)>,
    },

    /// Enum constructor: Result::Ok(42) or Option::None
    EnumLiteral {
        enum_name: String,
        variant: String,
        data: Option<Box<Expression>>, // Some(expr) or None for unit variants
    },

    /// Range: 0..10 (exclusive end)
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
    },

    /// RangeInclusive: 0..=10 (inclusive end)
    RangeInclusive {
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

    /// Match expression: match value { pattern => expr, ... }
    Match {
        value: Box<Expression>,
        arms: Vec<MatchArm>,
    },

    /// Block expression: { stmt1; stmt2; expr }
    Block {
        statements: Vec<Statement>,
        return_expr: Option<Box<Expression>>,
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

    /// Question mark operator: expr? (Result early return)
    /// Desugars to: match expr { Ok(v) => v, Err(e) => return Err(e) }
    QuestionMark(Box<Expression>),

    /// Increment/Decrement: x++ or x--
    PostfixOp {
        expr: Box<Expression>,
        op: PostfixOp,
    },

    /// Error creation: error.new("message")
    ErrorNew(Box<Expression>),

    /// Closure/Lambda: |x, y| expr or |x, y| { stmts; expr }
    Closure {
        params: Vec<Param>,        // Closure parameters
        return_type: Option<Type>, // Optional return type annotation
        body: Box<Expression>,     // Body expression (can be Block)
        capture_mode: CaptureMode, // How closure captures variables
    },
}

/// Closure capture mode (determines which trait it implements)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureMode {
    /// Inferred - will be determined during borrow checking
    Infer,
    /// Immutable capture - implements Callable trait (like Rust's Fn)
    /// Can be called multiple times, captures by immutable reference
    Immutable,
    /// Mutable capture - implements CallableMut trait (like Rust's FnMut)
    /// Can be called multiple times, captures by mutable reference
    Mutable,
    /// Move capture - implements CallableOnce trait (like Rust's FnOnce)
    /// Can only be called once, takes ownership of captures
    Once,
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
