# vex-formatter

Code formatter for the Vex programming language.

## Features

- ✅ Automatic code formatting with configurable rules
- ✅ Format individual files or directories
- ✅ In-place formatting with `--in-place` flag
- ✅ JSON-based configuration (`vexfmt.json`)
- ✅ AST-based formatting (preserves semantics)

## Usage

### Format and display output

```bash
vex format examples/test.vx
```

### Format in-place (overwrites file)

```bash
vex format examples/test.vx --in-place
```

### Format multiple files

```bash
vex format examples/**/*.vx --in-place
```

## Configuration

Create a `vexfmt.json` file in your project root:

```json
{
  "max_width": 100,
  "indent_size": 4,
  "brace_style": "same_line",
  "trailing_comma": "multiline",
  "quote_style": "double"
}
```

### Configuration Options

| Option           | Type   | Default       | Description                                                    |
| ---------------- | ------ | ------------- | -------------------------------------------------------------- |
| `max_width`      | number | 100           | Maximum line width before wrapping                             |
| `indent_size`    | number | 4             | Number of spaces per indentation level                         |
| `brace_style`    | string | `"same_line"` | Brace placement: `"same_line"` or `"next_line"`                |
| `trailing_comma` | string | `"multiline"` | Trailing comma policy: `"always"`, `"never"`, or `"multiline"` |
| `quote_style`    | string | `"double"`    | String quote style: `"double"`, `"single"`, or `"preserve"`    |

The formatter searches for `vexfmt.json` in:

1. Current directory
2. Parent directories (up to workspace root)
3. Defaults if no config found

## Formatting Rules

### Indentation

- 4 spaces per level (configurable)
- Consistent indentation for blocks, struct fields, enum variants

### Spacing

- Space around binary operators: `a + b`
- Space after commas: `fn(a, b, c)`
- Space after colons in type annotations: `x: i32`
- No space before colons: `fn main(): i32`

### Layout

- Function parameters on same line if fits
- Struct fields one per line
- Enum variants one per line
- Blank line between top-level items

## Supported Constructs

✅ **Supported:**

- Functions (with generics, async, methods)
- Structs (with fields, generics)
- Enums (with variants)
- Traits (with methods)
- Trait implementations
- Type aliases
- Constants
- Statements: let, let!, assign, return, if/elif/else
- Expressions: literals, binary ops, unary ops, calls, method calls, field access, arrays
- Types: primitives, named, references, arrays, generics, function types

🚧 **Partial Support:**

- Closures (parsed but formatted as placeholder)
- Pattern matching (work in progress)
- Import statements (placeholder)

❌ **Not Yet Supported:**

- Macro invocations
- Attributes (preserved but not formatted)
- Complex pattern matching
- String interpolation (f-strings)

## Example

**Input:**

```vex
fn main(): i32 {
let x=42;
let! y   = 10  ;
    let z:i32=x+y;
return z;
}

struct Point{x:i32,y:i32,z:i32}
```

**Output:**

```vex
fn main(): i32 {
    let x = 42;
    let! y = 10;
    let z: i32 = x + y;
    return z;
}

struct Point {
    x: i32,
    y: i32,
    z: i32,
}
```

## Implementation

vex-formatter uses a visitor pattern to traverse the AST produced by vex-parser:

```
Source Code → vex-lexer → Tokens → vex-parser → AST → vex-formatter → Formatted Code
```

**Architecture:**

- `lib.rs`: Public API (`format_file`, `format_source`)
- `config.rs`: Configuration parsing and defaults
- `formatter.rs`: Main formatting logic
- `visitor.rs`: AST traversal and output generation
- `rules/`: Formatting rule modules
  - `indentation.rs`: Indentation calculation
  - `spacing.rs`: Spacing around operators
  - `imports.rs`: Import sorting
  - `expressions.rs`: Expression formatting

## Testing

```bash
# Format test file
vex format examples/test_format.vx

# Format with in-place modification
vex format examples/test_format.vx --in-place

# Format multiple files
find examples -name "*.vx" -exec vex format {} --in-place \;
```

## Future Enhancements

- [ ] Closure formatting
- [ ] Pattern matching formatting
- [ ] Import sorting and grouping
- [ ] Attribute formatting
- [ ] Comment preservation
- [ ] Line wrapping for long lines
- [ ] Custom rule plugins
- [ ] Format-on-save integration

## License

Part of the Vex programming language project.
