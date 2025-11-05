# Builtin Types Implementation - Quick Start

**Status:** Planning → Implementation  
**Priority:** 🔴 HIGHEST (Phase 1 of language)  
**Timeline:** 14-20 days

---

## 📋 Overview

Core types are now **builtin** (language-level, no imports):

- `Option<T>` - Nullable types
- `Result<T, E>` - Error handling
- `Vec<T>` - Dynamic arrays
- `Map<K, V>` - Hash tables (not HashMap - hashing is default!)
- `Set<T>` - Hash sets
- `String` - UTF-8 strings
- `Iterator<T>` - Iteration protocol (trait + for-in loops)
- `Channel<T>` - Message passing

**Why builtin?**

- ✅ Zero imports needed
- ✅ Compiler integration (pattern matching, operators)
- ✅ Borrow checker aware
- ✅ Zero-cost (monomorphized, inlined)

---

## 🚀 Quick Reference

### Documents

- **`BUILTIN_TYPES_ARCHITECTURE.md`** - Full implementation plan (14-20 days, 6 phases)
- **`ITERATOR_SYSTEM_DESIGN.md`** - Iterator trait, for-in loops, map/filter/fold
- **`NAMING_DECISIONS.md`** - Why Map/Set (not HashMap/HashSet)
- **`VEX_RUNTIME_STDLIB_ROADMAP.md`** - Updated with builtin types as Sprint 0
- **`TODO.md`** - Added to Phase 2 (moved from Phase 3)

### Implementation Order

1. **Phase 1:** Option, Result, Vec foundation (3-4 days)
2. **Phase 2:** Pattern matching (2-3 days)
3. **Phase 3:** Methods & operators (2-3 days)
4. **Phase 4:** Map, Set & Iterator (3-4 days)
5. **Phase 5:** Channels (3-4 days)
6. **Phase 6:** Optimization (2-3 days)

### File Structure

```
vex-runtime/c/
├── vex_vec.c          # Generic vector (NEW)
├── vex_option.c       # Option helpers (NEW)
├── vex_result.c       # Result helpers (NEW)
├── vex_map.c          # Map wrapper (rename from hashmap, SwissTable)
├── vex_set.c          # Set wrapper (NEW)
├── vex_iterator.c     # Iterator state (NEW)
├── vex_string.c       # String ops (extend existing)
└── vex_channel.c      # MPSC channel (NEW)

vex-ast/src/lib.rs
└── Type enum:
    ├── Option(Box<Type>)           # NEW
    ├── Result(Box<Type>, Box<Type>) # NEW
    ├── Vec(Box<Type>)              # NEW
    ├── Map(Box<Type>, Box<Type>)   # NEW (hash-based)
    ├── Set(Box<Type>)              # NEW (hash-based)
    └── Iterator(Box<Type>)         # NEW (trait-based)

vex-compiler/src/codegen_ast/
└── builtin_types/     # NEW MODULE
    ├── mod.rs
    ├── option.rs      # Option codegen
    ├── result.rs      # Result codegen
    ├── vec.rs         # Vec codegen
    ├── map.rs         # Map codegen
    ├── set.rs         # Set codegen
    ├── iterator.rs    # Iterator trait
    └── drop.rs        # Drop trait
```

---

## 🎯 Start with Phase 1

### Step 1: C Runtime (1-2 days)

```bash
cd vex-runtime/c
vim vex_vec.c      # Generic vector implementation
vim vex_option.c   # Option unwrap helpers
vim vex_result.c   # Result unwrap helpers
./build.sh
```

### Step 2: AST Types (1 day)

```rust
// vex-ast/src/lib.rs
pub enum Type {
    // ... existing types ...
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Vec(Box<Type>),
    // ...
}
```

### Step 3: Parser (1 day)

```rust
// vex-parser/src/parser/types.rs
fn parse_generic_builtin(&mut self) -> Result<Type, ParseError> {
    match self.current.kind {
        TokenKind::Ident("Option") => { /* ... */ }
        TokenKind::Ident("Vec") => { /* ... */ }
        // ...
    }
}
```

### Step 4: Codegen (1-2 days)

```rust
// vex-compiler/src/codegen_ast/builtin_types/vec.rs
impl<'ctx> ASTCodeGen<'ctx> {
    pub fn compile_vec_type(&mut self, elem_ty: &Type) -> StructType<'ctx> {
        // Struct: { *T, len, capacity }
    }

    pub fn compile_vec_push(&mut self, ...) {
        // Call vex_vec_push(vec_ptr, &value)
    }
}
```

### Step 5: Tests (0.5 day)

```vex
// examples/10_builtins/vec_basic.vx
fn main(): i32 {
    let! vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    return vec[0]; // Should return 1
}
```

---

## 🔑 Key Principles

1. **No imports** - Available everywhere
2. **Zero-cost** - Same performance as C/Rust
3. **Type-safe** - Borrow checker integration
4. **RAII** - Automatic cleanup via Drop
5. **Ergonomic** - Pattern matching, operators, methods

---

## 📊 Success Criteria

- [ ] No `import std.collections` needed
- [ ] Pattern matching: `match opt { Some(x) => ..., None => ... }`
- [ ] Method calls: `vec.push(x)`, `opt.unwrap()`
- [ ] Operators: `vec[i]`, `result?`
- [ ] Performance: <5% overhead vs Rust
- [ ] Tests: All `examples/10_builtins/*.vx` passing

---

**Ready to implement?** Start with Phase 1 → `vex_vec.c` + `Type::Vec`
