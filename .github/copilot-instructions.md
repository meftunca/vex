# Vex Language Compiler - AI Agent Instructions

**Project:** Vex - Modern systems programming language  
**Version:** 0.2.0 (Syntax v0.9)  
**Last Updated:** November 8, 2025  
**Test Status:** 238/238 passing (100%) ✅🎉 - PRODUCTION READY!

## 🎯 Core Principles

1. **Check reference documentation first** - See TODO.md, docs/\*.md for specs
2. **No shortcuts** - Implement features properly, not quick hacks
3. **Comprehensive testing** - Test all edge cases, not just happy paths
4. **Parallel development** - If feature A needs feature B enhancement, develop both
5. **⚠️ ABSOLUTE SILENCE RULE** - **DO NOT** engage in conversation, explanations, or discussions unless explicitly asked. Work completely silently. Only provide minimal status updates at the very end.
6. **Minimal status format** - Final report MUST be: `✅ [Task] → [Result] ([files changed])` - Nothing more.
7. **Use absolute paths** - Binary is at `~/.cargo/target/debug/vex`
8. **Follow Vex syntax v0.9** - Not Rust syntax (no `mut`, `->`, `::`)
9. **⚠️ CRITICAL: NO `::` operator!** - Use `.` for all member access (`Vec.new()` not `Vec::new()`, `Some(x)` not `Option::Some(x)`)
10. **⚠️ FILE SIZE LIMIT: 400 LINES MAX** - **MANDATORY** Rust files MUST NOT exceed 400 lines. Split logically into modules when approaching this limit.
11. **⚠️ UPDATE THIS FILE!** - When adding new modules or reorganizing code, **ALWAYS** update the Project Structure section in this file with current line counts and organization.

## 📁 Project Structure

```
vex_lang/
├── .github/
│   └── copilot-instructions.md          # This file
├── vex-diagnostics/                     # Diagnostic system (NEW!)
│   └── src/
│       └── lib.rs (646)                 # Span, Diagnostic, DiagnosticEngine, error codes
├── vex-lexer/                           # Tokenization (logos)
├── vex-parser/                          # Recursive descent parser (WELL ORGANIZED)
│   └── src/parser/
│       ├── mod.rs (345)                 # Main parser coordinator
│       ├── expressions.rs (84)          # Expression parsing entry
│       ├── statements.rs (338)          # Statement parsing
│       ├── primaries.rs (240)           # Primary expressions
│       ├── operators.rs (414)           # Binary/unary operators
│       ├── patterns.rs (188)            # Pattern matching syntax
│       ├── types.rs (451)               # Type parsing (ALL types)
│       └── items/                       # Top-level items (organized)
│           ├── mod.rs (12)              # Re-exports
│           ├── functions.rs (113)       # Function declarations
│           ├── structs.rs (134)         # Struct definitions
│           ├── enums.rs (48)            # Enum definitions
│           ├── traits.rs (186)          # Trait definitions
│           ├── imports.rs (90)          # Import statements
│           ├── exports.rs (48)          # Export statements
│           ├── externs.rs (97)          # Extern declarations
│           ├── consts.rs (22)           # Const declarations
│           ├── aliases.rs (27)          # Type aliases
│           └── helpers.rs (43)          # Parsing utilities
├── vex-ast/                             # Abstract Syntax Tree
│   └── src/lib.rs                       # All AST node definitions
├── vex-compiler/                        # LLVM codegen (REORGANIZED!)
│   └── src/
│       ├── lib.rs                       # Public API
│       ├── diagnostics.rs               # Re-export vex-diagnostics
│       ├── trait_bounds_checker.rs      # Trait constraint validation
│       ├── module_resolver.rs           # Import/module system
│       ├── codegen_ast/                 # Code generation (WELL STRUCTURED)
│       │   ├── mod.rs (687)             # Core ASTCodeGen + DiagnosticEngine
│       │   ├── registry.rs (129)        # Type/function registry + diagnostics
│       │   ├── analysis.rs              # Pre-codegen analysis
│       │   ├── program.rs               # Program compilation entry
│       │   ├── types.rs (597)           # AST↔LLVM type conversion
│       │   ├── generics.rs              # Generic instantiation
│       │   ├── methods.rs               # Method compilation
│       │   ├── traits.rs                # Trait implementation
│       │   ├── enums.rs                 # Enum codegen
│       │   ├── defer.rs                 # Defer statement
│       │   ├── ffi.rs                   # FFI/extern support
│       │   ├── statements/              # Statement compilation (ORGANIZED)
│       │   │   ├── mod.rs (143)         # Statement dispatcher + diagnostics
│       │   │   ├── let_statement.rs (638) # Variable declarations
│       │   │   ├── assignment.rs        # Assignment expressions
│       │   │   ├── control_flow.rs      # If/match statements
│       │   │   └── loops.rs (399)       # For/while loops
│       │   ├── functions/               # Function compilation (ORGANIZED)
│       │   │   ├── mod.rs               # Function dispatcher
│       │   │   ├── declare.rs           # Function declarations
│       │   │   ├── compile.rs           # Function body compilation
│       │   │   └── asynchronous.rs      # Async function support
│       │   ├── expressions/             # Expression compilation (WELL SPLIT)
│       │   │   ├── mod.rs (500)         # Expression dispatcher
│       │   │   ├── binary_ops.rs        # +, -, *, /, %, ==, !=, <, >, etc.
│       │   │   ├── literals.rs (388)    # Numbers, strings, arrays, structs
│       │   │   ├── control.rs           # If/match expressions
│       │   │   ├── pattern_matching.rs (858) # Pattern matching codegen
│       │   │   ├── access/              # Member access (ORGANIZED)
│       │   │   │   ├── mod.rs           # Access dispatcher
│       │   │   │   ├── field_access.rs (494) # Struct field access
│       │   │   │   ├── indexing.rs      # Array/slice indexing
│       │   │   │   └── fstring.rs       # F-string formatting
│       │   │   ├── calls/               # Function calls (ORGANIZED)
│       │   │   │   ├── mod.rs           # Call dispatcher
│       │   │   │   ├── function_calls.rs (216) # Regular function calls
│       │   │   │   ├── method_calls.rs (288) # Method calls
│       │   │   │   └── builtins.rs      # Builtin function calls
│       │   │   └── special/             # Special expressions (ORGANIZED)
│       │   │       ├── mod.rs           # Special dispatcher
│       │   │       ├── unary.rs         # Unary operators (!, -, &)
│       │   │       ├── closures.rs (481) # Closure compilation
│       │   │       └── casts.rs         # Type casting
│       │   └── builtins/                # Builtin types & functions (COMPREHENSIVE)
│       │       ├── mod.rs (378)         # Builtin coordinator
│       │       ├── core.rs              # Core builtin setup
│       │       ├── hints.rs             # Type hints for builtins
│       │       ├── intrinsics.rs (318)  # LLVM intrinsics
│       │       ├── memory.rs (292)      # Memory operations
│       │       ├── memory_ops.rs (226)  # Alloc/dealloc helpers
│       │       ├── array.rs (220)       # Array operations
│       │       ├── string.rs            # String operations
│       │       ├── utf8.rs              # UTF-8 validation
│       │       ├── hashmap.rs (323)     # HashMap operations
│       │       ├── reflection.rs (205)  # Runtime reflection
│       │       ├── stdlib.rs (308)      # Standard library
│       │       ├── stdlib_logger.rs     # Logger module
│       │       ├── stdlib_testing.rs    # Testing framework
│       │       ├── stdlib_time.rs       # Time operations
│       │       └── builtin_types/       # Builtin type implementations
│       │           ├── mod.rs           # Type dispatcher
│       │           ├── option_result.rs (237) # Option<T>, Result<T,E>
│       │           ├── collections.rs (244) # Vec<T>, Box<T>
│       │           └── conversions.rs (250) # Type conversions
│       └── borrow_checker/              # Borrow checker (4-PHASE SYSTEM)
│           ├── mod.rs (365)             # Entry point + orchestration
│           ├── errors.rs (229)          # Error reporting
│           ├── builtin_metadata.rs (303) # Builtin type borrow info
│           ├── immutability.rs (399)    # Phase 1: let vs let!
│           ├── moves.rs (625)           # Phase 2: Use-after-move
│           ├── borrows.rs (610)         # Phase 3: Borrow rules
│           ├── lifetimes.rs (692)       # Phase 4: Lifetime analysis
│           └── closure_traits.rs (357)  # Closure trait inference
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

**Add C function:** vex.h → vex\_\*.c → builtins.rs → test  
**Build:** `cd vex-runtime/c && ./build.sh`

## 🛠️ Development Workflow

### Implementation Standards

- **No quick fixes** - Implement properly from the start
- **Test exhaustively** - All edge cases, error paths, boundary conditions
- **Parallel features** - If implementing X requires Y enhancement, do both
- **Silent execution** - Work without asking, report final summary only
- **⚠️ MANDATORY: File size discipline** - Keep Rust files under 400 lines

### Process

1. Read `TODO.md` + relevant `docs/`
2. Implement feature fully (parser → AST → codegen → borrow checker)
3. **Check file size** - If any .rs file approaches 400 lines, refactor into modules
4. Add comprehensive tests (happy path + edge cases + errors)
5. Run `./test_all.sh`
6. Update `TODO.md` + documentation
7. **⚠️ UPDATE `.github/copilot-instructions.md`** - If new modules added or code reorganized, update Project Structure section with line counts
8. **Report final progress summary**

## 📏 File Size Management (CRITICAL)

**RULE:** Rust source files MUST NOT exceed **400 lines** (excluding blank lines/comments)

### When to Split a File

**Triggers:**

- ✅ File reaches 250+ lines → Plan refactoring
- ✅ File reaches 280+ lines → Split IMMEDIATELY before adding more code
- ✅ Multiple logical concerns in one file → Split by responsibility

**How to Split:**

```rust
// ❌ BAD: expressions/mod.rs (1100 lines)
impl ASTCodeGen {
    fn compile_binary_op() { /* 100 lines */ }
    fn compile_unary_op() { /* 80 lines */ }
    fn compile_match() { /* 200 lines */ }
    fn compile_if() { /* 150 lines */ }
    // ... 500+ more lines
}

// ✅ GOOD: Split into logical modules
expressions/
├── mod.rs (200 lines)        // Dispatcher + common utilities
├── binary_ops.rs (150 lines) // Binary operations
├── unary_ops.rs (100 lines)  // Unary operations
├── pattern_match.rs (250 lines) // Pattern matching
└── control_flow.rs (200 lines)  // If/match expressions
```

### Refactoring Strategy

**Step 1: Identify logical boundaries**

```rust
// File with 400 lines - find natural split points:
// - Binary operations (150 lines)
// - Unary operations (100 lines)
// - Pattern matching (150 lines)
```

**Step 2: Extract into new module**

```rust
// 1. Create new file: binary_ops.rs
// 2. Move functions with `pub(super)` visibility
// 3. Update mod.rs: `mod binary_ops; pub use binary_ops::*;`
// 4. Test compilation
```

**Step 3: Verify**

```bash
# Check line counts
wc -l vex-compiler/src/codegen_ast/**/*.rs

# Target distribution:
# mod.rs:         150-250 lines (coordinator)
# feature_*.rs:   100-400 lines (implementation)
```

### Module Organization Patterns (CURRENT STRUCTURE - Updated Nov 6, 2025)

**Pattern 1: Deep Feature-based split (Parser)**

```
parser/
├── mod.rs (345)             # Main coordinator
├── expressions.rs (84)       # Expression entry
├── statements.rs (338)       # Statements
├── types.rs (451)           # All type parsing
└── items/                   # Top-level items (11 files)
    ├── mod.rs               # Re-exports
    ├── functions.rs (113)   # Function declarations
    ├── structs.rs (134)     # Struct definitions
    ├── traits.rs (186)      # Trait definitions
    └── ... (8 more specialized files)
```

**Pattern 2: Multi-level Feature split (Codegen Expressions)**

```
codegen_ast/expressions/
├── mod.rs (500)             # Main dispatcher
├── binary_ops.rs            # Arithmetic/comparison
├── literals.rs (388)        # Literal values
├── pattern_matching.rs (858) # Match expressions
├── access/                  # Member access (4 files)
│   ├── mod.rs
│   ├── field_access.rs (494)
│   ├── indexing.rs
│   └── fstring.rs
├── calls/                   # Function calls (4 files)
│   ├── mod.rs
│   ├── function_calls.rs (216)
│   ├── method_calls.rs (288)
│   └── builtins.rs
└── special/                 # Special expressions (4 files)
    ├── mod.rs
    ├── closures.rs (481)
    ├── unary.rs
    └── casts.rs
```

**Pattern 3: Category-based split (Builtins)**

```
codegen_ast/builtins/
├── mod.rs (378)             # Coordinator
├── core.rs                  # Core setup
├── intrinsics.rs (318)      # LLVM intrinsics
├── memory.rs (292)          # Memory operations
├── array.rs (220)           # Array operations
├── string.rs                # String operations
├── hashmap.rs (323)         # HashMap operations
├── stdlib_*.rs              # Stdlib modules (3 files)
└── builtin_types/           # Type implementations (4 files)
    ├── mod.rs
    ├── option_result.rs (237)
    ├── collections.rs (244)
    └── conversions.rs (250)
```

**Pattern 4: Phase-based split (Borrow Checker)**

```
borrow_checker/
├── mod.rs (365)             # Entry + orchestration
├── errors.rs (229)          # Error reporting
├── builtin_metadata.rs (303) # Builtin type metadata
├── immutability.rs (399)    # Phase 1: let vs let!
├── moves.rs (625)           # Phase 2: Use-after-move
├── borrows.rs (610)         # Phase 3: Borrow rules
├── lifetimes.rs (692)       # Phase 4: Lifetime analysis
└── closure_traits.rs (357)  # Closure trait inference
```

**Key Takeaways from Current Organization:**

1. ✅ **3-Level Hierarchy Works Well**: mod.rs → feature/ → subfeature.rs
2. ✅ **500-line Modules OK**: If well-organized dispatcher with clear sections
3. ✅ **Deep Nesting Acceptable**: expressions/calls/method_calls.rs is clear
4. ✅ **Line Count in Parentheses**: Helps track file sizes quickly
5. ⚠️ **Watch These Files**: pattern_matching.rs (858), lifetimes.rs (692), moves.rs (625)

### Enforcement

**Before committing code:**

1. Run: `find . -name "*.rs" -exec wc -l {} \; | awk '$1 > 400'`
2. If output exists → Files exceed 400 lines → MUST refactor
3. No exceptions - this ensures maintainability

**Why 400 lines?**

- ✅ AI can read entire file in 1-2 tool calls
- ✅ Human can understand file scope quickly
- ✅ Git diffs remain readable
- ✅ Merge conflicts easier to resolve
- ✅ Forces good separation of concerns

## 🐛 Common Issues

| Issue                   | Solution                                        |
| ----------------------- | ----------------------------------------------- |
| Binary not found        | Use `~/.cargo/target/debug/vex` not `./target/` |
| Rust syntax errors      | Use Vex v0.9: `let!` not `mut`, `:` not `->`    |
| LLVM codegen crash      | Check builder position, block terminators       |
| C runtime undefined ref | `cd vex-runtime/c && ./build.sh`                |
| Borrow checker miss     | Check all 4 phases handle new feature           |

## 📊 Testing

**Status:** 143/146 passing (97.9%) - See `./test_all.sh`

**Add test:** Create `.vx` in `examples/` → run `./test_all.sh` → update README

---

**Critical Reminder:**

1. **No shortcuts** - Implement fully, test exhaustively
2. **Parallel features** - Develop dependencies together
3. **Silent work** - Only report final progress summary
4. **Check TODO.md** for current priorities
5. **⚠️ ENFORCE 400-LINE LIMIT** - Split files immediately when approaching this limit
