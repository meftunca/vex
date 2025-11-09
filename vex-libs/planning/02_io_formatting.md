# Vex Stdlib Planning - 02: I/O and Formatting

**Priority:** 2
**Status:** Partial (io exists, others missing)
**Dependencies:** builtin, unsafe

## 📦 Packages in This Category

### 2.1 io
**Status:** ✅ Exists (implemented with extern "C")
**Description:** Basic I/O interfaces and primitives

#### Current Implementation
- println, print, eprintln, eprint functions implemented
- Extern "C" calls to vex_print, vex_println, etc.
- String integration with vex_string.c
- Tests: vex-libs/std/io/tests/test_io_*.vx
- Tests: vex-libs/std/io/tests/test_io_*.vx

#### Current Implementation
```vex
// Existing functions
fn println(s: str)
fn print(s: str)
fn readln(): str
```

#### Required Extensions
```vex
// Reader/Writer interfaces
trait Reader {
    fn read(self: &mut Self, buf: &mut [u8]): Result<usize, Error>;
}

trait Writer {
    fn write(self: &mut Self, buf: &[u8]): Result<usize, Error>;
}

trait Closer {
    fn close(self: Self) -> Result<(), Error>;
}

// Utility functions
fn copy(dst: Writer, src: Reader): Result<usize, Error>
fn copy_n(dst: Writer, src: Reader, n: usize): Result<usize, Error>
fn read_all(r: Reader): Result<Vec<u8>, Error>
fn write_string(w: Writer, s: str): Result<usize, Error>

// Byte operations
fn read_byte(r: Reader): Result<u8, Error>
fn write_byte(w: Writer, b: u8): Result<(), Error>
```

#### Dependencies
- builtin
- errors

### 2.2 fmt
**Status:** Partial (placeholder exists)
**Description:** Text formatting and printing

#### Current Implementation
- fmt module exists with basic format() function
- Returns template unchanged (placeholder implementation)
- Formatting logic exists in C runtime (vex_io.c)

#### Required Functions
```vex
// Print functions
fn print(args: ...any)
fn println(args: ...any)
fn printf(format: str, args: ...any)
fn sprintf(format: str, args: ...any): str

// Formatting interfaces
trait Stringer {
    fn string(self: &Self): str;
}

trait Formatter {
    fn format(self: &Self, f: &mut Formatter, verb: rune) -> Result<(), Error>;
}

// Format verbs
// %v - default format
// %+v - struct with field names
// %#v - Go syntax representation
// %T - type
// %d - decimal
// %x - hex
// %f - float
// %s - string
// %p - pointer
// %t - bool
```

#### Dependencies
- builtin
- io
- reflect

#### Notes
- **Language Issue:** Variadic functions (...any) support needed
- **Integration:** Should extend existing println

### 2.3 bufio
**Status:** ❌ Missing (important for performance)
**Description:** Buffered I/O operations

#### Required Types
```vex
struct Reader {
    inner: io.Reader,
    buf: [u8; DEFAULT_BUF_SIZE],
    rd_pos: usize,
    wr_pos: usize,
}

struct Writer {
    inner: io.Writer,
    buf: [u8; DEFAULT_BUF_SIZE],
    pos: usize,
}

struct Scanner {
    reader: Reader,
    buf: Vec<u8>,
    token: []u8,
    split_func: fn([]u8, bool): (usize, []u8, error),
}
```

#### Required Functions
```vex
// Buffered reading
fn new_reader(rd: io.Reader): Reader
fn peek(r: &mut Reader, n: usize): []u8
fn read_line(r: &mut Reader): Result<str, Error>
fn read_bytes(r: &mut Reader, delim: u8): Result<Vec<u8>, Error>
fn read_string(r: &mut Reader, delim: u8): Result<str, Error>

// Buffered writing
fn new_writer(wr: io.Writer): Writer
fn write_string(w: &mut Writer, s: str): Result<usize, Error>
fn write_byte(w: &mut Writer, b: u8): Result<(), Error>
fn flush(w: &mut Writer): Result<(), Error>

// Scanning
fn new_scanner(r: io.Reader): Scanner
fn scan(s: &mut Scanner): bool
fn text(s: &Scanner): str
fn bytes(s: &Scanner): []u8
```

#### Dependencies
- io
- strings

### 2.4 log
**Status:** Partial (logging exists in testing)
**Description:** Logging framework

#### Current Implementation
- Basic logging functions exist in testing module
- vex_log, vex_error, vex_fatal functions available
- No dedicated log package with Logger struct

#### Required Types
```vex
struct Logger {
    prefix: str,
    flag: int,
    out: io.Writer,
    buf: Vec<u8>,
}

enum Level {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}
```

#### Required Functions
```vex
// Logger operations
fn new(out: io.Writer, prefix: str, flag: int): Logger
fn set_output(l: &mut Logger, w: io.Writer)
fn set_prefix(l: &mut Logger, prefix: str)
fn set_flags(l: &mut Logger, flag: int)

// Logging functions
fn print(l: &Logger, args: ...any)
fn printf(l: &Logger, format: str, args: ...any)
fn println(l: &Logger, args: ...any)

// Level-based logging
fn debug(l: &Logger, args: ...any)
fn info(l: &Logger, args: ...any)
fn warn(l: &Logger, args: ...any)
fn error(l: &Logger, args: ...any)
fn fatal(l: &Logger, args: ...any)

// Global logger
static default_logger: Logger
fn print(args: ...any)
fn printf(format: str, args: ...any)
```

#### Dependencies
- io
- fmt
- time
- sync

### 2.5 flag
**Status:** ❌ Missing (useful for CLIs)
**Description:** Command-line flag parsing

#### Required Types
```vex
struct FlagSet {
    name: str,
    parsed: bool,
    actual: Map<str, Flag>,
    formal: Map<str, Flag>,
}

struct Flag {
    name: str,
    value: Value,
    usage: str,
}
```

#### Required Functions
```vex
// Flag definition
fn bool_var(p: &mut bool, name: str, value: bool, usage: str)
fn int_var(p: &mut i64, name: str, value: i64, usage: str)
fn string_var(p: &mut str, name: str, value: str, usage: str)

// Parsing
fn parse()
fn parsed(): bool
fn arg(i: int): str
fn args(): []str
fn n_arg(): int
fn n_flag(): int
```

#### Dependencies
- builtin
- reflect

## 🎯 Implementation Priority

1. **fmt** - Text formatting (extends existing print)
2. **bufio** - Buffered I/O for performance
3. **log** - Logging framework
4. **flag** - Command-line parsing
5. **io** - Extend existing io package

## ⚠️ Language Feature Issues

- **Variadic Functions:** `...any` syntax not confirmed in Vex
- **Global Variables:** Static logger instance needs verification
- **Interface Integration:** Reader/Writer traits with existing io

## 📋 Missing Critical Dependencies

- **Variadic Arguments:** Support for `fn f(args: ...any)`
- **Static Variables:** Global mutable state support
- **Interface Defaults:** Default implementations for traits

## 🚀 Next Steps

1. Implement fmt package with basic formatting
2. Add bufio for buffered operations
3. Create log package
4. Extend io with Reader/Writer interfaces