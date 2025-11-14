# Vex Language Development TODO

## Async/Await Runtime Integration - Priority 1 (Core Functionality)

### Task 1: Add Global Runtime Handle ✅ COMPLETE

- **Status:** COMPLETE
- **File:** `vex-compiler/src/codegen_ast/struct_def.rs`
- **Description:** Added `global_runtime: Option<PointerValue<'ctx>>` field to `ASTCodeGen` struct
- **Estimated Time:** 30 minutes

### Task 2: Declare runtime_spawn_global() in LLVM ✅ COMPLETE

- **Status:** COMPLETE
- **File:** `vex-compiler/src/codegen_ast/functions/asynchronous.rs`
- **Description:** Added `get_or_declare_runtime_spawn()` helper function + runtime_create/run/destroy
- **Estimated Time:** 15 minutes

### Task 3: Call runtime_spawn_global in Async Wrapper ✅ COMPLETE

- **Status:** COMPLETE
- **File:** `vex-compiler/src/codegen_ast/functions/asynchronous.rs` (line 197)
- **Description:** Replaced TODO comment with actual runtime_spawn_global() call
- **Estimated Time:** 45 minutes

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
