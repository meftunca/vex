# Vex Stdlib Planning - 04: Strings and Text

**Priority:** 4
**Status:** Partial (basic string ops exist, packages missing)
**Dependencies:** builtin, unsafe

## 📦 Packages in This Category

### 4.1 strings
**Status:** ❌ Missing (critical for text processing)
**Description:** String manipulation functions

#### Required Functions
```vex
// Basic operations
fn len(s: str): usize
fn compare(a: str, b: str): int  // -1, 0, 1
fn equal_fold(a: str, b: str): bool
fn has_prefix(s: str, prefix: str): bool
fn has_suffix(s: str, suffix: str): bool
fn contains(s: str, substr: str): bool

// Searching
fn index(s: str, substr: str): usize
fn last_index(s: str, substr: str): usize
fn index_byte(s: str, c: u8): usize
fn index_any(s: str, chars: str): usize
fn index_func(s: str, f: fn(rune): bool): usize

// Splitting
fn split(s: str, sep: str): []str
fn split_n(s: str, sep: str, n: usize): []str
fn split_after(s: str, sep: str): []str
fn split_after_n(s: str, sep: str, n: usize): []str
fn fields(s: str): []str
fn fields_func(s: str, f: fn(rune): bool): []str

// Joining
fn join(elems: []str, sep: str): str
fn repeat(s: str, count: usize): str

// Case conversion
fn to_lower(s: str): str
fn to_upper(s: str): str
fn to_title(s: str): str

// Trimming
fn trim(s: str, cutset: str): str
fn trim_left(s: str, cutset: str): str
fn trim_right(s: str, cutset: str): str
fn trim_space(s: str): str
fn trim_prefix(s: str, prefix: str): str
fn trim_suffix(s: str, suffix: str): str
```

#### Dependencies
- builtin
- unicode

### 4.2 bytes
**Status:** ❌ Missing (important for binary data)
**Description:** Byte slice manipulation

#### Required Types
```vex
type Buffer = Vec<u8>
```

#### Required Functions
```vex
// Similar to strings but for []u8
fn len(b: []u8): usize
fn compare(a: []u8, b: []u8): int
fn equal(a: []u8, b: []u8): bool
fn has_prefix(b: []u8, prefix: []u8): bool
fn has_suffix(b: []u8, suffix: []u8): bool
fn contains(b: []u8, sub: []u8): bool

// Searching
fn index(b: []u8, sub: []u8): usize
fn last_index(b: []u8, sub: []u8): usize
fn index_byte(b: []u8, c: u8): usize
fn index_any(b: []u8, chars: str): usize

// Splitting
fn split(b: []u8, sep: []u8): [][]u8
fn split_n(b: []u8, sep: []u8, n: usize): [][]u8
fn fields(b: []u8): [][]u8

// Joining
fn join(s: [][]u8, sep: []u8): []u8

// Case conversion
fn to_lower(b: []u8): []u8
fn to_upper(b: []u8): []u8
fn to_title(b: []u8): []u8

// Trimming
fn trim(b: []u8, cutset: str): []u8
fn trim_space(b: []u8): []u8
```

#### Dependencies
- builtin
- strings

### 4.3 strconv
**Status:** ❌ Missing (essential for parsing)
**Description:** String conversion utilities

#### Required Functions
```vex
// Integer conversions
fn atoi(s: str): Result<i64, Error>
fn itoa(i: i64): str
fn parse_int(s: str, base: int, bit_size: int): Result<i64, Error>
fn format_int(i: i64, base: int): str

// Float conversions
fn atof(s: str): Result<f64, Error>
fn ftoa(f: f64): str
fn parse_float(s: str, bit_size: int): Result<f64, Error>
fn format_float(f: f64, fmt: u8, prec: int, bit_size: int): str

// Boolean conversions
fn parse_bool(s: str): Result<bool, Error>
fn format_bool(b: bool): str

// Quote handling
fn quote(s: str): str
fn unquote(s: str): Result<str, Error>
fn quote_to_ascii(s: str): str

// Base conversions
fn append_int(dst: []u8, i: i64, base: int): []u8
fn append_float(dst: []u8, f: f64, fmt: u8, prec: int, bit_size: int): []u8
fn append_bool(dst: []u8, b: bool): []u8
fn append_quote(dst: []u8, s: str): []u8
```

#### Dependencies
- builtin
- strings
- errors

### 4.4 unicode
**Status:** ❌ Missing (important for internationalization)
**Description:** Unicode utilities

#### Required Types
```vex
type Rune = i32

struct CaseRange {
    lo: Rune,
    hi: Rune,
    delta: isize,
}
```

#### Required Functions
```vex
// Rune operations
fn is_letter(r: Rune): bool
fn is_digit(r: Rune): bool
fn is_space(r: Rune): bool
fn is_control(r: Rune): bool
fn is_mark(r: Rune): bool
fn is_symbol(r: Rune): bool
fn is_punct(r: Rune): bool

// Case conversion
fn to_lower(r: Rune): Rune
fn to_upper(r: Rune): Rune
fn to_title(r: Rune): Rune
fn simple_fold(r: Rune): Rune

// Categories
fn is_upper(r: Rune): bool
fn is_lower(r: Rune): bool
fn is_title(r: Rune): bool

// UTF-8 handling
fn encode_rune(p: []u8, r: Rune): usize
fn decode_rune(p: []u8): (Rune, usize)
fn decode_last_rune(p: []u8): (Rune, usize)
fn rune_count(p: []u8): usize
fn full_rune(p: []u8): bool
```

#### Dependencies
- builtin

### 4.5 text
**Status:** ❌ Missing (Go has text/scanner, text/tabwriter, etc.)
**Description:** Text processing utilities

#### text/scanner
```vex
struct Scanner {
    src: []u8,
    pos: usize,
    line: usize,
    col: usize,
    tok_pos: usize,
    tok_end: usize,
    tok: Token,
    val: str,
}

enum Token {
    EOF,
    Ident,
    Int,
    Float,
    Char,
    String,
    Comment,
    // ... more tokens
}

fn new(src: str): Scanner
fn next(s: &mut Scanner): Token
fn token_text(s: &Scanner): str
fn token_pos(s: &Scanner): (usize, usize)
```

#### text/tabwriter
```vex
struct Writer {
    output: io.Writer,
    buf: Vec<u8>,
    cell: Vec<u8>,
    widths: Vec<usize>,
    tab_width: usize,
    padding: usize,
    flags: u32,
}

fn new_writer(output: io.Writer, min_width: usize, tab_width: usize, padding: usize, pad_char: u8, flags: u32): Writer
fn write(w: &mut Writer, buf: []u8): Result<usize, Error>
fn flush(w: &mut Writer): Result<(), Error>
```

#### Dependencies
- builtin
- io
- strings

### 4.6 regex (Regular Expressions - Rust-inspired)
**Status:** ❌ Missing (critical for text processing - Rust stdlib'den)
**Description:** Regular expression matching and manipulation

#### Required Types
```vex
struct Regex {
    // compiled regex pattern
}

struct Captures<'t> {
    text: &'t str,
    names: Vec<Option<usize>>,
    matches: Vec<Option<(usize, usize)>>,
}

struct Match<'t> {
    text: &'t str,
    start: usize,
    end: usize,
}

struct Matches<'r, 't> {
    regex: &'r Regex,
    text: &'t str,
    last_end: usize,
    last_match: Option<usize>,
}

struct CaptureMatches<'r, 't> {
    regex: &'r Regex,
    text: &'t str,
    last_end: usize,
    last_match: Option<usize>,
}
```

#### Required Functions
```vex
// Regex construction
fn new(pattern: str): Result<Regex, Error>
fn is_match(regex: &Regex, text: str): bool

// Finding matches
fn find(regex: &Regex, text: str): Option<Match>
fn find_at(regex: &Regex, text: str, start: usize): Option<Match>
fn find_iter(regex: &Regex, text: str): Matches

// Capturing groups
fn captures(regex: &Regex, text: str): Option<Captures>
fn captures_at(regex: &Regex, text: str, start: usize): Option<Captures>
fn captures_iter(regex: &Regex, text: str): CaptureMatches

// Replacement
fn replace(regex: &Regex, text: str, rep: str): str
fn replace_all(regex: &Regex, text: str, rep: str): str

// Splitting
fn split(regex: &Regex, text: str): Vec<str>
fn splitn(regex: &Regex, text: str, limit: usize): Vec<str>

// Information
fn as_str(regex: &Regex): str
fn capture_names(regex: &Regex): Vec<Option<str>>
fn captures_len(regex: &Regex): usize
```

#### Required Types (Advanced)
```vex
struct RegexBuilder {
    pattern: str,
    case_insensitive: bool,
    multi_line: bool,
    dot_matches_new_line: bool,
    swap_greed: bool,
    ignore_whitespace: bool,
    unicode: bool,
    octal: bool,
}

fn builder(): RegexBuilder
fn build(builder: RegexBuilder): Result<Regex, Error>

// Iterator implementations
impl Iterator for Matches {
    type Item = Match;
    fn next(&mut self): Option<Match>
}

impl Iterator for CaptureMatches {
    type Item = Captures;
    fn next(&mut self): Option<Captures>
}
```

#### Dependencies
- builtin
- strings
- vec

#### Notes
- **Rust-inspired:** Regex API'si Rust'ın regex crate'ından esinlenilmiş
- **Performance:** Compiled regex patterns for efficiency
- **Safety:** Memory-safe string operations
- **Unicode:** Full Unicode support

## 🎯 Implementation Priority

1. **strings** - Core string manipulation
2. **strconv** - String/number conversions
3. **bytes** - Byte slice operations
4. **unicode** - Unicode support
5. **regex** - Regular expressions (Rust-inspired)
6. **text/scanner** - Text scanning
7. **text/tabwriter** - Table formatting

## ⚠️ Language Feature Issues

- **Rune Type:** i32 as rune may not be ideal
- **UTF-8 Handling:** Built-in UTF-8 support verification needed
- **String Immutability:** String operations need efficient implementation

## 📋 Missing Critical Dependencies

- **Rune Literals:** `'a'` syntax for rune constants
- **String Builders:** Efficient string concatenation
- **UTF-8 Validation:** Built-in UTF-8 checking

## 🚀 Next Steps

1. Implement strings package with core functions
2. Add strconv for parsing/conversion
3. Implement bytes package
4. Add basic unicode support
5. Create text processing utilities