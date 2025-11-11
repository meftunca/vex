# Lexical Structure

**Version:** 0.1.0 
**Last Updated:** November 3, 2025

This document defines the lexical structure of the Vex programming language, including tokens, identifiers, literals, operators, and comments.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1
7. \1
8. \1

---

## Source Code Encoding

Vex source files:

- **File Extension**: `.vx`
- **Encoding**: UTF-8
- **Line Endings**: LF (`\n`) or CRLF (`\r\n`)
- **BOM**: Not required, but accepted if present

---

## Comments

Vex supports two types of comments that are ignored by the lexer:

### Line Comments

Begin with `//` and continue until the end of the line.

``````vex
// This is a line comment
let x = 42; // Inline comment after code
```

### Block Comments

Begin with `/*` and end with `*/`. Can span multiple lines.

``````vex
/*
 * This is a multi-line
 * block comment
 */

/* Inline block comment */ let y = 100;
```

**Note**: Block comments do not nest in the current implementation.

---

## Whitespace and Line Terminators

The following characters are considered whitespace and are skipped by the lexer:

- Space (U+0020)
- Tab (U+0009)
- Line Feed (U+000A)
- Form Feed (U+000C)

**Regex Pattern**: `[ \t\n\f]+`

Whitespace is used to separate tokens but is otherwise ignored.

---

## Identifiers

Identifiers are names for variables, functions, types, and other program entities.

### Syntax Rules

- **First Character**: Must be a letter (`a-z`, `A-Z`) or underscore (`_`)
- **Subsequent Characters**: Letters, digits (`0-9`), or underscores
- **Case Sensitive**: `myVar`, `MyVar`, and `myvar` are different identifiers

**Regex Pattern**: `[a-zA-Z_][a-zA-Z0-9_]*`

### Valid Identifiers

``````vex
variable
_private
count_123
camelCase
snake_case
PascalCase
__double_underscore
```

### Invalid Identifiers

``````vex
123start     // Cannot start with digit
my-var       // Hyphen not allowed
my.var       // Dot not allowed
fn           // Reserved keyword
```

### Naming Conventions

While not enforced by the compiler, the following conventions are recommended:

• Entity — Convention — Example
• ---------------------- — ---------------- — -----------------
| Variables | snake_case | `user_count` |
| Constants | UPPER_SNAKE_CASE | `MAX_SIZE` |
| Functions | snake_case | `calculate_total` |
| Types (Structs, Enums) | PascalCase | `UserAccount` |
| Traits | PascalCase | `Serializable` |
| Internal/Helper | Prefix with `_` | `_internal_fn` |

---

## Keywords

Keywords are reserved identifiers with special meaning in the language.

### Control Flow Keywords

```
if          else        elif        for
while       in          match       switch
case        default     return      break
continue    defer
```

**Answer**: ✅ `for` keyword mevcut ve çalışıyor. 06_Control_Flow.md'de detaylı dokümante edilmiş.
**Answer**: ✅ `defer` keyword IMPLEMENTED! (Nov 9, 2025) - Go-style resource cleanup with LIFO execution.

### Declaration Keywords

```
fn          let         const       struct
enum        type        trait       impl
extern
```

**Answer**: ❌ `static` keyword eklemiyoruz. Rust'taki `static` yerine Vex'te `const` kullanılıyor. Global değişkenler için gelecekte düşünülebilir ama şu an öncelik değil.

### Type Keywords

```
i8          i16         i32         i64
u8          u16         u32         u64
f32         f64         bool        string
byte        error       nil
```

**Answer**:

- ❌ `void` - Zaten `nil` kullanıyoruz (unit type)
- 🟡 `i128/u128` - Gelecekte eklenebilir (crypto/big numbers için), şu an Low Priority
- ❌ `i256/u256` - Gerek yok, çok spesifik use case (blockchain)
- ❌ `f16/f8/f128` - LLVM desteği sınırlı, şu an öncelik değil. f32/f64 yeterli.

### Module Keywords

```
import      export      from        as
```

### Concurrency Keywords

```
async       await       go          gpu
launch      select
```

### Advanced Keywords

```
unsafe      new         make        try
extends     infer       interface
```

### Boolean Literals

```
true        false
```

**Total Reserved Keywords**: 66

### Deprecated Keywords

- `mut` - Removed in v0.1 (use `let!` for mutable variables)
- `interface` - Use `trait` instead

---

## Operators and Punctuation

### Arithmetic Operators

• Operator — Symbol — Description — Example
• -------------- — ------ — --------------- — -------
| Addition | `+` | Add two values | `a + b` |
| Subtraction | `-` | Subtract values | `a - b` |
| Multiplication | `*` | Multiply values | `a * b` |
| Division | `/` | Divide values | `a / b` |
| Modulo | `%` | Remainder | `a % b` |

### Comparison Operators

• Operator — Symbol — Description
• ---------------- — ------ — ---------------------
| Equal | `==` | Equality test |
| Not Equal | `!=` | Inequality test |
| Less Than | `<` | Less than |
| Less or Equal | `<=` | Less than or equal |
| Greater Than | `>` | Greater than |
| Greater or Equal | `>=` | Greater than or equal |

### Logical Operators

• Operator — Symbol — Description
• ----------- — ------ — ---------------------------
| Logical AND | `&&` | Both conditions true |
| Logical OR | `\|\|` | At least one condition true |
| Logical NOT | `!` | Negate condition |

### Bitwise Operators (Future)

• Operator — Symbol — Description
• ----------- — ------ — -----------
| Bitwise AND | `&` | Bitwise AND |
| Bitwise OR | `\|` | Bitwise OR |
| Bitwise XOR | `^` | Bitwise XOR |
| Left Shift | `<<` | Shift left |
| Right Shift | `>>` | Shift right |

### Assignment Operators

• Operator — Symbol — Description
• --------------- — ------ — -------------------
| Assign | `=` | Assignment |
| Add Assign | `+=` | Add and assign |
| Subtract Assign | `-=` | Subtract and assign |
| Multiply Assign | `*=` | Multiply and assign |
| Divide Assign | `/=` | Divide and assign |
| Modulo Assign | `%=` | Modulo and assign |
| Bitwise AND | `&=` | AND and assign |
| Bitwise OR | `\|=` | OR and assign |
| Bitwise XOR | `^=` | XOR and assign |
| Left Shift | `<<=` | Shift left assign |
| Right Shift | `>>=` | Shift right assign |

**Answer**: ✅ Bitwise assignment operators eklenmeli (Medium Priority 🟡). Bitwise operatörler zaten planned olduğu için bunlar da eklenecek.

**Answer**: ❌ Increment/Decrement (`++`/`--`) operatörleri eklenmeyecek. Açıkça `x = x + 1` veya `x += 1` kullanılmalı (Go ve Rust'ın yaklaşımı gibi). Belirsizliği önler (prefix vs postfix).

### Reference Operators

• Operator — Symbol — Description — Example
• ----------- — ------ — ------------------- — -------
| Reference | `&` | Take reference | `&x` |
| Dereference | `*` | Dereference pointer | `*ptr` |
| Mutable Ref | `!` | Mutable marker | `&x!` |

**Answer**: Stack için `&` reference, heap allocation için `new` keyword kullanılacak (future). Raw pointer için `unsafe` blok içinde manual allocation gerekecek.

**Answer**: ❌ `++`/`--` operatörleri desteklenmeyecek. Açık assignment kullanılmalı: `x += 1` veya `x -= 1`.

### Other Operators

• Operator — Symbol — Description
• ------------- — ------ — ----------------------
| Member Access | `.` | Access field or method |
| Range | `..` | Range operator |
| Variadic | `...` | Variadic arguments |
| Try | `?` | Error propagation |
| Pipe | `\|` | OR pattern in match |

**Answer**: 🟡 Spread/Rest operators (Medium Priority)

- `...arr` - Spread operator (array unpacking)
- Rest parameters in functions: `fn sum(...numbers: i32[])`
- Gelecekte eklenebilir ama şu an öncelik değil. JavaScript/TypeScript pattern'i.

### Delimiters

• Symbol — Name — Usage
• ------- — ----------- — --------------------------------
| `(` `)` | Parentheses | Function calls, grouping, tuples |
| `{` `}` | Braces | Blocks, struct literals |
| `[` `]` | Brackets | Arrays, indexing |
| `,` | Comma | Separate items |
| `;` | Semicolon | Statement terminator |
| `:` | Colon | Type annotations |
| `_` | Underscore | Wildcard pattern |

### Special Symbols

• Symbol — Name — Usage
• ------ — --------- — ---------------------------------
| `=>` | Fat Arrow | Match arms, lambdas |
| `@` | At | Intrinsics (`@vectorize`, `@gpu`) |

**Note**: Vex does NOT use Rust-style `#[attribute]` syntax. Attributes are not part of the language.

---

## Literals

### Integer Literals

Decimal integers without any prefix:

``````vex
0           // Zero
42          // Positive integer
-100        // Negative integer (unary minus + literal)
```

**Type**: `i64` by default (can be inferred or explicitly typed)

**Regex Pattern**: `[0-9]+`

**Future Extensions**:

- Hexadecimal: `0xFF`, `0x1A2B`
- Octal: `0o77`, `0o644`
- Binary: `0b1010`, `0b11110000`
- Underscores: `1_000_000`, `0xFF_FF_FF`

### Floating-Point Literals

Decimal numbers with a decimal point:

``````vex
0.0
3.14
2.71828
-0.5        // Negative (unary minus + literal)
```

**Type**: `f64` by default

**Regex Pattern**: `[0-9]+\.[0-9]+`

**Future Extensions**:

- Scientific notation: `1.5e10`, `2.0E-5`
- Type suffix: `3.14f32`, `2.0f64`

### Boolean Literals

``````vex
true        // Boolean true
false       // Boolean false
```

**Type**: `bool`

### String Literals

Enclosed in double quotes with escape sequences:

``````vex
"Hello, World!"
"Line 1\nLine 2"
"Tab\tseparated"
"Quote: \"Hello\""
"Backslash: \\"
```

**Type**: `string`

**Regex Pattern**: `"([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*"`

**Supported Escape Sequences**:

• Sequence — Meaning
• -------- — ---------------------------------
| `\"` | Double quote |
| `\\` | Backslash |
| `\n` | Newline (LF) |
| `\r` | Carriage return |
| `\t` | Tab |
| `\b` | Backspace |
| `\f` | Form feed |
| `\uXXXX` | Unicode code point (4 hex digits) |

### F-String Literals (Interpolation)

Strings with embedded expressions, prefixed with `f`:

``````vex
let name = "Alice";
let age = 30;
let greeting = f"Hello, {name}! You are {age} years old.";
```

**Type**: `string`

**Regex Pattern**: `f"([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*"`

**Note**: Current implementation parses f-strings but full interpolation support is in progress.

### Nil Literal

``````vex
nil         // Represents absence of value
```

**Type**: Unit type (void)

### Struct Tags (Go-Style)

Metadata attached to struct fields, enclosed in backticks:

``````vex
struct User {
    id: i64       `json:"id" db:"primary_key"`,
    name: string  `json:"name" validate:"required"`,
}
```

**Type**: Metadata (compile-time only)

**Regex Pattern**: `` `[^`]*` ``

---

## Token Types

### Token Categories

The lexer produces tokens in the following categories:

### 1. Keywords (67 tokens)

- Control flow: `if`, `else`, `elif`, `for`, `while`, `match`, `switch`, etc.
- Declarations: `fn`, `let`, `const`, `struct`, `enum`, `trait`, `impl`
- Types: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `bool`, `string`
- Concurrency: `async`, `await`, `go`, `gpu`
- Other: `import`, `export`, `return`, `nil`, `true`, `false`

### 2. Operators (37 tokens)

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Logical: `&&`, `||`, `!`
- Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `%=`
- Reference: `&`, `*`
- Other: `.`, `..`, `?`, `|`

### 3. Delimiters (15 tokens)

- Parentheses: `(`, `)`
- Braces: `{`, `}`
- Brackets: `[`, `]`
- Separators: `,`, `;`, `:`
- Special: `->`, `=>`, `_`, `#`, `...`

### 4. Literals (5 token types)

- `IntLiteral(i64)` - Integer values
- `FloatLiteral(f64)` - Floating-point values
- `StringLiteral(String)` - Regular strings
- `FStringLiteral(String)` - Interpolated strings
- `Tag(String)` - Struct field tags

### 5. Identifiers (1 token type)

- `Ident(String)` - User-defined names

### 6. Intrinsics (2 tokens)

- `@vectorize` - SIMD vectorization hint
- `@gpu` - GPU kernel marker

**Total Token Types**: ~127

### Token Representation

Internally, tokens are represented as:

``````rust
pub struct TokenSpan {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}
```

Where:

- `token`: The token type and associated value
- `span`: Source position (start..end byte offsets)

---

## Lexing Process

### Tokenization Steps

1. **Whitespace Skipping**: Spaces, tabs, newlines, and form feeds are skipped
2. **Comment Removal**: Line and block comments are ignored
3. **Token Matching**: Longest match wins using Logos lexer
4. **Error Handling**: Invalid characters produce `LexError::InvalidToken`

### Ambiguity Resolution

When multiple patterns match, the lexer uses the following rules:

1. **Longest Match**: Prefer longer token (e.g., `==` over `=`)
2. **Keyword Priority**: Keywords take precedence over identifiers
3. **Operator Priority**: Compound operators over simple ones (e.g., `<=` over `<`)

**Examples**:

- `let` → Keyword `Let`, not identifier
- `<=` → Single token `LtEq`, not `Lt` + `Eq`
- `f"string"` → `FStringLiteral`, not `Ident` + `StringLiteral`

### Error Handling

Invalid tokens produce a `LexError`:

``````rust
pub enum LexError {
    InvalidToken { span: std::ops::Range<usize> }
}
```

**Example Error**:

``````vex
let x = @;  // '@' alone is invalid (only @vectorize, @gpu are valid)
```

---

## Implementation Notes

### Lexer Technology

Vex uses the **Logos** lexer generator for efficient tokenization:

- **Declarative**: Token definitions via Rust attributes
- **Zero-Copy**: Slices source without allocation where possible
- **Fast**: Compiled to optimized DFA
- **Error Recovery**: Continues after invalid tokens

### Performance Characteristics

- **Time Complexity**: O(n) where n is source length
- **Space Complexity**: O(1) (streaming, no full token buffer)
- **Throughput**: ~500 MB/s on modern hardware

---

## Examples

### Complete Lexing Example

**Input**:

``````vex
fn add(a: i32, b: i32): i32 {
    return a + b;
}
```

**Tokens**:

[20 lines code: (unknown)]

### String Literals

**Input**:

``````vex
"Hello, \"World\"!\n"
f"User: {name}, Age: {age}"
`json:"user_id"`
```

**Tokens**:

```
StringLiteral("Hello, \"World\"!\n")
FStringLiteral("User: {name}, Age: {age}")
Tag("json:\"user_id\"")
```

---

**Previous**: \1 
**Next**: \1

**Maintained by**: Vex Language Team
