# Vex VS Code Extension

## 📦 Structure

```
vscode-ext/
├── package.json                    # Extension manifest
├── language-configuration.json     # Brackets, comments, indentation
├── syntaxes/
│   └── vex.tmLanguage.json        # Syntax highlighting rules
├── snippets/
│   └── vex.json                   # Code snippets
├── themes/
│   └── vex-dark.json              # Custom dark theme
├── icons/
│   └── vex-icon.svg               # Extension icon
├── install.sh                      # Symlink installer
├── uninstall.sh                    # Uninstaller
├── README.md                       # User documentation
└── CHANGELOG.md                    # Version history
```

## 🎨 Features Implemented

### 1. Syntax Highlighting

- **Keywords**: `fn`, `struct`, `enum`, `trait`, `impl`, `let`, `const`, `export`
- **Control Flow**: `if`, `else`, `match`, `for`, `while`, `return`, `break`, `continue`
- **Types**: All primitive types (i8-i64, u8-u64, f32, f64, bool, string, char)
- **Operators**: Arithmetic, logical, bitwise, comparison
- **Special**:
  - `!` suffix for mutability (highlighted in red)
  - `&T` and `&T!` references
  - `@attribute` syntax
  - `macro!()` syntax

### 2. Code Snippets (30+ snippets)

- `main` - Main function
- `fn` / `efn` - Function / Exported function
- `struct` / `estruct` - Struct / Exported struct
- `impl` / `implt` - Implementation blocks
- `let` / `letm` - Immutable / Mutable variables
- `import` / `importall` - Import statements
- `match`, `for`, `while` - Control flow
- And many more...

### 3. Language Configuration

- Auto-closing: `{}`, `[]`, `()`, `""`, `''`
- Comment toggling: `//` and `/* */`
- Smart indentation
- Bracket matching
- Folding regions

### 4. Vex Dark Theme

Custom color scheme optimized for Vex syntax:

- **Blue** (#569CD6) - Keywords
- **Red** (#FF6B6B) - Mutable markers (`!`)
- **Teal** (#4EC9B0) - Types
- **Yellow** (#DCDCAA) - Functions
- **Orange** (#CE9178) - Strings
- **Green** (#B5CEA8) - Numbers
- **Purple** (#C586C0) - Macros & references

## 🚀 Installation

```bash
cd vscode-ext
./install.sh
```

Then reload VS Code (Cmd+Shift+P → "Developer: Reload Window")

## 🧪 Testing

1. Open any `.vx` file in the project (e.g., `examples/01_basics/hello_world.vx`)
2. Verify syntax highlighting works
3. Try snippets: type `main` and press Tab
4. Test auto-completion: type `{` and it should auto-close with `}`

## 📝 Notes

- Extension is symlinked, so changes take effect immediately after reloading
- No need to rebuild or reinstall after modifications
- Icon is SVG for crisp display at any size

## 🎯 Vex v0.1 Specific Features

✅ **Correctly Highlights:**

- `let!` mutable variables (NOT `let mut`)
- `export` visibility (NOT `pub`)
- `&T!` mutable references (NOT `&mut T`)
- `import { ... } from "path"` ES6-style imports
- Trait implementations with `impl Trait for Type`
- Pattern matching with `match`
- Generics `<T>`

❌ **Does NOT Highlight (Deprecated):**

- `pub` keyword
- `let mut` syntax
- `&mut T` syntax
- `mod` / `use` keywords (use `import`/`export` instead)
