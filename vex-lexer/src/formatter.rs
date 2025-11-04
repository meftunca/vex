use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum FmtToken {
    // --- Trivia (skip ETMİYORUZ) ---
    #[regex(r"[ \t\f]+")]
    Ws, // boşluk/tabs (satır sonu yok)
    #[regex(r"\r?\n")]
    Newline, // satır sonu tekil token
    #[regex(r"//[^\n]*")]
    LineComment, // // yorum (sonunda newline gelirse ayrı Newline gelecek)
    #[regex(r"/\*([^*]|\*[^/])*\*/")]
    BlockComment,

    // --- Delimiters / Operators / Keywords / Ident/Lit ---
    // (Mevcut Token listenizde ne varsa mümkün olduğunca yeniden kullanın)
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
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

    // Types
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
    StringTy,
    #[token("byte")]
    Byte,
    #[token("error")]
    ErrorTy,
    #[token("Map")]
    Map,

    // Intrinsics
    #[token("@vectorize")]
    Vectorize,
    #[token("@gpu")]
    GpuIntrinsic,

    // Operators
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
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
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("::")]
    DoubleColon,
    #[token("..")]
    DotDot,
    #[token("...")]
    DotDotDot,
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
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("!")]
    Not,
    #[token("?")]
    Question,
    #[token(".")]
    Dot,
    #[token("#")]
    Hash,

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

    // Literals/Idents (stringleri olduğu gibi bırakıyoruz)
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)]
    StringLit,
    #[regex(r#"f"([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)]
    FStringLit,
    #[regex(r"[0-9]+\.[0-9]+")]
    FloatLit,
    #[regex(r"[0-9]+")]
    IntLit,
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,
    #[token("_", priority = 10)]
    Underscore,

    #[error]
    Error,
}

#[derive(Debug, Clone)]
pub struct FmtSpan {
    pub tok: FmtToken,
    pub text: String,
}

pub fn lex_for_formatting(input: &str) -> Vec<FmtSpan> {
    let mut out = Vec::new();
    let mut lx = FmtToken::lexer(input);
    while let Some(tok) = lx.next() {
        out.push(FmtSpan {
            tok,
            text: lx.slice().to_string(),
        });
    }
    out
}

pub fn format_vex(src: &str) -> String {
    let toks = lex_for_formatting(src);
    let mut out = String::new();

    let mut indent: usize = 0;
    let mut need_space = false;
    let mut need_newline = false;
    let mut last_was_newline = true;

    // yardımcılar
    let mut write_indent = |out: &mut String, indent: usize| {
        for _ in 0..indent {
            out.push_str("    ");
        } // 4-space indent
    };

    // küçük yardımcı kontroller
    let is_binop = |t: &FmtToken| {
        matches!(
            t,
            FmtToken::Plus
                | FmtToken::Minus
                | FmtToken::Star
                | FmtToken::Slash
                | FmtToken::Percent
                | FmtToken::Eq
                | FmtToken::EqEq
                | FmtToken::NotEq
                | FmtToken::Lt
                | FmtToken::Gt
                | FmtToken::LtEq
                | FmtToken::GtEq
                | FmtToken::And
                | FmtToken::Or
                | FmtToken::PlusEq
                | FmtToken::MinusEq
                | FmtToken::StarEq
                | FmtToken::SlashEq
        )
    };

    // bir sonraki token’a bakmak için
    let mut i = 0usize;
    while i < toks.len() {
        let cur = &toks[i];
        let next = toks.get(i + 1);

        // Newline normalizasyonu: Birden fazla Newline → tek boş satırı korumak istersen burada sayabilirsin.
        match cur.tok {
            FmtToken::Newline => {
                if !last_was_newline {
                    out.push('\n');
                    last_was_newline = true;
                }
                need_space = false;
                need_newline = false;
                i += 1;
                continue;
            }
            FmtToken::Ws => {
                // Tüm ham boşlukları normalize ediyoruz (space bırakma kararını kurallar veriyor)
                i += 1;
                continue;
            }
            FmtToken::LineComment => {
                if last_was_newline {
                    write_indent(&mut out, indent);
                } else if need_space {
                    out.push(' ');
                }
                out.push_str(&cur.text);
                out.push('\n');
                last_was_newline = true;
                need_space = false;
                need_newline = false;
                i += 1;
                continue;
            }
            FmtToken::BlockComment => {
                // Blok yorum: satır içinde ise bir boşlukla ayır
                if last_was_newline {
                    write_indent(&mut out, indent);
                } else if need_space {
                    out.push(' ');
                }
                out.push_str(&cur.text);
                // Ardından tek boşluk bırak (istersen satır kır da diyebilirsin)
                out.push(' ');
                need_space = false;
                last_was_newline = false;
                i += 1;
                continue;
            }
            _ => {}
        }

        // Satır başındaysak girinti yaz
        if last_was_newline {
            write_indent(&mut out, indent);
            last_was_newline = false;
        }

        match cur.tok {
            FmtToken::LBrace => {
                // " ... {" → önce bir boşluk gerekiyorsa koy
                if need_space {
                    out.push(' ');
                }
                out.push('{');
                out.push('\n');
                indent += 1;
                last_was_newline = true;
                need_space = false;
            }
            FmtToken::RBrace => {
                // Kapanmadan önce yeni satır ve indent azalt
                if !last_was_newline {
                    out.push('\n');
                }
                if indent > 0 {
                    indent -= 1;
                }
                write_indent(&mut out, indent);
                out.push('}');
                // 'else' geliyorsa aynı satırda tutmak istersen:
                if let Some(n) = next {
                    if matches!(n.tok, FmtToken::Else | FmtToken::Elif) {
                        out.push(' ');
                        need_space = false;
                        last_was_newline = false;
                    } else {
                        out.push('\n');
                        last_was_newline = true;
                    }
                } else {
                    out.push('\n');
                    last_was_newline = true;
                }
            }
            FmtToken::Semicolon => {
                out.push(';');
                out.push('\n');
                last_was_newline = true;
                need_space = false;
            }
            FmtToken::Comma => {
                out.push(',');
                out.push(' ');
                need_space = false;
            }
            FmtToken::LParen | FmtToken::LBracket => {
                // aç parantez: önce gerekliyse boşluk, sonra parantez; içeri girerken space yok
                if need_space {
                    out.push(' ');
                }
                out.push_str(&cur.text);
                need_space = false;
            }
            FmtToken::RParen | FmtToken::RBracket => {
                // kapanan parantez öncesinde boşluk bırakma
                // önceki token çıktısı zaten yazıldı; sadece parantezi yaz
                out.push_str(&cur.text);
                // kapanıştan sonra; bir sonraki token identifier/keyword ise bir boşluk isteyebilir
                need_space = matches!(
                    next.map(|n| &n.tok),
                    Some(
                        FmtToken::Ident
                            | FmtToken::Fn
                            | FmtToken::Let
                            | FmtToken::If
                            | FmtToken::While
                            | FmtToken::For
                            | FmtToken::Struct
                            | FmtToken::Enum
                            | FmtToken::Trait
                    )
                );
            }
            t if is_binop(&t) => {
                // Operatörlerin etrafına space
                out.push(' ');
                out.push_str(&cur.text);
                out.push(' ');
                need_space = false;
            }
            FmtToken::Dot | FmtToken::DoubleColon | FmtToken::Arrow | FmtToken::FatArrow => {
                // Nokta ve :: ve oklar: çevresinde genellikle space (->, =>) ister;
                // nokta/:: öncesi space yok; sonrası da yok (nitelikli ad).
                if matches!(cur.tok, FmtToken::Arrow | FmtToken::FatArrow) {
                    out.push(' ');
                    out.push_str(&cur.text);
                    out.push(' ');
                    need_space = false;
                } else {
                    out.push_str(&cur.text);
                    need_space = false;
                }
            }
            FmtToken::Not => {
                // '!' genellikle unary; ardından space yok
                out.push('!');
                need_space = false;
            }
            // anahtar kelimeler/ident/literaller
            _ => {
                if need_space {
                    out.push(' ');
                }
                out.push_str(&cur.text);
                // sonraki token türüne göre space isteği
                need_space = matches!(next.map(|n| &n.tok),
                    // identifier veya literal araya giriyorsa space iste
                    Some(FmtToken::Ident | FmtToken::IntLit | FmtToken::FloatLit |
                         FmtToken::StringLit | FmtToken::FStringLit |
                         FmtToken::True | FmtToken::False | FmtToken::Nil |
                         FmtToken::Fn | FmtToken::Let | FmtToken::Struct | FmtToken::Enum |
                         FmtToken::Trait | FmtToken::Impl | FmtToken::If | FmtToken::While |
                         FmtToken::For | FmtToken::Return | FmtToken::Async | FmtToken::Await)
                )
                // fakat şu parantez/virgül/operatörler geliyorsa space bırakma
                && !matches!(next.map(|n| &n.tok),
                    Some(FmtToken::LParen | FmtToken::Comma | FmtToken::Semicolon |
                         FmtToken::RParen | FmtToken::RBracket | FmtToken::Dot |
                         FmtToken::DoubleColon | FmtToken::Arrow | FmtToken::FatArrow));
            }
        }

        i += 1;
    }

    // Sonda tek newline normalizasyonu
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/*
Ne elde edersin?

İdempotent (aynı kodu tekrar biçimleyince değişmez).

Yorumlar korunur (// … satır sonuna sabitlenir, /* … */ yerinde kalır).

Operatör boşlukları ve blok girintisi normalize edilir.

else’ün } ile aynı satırda kalması gibi küçük stil kurallarını yukarıdaki RBrace dalında ayarlayabilirsin.

İleri adımlar (daha profesyonel formatter için)

Lossless CST: rowan veya benzeri bir green tree yapısı ile tüm trivia’yı düğümlere bağlayıp kural bazlı format uygulayın (Rust rust-analyzer tarzı).

Belge modeli: Wadler-Leijen “doc” modeli (group/indent/line) ile satır kırma kararlarını maksimum satır genişliği’ne göre otomatik verin.

Kural tabloları: let, fn, if/else, match, struct alan listesi, enum varyantları, argüman çağırma listeleri… için ayrı grup/indent politikaları.

İstersen, format_vex için unit test’ler ve birkaç stil bayrağı (örn. indent_width, space_before_colon, brace_style) eklenmiş bir sürümü de hazırlayabilirim.
fn main() {
    let src = r#"
fn  main( ){let  x=1+2; if(x>0){ // comment
return x; }else{y=3;}
}
"#;
    let pretty = format_vex(src);
    println!("{pretty}");
}

*/
