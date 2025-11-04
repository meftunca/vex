# Vex Language Support for Visual Studio Code

Syntax highlighting, snippets, and language support for the Vex programming language.

## Features

- **Syntax Highlighting**: Full syntax highlighting for Vex v0.9
- **Code Snippets**: Common patterns and boilerplate
- **Auto-completion**: Brackets, quotes, and comments
- **Indentation**: Smart indentation for Vex code
- **Theme**: Custom Vex Dark theme optimized for Vex syntax

## Installation

### From Source (Symlink)

```bash
cd vscode-ext
ln -s "$(pwd)" ~/.vscode/extensions/vex-language-0.2.0
```

### Manual Installation

1. Copy the `vscode-ext` folder to `~/.vscode/extensions/vex-language-0.2.0`
2. Reload VS Code

## Syntax v0.9 Features

- `let` / `let!` / `const` variables
- `export` visibility (NOT `pub`)
- `&T` / `&T!` references (NOT `&mut T`)
- `import { ... } from "path"` ES6-style imports
- Traits with `impl` blocks
- Pattern matching with `match`
- Generics with `<T>`

## File Association

Files with `.vx` extension are automatically recognized as Vex files.

## Development

After making changes, run:

```bash
# Reload VS Code window
# Press: Cmd+Shift+P -> "Developer: Reload Window"
```

## License

MIT
