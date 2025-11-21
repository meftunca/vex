use logos::Logos;

/// Helper function to unescape string literals
fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\x08'),
                Some('f') => result.push('\x0C'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('u') => {
                    // Unicode escape: \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(unicode_char) = char::from_u32(code) {
                            result.push(unicode_char);
                        }
                    }
                }
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Token types for the Vex programming language
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    // Keywords
    #[token("fn")]
    Fn,
    #[token("let!")]
    LetMut,
    #[token("let")]
    Let,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,

    #[token("contract")]
    Contract,
    #[token("impl")]
    Impl,
    #[token("policy")]
    Policy,
    #[token("with")]
    With,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("go")]
    Go,
    #[token("gpu")]
    Gpu,
    #[token("launch")]
    Launch,
    #[token("try")]
    Try,
    #[token("return")]
    Return,
    #[token("defer")]
    Defer,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("elif")]
    Elif,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("loop")]
    Loop,
    #[token("in")]
    In,
    #[token("import")]
    Import,
    #[token("export")]
    Export,
    #[token("from")]
    From,
    #[token("as")]
    As,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("nil")]
    Nil,
    #[token("type")]
    Type,
    #[token("extends")]
    Extends,
    #[token("infer")]
    Infer,
    #[token("where")]
    Where,
    #[token("const")]
    Const,
    #[token("typeof")]
    Typeof,
    #[token("unsafe")]
    Unsafe,
    #[token("new")]
    New,
    #[token("make")]
    Make,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("match")]
    Match,
    #[token("select")]
    Select,
    #[token("extern")]
    Extern,

    // Primitive Types
    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("i128")]
    I128,
    #[token("u128")]
    U128,
    #[token("f16")]
    F16,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,
    #[token("string")]
    String,
    #[token("any")]
    Any,
    #[token("byte")]
    Byte,
    #[token("error")]
    Error,
    #[token("Map")]
    Map,
    #[token("Set")]
    Set,

    // // Intrinsics
    // #[token("@vectorize")]
    // Vectorize,
    // #[token("@gpu")]
    // GpuIntrinsic,

    // Operators (base operators before compound assignments)
    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("<<")]
    LShift,
    #[token(">>")]
    RShift,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("?")]
    Question,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token("::")]
    DoubleColon,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEq,
    #[token("->>")]
    Arrow,
    #[token("<-")]
    LeftArrow, // Go-style channel receive
    #[token("=>")]
    FatArrow,
    #[token("...")]
    DotDotDot,

    // Advanced operators
    #[token("**")]
    StarStar,
    #[token("??")]
    QuestionQuestion,

    // Compound Assignment Operators (must come BEFORE single operators)
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("%=")]
    PercentEq,
    #[token("&=")]
    AmpersandEq,
    #[token("|=")]
    PipeEq,
    #[token("^=")]
    CaretEq,
    #[token("<<=")]
    LShiftEq,
    #[token(">>=")]
    RShiftEq,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    // Increment/Decrement
    #[token("++")]
    Increment,
    #[token("--")]
    Decrement,

    // Literals
    // Hex literal with optional type suffix: 0x1A3F, 0xFFu8, 0x1000i64
    // Store as String to support i128/u128 range - parser will validate
    #[regex(r"0[xX][0-9a-fA-F]+(?:i8|i16|i32|i64|i128|u8|u16|u32|u64|u128)?", |lex| lex.slice().to_string())]
    HexLiteral(String),

    // Binary literal with optional type suffix: 0b1010, 0B1111u8, 0b1010i32
    // Store as String to support i128/u128 range - parser will validate
    #[regex(r"0[bB][01]+(?:i8|i16|i32|i64|i128|u8|u16|u32|u64|u128)?", |lex| lex.slice().to_string())]
    BinaryLiteral(String),

    // Octal literal with optional type suffix: 0o777, 0O123u16, 0o777i64
    // Store as String to support i128/u128 range - parser will validate
    #[regex(r"0[oO][0-7]+(?:i8|i16|i32|i64|i128|u8|u16|u32|u64|u128)?", |lex| lex.slice().to_string())]
    OctalLiteral(String),

    // Decimal integer with optional type suffix: 42, 42i64, 100u32, etc.
    // Store as String to support i128/u128 range - parser will validate range
    #[regex(r"[0-9]+(?:i8|i16|i32|i64|i128|u8|u16|u32|u64|u128)?", |lex| lex.slice().to_string())]
    IntLiteral(String),

    // Float literal with optional scientific notation: 3.14, 1.5e10, 2.0E-5
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse().ok())]
    FloatLiteral(f64),

    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        unescape_string(&s[1..s.len()-1])
    })]
    StringLiteral(String),

    // Formatted string (f"...")
    #[regex(r#"f"([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        unescape_string(&s[2..s.len()-1])
    })]
    FStringLiteral(String),

    // Struct tag (Go-style): `json:"id" db:"pk"`
    #[regex(r"`[^`]*`", |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    Tag(String),

    // Operator methods: op+, op-, op*, op/, op%, op==, op[], op++, etc.
    // Must come BEFORE regular identifiers to match first
    // Order matters: longer operators first (op<<= before op<<, op[]= before op[])
    // Operator methods: match only when an operator is present after 'op' (e.g., 'op+')
    // NOTE: op(, op) are NOT operators - they're for function call syntax
    #[regex(r"op(?:\+\+|--|\*\*|<<=|>>=|\.\.=|\?\?|\[\]=|\[\]|[+\-*/%&|^]=|==|!=|<=|>=|<<|>>|&&|\|\||\.\.|\^|~|[+\-*/%<>&|!])", |lex| lex.slice().to_string(), priority = 15)]
    OperatorMethod(String),

    // Identifiers - defined after operator methods
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // Underscore wildcard - higher priority than Ident
    #[token("_", priority = 10)]
    Underscore,
    // Comments (skip)
    #[regex(r"//[^\n]*", logos::skip)]
    LineComment,

    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    BlockComment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSpan {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}

pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<TokenSpan, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.inner.next()?;
        let span = self.inner.span();

        match token {
            Ok(tok) => Some(Ok(TokenSpan { token: tok, span })),
            Err(_) => Some(Err(LexError::InvalidToken { span: span.clone() })),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LexError {
    #[error("Invalid token at {span:?}")]
    InvalidToken { span: std::ops::Range<usize> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let source = "fn let struct async await contract";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Fn);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Let);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Struct);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Async);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Await);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Contract);
    }

    #[test]
    fn test_literals() {
        let source = r#"42 3.14 "hello" f"world {x}""#;
        let mut lexer = Lexer::new(source);

        assert_eq!(
            lexer.next().unwrap().unwrap().token,
            Token::IntLiteral("42".to_string())
        );
        assert_eq!(
            lexer.next().unwrap().unwrap().token,
            Token::FloatLiteral(3.14)
        );
        assert_eq!(
            lexer.next().unwrap().unwrap().token,
            Token::StringLiteral("hello".to_string())
        );
        assert_eq!(
            lexer.next().unwrap().unwrap().token,
            Token::FStringLiteral("world {x}".to_string())
        );
    }

    #[test]
    fn test_identifiers() {
        let source = "my_var count_123 _private";
        let mut lexer = Lexer::new(source);

        assert_eq!(
            lexer.next().unwrap().unwrap().token,
            Token::Ident("my_var".to_string())
        );
        assert_eq!(
            lexer.next().unwrap().unwrap().token,
            Token::Ident("count_123".to_string())
        );
        assert_eq!(
            lexer.next().unwrap().unwrap().token,
            Token::Ident("_private".to_string())
        );
    }

    #[test]
    fn test_function_declaration() {
        let source = "fn main(): error { return nil; }";
        let tokens: Vec<_> = Lexer::new(source).map(|r| r.unwrap().token).collect();

        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Ident("main".to_string()));
        assert_eq!(tokens[2], Token::LParen);
        assert_eq!(tokens[3], Token::RParen);
        assert_eq!(tokens[4], Token::Colon);
        assert_eq!(tokens[5], Token::Error);
    }

    #[test]
    fn test_contract_keyword() {
        let source = "contract Add";
        let tokens: Vec<_> = Lexer::new(source).map(|r| r.unwrap().token).collect();

        assert_eq!(
            tokens[0],
            Token::Contract,
            "Expected Contract token, got {:?}",
            tokens[0]
        );
        assert_eq!(tokens[1], Token::Ident("Add".to_string()));
    }

    #[test]
    fn test_operator_method() {
        let source = "fn op+(x: i32)";
        let tokens: Vec<_> = Lexer::new(source).map(|r| r.unwrap().token).collect();

        println!("Tokens: {:?}", tokens);
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(
            tokens[1],
            Token::OperatorMethod("op+".to_string()),
            "Expected OperatorMethod, got {:?}",
            tokens[1]
        );
        assert_eq!(tokens[2], Token::LParen);
    }
}
