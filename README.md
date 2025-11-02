# Vex Programming Language

**Version:** 0.2.0  
**Status:** Production-ready core, expanding features  
**Test Coverage:** 29/59 passing (49.2%)

A modern systems programming language with Go/Rust ergonomics, async-first design, and GPU compute capabilities.

---

## 🚀 Quick Start

### Install & Build

```bash
git clone https://github.com/yourusername/vex_lang.git
cd vex_lang
cargo build --release
```

### Run Examples

```bash
# Run directly
cargo run --bin vex run examples/fibonacci.vx

# Compile to binary
cargo run --bin vex compile examples/calculator.vx -o calculator
./calculator
```

### Hello World

```vex
fn main(): i32 {
    print("Hello, Vex!");
    return 0;
}
```

---

## ✨ Features

### ✅ Fully Working

- **Functions:** Regular, generic, methods with receivers
- **Control Flow:** if/else, while, for, switch/case ✨NEW
- **Data Structures:** Structs, enums (with constructors ✨NEW), arrays, tuples
- **Type System:** Generics, interfaces, type aliases, references
- **Operators:** Arithmetic, comparison, logical
- **Strings:** Literals and f-strings

### 🚧 In Progress

- **Async/Await:** Parser in progress
- **Match Expressions:** Planned for union types
- **Traits:** Parser in progress
- **Union Types:** Parsed, codegen pending

### 📋 Planned

- **GPU/CUDA:** Kernel support
- **SIMD:** Vectorization
- **Full Async Runtime:** io_uring integration

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **[LANGUAGE_FEATURES.md](LANGUAGE_FEATURES.md)** | **📖 Complete feature list with examples** |
| [Specification.md](Specification.md) | Language specification |
| [REFACTORING_SUCCESS.md](REFACTORING_SUCCESS.md) | Compiler refactoring details |
| [MISSING_FEATURES.md](MISSING_FEATURES.md) | Feature implementation tracker |
| [archive-docs/](archive-docs/) | Old documentation |

---

## 🎯 Working Examples

### Algorithms ✅
```bash
cargo run --bin vex run examples/fibonacci.vx      # Output: 55
cargo run --bin vex run examples/factorial.vx      # Output: 120
cargo run --bin vex run examples/power.vx          # Output: 1024
cargo run --bin vex run examples/gcd.vx            # Output: 6
```

### Data Structures ✅
```bash
cargo run --bin vex run examples/struct_test.vx           # Structs
cargo run --bin vex run examples/enum_constructor_test.vx # Enums: 0,1,2
cargo run --bin vex run examples/switch_test.vx           # Switch: Exit 20
```

---

## 🎓 Language Examples

### Switch Statement ✨NEW
```vex
fn classify(x: i32): i32 {
    switch x {
        case 1:
            return 10;
        case 2, 3:
            return 20;
        default:
            return 0;
    }
}
```

### Enums with Constructors ✨NEW
```vex
enum Status {
    Pending,   // 0
    Active,    // 1
    Complete,  // 2
}

fn main(): i32 {
    status := Status_Active();
    return status;  // Returns 1
}
```

### More Examples
See [LANGUAGE_FEATURES.md](LANGUAGE_FEATURES.md) for complete feature list.

---

## 📊 Progress

**Current:** 29/59 tests passing (49.2%)

- ✅ **Phase 1 Complete:** Core language + Quick wins
- 🚧 **Phase 2 In Progress:** Type system (Match, Union, Traits)
- 📋 **Phase 3 Planned:** Advanced features (GPU, Async runtime)

**Roadmap:** See [MISSING_FEATURES.md](MISSING_FEATURES.md)

---

## 🤝 Contributing

Contributions welcome! See [LANGUAGE_FEATURES.md](LANGUAGE_FEATURES.md) for implementation status.

---

## 📜 License

MIT License

---

**Last Updated:** 2 Kasım 2025
