# Vex Compiler - Critical Instructions

## ⚠️ IMPORTANT: Cargo Build Location

**Cargo builds to workspace root, NOT project directory!**

```bash
# ❌ WRONG - Don't look here:
./target/release/vex

# ✅ CORRECT - Binary is here:
~/.cargo/target/debug/vex
~/.cargo/target/release/vex
```

### Build Commands

```bash
# Build in release mode
cargo build --release

# Binary location after build
~/.cargo/target/debug/vex
~/.cargo/target/release/vex

# Run the compiler
~/.cargo/target/debug/vex compile examples/calculator.vx
~/.cargo/target/release/vex compile examples/calculator.vx
```

## 📁 Project Structure

```
vex_lang/
├── vex-ast/          # AST definitions
├── vex-lexer/        # Tokenizer
├── vex-parser/       # Parser (recursive descent)
├── vex-compiler/     # LLVM codegen + module resolver
├── vex-cli/          # CLI tool
├── vex-libs/
│   └── std/          # Standard library (8 modules)
├── examples/         # Example programs
└── vex-builds/       # Compiled binaries output
```

## 🔧 Module Resolution

Import system is now integrated:

- ModuleResolver loads from `vex-libs/std/`
- Path conversion: `"std::io"` → `vex-libs/std/io/mod.vx`
- Imports merged into main AST before codegen

## ✅ Working Features

- [x] Parser (recursive descent, 720 lines)
- [x] String type (i8\* pointers, global constants)
- [x] Import parsing (two syntax patterns)
- [x] Module resolver (caching, path conversion)
- [x] Basic codegen (functions, variables, arrays, loops)

## 🚧 In Progress

- [ ] Trait parsing (tokens exist, parser TODO)
- [ ] Async/await codegen
- [ ] Go/launch keywords
- [ ] Vex Runtime (Rust + io_uring)

## 📝 Standard Library Status

8 modules ready in `vex-libs/std/`:

- io, net, http, sync, testing, unsafe, ffi, hpc
- ~2000 lines of Vex code
- Layered architecture (unsafe → safe → app)

## 🧪 Testing

```bash
# Test working examples
/Users/mapletechnologies/.cargo/target/release/vex compile examples/calculator.vx
./vex-builds/calculator

# Test import system
/Users/mapletechnologies/.cargo/target/release/vex compile examples/import_test.vx
```

## 📚 Documentation

- `examples/README.md` - All example programs with status
- `vex-libs/std/README.md` - Standard library API docs
- `vex-libs/STD_IMPLEMENTATION_SUMMARY.md` - Architecture
- `PROGRESS.md` - Development status

## 🎯 Current Task

Integrating ModuleResolver into CLI - imports now resolve and load std library modules!

---

**Last Updated:** 1 Kasım 2025
**Version:** 0.2.0
