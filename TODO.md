# Vex Language - TODO

## 🎯 Aktif Geliştirme

### Phase 4: Lifetime Analysis (Sırada) 🔴

- [ ] Scope-based lifetime tracking
- [ ] Dangling reference prevention
- [ ] Integration with borrow checker (Phase 1-3)
- **Estimated**: 5-6 days
- **Priority**: High

### Bekleyen Özellikler

#### Yüksek Öncelik 🔴

1. **Lifetime Analysis** (Phase 4)
2. **Default trait methods** - Trait inheritance implementation
3. **Data-carrying enum patterns** - `Some(x)`, `Ok(val)` destructuring
4. **Reference expression codegen** - `&expr` in function calls

#### Orta Öncelik 🟡

1. **Trait bounds** - Generic constraint checking
2. **Dynamic dispatch** - Vtable generation for trait objects
3. **Closures** - Lambda expressions
4. **Async runtime** - State machine transformation
5. **Memory allocator** - `new()` built-in function with RC

#### Düşük Öncelik 🟢

1. **GPU/SIMD runtime** - Kernel execution
2. **Macro system** - Compile-time code generation
3. **Advanced optimizations** - LLVM passes

## ✅ Tamamlanan Özellikler

### v0.9 Syntax (3 Kasım 2025) ✅ VERIFIED

- [x] **Mutable references**: `&T!` instead of `&mut T`
- [x] **Immutability**: `let` (immutable) vs `let!` (mutable)
- [x] **Keyword removed**: `mut` keyword DELETED from lexer
- [x] **Deprecated**: `interface` keyword returns error (use `trait`)
- [x] **Parser**: Updated to v0.9 syntax (no `mut`, uses `!`)
- [x] **Tests**: Verified `mut` rejected, `let!` works
- [x] **Documentation**: `Syntax.md` updated

### Borrow Checker (3 Kasım 2025)

- [x] **Phase 1: Immutability Check** (7 tests ✅)
  - Enforces `let` vs `let!` semantics
  - Prevents assignment to immutable variables
- [x] **Phase 2: Move Semantics** (5 tests ✅)
  - Prevents use-after-move
  - Tracks Copy vs Move types
  - Supports shadowing
- [x] **Phase 3: Borrow Rules** (5 tests ✅)
  - Enforces: 1 mutable XOR N immutable references
  - Tracks active borrows
  - Prevents mutation while borrowed
- [x] **Parser**: `&T!` syntax support in types and expressions
- [x] **CLI Integration**: Automatic checking on compile/run
- [x] **Examples**: 9 files in `examples/00_borrow_checker/` with README

### Trait System v1.3 (3 Kasım 2025)

- [x] **Inline implementation**: `struct Foo impl Trait1, Trait2 { ... }`
- [x] **AST Changes**:
  - `Struct.impl_traits` and `Struct.methods` fields
  - `Trait.super_traits` for inheritance
  - `TraitMethod.body` for default implementations
- [x] **Parser**:
  - Multiple trait support (comma-separated)
  - Inline method syntax: `fn (self: &Type) method() { ... }`
  - Trait inheritance parsing: `trait A: B, C`
  - Interface deprecation error
- [x] **Codegen**:
  - Method mangling: `StructName_methodName`
  - Inline method compilation
- [x] **Examples**: 4 files in `examples/09_trait/` with README
- [ ] Default method inheritance (pending)
- [ ] Trait bounds checking (pending)
- [ ] Dynamic dispatch (pending)

### Pattern Matching (2 Kasım 2025)

- [x] **Match expressions**: Basic support
- [x] **Pattern types**: Wildcard, literal, identifier
- [x] **Guard clauses**: `if` conditions in match arms
- [x] **Tuple patterns**: `(x, y)` destructuring ✅
- [x] **Struct patterns**: `Point { x, y }` destructuring ✅
- [x] **Enum patterns**: Unit variant matching ✅
- [x] **Binding**: Pattern variable binding works
- [ ] Data-carrying enum patterns (pending)

### Core Language Features

- [x] Variables: `let` (immutable), `let!` (mutable)
- [x] Types: i8/16/32/64, u8/16/32/64, f32/64, bool, string
- [x] Functions: params, return types, generics
- [x] Control flow: if/else, while, for, switch
- [x] Operators: arithmetic, comparison, logical, bitwise, compound assignment
- [x] Data structures: struct, enum, tuple, array
- [x] Generics: functions and structs
- [x] References: `&T` (immutable), `&T!` (mutable)
- [x] Module system: Go + JS + Rust hybrid
- [x] Async/await: parsing (codegen pending)
- [x] Try operator: `?` syntax
- [x] Go keyword: goroutine-style syntax

### Tooling

- [x] CLI: `vex compile`, `vex run`
- [x] Inline execution: `vex run -c "code"`
- [x] LLVM backend: IR generation
- [x] Standard library: `std/log`

## 📊 Test Status

**Current**: 29/50 tests passing (58%)

- Borrow checker: 17/17 tests ✅
- Trait system: 4/4 examples ✅
- Pattern matching: Working ✅
- Core features: Stable ✅

## 📁 Project Structure

```
examples/
├── 00_borrow_checker/     # 9 files + README (Borrow checker examples)
├── 09_trait/              # 4 files + README (Trait system v1.3)
└── ...                    # Other examples

vex-ast/                   # AST definitions (v1.3 trait system)
vex-parser/                # Parser (v0.9 syntax support)
vex-compiler/              # Compiler + Borrow checker
vex-cli/                   # CLI tool
vex-libs/                  # Standard library
```

## 📖 Documentation

- `TRAIT_SYSTEM_MIGRATION_STATUS.md` - Trait system v1.3 details
- `Syntax.md` - v0.9 syntax reference
- `Specification.md` - Language specification
- `examples/00_borrow_checker/README.md` - Borrow checker guide
- `examples/09_trait/README.md` - Trait system guide

## 🚀 Next Steps

1. **Immediate**: Phase 4 - Lifetime Analysis
2. **Short-term**: Default trait methods, trait bounds
3. **Mid-term**: Closures, async runtime, memory allocator
4. **Long-term**: GPU/SIMD, macros, optimizations

---

**Last Updated**: 3 Kasım 2025
**Version**: 0.9 (Borrow checker + Trait system v1.3)
**Status**: `mut` keyword REMOVED ✅ | `interface` keyword DEPRECATED ✅
