# Vex Development TODO

**Status:** DEPRECATED - See CORE_STRATEGY.md for current roadmap

This file is kept for historical reference only.
All active tasks are now tracked in CORE_STRATEGY.md.

---

## Current Focus: Foundation First

**Critical Priority:** Contract System & Builtin Standardization

See: `CORE_STRATEGY.md` - The Two Critical Blockers

---

## Historical TODO Items (Reference Only)

- [x] **Loop syntax support** (2 hours) - PARSER MISSING FEATURE

  - Current: Only `while condition { }` supported
  - Needed: `loop { }` for infinite loops (Rust/Vex standard)
  - Error: Parser treats `loop {` as expression, fails with "map literals need key: value pairs"
  - File: `vex-parser/src/parser/statements.rs`
  - Implementation: Add `Token::Loop` handling similar to `Token::While`
  - Test case: `examples/09_trait/test_collect.vx` (tried to use `loop { }`)
  - Priority: HIGH - Common pattern, spec compliant
  - Status: ✅ **COMPLETED** - Added Token::Loop, Statement::Loop, parser/codegen/borrow-checker support

- [ ] **Mutable method auto out-parameter** (4 hours) - CODEGEN BUG

  - Current: Method calls with `fn method()!: T` require manual out-parameter
  - Expected: `let val = obj.method()` should auto-inject out-param for mutable methods
  - Error: "Range.next() requires exactly 1 argument (output pointer)"
  - File: `vex-compiler/src/codegen_ast/expressions/call.rs`
  - Root cause: Mutable methods use out-parameter ABI but syntax doesn't auto-inject
  - Solution: Detect mutable method signature, auto-add allocation + pass pointer
  - Test case: `examples/09_trait/test_collect.vx` line 31
  - Priority: HIGH - Breaks ergonomics, forces manual workarounds

- [ ] **Struct method inline compilation** (8 hours) - CODEGEN ARCHITECTURE

  - Current: Struct methods compiled as external C functions `_vex_typename_method`
  - Expected: Inline LLVM IR generation for user-defined struct methods
  - Error: "Undefined symbols: \_vex_range_next"
  - File: `vex-compiler/src/codegen_ast/structs.rs`
  - Root cause: Fourth pass compiles method bodies but emits external function declarations
  - Solution: Generate LLVM IR directly, not C extern declarations
  - Test case: `examples/09_trait/test_collect.vx` - any method call on Range
  - Priority: CRITICAL - Blocks all custom struct methods
  - Note: Built-in types (Vec, Box) work because they have C runtime implementations

- [ ] **String indexing** (1 day)

  - Syntax: `text[3]` returns character at index 3
  - Return type: `char` or `u8`?
  - UTF-8 handling: byte index vs character index
  - File: `vex-parser/src/parser/expressions.rs` (Index expression)
  - Test: `examples/test_string_index.vx`

- [ ] **Rename Symbol** (1 day) - Already in LSP, enhance with preview

  - Show all rename locations before applying
  - File: `vex-lsp/src/backend.rs::rename()` (enhance)

- [ ] **Extract Variable** (4 hours)

- [ ] **Extract Function** (8 hours)

  - Right-click → Refactor menu
  - Extract variable/function with one click
  - Rename with preview of all changes

  ***

  - server.x64.vx, server.arm64.vx, server.wasm.vx
  - server.linux.x64.vx, server.macos.arm64.vx
  - utils.linux.vx, utils.windows.vx

- [ ] Manifest parser (`vex.json` → AST)

- [ ] Git integration (clone, checkout tags)

- [ ] Dependency resolver (MVS algorithm)

- [ ] Global cache manager (`~/.vex/cache/`)

- [ ] Lock file generator (`vex.lock` with SHA-256)

- [ ] Platform-specific file selector

- [ ] Stdlib module system integration

- [ ] CLI commands (new, add, remove, build)

  - ❌ HTTP direct download (Phase 2)
  - ❌ FFI dependencies (Phase 3)
  - ❌ Nexus mirror (Phase 4)

- [ ] Phase 1.8: Unused Variable Warnings (2h)

  - Implement W0001 (unused variable) detection
  - Add to codegen after function compilation
  - Track variable usage with HashMap<String, bool>

- [ ] Phase 1.9: Type Error Sites (2h)

  - Convert more string errors to Diagnostics in expressions/mod.rs
  - Use helper methods: type_mismatch(), undefined_type()
  - Focus on user-facing errors (not internal LLVM errors)
  - Error messages are now **Rust-quality** with spans, colors, suggestions
  - IDEs can consume JSON diagnostics for real-time feedback
  - Developers get helpful "did you mean?" suggestions
  - Trait bound errors are clear and actionable
  - 🔵 Phase 2 (future): Runtime HashMap codegen, Type.metadata() API

- [ ] LLVM vector types (f32x4, f32x8, i32x4, etc.) - 2h

- [ ] SIMD intrinsics (add, mul, fma, sqrt, etc.) - 4h

- [ ] Operator overloading integration - 2h

- [ ] Auto-vectorization hints - 2h

- [ ] Platform detection (SSE, AVX, NEON) - 2h

- [ ] Benchmarks (4-8x speedup) - 4h

- [ ] LLVM vector types (f32x4, f32x8, i32x4, etc.) - 2h

- [ ] SIMD intrinsics (add, mul, fma, sqrt, etc.) - 4h

- [ ] Operator overloading integration - 2h

- [ ] Auto-vectorization hints - 2h

- [ ] Platform detection (SSE, AVX, NEON) - 2h

- [ ] Benchmarks (4-8x speedup) - 4h

  - ~~Dynamic Dispatch (`dyn Trait`)~~ - Not needed, enum + match sufficient
  - ~~Variant Type~~ - Already have enum (tagged unions)

- [x] **Trait method location validation** (in struct body vs external) - ✅ **DONE!**

  - Implementation: Parser rejects `impl Trait for Struct { }` syntax with clear error
  - Error Message: "External trait implementations are not allowed. Use 'struct S impl T' instead"
  - File: `vex-parser/src/parser/items/traits.rs::parse_trait_impl()`
  - Tests Updated: `trait_bounds_separate_impl.vx`, `03_associated_type_impl.vx`
  - Status: **241/241 tests passing (100%)** ✅

- [ ] **`self!` syntax enforcement** (currently method-level, not receiver-level) - ⚠️ **WON'T IMPLEMENT**

  - Issue: `self!` expression syntax not in AST (only in receiver type)
  - Current: Method-level mutability works: `fn method()!` → `self.field` mutation OK
  - Proposed: Receiver-level enforcement would require AST changes
  - Decision: Method-level `!` is sufficient, no need for `self!` in expressions
  - Alternative: Keep `fn method()!` as the mutability declaration point

- [ ] Update SYNTAX.md with method mutability + location rules

- [ ] Update VEX_SYNTAX_GUIDE.md with comprehensive examples

- [ ] Update trait system documentation

- [ ] Create migration guide (v0.1 → v0.1.1)

- [ ] ✅ Created METHOD_MUTABILITY_FINAL.md (complete spec)

- [ ] ✅ Removed METHOD_DEFINITION_ARCHITECTURE_DISCUSSION.md (old)

  - **Failing:** `operator/04_builtin_add` - ONLY REMAINING TEST!
  - **Task:** Implement builtin Add/Sub/Mul traits for i32, f32, String
  - **Code Location:** `codegen_ast/expressions/binary_ops.rs`
  - **Fix Strategy:**
  - **Estimate:** 8 hours → **100% test coverage when complete!** 🎯

- [ ] **State Machine Transformation** (~3 days) - async/await codegen

- [ ] **Future Trait** (~2 days) - Core async abstraction

- [ ] **Runtime Integration** (~2 days) - C runtime already exists

- [ ] **Module imports** - Already partially working

- [ ] **Package manager** - See `PACKAGE_MANAGER_DRAFT.md`

  - **Initial Report:** 122/237 passing (51.5%) - looked like massive regression
  - **Investigation:** Checked Statement::If, Statement::For implementations
  - **Discovery:** Code was correct, binary needed rebuild after recent changes
  - **Resolution:** `cargo build` fixed all issues
  - **Final Status:** 235/237 passing (99.2%) ✅
  - **Failing:** Only 2 tests (async runtime, operator overloading)
  - **Status:** 210/210 tests passing (100%)
  - **Note:** Test suite was smaller, didn't include operator/async tests
  - Borrow checker (14 tests: 10 errors correctly detected, 4 valid cases)
  - Functions & closures (24 tests)
  - Control flow (14 tests: if/elif/else, for, while, switch)
  - Types (structs, enums, tuples) (20 tests)
  - Generics (15 tests including deep nesting)
  - Pattern matching (12 tests)
  - Strings (5 tests)
  - Algorithms (5 tests: fibonacci, factorial, gcd, power, prime)
  - Traits (10 tests)
  - Builtins (35 tests: Vec, Box, Option, Result, Map, Set, Channel)
  - Advanced (10 tests: never, rawptr, casts)
  - Async (0/2 tests - 1 failing)
  - Operators (3/4 tests - 1 failing)
  - Policy system (7 tests)
  - Stdlib (12 tests)
  - Diagnostics (10 tests: error detection, suggestions)
  - Issue: Add/Sub/Mul trait implementations for i32, f32, String not registered
  - Fix: Implement trait dispatch in binary_ops.rs codegen
  - Estimate: 8 hours
  - Issue: State machine transform exists, C runtime exists, integration incomplete
  - Fix: Wire async codegen to runtime event loop
  - Estimate: 12-16 hours
  - `test_move_diagnostic` - Diagnostic format differences
  - (Previously) `04_types/error_handling_try` - Now passing with ? operator ✅

- [ ] Test RAG TODO integration

- [ ] Iterator adapters blocked - Need: 1) Match expression return values 2) Method calls on struct fields (self.field.method()) 3) Closure/function pointer support
