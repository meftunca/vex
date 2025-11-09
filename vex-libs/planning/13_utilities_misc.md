# Vex Stdlib Planning - 13: Utilities and Misc

**Priority:** 13
**Status:** Partial (some utilities exist, many missing)
**Dependencies:** builtin, io, time, sync

## 📦 Packages in This Category

### 13.1 archive/tar
**Status:** ❌ Missing (archive handling)
**Description:** Tar archive format

#### Required Types
```vex
struct Reader {
    r: io.Reader,
    pad: usize,
    curr: *fileReader,
    blk: block,
}

struct Writer {
    w: io.Writer,
    pad: usize,
    curr: *fileWriter,
    hdr: Header,
    blk: block,
}

struct Header {
    name: str,
    mode: i64,
    uid: int,
    gid: int,
    size: i64,
    mod_time: time.Time,
    type_flag: u8,
    link_name: str,
    uname: str,
    gname: str,
    dev_major: i64,
    dev_minor: i64,
    access_time: time.Time,
    change_time: time.Time,
    pax_records: Map<str, str>,
    format: Format,
}
```

#### Required Functions
```vex
fn new_reader(r: io.Reader): *Reader
fn next(r: *Reader): Result<Header, Error>
fn read(r: *Reader, b: []u8): Result<usize, Error>

fn new_writer(w: io.Writer): *Writer
fn write_header(w: *Writer, hdr: &Header): Result<(), Error>
fn write(w: *Writer, b: []u8): Result<usize, Error>
fn close(w: *Writer): Result<(), Error>
```

#### Dependencies
- builtin
- io
- time

### 13.2 archive/zip
**Status:** ❌ Missing (zip archive handling)
**Description:** Zip archive format

#### Required Types
```vex
struct Reader {
    r: io.ReaderAt,
    file: []*File,
    comment: str,
    // internal
}

struct ReadCloser {
    f: *os.File,
    reader_at: io.ReaderAt,
    closer: io.Closer,
    reader: *Reader,
}

struct Writer {
    cw: *countWriter,
    dir: []*header,
    last: *fileWriter,
    closed: bool,
    comment: str,
    compress: CompressionMethod,
}

struct File {
    file_header: FileHeader,
    // internal
}

struct FileHeader {
    name: str,
    creator_version: u16,
    reader_version: u16,
    flags: u16,
    method: u16,
    modified: time.Time,
    modified_time: u16,
    modified_date: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    compressed_size64: u64,
    uncompressed_size64: u64,
    extra: []u8,
    external_attrs: u32,
}
```

#### Required Functions
```vex
fn new_reader(r: io.ReaderAt, size: i64): Result<*Reader, Error>
fn open_reader(name: str): Result<*ReadCloser, Error>

fn new_writer(w: io.Writer): *Writer
fn create(w: *Writer, name: str): Result<io.Writer, Error>
fn create_header(w: *Writer, fh: &FileHeader): Result<io.Writer, Error>
fn close(w: *Writer): Result<(), Error>
```

#### Dependencies
- builtin
- io
- time
- os

### 13.3 compress/gzip
**Status:** ❌ Missing (compression)
**Description:** Gzip compression

#### Required Types
```vex
struct Reader {
    header: Header,
    r: io.Reader,
    decompressor: flate.Reader,
    digest: hash.Hash32,
    size: uint32,
    err: Error,
    multistream: bool,
}

struct Writer {
    header: Header,
    w: io.Writer,
    compressor: flate.Writer,
    digest: hash.Hash32,
    size: uint32,
    closed: bool,
    buf: [10]u8,
    err: Error,
}

struct Header {
    comment: str,
    extra: []u8,
    mod_time: time.Time,
    name: str,
    os: u8,
}
```

#### Required Functions
```vex
fn new_reader(r: io.Reader): Result<*Reader, Error>
fn new_writer(w: io.Writer): *Writer
fn new_writer_level(w: io.Writer, level: int): Result<*Writer, Error>
```

#### Dependencies
- builtin
- io
- time
- flate

### 13.4 compress/zlib
**Status:** ❌ Missing (zlib compression)
**Description:** Zlib compression

#### Required Types
```vex
struct Reader {
    r: flate.Reader,
    // internal
}

struct Writer {
    w: flate.Writer,
    // internal
}
```

#### Required Functions
```vex
fn new_reader(r: io.Reader): Result<io.ReadCloser, Error>
fn new_reader_dict(r: io.Reader, dict: []u8): Result<io.ReadCloser, Error>
fn new_writer(w: io.Writer): *Writer
fn new_writer_level(w: io.Writer, level: int): Result<*Writer, Error>
fn new_writer_level_dict(w: io.Writer, level: int, dict: []u8): Result<*Writer, Error>
```

#### Dependencies
- builtin
- io
- flate

### 13.5 expvar
**Status:** ❌ Missing (runtime metrics)
**Description:** Public variable export

#### Required Types
```vex
trait Var {
    fn string(self: &Self): str
}

struct Int {
    i: i64,
}

struct Float {
    f: f64,
}

struct String {
    s: str,
}

struct Func {
    f: fn(): any,
}

struct Map {
    m: sync.Map,
}
```

#### Required Functions
```vex
fn new_int(name: str): *Int
fn new_float(name: str): *Float
fn new_string(name: str): *String
fn new_map(name: str): *Map
fn publish(name: str, v: Var)
fn get(name: str): Var
fn do_func(fn(): any): Var
fn handler(): http.Handler
```

#### Dependencies
- builtin
- sync
- http

### 13.6 flag
**Status:** ❌ Missing (command-line flags - also in I/O section)
**Description:** Command-line flag parsing

#### Required Types
```vex
struct FlagSet {
    usage: fn(),
    name: str,
    parsed: bool,
    actual: Map<str, *Flag>,
    formal: Map<str, *Flag>,
}

struct Flag {
    name: str,
    value: Value,
    def_value: str,
    changed: bool,
    value_set: bool,
}
```

#### Required Functions
```vex
fn new_flag_set(name: str, error_handling: ErrorHandling): *FlagSet
fn parse()
fn parsed(): bool
fn arg(i: int): str
fn args(): []str
fn n_arg(): int
fn n_flag(): int
fn bool_var(p: *bool, name: str, value: bool, usage: str)
fn int_var(p: *i64, name: str, value: i64, usage: str)
fn string_var(p: *str, name: str, value: str, usage: str)
```

#### Dependencies
- builtin
- reflect

### 13.7 mime
**Status:** ❌ Missing (MIME type handling)
**Description:** MIME type parsing and handling

#### Required Functions
```vex
fn type_by_extension(ext: str): str
fn extensions_by_type(typ: str): []str
fn add_extension_type(ext: str, typ: str)
fn format_media_type(t: str, param: Map<str, str>): str
fn parse_media_type(v: str): Result<(str, Map<str, str>), Error>
```

#### Dependencies
- builtin
- strings

### 13.8 text/scanner
**Status:** ❌ Missing (text scanning - also in strings section)
**Description:** Text scanner for parsing

#### Required Types
```vex
struct Scanner {
    src: []u8,
    srcp: usize,
    src_end: usize,
    line: usize,
    column: usize,
    last_line_len: usize,
    last_char_len: usize,
    tok: Token,
    tokbuf: []u8,
    buf: []u8,
    error: fn(s: *Scanner, msg: str),
    error_count: usize,
    mode: u32,
    whitespace: u64,
    is_ident_rune: fn(ch: rune, i: int): bool,
    skip_comments: bool,
    comments: []u8,
}
```

#### Required Functions
```vex
fn new(src: str): *Scanner
fn init(s: *Scanner, src: str)
fn next(s: *Scanner): rune
fn peek(s: *Scanner): rune
fn token_text(s: *Scanner): str
fn token_bytes(s: *Scanner): []u8
fn skip_whitespace(s: *Scanner)
fn scan(s: *Scanner): Token
```

#### Dependencies
- builtin
- strings
- unicode

### 13.9 plugin
**Status:** ❌ Missing (plugin system)
**Description:** Dynamic plugin loading

#### Required Functions
```vex
fn open(path: str): Result<*Plugin, Error>
fn lookup(p: *Plugin, sym_name: str): Result<Symbol, Error>
```

#### Dependencies
- builtin
- reflect

### 13.10 panic (Panic Handling - Rust-inspired)
**Status:** ❌ Missing (important for error handling)
**Description:** Panic handling and recovery (Rust std::panic)

#### Required Types
```vex
struct PanicInfo {
    payload: Box<dyn Any>,
    message: Option<&'static str>,
    location: Option<&Location>,
}

struct Location {
    file: &'static str,
    line: u32,
    col: u32,
}

struct Backtrace {
    // captured stack trace
}
```

#### Required Functions
```vex
// Panic functions
fn panic_any(payload: Box<dyn Any>) -> !
fn catch_unwind<F, R>(f: F): Result<R, Box<dyn Any>> where F: FnOnce() -> R
fn resume_unwind(payload: Box<dyn Any>) -> !
fn set_hook(hook: fn(&PanicInfo))
fn take_hook(): fn(&PanicInfo)

// Panic info access
fn payload(p: &PanicInfo): &dyn Any
fn message(p: &PanicInfo): Option<&str>
fn location(p: &PanicInfo): Option<&Location>

// Location info
fn file(loc: &Location): &str
fn line(loc: &Location): u32
fn column(loc: &Location): u32
```

#### Dependencies
- builtin
- any
- backtrace

### 13.11 backtrace (Stack Traces - Rust-inspired)
**Status:** ❌ Missing (debugging support)
**Description:** Stack trace capture and printing (Rust std::backtrace)

#### Required Types
```vex
struct Backtrace {
    // internal representation
}

enum BacktraceStatus {
    Unsupported,
    Disabled,
    Captured,
}

struct BacktraceFrame {
    ip: *mut u8,
    symbol_address: *mut u8,
    // symbol information
}
```

#### Required Functions
```vex
// Backtrace capture
fn capture() -> Backtrace
fn force_capture() -> Backtrace

// Backtrace status
fn status(bt: &Backtrace): BacktraceStatus

// Frame access
fn frames(bt: &Backtrace): &[BacktraceFrame]

// Frame information
fn ip(frame: &BacktraceFrame): *mut u8
fn symbol_address(frame: &BacktraceFrame): *mut u8
fn filename(frame: &BacktraceFrame): Option<&str>
fn lineno(frame: &BacktraceFrame): Option<u32>
fn name(frame: &BacktraceFrame): Option<&str>

// Formatting
fn resolve(frame: &BacktraceFrame): BacktraceSymbol
fn print(bt: &Backtrace, fmt: &mut fmt::Formatter) -> fmt::Result
```

#### Dependencies
- builtin
- fmt

#### Notes
- **Platform Dependent:** Stack trace capture may not work on all platforms
- **Symbol Resolution:** Requires debug symbols for meaningful output
- **Performance:** Stack trace capture can be expensive

## 🎯 Implementation Priority

1. **flag** - Command-line flag parsing
2. **mime** - MIME type handling
3. **text/scanner** - Text scanning utilities
4. **panic** - Panic handling (Rust-inspired)
5. **backtrace** - Stack traces (Rust-inspired)
6. **compress/gzip** - Gzip compression
7. **compress/zlib** - Zlib compression
8. **archive/tar** - Tar archives
9. **archive/zip** - Zip archives
10. **expvar** - Runtime metrics
11. **plugin** - Plugin system

## ⚠️ Language Feature Issues

- **Dynamic Loading:** Plugin system needs runtime symbol resolution
- **Compression Algorithms:** May require C library bindings
- **Archive Formats:** Complex format parsing

## 📋 Missing Critical Dependencies

- **Dynamic Linking:** Runtime plugin loading
- **Compression Libraries:** Efficient compression algorithms
- **Archive Parsers:** Robust archive format handling

## 🚀 Next Steps

1. Implement flag package for CLI parsing
2. Add MIME type utilities
3. Create text scanner
4. Implement compression algorithms
5. Add archive format support
6. Create runtime metrics system
7. Implement plugin loading system