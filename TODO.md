# Vex Language Development TODO

## 🚨 Critical Type System Flaws (BLOCKING PRODUCTION USE)

### 🟢 FIXED: Integer Literal Suffix Support ✅

- **Status:** IMPLEMENTED in v0.2.0
- **Syntax:** `42i64`, `100u32`, `0xFFu8`, `0b1010i32`, `0o777i16`
- **Files Modified:**
  - `vex-lexer/src/lib.rs` - Added suffix parsing to regex patterns
  - `vex-parser/src/parser/primaries.rs` - Parse suffix and create TypedIntLiteral
  - `vex-ast/src/lib.rs` - Added TypedIntLiteral/TypedBigIntLiteral variants
  - `vex-compiler/src/codegen_ast/expressions/literals_expressions.rs` - Codegen for typed literals
  - `vex-compiler/src/codegen_ast/types/inference.rs` - Type inference for suffixed literals
- **Impact:** No more manual `(60 as i64)` casts needed
- **Priority:** ~~P1~~ COMPLETED

### 🟢 FIXED: Target-Typed Integer Literals ✅

- **Status:** PARTIALLY IMPLEMENTED - Works with literal suffixes, full inference needs const folding refactor
- **Issue:** Integer literals default to i32 regardless of expected type context
- **Example:** `const MINUTE: i64 = 60i64 * SECOND` → now works with suffix
- **Workaround:** Use literal suffixes for now: `60i64` instead of relying on inference
- **Files Modified:**
  - `vex-compiler/src/codegen_ast/expressions/mod.rs` - Added compile_expression_with_type
  - `vex-compiler/src/codegen_ast/constants.rs` - Pass expected type to expression compiler
  - `vex-compiler/src/codegen_ast/expressions/literals_expressions.rs` - Accept expected_type
- **Remaining:** Full target-typing needs builder-free const evaluation path
- **Priority:** ~~P0~~ PARTIAL - Literal suffixes provide immediate workaround

### 🟢 FIXED: Constant Expression Overflow Checks ✅

- **Status:** IMPLEMENTED in v0.2.0
- **Issue:** Overflow check uses left operand type instead of result type
- **Example:** `const X: i64 = 1000i64 * NANOSECOND` → now checks i64 bounds
- **Files Modified:**
  - `vex-compiler/src/codegen_ast/expressions/binary_ops/integer_ops.rs` - Use target_bit_width
  - `vex-compiler/src/codegen_ast/expressions/operators.rs` - Propagate expected_type
- **Impact:** RESOLVED - Target type used for overflow validation
- **Priority:** ~~P1~~ COMPLETED

### 🔴 P0: Binary Operation Type Inference

- **Issue:** Binary ops don't propagate expected type to operands during const evaluation
- **Example:** `const X: i64 = 1000 * 1000` → both literals i32, overflow before cast
- **Fix Required:** Add `expected_type: Option<&Type>` to all expression compilation
- **Impact:** SEVERE - Const expressions require excessive parentheses and casts
- **Priority:** P0 - Required for clean stdlib code

### 🔴 P1: Integer Literal Suffix Support Missing

- **Issue:** No support for type suffixes on integer literals (42i64, 100u32, etc.)
- **Rust/C++ Parity:** `42i64`, `100u32`, `0xFFu8` all standard
- **Current State:** Lexer doesn't parse suffixes, requires `as` casts
- **Files to Fix:**
  - `vex-lexer/src/lib.rs` - Add suffix parsing to IntLiteral token
  - `vex-parser/src/parser/primaries.rs` - Parse suffix and create typed literal
- **Impact:** HIGH - Every large constant needs explicit cast
- **Priority:** P1 - DX blocker for numeric code

### 🔴 P2: No Compile-Time Constant Folding

- **Issue:** Const expressions evaluated at codegen, not semantic analysis
- **Example:** `const X = 1000 * 1000 * 1000` → overflow at codegen, not parse
- **Rust Behavior:** Const eval in semantic pass, better error messages
- **Impact:** MEDIUM - Late error detection, confusing error messages
- **Priority:** P2 - Quality of life

## 🚨 Memory Safety & Runtime Issues

### 🔴 P0: No Automatic Drop/Destructor Calls

- **Issue:** Resources not freed when going out of scope (File, Socket, etc.)
- **Current State:** Manual `close()` required, easy to leak
- **Required:** RAII/Drop trait implementation
- **Files:** New `vex-compiler/src/codegen_ast/destructors.rs`
- **Impact:** CRITICAL - Memory/resource leaks in production
- **Priority:** P0 - BLOCKER for production use

### 🔴 P0: No Borrow Checker

- **Issue:** Dangling pointer bugs possible, no lifetime validation
- **Example:** Return reference to local variable compiles
- **Required:** Full borrow checker implementation
- **Impact:** CRITICAL - Unsafe memory access allowed
- **Priority:** P0 - Systems language requirement

### 🔴 P1: Uninitialized Variable Detection Weak

- **Issue:** `let x: i32; use(x);` sometimes allowed
- **Required:** Definite assignment analysis
- **Impact:** HIGH - Undefined behavior possible
- **Priority:** P1 - Safety critical

## 🚨 Type System Completeness

### 🔴 P1: No Associated Types in Traits

- **Issue:** Can't express `trait Iterator { type Item; }`
- **Workaround:** Generic parameters everywhere
- **Impact:** HIGH - Can't write idiomatic iterators
- **Priority:** P1 - API design blocker

### 🔴 P2: No Higher-Ranked Trait Bounds (HRTBs)

- **Issue:** Can't express `for<'a> F: Fn(&'a T)`
- **Impact:** MEDIUM - Limits closure APIs
- **Priority:** P2 - Advanced feature gap

### 🔴 P2: No Const Generics

- **Issue:** Can't write `Array<T, N: const usize>`
- **Workaround:** Runtime size only
- **Impact:** MEDIUM - Performance hit for fixed arrays
- **Priority:** P2 - Zero-cost abstraction gap

## 🚨 Error Handling Issues

### 🟢 FIXED: ? Operator Parsing ✅

- **Status:** PARSER IMPLEMENTED in v0.2.0
- **Example:** `let x = try_fn()?;` now parses correctly
- **Files Modified:**
  - `vex-ast/src/lib.rs` - Added TryOp expression variant
  - `vex-parser/src/parser/operators.rs` - Parse ? as postfix operator
  - `vex-compiler/src/codegen_ast/expressions/control_flow.rs` - Codegen infrastructure
  - `vex-compiler/src/borrow_checker/**/*.rs` - Updated pattern matching
- **Remaining:** Codegen needs Result enum support for unwrapping
- **Priority:** ~~P1~~ PARSER COMPLETE - Codegen blocked on Result enum

### 🔴 P1: No ? Operator

- **Issue:** Error propagation requires manual match
- **Example:** `let x = try_fn()?;` not supported
- **Current:** `let x = match try_fn() { Ok(v) => v, Err(e) => return Err(e) }`
- **Impact:** HIGH - Verbose error handling
- **Priority:** P1 - Ergonomics critical

### 🔴 P2: No Custom Error Types

- **Issue:** Result<T, string> only, no structured errors
- **Required:** Error trait, custom error types
- **Impact:** MEDIUM - Poor error context
- **Priority:** P2 - Production readiness

## 🚨 Standard Library Gaps

### 🔴 P0: No Vec/String Drop Implementation

- **Issue:** Vec and String leak memory when going out of scope
- **Current:** Manually call `.clear()` or `.free()`
- **Required:** Automatic deallocation
- **Impact:** CRITICAL - Every program leaks memory
- **Priority:** P0 - SHOWSTOPPER

### 🔴 P1: No Iterator Trait

- **Issue:** Can't write generic iteration code
- **Workaround:** for loops on Vec/Array only
- **Impact:** HIGH - No adapters (map, filter, etc.)
- **Priority:** P1 - Functional programming blocker

### 🔴 P1: No Option::unwrap_or / Result::unwrap_or_else

- **Issue:** Missing basic convenience methods
- **Impact:** HIGH - Verbose unwrapping code
- **Priority:** P1 - DX issue

## 🚨 Performance & Codegen

### 🔴 P1: Excessive Clone in Compiler

- **Issue:** Type/Expression cloned unnecessarily (see COMPATIBLE_CHECKS/02)
- **Impact:** HIGH - Slow compile times
- **Priority:** P1 - Performance critical

### 🔴 P2: No SIMD Support

- **Issue:** Can't use vector instructions
- **Impact:** MEDIUM - Slow numeric code
- **Priority:** P2 - Performance feature

### 🔴 P2: No Inline Assembly

- **Issue:** Can't write low-level kernel code
- **Impact:** MEDIUM - Not suitable for OS dev
- **Priority:** P2 - Systems programming gap

## 🚨 Concurrency Safety

### 🔴 P0: No Send/Sync Traits

- **Issue:** Can share non-thread-safe data across threads
- **Required:** Send/Sync marker traits
- **Impact:** CRITICAL - Data races possible
- **Priority:** P0 - Concurrency safety

### 🔴 P1: No Atomic Operations

- **Issue:** No atomic primitives (AtomicI32, etc.)
- **Workaround:** Mutex everywhere
- **Impact:** HIGH - Poor lock-free performance
- **Priority:** P1 - Concurrency performance

## 🚨 Macro System

### 🔴 P2: No Declarative Macros

- **Issue:** Can't write `vec![1, 2, 3]` style macros
- **Impact:** MEDIUM - Code generation limited
- **Priority:** P2 - DX feature

### 🔴 P3: No Procedural Macros

- **Issue:** Can't derive traits automatically
- **Impact:** LOW - Boilerplate in user code
- **Priority:** P3 - Nice to have

---

## 🚨 Critical Compiler Bugs (High Priority)

### 🔴 Constant Expression Crash (Builder Position Not Set)

- **Issue:** Compiler crashes with "Builder position is not set" when exporting constants defined by expressions involving other constants (e.g., `export const MINUTE = 60 * SECOND`).
- **Workaround:** Currently using hardcoded integer literals in `std/time`.
- **Fix Required:** Ensure `codegen_global_const` or equivalent handles expression evaluation properly without needing a function builder context, or creates a temporary context.
- **Impact:** Makes standard library code brittle and hard to read.

### Task 4: Initialize Runtime in Main ⚠️ IN PROGRESS

- **Status:** 90% COMPLETE - DEBUGGING SEGFAULT
- **File:** `vex-compiler/src/codegen_ast/program.rs`
- **Description:** Added runtime_create() call in main() when async functions exist
- **Issue:** Segfault during LLVM IR generation or linking
- **Completed:**
  - ✅ Runtime initialization before function compilation
  - ✅ runtime_run() and runtime_destroy() calls in main() return
  - ✅ Skip duplicate entry block for main()
  - ❌ Segfault happening after "Finished compiling function body"
- **Estimated Time:** 1 hour

---

## Debugging Notes

**Current Issue:** Segmentation fault when compiling async functions

- Location: After "Finished compiling function body" message
- Likely cause: LLVM IR generation issue in async state machine or wrapper
- Evidence:
  - Non-async code compiles and runs fine
  - Async detection and runtime init work correctly
  - runtime_spawn_global() is called in wrapper
  - Segfault happens before linking stage

**Next Steps:**

1. Add error handling around LLVM IR generation
2. Check if async wrapper/resume function generation is valid
3. Verify all LLVM function declarations match C signatures
4. Test with LLVM IR dump to see generated code

---

## Async/Await Type System - Priority 2

### Task 5: Add Future<T> Type

- **Status:** NOT STARTED
- **File:** `vex-ast/src/lib.rs`
- **Description:** Add `Future(Box<Type>)` variant to `Type` enum
- **Estimated Time:** 2 hours

### Task 6: Modify Async Function Return Type

- **Status:** NOT STARTED
- **File:** `vex-compiler/src/codegen_ast/functions/asynchronous.rs`
- **Description:** Change wrapper to return `Future<T>` instead of void
- **Estimated Time:** 1 hour

### Task 7: Implement await Result Handling

- **Status:** NOT STARTED
- **File:** `vex-compiler/src/codegen_ast/expressions/control_flow.rs`
- **Description:** Load result from Future when resuming after await
- **Estimated Time:** 1.5 hours

### Task 8: Add Promise<T> for Manual Futures

- **Status:** NOT STARTED
- **File:** `stdlib/core/async.vx` (new file)
- **Description:** Implement Promise API for manual future creation
- **Estimated Time:** 2 hours

---

## Async/Await Advanced Features - Priority 3

### Task 9: Async Blocks

- **Status:** NOT STARTED
- **Syntax:** `let fut = async { ... }`
- **File:** `vex-parser/src/parser/expressions.rs`
- **Estimated Time:** 3 hours

### Task 10: .await? Syntax

- **Status:** NOT STARTED
- **Syntax:** `let result = async_fn().await?`
- **File:** `vex-parser/src/parser/expressions.rs`
- **Estimated Time:** 1 hour

### Task 11: select! Macro

- **Status:** NOT STARTED
- **Syntax:** `select! { fut1 => ..., fut2 => ... }`
- **Estimated Time:** 8 hours

### Task 12: Stream<T> for Async Iteration

- **Status:** NOT STARTED
- **Syntax:** `for await item in stream { ... }`
- **Estimated Time:** 4 hours

---

## Async/Await Optimization - Priority 4

### Task 13: Inline Small Async Functions

- **Status:** NOT STARTED
- **Description:** Inline async functions < 50 LLVM IR lines
- **Estimated Time:** 2 hours

### Task 14: Optimize State Machine Size

- **Status:** NOT STARTED
- **Description:** Pack state fields, remove padding
- **Estimated Time:** 1 hour

### Task 15: Zero-Copy Future Chaining

- **Status:** NOT STARTED
- **Description:** Reuse state struct across await chain
- **Estimated Time:** 3 hours

---

## Current Status Summary

**Completed:**

- ✅ Async/await syntax parsing (100%)
- ✅ State machine codegen (100%)
- ✅ M:N async runtime infrastructure (90%)
- ✅ Channel<T> implementation (8/8 tests)
- ✅ go blocks (working)

**In Progress:**

- 🟡 Runtime integration (0% - Priority 1 tasks needed)

**Blocked By:**

- ❌ async fn execution (needs Task 1-4)
- ❌ Future<T> types (needs Task 5-8)

**Total Estimated Time:**

- Priority 1 (Tasks 1-4): ~2.5 hours → **Basic async/await working**
- Priority 2 (Tasks 5-8): +6.5 hours → **Production-ready types**
- Priority 3 (Tasks 9-12): +16 hours → **Advanced features**
- Priority 4 (Tasks 13-15): +6 hours → **Performance optimization**
- **Grand Total: ~31 hours for complete async/await ecosystem**
