# Vex Language - TODO

**Current Status:** 143/146 tests passing (97.9%)

**Last Updated:** November 5, 2025

## 🎯 Phase 1: Core Language Features (Priority 🔴)

### Immediate Priority

- [ ] **`?` Operator** (~1 day) - Result<T,E> early return, desugars to match
- [ ] **Dynamic Dispatch** (~3 days) - Vtable generation for `dyn Trait`
- [ ] **Where Clauses** (~1 day) - `where T: Display` syntax
- [ ] **Associated Types** (~2 days) - `trait Container { type Item; }`
- [ ] **Reference Lifetime Validation** (~2 days) - Advanced rules
- [ ] **Lifetime Elision** (~1 day) - Auto-infer lifetimes
- [ ] **Explicit Lifetime Parameters** (~1 day) - `'a` syntax

---

## 🎯 Phase 2: Builtin Types Completion

**See:** `BUILTIN_TYPES_ARCHITECTURE.md`, `ITERATOR_SYSTEM_DESIGN.md`

**Tier 0 (Core - 10 types):** ✅ Vec, Box, Option, Result, Tuple | ⏳ Slice, String, str, Range, RangeInclusive  
**Tier 1 (Collections - 4 types):** Map, Set, Channel, Iterator  
**Tier 2 (Advanced - 3 types):** Array<T,N>, Never (!), RawPtr (\*T)

### Remaining Tier 0 Types

- [ ] **Range & RangeInclusive** (2-3 days)
  - Critical for for-in loops: `for i in 0..10 {}`
  - Parser: `0..10` (Range), `0..=10` (RangeInclusive)
  - C Runtime: `vex_range.c` (iterator protocol)
  - Codegen: Range construction, iterator methods
- [ ] **String & str** (3-4 days)
  - Parser: String literals, str type
  - C Runtime: Already exists (`vex_string.c`, `vex_simd_utf.c`)
  - Codegen: String operations, UTF-8 validation
  - Methods: len, chars, slice, concat
- [ ] **Slice<T>** (2-3 days)
  - Parser: `&[T]` syntax
  - C Runtime: Slice view operations
  - Codegen: Slice from Vec, arrays
  - Methods: len, get, iter

### Tier 1 Collections

- [ ] **Map<K,V> & Set<T>** (4-5 days)

  - C Runtime: Already exists (`vex_swisstable.c` - Google Swiss Tables)
  - Parser: Map/Set syntax
  - Codegen: Collection operations
  - Methods: insert, get, remove, contains

- [ ] **Iterator<T>** (3-4 days)

  - See `ITERATOR_SYSTEM_DESIGN.md`
  - Trait: `trait Iterator { fn next(&self!): Option<Self.Item>; }`
  - Adapters: map, filter, fold, collect
  - Critical for for-in loops

- [ ] **Channel<T>** (2-3 days)
  - CSP-style concurrency
  - C Runtime: Lock-free queue
  - Methods: send, recv, try_recv

### Tier 2 Advanced

- [ ] **Array<T,N>** (2-3 days)

  - Fixed-size arrays with const generics
  - Parser: `[T; N]` syntax
  - Stack-allocated, no heap

- [ ] **Never (!)** (1 day)

  - Diverging function return type
  - For panic, exit, infinite loops

- [ ] **RawPtr (\*T)** (1-2 days)
  - FFI/C interop
  - Parser: `*T` syntax
  - Unsafe operations

---

## 🎯 Phase 3: Runtime & Advanced Features

### Async/Await

- [ ] **State Machine Transformation** (~3 days) - async/await codegen
- [ ] **Future Trait** (~2 days) - Core async abstraction
- [ ] **Runtime Integration** (~2 days) - C runtime already exists

### Module System

- [ ] **Module imports** - Already partially working
- [ ] **Package manager** - See `PACKAGE_MANAGER_DRAFT.md`

---

## 📊 Known Issues

**Test Status:** 143/146 passing (97.9%)

**Failing Tests:**

1. `error_handling_try.vx` - Needs `?` operator
2. `nested_extreme.vx` - Parser depth limit (70-level nesting)
3. 1 test script false positive

**Expected Failures:** Tests 2-3 are edge cases, not critical bugs
