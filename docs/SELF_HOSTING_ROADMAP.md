# Vex Self-Hosting Roadmap

**Goal:** Make Vex self-hosted - write core builtins in Vex instead of Rust
**Status:** Planning Phase
**Last Updated:** November 13, 2025

---

## 🎯 Vision

Replace Rust builtin implementations with **Vex + C FFI**, enabling:
- ✅ Better language dogfooding (using Vex to write Vex)
- ✅ Cleaner separation: Compiler (Rust) vs Runtime (Vex + C)
- ✅ Easier contribution (Vex syntax vs Rust/LLVM internals)
- ✅ Faster iteration on stdlib features

---

## 📊 Current State

### What's Done ✅

```
vex-runtime/c/        → 13,258 lines of production C runtime
├── vex_vec_*         → Vec<T> operations (new, push, get, len, etc.)
├── vex_string_*      → String operations
├── vex_map_*         → Map<K,V> operations
├── vex_set_*         → Set<T> operations
├── vex_box_*         → Box<T> operations
└── vex_option_*      → Option/Result helpers

vex-compiler/         → Rust compiler with LLVM IR generation
├── Type system       → Generics, traits, borrow checking
├── Builtin dispatch  → Method call routing to C functions
└── LLVM codegen      → Zero-overhead native code

stdlib/vex/           → BETA Vex wrappers (proof of concept)
├── beta_vec.vx       → Vec<T> wrapper around vex_vec_*
├── beta_option.vx    → Option<T> enum with helper functions
└── beta_result.vx    → Result<T,E> enum with helpers
```

### What's Missing ❌

```
stdlib/core/src/      → Layer 1 global builtins (NOT IMPLEMENTED)
├── vec.vx            → Global Vec<T> (auto-imported)
├── box.vx            → Global Box<T>
├── option.vx         → Global Option<T>
├── result.vx         → Global Result<T,E>
├── string.vx         → Global String
├── map.vx            → Global Map<K,V>
├── set.vx            → Global Set<T>
└── slice.vx          → Global Slice<T>
```

---

## 🏗️ Architecture Layers

```
┌────────────────────────────────────────────────────────┐
│ Layer 3: User Code (examples/, vex-libs/)             │
│ import fs from "std/fs"                                │
│ let files = fs.read_dir("/tmp")                        │
└────────────────┬───────────────────────────────────────┘
                 │
┌────────────────▼───────────────────────────────────────┐
│ Layer 2: Standard Library (vex-libs/std/)             │
│ - fs, io, time, net, json, http                       │
│ - Uses Layer 1 builtins (Vec, String, Result)         │
│ - Calls C FFI for OS-specific operations              │
└────────────────┬───────────────────────────────────────┘
                 │
┌────────────────▼───────────────────────────────────────┐
│ Layer 1: Core Builtins (stdlib/core/)  ⭐ SELF-HOSTED │
│ - Vec<T>, Box<T>, Option<T>, Result<T,E>              │
│ - String, Map<K,V>, Set<T>, Slice<T>                  │
│ - Written in VEX with C FFI bindings                   │
│ - Auto-imported globally (no import needed)           │
└────────────────┬───────────────────────────────────────┘
                 │ extern "C" fn calls
┌────────────────▼───────────────────────────────────────┐
│ Layer 0: C Runtime (vex-runtime/c/)                   │
│ - vex_vec_new(), vex_vec_push(), vex_string_len()     │
│ - Production-ready, battle-tested C code              │
│ - Memory allocators, data structures                  │
└────────────────────────────────────────────────────────┘
```

---

## 📋 Implementation Plan

### **Phase 1: Core Data Structures** (Week 1-2)

#### 1.1 Vec<T> - Dynamic Array

**File:** `stdlib/core/src/vec.vx`

```vex
// Vec<T> - Self-hosted dynamic array
// Status: PRODUCTION (migrated from beta_vec.vx)

extern "C" {
    fn vex_vec_new(elem_size: i64): ptr;
    fn vex_vec_push(vec: ptr, elem: ptr);
    fn vex_vec_get(vec: ptr, index: i64): ptr;
    fn vex_vec_len(vec: ptr): i64;
    fn vex_vec_free(vec: ptr);
    fn vex_vec_reserve(vec: ptr, additional: i64);
    fn vex_vec_pop(vec: ptr, out: ptr): bool;
    fn vex_vec_clear(vec: ptr);
    fn vex_vec_is_empty(vec: ptr): bool;
    fn vex_vec_capacity(vec: ptr): i64;
}

export struct Vec<T> impl Drop {
    _ptr: ptr,
    
    fn drop()! {
        vex_vec_free(self._ptr);
    },
    
    // Constructor
    pub new(): Vec<T> {
        return Vec { _ptr: vex_vec_new(sizeof(T)) };
    },
    
    pub with_capacity(capacity: i64): Vec<T> {
        let v = Vec.new();
        vex_vec_reserve(v._ptr, capacity);
        return v;
    },
    
    // Methods
    pub fn push(self, value: T)! {
        vex_vec_push(self._ptr, &value);
    },
    
    pub fn get(self, index: i64): T {
        let ptr = vex_vec_get(self._ptr, index);
        return *ptr as T;
    },
    
    pub fn len(self): i64 {
        return vex_vec_len(self._ptr);
    },
    
    pub fn is_empty(self): bool {
        return vex_vec_is_empty(self._ptr);
    },
    
    pub fn pop(self): Option<T> {
        if self.is_empty() {
            return Option.None;
        }
        let! result: T;
        if vex_vec_pop(self._ptr, &result) {
            return Option.Some(result);
        }
        return Option.None;
    },
}
```

**Tasks:**
- [ ] Migrate `stdlib/vex/beta_vec.vx` → `stdlib/core/src/vec.vx`
- [ ] Add `pub` visibility to all methods
- [ ] Export Vec<T> globally (no import needed)
- [ ] Add Display, Clone, Debug contracts
- [ ] Write tests in `stdlib/core/tests/vec_test.vx`

#### 1.2 Box<T> - Heap Allocation

**File:** `stdlib/core/src/box.vx`

```vex
extern "C" {
    fn vex_box_new(elem_size: i64): ptr;
    fn vex_box_get(box: ptr): ptr;
    fn vex_box_free(box: ptr);
}

export struct Box<T> impl Drop {
    _ptr: ptr,
    
    fn drop()! {
        vex_box_free(self._ptr);
    },
    
    pub new(value: T): Box<T> {
        let b = Box { _ptr: vex_box_new(sizeof(T)) };
        let p = vex_box_get(b._ptr);
        *p = value;
        return b;
    },
    
    pub fn unwrap(self): T {
        let ptr = vex_box_get(self._ptr);
        return *ptr as T;
    },
}
```

**Tasks:**
- [ ] Create `stdlib/core/src/box.vx`
- [ ] Export Box<T> globally
- [ ] Write tests

#### 1.3 Option<T> - Optional Values

**File:** `stdlib/core/src/option.vx`

```vex
export enum Option<T> {
    Some(T),
    None,
}

export impl<T> Option<T> {
    pub fn is_some(self): bool {
        match self {
            Option.Some(_) => true,
            Option.None => false,
        }
    },
    
    pub fn is_none(self): bool {
        !self.is_some()
    },
    
    pub fn unwrap(self): T {
        match self {
            Option.Some(value) => value,
            Option.None => panic("Called unwrap on None"),
        }
    },
    
    pub fn unwrap_or(self, default: T): T {
        match self {
            Option.Some(value) => value,
            Option.None => default,
        }
    },
    
    pub fn map<U>(self, f: fn(T): U): Option<U> {
        match self {
            Option.Some(value) => Option.Some(f(value)),
            Option.None => Option.None,
        }
    },
}
```

**Tasks:**
- [ ] Migrate `stdlib/vex/beta_option.vx` → `stdlib/core/src/option.vx`
- [ ] Add impl block with methods
- [ ] Export globally
- [ ] Write tests

#### 1.4 Result<T, E> - Error Handling

**File:** `stdlib/core/src/result.vx`

```vex
export enum Result<T, E> {
    Ok(T),
    Err(E),
}

export impl<T, E> Result<T, E> {
    pub fn is_ok(self): bool {
        match self {
            Result.Ok(_) => true,
            Result.Err(_) => false,
        }
    },
    
    pub fn is_err(self): bool {
        !self.is_ok()
    },
    
    pub fn unwrap(self): T {
        match self {
            Result.Ok(value) => value,
            Result.Err(_) => panic("Called unwrap on Err"),
        }
    },
    
    pub fn expect(self, msg: str): T {
        match self {
            Result.Ok(value) => value,
            Result.Err(_) => panic(msg),
        }
    },
}
```

**Tasks:**
- [ ] Migrate `stdlib/vex/beta_result.vx` → `stdlib/core/src/result.vx`
- [ ] Add impl block
- [ ] Export globally
- [ ] Write tests

---

### **Phase 2: String & Collections** (Week 3-4)

#### 2.1 String - UTF-8 Strings

**File:** `stdlib/core/src/string.vx`

```vex
extern "C" {
    fn vex_string_new(): ptr;
    fn vex_string_from_cstr(cstr: ptr): ptr;
    fn vex_string_len(s: ptr): i64;
    fn vex_string_is_empty(s: ptr): bool;
    fn vex_string_free(s: ptr);
    fn vex_string_clone(s: ptr): ptr;
    fn vex_string_concat(a: ptr, b: ptr): ptr;
}

export struct String impl Drop, Clone, Display {
    _ptr: ptr,
    
    fn drop()! {
        vex_string_free(self._ptr);
    },
    
    fn clone(): String {
        return String { _ptr: vex_string_clone(self._ptr) };
    },
    
    fn display(): str {
        // Return internal C string pointer
        return self._ptr as str;
    },
    
    pub new(): String {
        return String { _ptr: vex_string_new() };
    },
    
    pub from(s: str): String {
        return String { _ptr: vex_string_from_cstr(s as ptr) };
    },
    
    pub fn len(self): i64 {
        return vex_string_len(self._ptr);
    },
    
    pub fn is_empty(self): bool {
        return vex_string_is_empty(self._ptr);
    },
}
```

**Tasks:**
- [ ] Create `stdlib/core/src/string.vx`
- [ ] Implement Drop, Clone, Display
- [ ] Export globally
- [ ] Write tests

#### 2.2 Map<K, V> - Hash Map

**File:** `stdlib/core/src/map.vx`

```vex
extern "C" {
    fn vex_map_new(key_size: i64, val_size: i64): ptr;
    fn vex_map_insert(map: ptr, key: ptr, val: ptr): bool;
    fn vex_map_get(map: ptr, key: ptr): ptr;
    fn vex_map_contains(map: ptr, key: ptr): bool;
    fn vex_map_remove(map: ptr, key: ptr): bool;
    fn vex_map_len(map: ptr): i64;
    fn vex_map_free(map: ptr);
}

export struct Map<K, V> impl Drop {
    _ptr: ptr,
    
    fn drop()! {
        vex_map_free(self._ptr);
    },
    
    pub new(): Map<K, V> {
        return Map { _ptr: vex_map_new(sizeof(K), sizeof(V)) };
    },
    
    pub fn insert(self, key: K, value: V)!: bool {
        return vex_map_insert(self._ptr, &key, &value);
    },
    
    pub fn get(self, key: K): Option<V> {
        let ptr = vex_map_get(self._ptr, &key);
        if ptr == nil {
            return Option.None;
        }
        return Option.Some(*ptr as V);
    },
    
    pub fn contains(self, key: K): bool {
        return vex_map_contains(self._ptr, &key);
    },
    
    pub fn len(self): i64 {
        return vex_map_len(self._ptr);
    },
}
```

**Tasks:**
- [ ] Create `stdlib/core/src/map.vx`
- [ ] Export globally
- [ ] Write tests

#### 2.3 Set<T> - Hash Set

**File:** `stdlib/core/src/set.vx`

```vex
extern "C" {
    fn vex_set_new(elem_size: i64): ptr;
    fn vex_set_insert(set: ptr, elem: ptr): bool;
    fn vex_set_contains(set: ptr, elem: ptr): bool;
    fn vex_set_remove(set: ptr, elem: ptr): bool;
    fn vex_set_len(set: ptr): i64;
    fn vex_set_free(set: ptr);
}

export struct Set<T> impl Drop {
    _ptr: ptr,
    
    fn drop()! {
        vex_set_free(self._ptr);
    },
    
    pub new(): Set<T> {
        return Set { _ptr: vex_set_new(sizeof(T)) };
    },
    
    pub fn insert(self, value: T)!: bool {
        return vex_set_insert(self._ptr, &value);
    },
    
    pub fn contains(self, value: T): bool {
        return vex_set_contains(self._ptr, &value);
    },
    
    pub fn len(self): i64 {
        return vex_set_len(self._ptr);
    },
}
```

**Tasks:**
- [ ] Create `stdlib/core/src/set.vx`
- [ ] Export globally
- [ ] Write tests

---

### **Phase 3: Compiler Integration** (Week 5)

#### 3.1 Auto-import stdlib/core

**File:** `vex-compiler/src/module_resolver.rs`

```rust
// Automatically import core types into every file
fn inject_core_prelude(program: &mut Program) {
    let core_imports = vec![
        "Vec", "Box", "Option", "Result", 
        "String", "Map", "Set", "Slice"
    ];
    
    for type_name in core_imports {
        program.add_implicit_import("core", type_name);
    }
}
```

**Tasks:**
- [ ] Modify compiler to auto-import `stdlib/core/src/*.vx`
- [ ] Remove manual `import Vec from "core"` requirement
- [ ] Update all example files to remove explicit imports

#### 3.2 Builtin Method Dispatch

Currently: Rust code in `vex-compiler/src/codegen_ast/builtins/`
Future: Route to Vex stdlib methods

**Tasks:**
- [ ] Keep C FFI declarations in Rust (for type safety)
- [ ] Remove Rust method implementations
- [ ] Let Vex stdlib handle method calls

---

### **Phase 4: Testing & Validation** (Week 6)

#### 4.1 Test Suite

```
stdlib/core/tests/
├── vec_test.vx
├── box_test.vx
├── option_test.vx
├── result_test.vx
├── string_test.vx
├── map_test.vx
└── set_test.vx
```

**Tasks:**
- [ ] Port existing examples to use new stdlib
- [ ] Run `./test_all.sh` - ensure 407/407 passing
- [ ] Benchmark performance vs current Rust impl

#### 4.2 Documentation

**Tasks:**
- [ ] Update `docs/REFERENCE.md` with new stdlib
- [ ] Add stdlib API documentation
- [ ] Write migration guide from beta → production

---

## 🎯 Success Criteria

- ✅ All core builtins (Vec, Box, Option, Result, String, Map, Set) written in Vex
- ✅ Zero performance regression vs Rust implementation
- ✅ All 407+ tests passing
- ✅ Auto-import working (no manual imports needed)
- ✅ `vex-libs/std/` using new core stdlib
- ✅ Examples updated and working

---

## 🚀 Benefits of Self-Hosting

1. **Dogfooding** - Use Vex to write Vex (confidence in language)
2. **Contribution** - Easier for contributors (Vex vs Rust internals)
3. **Iteration Speed** - Change stdlib without recompiling Rust
4. **Clarity** - Clear separation: Compiler (Rust) vs Runtime (Vex+C)
5. **Testing** - stdlib becomes comprehensive language test suite

---

## 📝 Next Steps

1. **Review this roadmap** - Get feedback from team
2. **Start Phase 1.1** - Migrate Vec<T> to stdlib/core/
3. **Set up CI** - Automated testing for each phase
4. **Track progress** - Update this doc weekly

---

**Last Updated:** November 13, 2025
**Status:** 🟡 Planning → Ready to start Phase 1
