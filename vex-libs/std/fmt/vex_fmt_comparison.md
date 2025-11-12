# Format Library Feature Comparison

## Go fmt Package Features

### Format Verbs

- `%v` - default format
- `%+v` - struct with field names
- `%#v` - Go-syntax representation
- `%T` - type of value
- `%d` - decimal integer
- `%f` - floating point
- `%s` - string
- `%q` - quoted string
- `%x` - hex lowercase
- `%X` - hex uppercase
- `%b` - binary
- `%o` - octal
- `%p` - pointer address
- `%t` - bool

### Width & Precision

- `%5d` - width 5
- `%.2f` - precision 2
- `%5.2f` - width 5, precision 2

### Flags

- `-` - left justify
- `+` - always show sign
- `#` - alternate format (0x for hex)
- `0` - zero padding
- ` ` (space) - space for positive numbers

### Custom Types

- `Stringer` interface: `String() string`

---

## Rust fmt Module Features

### Format Traits

- `Display` - `{}` user-facing
- `Debug` - `{:?}` programmer-facing
- `LowerHex` - `{:x}` hex lowercase
- `UpperHex` - `{:X}` hex uppercase
- `Binary` - `{:b}` binary
- `Octal` - `{:o}` octal
- `Pointer` - `{:p}` memory address

### Format Specs

- `{:5}` - width 5
- `{:.2}` - precision 2
- `{:5.2}` - width 5, precision 2
- `{:>5}` - right align, width 5
- `{:<5}` - left align, width 5
- `{:^5}` - center align, width 5
- `{:*>5}` - fill with \*, right align, width 5
- `{:+}` - always show sign
- `{:#x}` - alternate (0x prefix for hex)
- `{:#?}` - pretty-print debug

### Macros

- `format!("{}", x)` - return String
- `print!("{}", x)` - stdout without newline
- `println!("{}", x)` - stdout with newline
- `eprint!("{}", x)` - stderr without newline
- `eprintln!("{}", x)` - stderr with newline

---

## Vex Current Implementation

### ✅ Implemented

- `{}` - basic placeholder
- Type-safe primitive formatting (i32, i64, u32, u64, f32, f64, bool, String)
- Format spec struct (width, precision, flags, base)
- Native C formatting functions (vex_fmt_i32, vex_fmt_f64, etc.)
- Zero-cost compile-time type dispatch
- C-style variadic printf/sprintf

### ❌ Missing Features

#### 1. Format Spec Parsing

- `{:5}` - width
- `{:.2}` - precision
- `{:5.2}` - width + precision
- `{:x}` - hex format
- `{:b}` - binary format
- `{:o}` - octal format
- `{:>5}` - alignment (left, right, center)
- `{:*>5}` - fill character
- `{:+}` - sign control
- `{:#x}` - alternate format (0x prefix)

#### 2. Display Trait

- Custom Display implementation for user types
- Auto-impl for primitives (partially done in C)

#### 3. Debug Trait

- `{:?}` - debug format
- `{:#?}` - pretty-print debug
- Auto-derive for structs/enums

#### 4. Special Format Traits

- `{:x}` / `{:X}` - hex format trait
- `{:b}` - binary format trait
- `{:o}` - octal format trait
- `{:p}` - pointer format

#### 5. Macros/Builtins

- `println!("{}", x)` - currently uses simple println(x)
- `print!("{}", x)`
- `eprintln!("{}", x)`
- `eprint!("{}", x)`

#### 6. Advanced Features

- Positional arguments: `{0}, {1}, {0}` (reuse args)
- Named arguments: `{name}, {age}`
- Escape sequences: `{{` for `{` (partially implemented)

---

## Priority Implementation Order

### Phase 1: Link vex_fmt.c to Runtime (CRITICAL - BLOCKING)

**Status:** ❌ Blocking all testing
**Action:** Add vex_fmt.c to vex-runtime/build.rs sources

```rust
// In vex-runtime/build.rs, add to sources vec:
c_dir.join("vex_fmt.c"),  // Format library
```

**Files:**

- Source: `vex-libs/std/fmt/native/src/vex_fmt.c` (751 lines)
- Header: `vex-libs/std/fmt/native/src/vex_fmt.h` (143 lines)
- Copy to: `vex-runtime/c/vex_fmt.c` and `vex-runtime/c/vex_fmt.h`

### Phase 2: Format Spec Parsing

**Status:** ⚠️ Parser exists but only handles `{}`
**Action:** Enhance parse_format_string() to parse `{:spec}` syntax
**Features:**

1. Width: `{:5}`
2. Precision: `{:.2}`
3. Combined: `{:5.2}`
4. Format type: `{:x}`, `{:b}`, `{:o}`
5. Alignment: `{:>5}`, `{:<5}`, `{:^5}`
6. Fill: `{:*>5}`
7. Sign: `{:+}`
8. Alternate: `{:#x}`

**Example Parser Output:**

```
"{:5.2}" → FormatSpec { width: 5, precision: 2, ... }
"{:#x}" → FormatSpec { alternate: true, base: 16, ... }
"{:*>5}" → FormatSpec { fill: '*', align: Right, width: 5, ... }
```

### Phase 3: Extended Format Types

**Status:** ❌ Not implemented
**Action:** Add format type dispatch in compile_format_arg()

1. Hex: `{:x}`, `{:X}`, `{:#x}` (0x prefix)
2. Binary: `{:b}`, `{:#b}` (0b prefix)
3. Octal: `{:o}`, `{:#o}` (0o prefix)
4. Pointer: `{:p}` (0x... address)

**Implementation:**

- Parse format type from spec
- Set base field in FormatSpec (2, 8, 10, 16)
- C functions already support bases 2-36

### Phase 4: Display Trait System

**Status:** ❌ Not defined
**Action:** Create Display trait in stdlib

```vex
trait Display {
    fn fmt(self, f: Formatter): Result<(), Error>;
}
```

- Compiler auto-impl for primitives
- Allow custom implementations
- Integrate with format() calls

### Phase 5: Debug Trait

**Status:** ❌ Not defined
**Action:** Create Debug trait

```vex
trait Debug {
    fn fmt(self, f: Formatter): Result<(), Error>;
}
```

- Auto-derive for structs/enums
- `{:?}` calls Debug.fmt()
- `{:#?}` pretty-print (indentation)

### Phase 6: Advanced Arguments

**Status:** ❌ Not implemented
**Features:**

1. Positional: `format("{0} {1} {0}", x, y)` → "x y x"
2. Named: `format("{name} is {age}", name: "Alice", age: 30)`
3. Mix: `format("{0} {name} {1}", x, y, name: "test")`

### Phase 7: Macros/Builtins

**Status:** ❌ Not implemented
**Action:** Add builtin macros

- `println!("{}", x)` - format + println
- `print!("{}", x)` - format + print
- `eprint!("{}", x)` - format + stderr
- `eprintln!("{}", x)` - format + stderr + newline

---

## Feature Parity Score

| Feature Category           | Go  | Rust | Vex | Gap                                 |
| -------------------------- | --- | ---- | --- | ----------------------------------- |
| Basic placeholders         | ✅  | ✅   | ✅  | 0%                                  |
| Width/precision            | ✅  | ✅   | ❌  | 100%                                |
| Alignment                  | ✅  | ✅   | ❌  | 100%                                |
| Number bases (hex/bin/oct) | ✅  | ✅   | 🟡  | 50% (C impl exists, parser missing) |
| Sign control               | ✅  | ✅   | ❌  | 100%                                |
| Fill character             | ❌  | ✅   | ❌  | -                                   |
| Custom Display             | ✅  | ✅   | ❌  | 100%                                |
| Debug format               | ✅  | ✅   | ❌  | 100%                                |
| Positional args            | ❌  | ✅   | ❌  | -                                   |
| Named args                 | ❌  | ✅   | ❌  | -                                   |
| Macros                     | ❌  | ✅   | ❌  | -                                   |
| Variadic (printf-style)    | ✅  | ❌   | ✅  | 0%                                  |

**Overall Gap: ~70% missing features**

---

## Current Blockers

### 🔴 CRITICAL: Linking Error

```
Undefined symbols for architecture arm64:
  "_vex_fmt_buffer_append_str"
  "_vex_fmt_buffer_free"
  "_vex_fmt_buffer_new"
  "_vex_fmt_buffer_to_string"
  "_vex_fmt_f64"
  "_vex_fmt_i32"
  "_vex_fmt_string"
```

**Root Cause:** vex_fmt.c not compiled into vex-runtime
**Solution:** Add to build.rs sources and copy files to vex-runtime/c/

### 🟡 Format Spec Parser Limited

**Current:** Only handles `{}`
**Needed:** Parse `{:5.2}`, `{:#x}`, `{:>5}`, etc.

### 🟡 No Trait System Integration

**Current:** Hardcoded primitive formatting
**Needed:** Display/Debug trait dispatch

---

## Next Steps

1. **Copy vex_fmt.c/h to runtime** → Fix linking
2. **Test basic format()** → Verify `format("{}", 42)` works
3. **Enhance parser** → Support `{:5.2}` syntax
4. **Add format types** → hex, binary, octal
5. **Display trait** → Custom type formatting
6. **Debug trait** → Struct/enum pretty-print
7. **Macros** → println!("{}", x)
