# Vex Language Compiler - AI Agent Instructions

**Project:** Vex - Modern systems programming language with Rust's safety and Go's simplicity  
**Version:** 0.2.0 (Syntax v0.9)  
**Last Updated:** November 3, 2025

## 🏗️ Architecture Overview

**Rust Workspace with 6 Crates:**

```
vex-lexer (logos) → vex-parser (recursive descent) → vex-ast
  → vex-compiler (LLVM/inkwell) → vex-cli (clap) → vex-runtime (tokio)
```

**Key Design Philosophy:**

- Go's simplicity (interfaces, clean syntax) + Rust's safety (borrow checker) + TypeScript's type system (generics, unions)
- Compiler is in Rust, target language is Vex (`.vx` files)
- 71% test coverage, 29/59 examples passing

## ⚠️ Critical Build Information

**Cargo builds to ~/.cargo/target/, NOT workspace directory!**

```bash
# ✅ CORRECT binary locations:
~/.cargo/target/debug/vex
~/.cargo/target/release/vex

# ❌ NEVER use ./target/ - Cargo.toml workspace config sets:
#    target-dir = "~/.cargo/target"
```

**Always use absolute paths when running the compiler:**

```bash
# Build
cargo build --release

# Run examples
~/.cargo/target/debug/vex run examples/01_basics/hello_world.vx
~/.cargo/target/debug/vex compile examples/08_algorithms/fibonacci.vx -o fib.o
```

## 📋 Syntax v0.9 Conventions (Nov 2025)

**Variable System (Unified):**

```vex
let x = 42;              // Immutable (default, Rust-style)
let! counter = 0;        // Mutable (explicit with !)
counter = counter + 1;   // OK, marked mutable
const MAX = 100;         // Compile-time constant
```

**References:**

```vex
&T                       // Immutable reference
&T!                      // Mutable reference (NOT &mut T)
```

**Deprecated Keywords:**

- ❌ `mut` keyword removed from lexer (use `!` suffix instead)
- ❌ `interface` keyword returns parser error (use `trait`)
- ❌ `:=` operator removed (use `let` instead)

**See:** `docs/VARIABLE_SYSTEM_V09.md` for full specification

## 🔍 Module Structure & Entry Points

### Compiler Pipeline (vex-compiler/src/)

**Core Files:**

- `lib.rs` - Public API exports
- `codegen.rs` - Legacy standalone examples (hello_world, fibonacci, etc.)
- `codegen_ast/mod.rs` - Main `ASTCodeGen<'ctx>` struct (184 lines)
- `module_resolver.rs` - Import system, loads from `vex-libs/std/`
- `borrow_checker/mod.rs` - 3-phase checker (immutability, moves, borrows)

**Modular Codegen Structure (see REFACTORING_SUCCESS.md):**

```
codegen_ast/
├── mod.rs              - Core struct, helpers, printf
├── types.rs            - AST↔LLVM type conversions (230 lines)
├── statements.rs       - Let, if, while, for, return (408 lines)
├── functions.rs        - Program compilation, generics (540 lines)
├── builtins.rs         - Built-in functions registry
└── expressions/
    ├── mod.rs          - Main dispatcher
    ├── binary_ops.rs   - Arithmetic, comparisons
    ├── calls.rs        - Function/method calls
    ├── literals.rs     - Arrays, structs, tuples
    ├── access.rs       - Field access, indexing
    └── special.rs      - Unary, postfix ops
```

**When modifying codegen:**

1. Identify the expression/statement type from `vex-ast/src/lib.rs`
2. Find the relevant module in `codegen_ast/`
3. Add case to dispatcher or extend existing `impl<'ctx> ASTCodeGen<'ctx>` block
4. Follow LLVM patterns from existing code (use `IntValue`, `BasicBlock`, etc.)

### Parser (vex-parser/src/)

**Entry Point:** `grammar.lalrpop` (LALRPOP grammar) - currently being migrated  
**Recursive Descent:** `parser/` directory (items.rs, expressions.rs, types.rs)

**Parser is in flux** - check `TODO.md` for migration status

### Borrow Checker (vex-compiler/src/borrow_checker/)

**3-Phase System:**

1. **Immutability** (`immutability.rs`) - Enforces `let` vs `let!` semantics (7 tests ✅)
2. **Move Semantics** (`moves.rs`) - Prevents use-after-move (5 tests ✅)
3. **Borrow Rules** (`borrows.rs`) - 1 mutable XOR N immutable refs (5 tests ✅)
4. **Phase 4 TODO:** Lifetime analysis (5-6 days estimated)

**Integration:** Automatically runs on `vex compile` and `vex run` via `vex-cli/src/main.rs`

## 🧪 Testing & Development Workflow

**Test Scripts:**

```bash
# Run all examples (shell script)
./test_all.sh

# Individual example test
~/.cargo/target/debug/vex run examples/02_functions/recursion.vx
echo $?  # Check exit code
```

**Working Examples (29/59 passing):**

- `examples/01_basics/` - Variables, types
- `examples/02_functions/` - Recursion, methods
- `examples/03_control_flow/` - If, switch, loops
- `examples/08_algorithms/` - Fibonacci (returns 55), factorial, GCD

**Borrow Checker Tests:**

```bash
# All tests in examples/00_borrow_checker/
~/.cargo/target/debug/vex run examples/00_borrow_checker/01_immutable_assign.vx
# Should fail with borrow error
```

**Trait System Tests:**

```bash
# All tests in examples/09_trait/
~/.cargo/target/debug/vex run examples/09_trait/01_basic_trait.vx
```

**Unit Tests:**

```bash
# Run Rust tests in specific crate
cargo test -p vex-lexer
cargo test -p vex-compiler

# Run all workspace tests
cargo test --workspace
```

## 🎯 Feature Implementation Status (see TODO.md)

**✅ Fully Working:**

- Basic types: i8/16/32/64, u8/16/32/64, f32/64, bool, string
- Variables: `let`, `let!`, `const` with v0.9 syntax
- Functions: Basic, generic, recursive, methods with receivers
- Control flow: if/else, switch/case, while, for
- Data structures: Structs, enums (C-style), tuples (parsed)
- Pattern matching: Basic match, tuple/struct destructuring
- Borrow checker: Phases 1-3 complete (17 tests passing)
- Trait system v1.3: Inline implementation (`struct Foo impl Trait`)

**🚧 Partial/In Progress:**

- Generics: Monomorphization works, edge cases remain
- F-strings: Parsing complete, interpolation limited
- Default trait methods: AST ready, codegen pending
- Data-carrying enums: `Some(x)`, `Ok(val)` pattern matching pending

**❌ Not Yet Implemented:**

- Async/await: Parser exists, no runtime integration
- Dynamic dispatch: Vtable generation pending
- Closures/lambdas
- Advanced optimizations

## 📦 Standard Library (vex-libs/std/)

**Layered Architecture (see vex-libs/std/README.md):**

```
Layer 3: Application (100% Safe Vex) - http, json, xml
Layer 2: Protocol (100% Safe Vex)    - net, sync, testing
Layer 1: I/O Core (Unsafe Bridge)    - io, ffi, unsafe, hpc
Layer 0: Vex Runtime (Rust)          - io_uring, async scheduler
```

**Import Resolution:**

- ModuleResolver in `vex-compiler/src/module_resolver.rs`
- Path conversion: `"std::io"` → `vex-libs/std/io/mod.vx`
- Imports merged into main AST before codegen

**Example Import:**

```vex
import { io, log } from "std";  // Loads from vex-libs/std/
```

## 🔧 Common Tasks

### Adding a New Built-in Function

1. Register in `codegen_ast/builtins.rs` → `BuiltinRegistry::new()`
2. Implement generator function with signature `fn(&mut ASTCodeGen, Vec<Expression>) -> Result<BasicValueEnum, String>`
3. Add test in examples with `@intrinsic` or direct call

### Adding a New Statement Type

1. Define AST node in `vex-ast/src/lib.rs` → `Statement` enum
2. Add parser case in `vex-parser/src/parser/items.rs`
3. Implement codegen in `vex-compiler/src/codegen_ast/statements.rs`
4. Add borrow checker logic if needed in `borrow_checker/` modules
5. Create test in `examples/` with expected behavior

### Debugging LLVM Issues

```bash
# Emit LLVM IR to inspect
~/.cargo/target/debug/vex compile examples/test.vx --emit-llvm

# Check generated IR in vex-builds/
cat vex-builds/test.ll

# Verify LLVM module validity
# Look for verify_module() calls in codegen
```

**Common LLVM Patterns:**

- Use `builder.position_at_end(block)` before emitting instructions
- Check terminator with `block.get_terminator().is_some()` before adding branches
- Float comparisons need `FloatPredicate`, ints need `IntPredicate`

## 📚 Key Documentation Files

- `README.md` - Quick start, feature overview
- `TODO.md` - Active development tasks with priorities (🔴🟡🟢)
- `LANGUAGE_FEATURES.md` - Complete feature list with test status
- `REFACTORING_SUCCESS.md` - Codegen modular structure explanation
- `Specification.md` - Language spec (Turkish, detailed syntax rules)
- `examples/README.md` - All examples organized by category with status
- `vex-libs/std/README.md` - Standard library API documentation

## 🎨 Code Style & Patterns

**Rust Conventions:**

- Use `Result<T, String>` for errors in codegen
- LLVM lifetimes: `ASTCodeGen<'ctx>` tracks Inkwell context
- Prefer pattern matching over if-let chains
- Use `log::info!()` / `log::debug!()` for debugging

**Vex Language Conventions:**

- File extension: `.vx`
- Main entry point: `fn main(): i32 { return 0; }`
- Comments: `//` and `/* */` C-style
- Naming: snake_case for variables/functions, PascalCase for types

## 🚨 Known Gotchas

1. **Cargo binary location** - Always use `~/.cargo/target/` not `./target/`
2. **`mut` keyword removed** - Use `let!` instead, parser will error on `mut`
3. **Module imports** - Must match directory structure in `vex-libs/std/`
4. **Generic monomorphization** - Each type instantiation generates new function
5. **Borrow checker runs automatically** - No need to invoke separately in CLI
6. **Trait vs Interface** - `interface` keyword deprecated, use `trait` only
7. **Pattern matching** - Data-carrying enum destructuring not yet implemented

## 🎯 Current Development Focus (November 2025)

**Active Work (see TODO.md):**

- Phase 4: Lifetime Analysis (high priority 🔴)
- Default trait methods implementation
- Data-carrying enum pattern matching (`Some(x)`, `Ok(val)`)

**Next Up:**

- Trait bounds in generics
- Dynamic dispatch with vtables
- Closures and lambda expressions

---

**For Questions:** Check `TODO.md` for priorities, `LANGUAGE_FEATURES.md` for implementation status, or search `examples/` for working code patterns.
