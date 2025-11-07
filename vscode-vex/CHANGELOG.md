# Change Log

## [0.2.0] - 2025-11-03

### Added

- Initial release with Vex v0.9 syntax support
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
