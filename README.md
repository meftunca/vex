# Vex Programming Language (v0.9)

**Version:** 0.2.0 (Syntax v0.9)  
**Status:** Core features working, clean examples  
**Test Coverage:** 71% working examples

A modern systems programming language with clean syntax, zero-overhead abstractions, and a focus on correctness.

**v0.9 Highlights:**

- ✅ Unified variable system: `let` (immutable) vs `let!` (mutable)
- ✅ Consistent reference syntax: `&T` vs `&T!`
- ✅ Removed redundant syntax (`:=`, `var`, `&mut`)
- ✅ 23 working examples in organized structure

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
# Run directly (using global cargo binary)
~/.cargo/target/debug/vex run examples/08_algorithms/fibonacci.vx
echo $?  # Prints 55

# Explore organized examples
~/.cargo/target/debug/vex run examples/01_basics/variables.vx
~/.cargo/target/debug/vex run examples/02_functions/recursion.vx
~/.cargo/target/debug/vex run examples/03_control_flow/loops.vx

# Check examples README
cat examples/README.md
```

### Hello World (v0.9)

```vex
fn main() : i32 {
    // Immutable variable (default)
    let x = 42;

    // Mutable variable (explicit)
    let! counter = 0;
    counter = counter + 1;

    return counter;
}
```

---

## ✨ Features (v0.9)

### ✅ Fully Implemented & Tested

- **Variables:** `let` (immutable), `let!` (mutable) - unified system
- **Functions:** Basic, generic, recursive, methods with receivers
- **Control Flow:** if/else, switch/case, while loops
- **Data Structures:** Structs, enums (C-style), tuples (parsed)
- **Type System:** Generics (basic), interfaces, type aliases, references
- **Operators:** Arithmetic, comparison, logical
- **Strings:** Literals, f-strings (parsed)
- **References:** `&T` (immutable), `&T!` (mutable)

**See `examples/` directory for 23 working examples organized by category!**

### 🚧 Partial Support

- **Pattern Matching:** Parser complete, codegen incomplete
- **Generics:** Monomorphization works, some edge cases
- **F-strings:** Parsing works, interpolation limited

### ❌ Not Yet Implemented

- **Async/Await:** Planned for future
- **Traits:** Parser only, no codegen
- **Union Types:** Parser only, no codegen
- **Match Expressions:** Requires union codegen
- **GPU/SIMD:** Future feature
- **Standard Library:** io, fs, net modules planned

---

## 📚 Documentation

| Document                                         | Description                                |
| ------------------------------------------------ | ------------------------------------------ |
| **[LANGUAGE_FEATURES.md](LANGUAGE_FEATURES.md)** | **📖 Complete feature list with examples** |
| [Specification.md](Specification.md)             | Language specification                     |
| [REFACTORING_SUCCESS.md](REFACTORING_SUCCESS.md) | Compiler refactoring details               |
| [MISSING_FEATURES.md](MISSING_FEATURES.md)       | Feature implementation tracker             |
| [archive-docs/](archive-docs/)                   | Old documentation                          |

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
