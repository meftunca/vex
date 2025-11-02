use logos::Logos;

/// Token types for the Vex programming language
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    // Keywords
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("interface")]
    Interface,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
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
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("while")]
    While,
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
    #[token("const")]
    Const,
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
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,
    #[token("string")]
    String,
    #[token("byte")]
    Byte,
    #[token("error")]
    Error,

    // Intrinsics
    #[token("@vectorize")]
    Vectorize,
    #[token("@gpu")]
    GpuIntrinsic,

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
    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token("<=")]
    LtEq,
    #[token(">")]
    Gt,
    #[token(">=")]
    GtEq,
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
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("_")]
    Underscore,
    #[token("#")]
    Hash,
    #[token("...")]
    DotDotDot,

    // Increment/Decrement
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("++")]
    Increment,
    #[token("--")]
    Decrement,
    #[token(":=")]
    ColonEq, // Go-style short variable declaration

    // Literals
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    IntLiteral(i64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse().ok())]
    FloatLiteral(f64),

    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    StringLiteral(String),

    // Formatted string (f"...")
    #[regex(r#"f"([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        s[2..s.len()-1].to_string()
    })]
    FStringLiteral(String),

    // Struct tag (Go-style): `json:"id" db:"pk"`
    #[regex(r"`[^`]*`", |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    Tag(String),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

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
        let source = "fn let mut struct async await";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Fn);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Let);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Mut);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Struct);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Async);
        assert_eq!(lexer.next().unwrap().unwrap().token, Token::Await);
    }

    #[test]
    fn test_literals() {
        let source = r#"42 3.14 "hello" f"world {x}""#;
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next().unwrap().unwrap().token, Token::IntLiteral(42));
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
}
