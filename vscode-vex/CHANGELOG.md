# Change Log

## [0.1.1] - 2025-01-07

### Added

- Extended primitive types: `i128`, `u128`, `f16`, `error`
- Collection types: `Map`, `Set`, `Channel`, `Vec`, `Box`
- Option/Result constructors: `Some`, `None`, `Ok`, `Err`
- Goroutine support: `go` keyword
- Switch statement support
- Defer statement support
- Extensive builtin functions:
  - Memory operations: `alloc`, `free`, `realloc`, `memcpy`, `memset`, `memcmp`, `memmove`
  - String operations: `strlen`, `strcmp`, `strcpy`, `strcat`, `strdup`
  - UTF-8 support: `utf8_valid`, `utf8_char_count`, `utf8_char_at`
  - Type reflection: `typeof`, `type_id`, `type_size`, `type_align`, `is_int_type`, `is_float_type`, `is_pointer_type`
  - LLVM intrinsics: `ctlz`, `cttz`, `ctpop`, `bswap`, `bitreverse`, `sadd_overflow`, `ssub_overflow`, `smul_overflow`
  - Compiler hints: `assume`, `likely`, `unlikely`, `prefetch`
  - Stdlib modules: `logger::*`, `time::*`, `testing::*`

### Updated

- Numeric literal suffixes now include `i128`, `u128`, `f16`
- Enhanced code snippets for new language features:
  - Switch statements
  - Goroutines
  - Async functions
  - Channel, Vec, Map, Set, Box creation
  - Option/Result constructors
  - Logger and testing utilities

### Syntax Features

- All v0.1.1 language features fully supported
- LSP integration ready
- Comprehensive builtin function highlighting

## [0.2.0] - 2025-11-03

### Added

- Initial release with Vex v0.1 syntax support
- Syntax highlighting for all Vex keywords
- Support for `let!` mutable variables
- Support for `&T!` mutable references
- `export` keyword highlighting (NOT `pub`)
- ES6-style import/export syntax
- Code snippets for common patterns
- Vex Dark theme optimized for Vex syntax
- Auto-closing pairs and brackets
- Comment toggling support

### Syntax Features

- Keywords: `fn`, `struct`, `enum`, `trait`, `impl`, `let`, `const`, `export`
- Control flow: `if`, `else`, `match`, `for`, `while`, `loop`, `break`, `continue`, `return`
- Types: `i8-i64`, `u8-u64`, `f32`, `f64`, `bool`, `string`, `char`, `void`
- Operators: All arithmetic, logical, bitwise operators
- Attributes: `@test`, `@inline`, etc.
- Built-in functions: `println`, `assert`, `sizeof`, `typeof`, etc.

### Theme

- Dark theme with distinct colors for:
  - Mutable markers (`!`) in red
  - Keywords in blue
  - Types in teal
  - Functions in yellow
  - Strings in orange
  - Numbers in green
  - Comments in green italic
