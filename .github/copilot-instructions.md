# Vex Language Compiler - AI Agent Instructions

**Project:** Vex - Modern systems programming language  
**Version:** 0.2.0 (Syntax v0.9)  
**Last Updated:** November 4, 2025  
**Test Status:** 86/101 passing (85.1%)

## 🎯 Core Principles

1. **Check reference documentation first** - See TODO.md, docs/*.md for specs
2. **No shortcuts** - Implement features properly, not quick hacks
3. **Comprehensive testing** - Test all edge cases, not just happy paths
4. **Parallel development** - If feature A needs feature B enhancement, develop both
5. **Work silently** - No chat discussion during implementation, only final progress summary
6. **Use absolute paths** - Binary is at `~/.cargo/target/debug/vex`
7. **Follow Vex syntax v0.9** - Not Rust syntax (no `mut`, `->`, `::`)

## 📁 Project Structure

```
vex_lang/
├── .github/
│   └── copilot-instructions.md          # This file
├── vex-lexer/                           # Tokenization (logos)
├── vex-parser/                          # Recursive descent parser
│   └── src/parser/
│       ├── expressions.rs               # Expression parsing
│       ├── items.rs                     # Functions, traits, structs
│       └── types.rs                     # Type parsing
├── vex-ast/                             # Abstract Syntax Tree
│   └── src/lib.rs                       # All AST node definitions
├── vex-compiler/                        # LLVM codegen
│   └── src/
│       ├── codegen_ast/
│       │   ├── mod.rs                   # Core ASTCodeGen struct
│       │   ├── types.rs                 # AST↔LLVM type conversion
│       │   ├── statements.rs            # Let, if, while, for, return
│       │   ├── functions.rs             # Function compilation, generics
│       │   └── expressions/
│       │       ├── mod.rs               # Expression dispatcher
│       │       ├── binary_ops.rs        # Arithmetic, comparisons
│       │       ├── calls.rs             # Function/method calls
│       │       ├── literals.rs          # Arrays, structs, tuples
│       │       ├── access.rs            # Field access, indexing
│       │       └── special.rs           # Unary, postfix, closures
│       ├── borrow_checker/
│       │   ├── mod.rs                   # Entry point
│       │   ├── immutability.rs          # Phase 1: let vs let!
│       │   ├── moves.rs                 # Phase 2: Use-after-move
│       │   ├── borrows.rs               # Phase 3: Borrow rules
│       │   └── lifetimes.rs             # Phase 4: Lifetime analysis
│       └── module_resolver.rs           # Import system
├── vex-cli/                             # Command-line interface
├── vex-runtime/                         # Runtime (async, SIMD, C ABI)
│   ├── src/                             # Rust FFI bindings
│   ├── c/                               # ⚠️ C ABI RUNTIME (CRITICAL)
│   │   ├── vex.h                        # Main C header
│   │   ├── vex_intrinsics.h             # Intrinsic functions
│   │   ├── vex_alloc.c                  # Memory allocation
│   │   ├── vex_array.c                  # Array operations
│   │   ├── vex_string.c                 # String handling
│   │   ├── vex_simd_utf.c               # SIMD UTF-8 (simdutf)
│   │   ├── vex_swisstable.c             # HashMap (Google Swiss Tables)
│   │   ├── vex_io.c                     # I/O operations
│   │   ├── vex_file.c                   # File operations
│   │   ├── vex_time.c                   # Time operations
│   │   ├── vex_error.c                  # Error handling
│   │   ├── vex_testing.c                # Test utilities
│   │   └── async_runtime/               # Async/await runtime (C)
│   │       ├── include/runtime.h        # Runtime API
│   │       ├── src/                     # Event loop, scheduler
│   │       └── tests/                   # Runtime tests
│   ├── README.md                        # Runtime documentation
│   ├── IMPLEMENTATION_STATUS.md         # Feature status
│   ├── UTF8_SUPPORT.md                  # UTF-8 implementation
│   └── ARRAY_SAFETY.md                  # Array safety details
├── vex-libs/                            # Standard library
│   └── std/                             # Vex stdlib modules
├── examples/                            # Test examples (.vx files)
│   ├── 00_borrow_checker/               # Borrow checker tests
│   ├── 01_basics/                       # Variables, types
│   ├── 02_functions/                    # Functions, closures
│   ├── 03_control_flow/                 # If, loops, match
│   ├── 04_types/                        # Structs, enums
│   ├── 05_generics/                     # Generic functions
│   ├── 06_patterns/                     # Pattern matching
│   ├── 07_strings/                      # String operations
│   ├── 08_algorithms/                   # Fibonacci, factorial
│   └── 09_trait/                        # Trait system
├── docs/                                # Documentation
│   ├── CLOSURE_IMPLEMENTATION_COMPLETE.md
│   ├── VARIABLE_SYSTEM_V09.md
│   └── ...
├── TODO.md                              # ⚠️ PRIMARY TASK LIST
├── README.md                            # Project overview
├── Specification.md                     # Language spec (Turkish)
├── SYNTAX.md                            # Syntax reference
└── test_all.sh                          # Run all tests

Binary location: ~/.cargo/target/debug/vex (NOT ./target/)
Build output:    vex-builds/              (LLVM IR and binaries)
```

## 📚 Reference Documentation (Always Check These First!)

### Primary References
- **`TODO.md`** - Current tasks, priorities, recent completions, test status
- **`SYNTAX.md`** - Language syntax reference
- **`Specification.md`** - Detailed language specification (Turkish)
- **`README.md`** - Quick start, feature overview

### Feature Documentation
- **`docs/CLOSURE_IMPLEMENTATION_COMPLETE.md`** - Closure implementation details
- **`docs/VARIABLE_SYSTEM_V09.md`** - let/let! syntax, references
- **`DEFER_IMPLEMENTATION.md`** - Defer statement implementation
- **`CLOSURE_PARSER_FIX_SUMMARY.md`** - Closure parsing fix details

### Architecture
- **`REFACTORING_PLAN.md`** - Codegen module organization
- **`vex-libs/std/README.md`** - Standard library structure
- **`examples/README.md`** - Example organization and status

### Test Results
- **`TEST_RESULTS.md`** - Historical test data
- **`test_all.sh`** - Run to get current test status

## ⚙️ Build & Run Commands

```bash
# Build
cargo build

# Run file
~/.cargo/target/debug/vex run examples/02_functions/closure_simple.vx

# Run inline code
~/.cargo/target/debug/vex run -c "fn main(): i32 { return 42; }"

# Compile to binary
~/.cargo/target/debug/vex compile examples/08_algorithms/fibonacci.vx

# Run all tests
./test_all.sh

# Emit LLVM IR
~/.cargo/target/debug/vex compile examples/test.vx --emit-llvm
cat vex-builds/test.ll
```

## 🔑 Key Syntax Rules (v0.9)

### Variables
```vex
let x = 42;              // Immutable (default)
let! counter = 0;        // Mutable (! suffix)
const MAX = 100;         // Compile-time constant
```

### References
```vex
&T                       // Immutable reference
&T!                      // Mutable reference (NOT &mut T)
```

### Function Types
```vex
fn(i32, i32): i32        // Use : not ->
fn add(x: i32): i32      // Return type with :
```

### Closures
```vex
|x: i32| x * 2           // Basic closure
|x: i32|: i32 { x * 2 }  // With explicit return type
```

### Deprecated (Will Error)
```vex
❌ mut x = 42;           // Use let! instead
❌ fn(): i32 -> { }      // Use : not ->
❌ interface Foo {}      // Use trait instead
❌ x := 42;              // Use let instead
```

## 🎯 Current Implementation Status

### Implementation Status (See TODO.md)
- ✅ Variables, functions, control flow, structs, enums, pattern matching
- ✅ Trait system v1.3, borrow checker (4 phases), defer statement
- ✅ Closures: parser, borrow checker, basic codegen, environment detection
- 🚧 Closure environment binding, closure traits (Fn/FnMut/FnOnce)
- ❌ Async/await runtime, dynamic dispatch, full optimizations

## ⚠️ C ABI Runtime (Critical)

**Why C?** SIMD-optimized (20 GB/s UTF-8), Swiss Tables HashMap, cross-platform

**Key Files:**
```
vex-runtime/c/
├── vex.h, vex_intrinsics.h  - API headers
├── vex_alloc.c, vex_array.c - Memory, arrays
├── vex_simd_utf.c           - SIMD UTF-8 (simdutf)
├── vex_swisstable.c         - HashMap
└── async_runtime/           - Async event loop
```

**Add C function:** vex.h → vex_*.c → builtins.rs → test  
**Build:** `cd vex-runtime/c && ./build.sh`

## 🛠️ Development Workflow

### Implementation Standards
- **No quick fixes** - Implement properly from the start
- **Test exhaustively** - All edge cases, error paths, boundary conditions
- **Parallel features** - If implementing X requires Y enhancement, do both
- **Silent execution** - Work without asking, report final summary only

### Process
1. Read `TODO.md` + relevant `docs/`
2. Implement feature fully (parser → AST → codegen → borrow checker)
3. Add comprehensive tests (happy path + edge cases + errors)
4. Run `./test_all.sh`
5. Update `TODO.md` + documentation
6. **Report final progress summary**

## 🐛 Common Issues

| Issue | Solution |
|-------|----------|
| Binary not found | Use `~/.cargo/target/debug/vex` not `./target/` |
| Rust syntax errors | Use Vex v0.9: `let!` not `mut`, `:` not `->` |
| LLVM codegen crash | Check builder position, block terminators |
| C runtime undefined ref | `cd vex-runtime/c && ./build.sh` |
| Borrow checker miss | Check all 4 phases handle new feature |

## 📊 Testing

**Status:** 86/101 passing (85.1%) - See `./test_all.sh`

**Add test:** Create `.vx` in `examples/` → run `./test_all.sh` → update README

---

**Critical Reminder:**
1. **No shortcuts** - Implement fully, test exhaustively
2. **Parallel features** - Develop dependencies together
3. **Silent work** - Only report final progress summary
4. **Check TODO.md** for current priorities
