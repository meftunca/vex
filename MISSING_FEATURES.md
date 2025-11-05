# Missing Features - Implementation Backlog

## High Priority

### 1. Try/Catch Operator (`?`)

**Status:** Not Implemented  
**Usage:** `let x = divide(10, 2)?;` - Early return on error  
**Why:** Essential for ergonomic error handling with Result<T,E> types. Eliminates nested match blocks, improves readability in error-prone code paths.

### 2. Unit Enum Literal in Match

**Status:** Partially Implemented  
**Usage:** `match status { Status.Idle => 0, ... }`  
**Why:** Unit enums (no data) need direct pattern matching support. Currently fails to match enum variants as literals in patterns.

### 3. Deep Generic Type Parsing (>5 levels)

**Status:** Parser Limitation  
**Usage:** `Box<Box<Box<Box<Box<i32>>>>>`  
**Why:** Recursive generic types beyond depth 5-10 cause parser errors. Need depth limit check or better recursive type parsing.

## Medium Priority

### 4. Nested Generic Field Access Edge Cases

**Status:** Partially Working  
**Usage:** `outer.value.value` with Box<Box<T>>  
**Why:** Type tracking works but runtime crashes on deep nesting (>3 levels). Need better pointer chain handling.

### 5. Enum Method Calls

**Status:** Not Implemented  
**Usage:** `result.is_ok()` or `option.unwrap()`  
**Why:** Common pattern for ergonomic enum handling. Currently requires manual match blocks for simple checks.

### 6. String Interpolation in Match Arms

**Status:** Not Implemented  
**Usage:** `match x { n => f"Value: {n}" }`  
**Why:** F-strings in match expressions crash. Need proper scope handling for pattern bindings in interpolation.

## Low Priority

### 7. Async/Await Support

**Status:** Runtime Partial, Syntax Not Implemented  
**Usage:** `async fn fetch() -> Result<T,E>` + `await`  
**Why:** C runtime has async support but no language syntax. Needed for I/O-heavy applications.

### 8. Dynamic Dispatch (dyn Trait)

**Status:** Not Implemented  
**Usage:** `let obj: dyn Display = ...`  
**Why:** Currently only static dispatch via generics. Dynamic needed for heterogeneous collections.

### 9. Closure Trait Bounds (Fn/FnMut/FnOnce)

**Status:** Environment Detection Only  
**Usage:** `fn map<F: Fn(T) -> U>(f: F)`  
**Why:** Closures compile but no trait-based constraints. Needed for generic higher-order functions.

### 10. Module System (Full Import Resolution)

**Status:** Basic Only  
**Usage:** `use std.collections.{Vec, HashMap}`  
**Why:** Only simple imports work. Need complex paths, re-exports, visibility control.

## Parser Fixes Needed

### 11. Negative Number Literals

**Status:** Bug  
**Usage:** `let x = -42;`  
**Why:** Parsed as UnaryOp(-, 42) instead of IntLiteral(-42). Breaks in some contexts.

### 12. Multi-line Type Annotations

**Status:** Parser Limitation  
**Usage:** `let x: Box<Box<...>>` over multiple lines  
**Why:** Parser expects `>` immediately, fails on newlines in deep generic types.

## Test Coverage Gaps

- **10 failing tests** (92.7% pass rate, 127/137)
- Most failures: Error handling (? operator), deep generics, enum match edge cases
- Runtime crashes (bus errors) in pattern binding with enums

---

**Last Updated:** 5 Kasım 2025  
**Test Status:** 127/137 passing (92.7%)
