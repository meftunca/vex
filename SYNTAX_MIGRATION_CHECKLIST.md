# Syntax Migration Script

**Purpose:** Fix all `::` usage in documentation and examples

---

## 🔍 Search Patterns

```bash
# Find all :: usage in markdown files
grep -r "::" docs/*.md
grep -r "::" *.md

# Find in Vex examples
grep -r "::" examples/*.vx
```

---

## 🔧 Replacement Patterns

### Type Constructors

```bash
Vec::new()           → Vec.new()
Vec::with_capacity   → Vec.with_capacity
Vec::from            → Vec.from
Map::new()           → Map.new()
Set::new()           → Set.new()
String::new()        → String.new()
String::from         → String.from
Channel::new         → Channel.new
```

### Enum Variants

```bash
Option::Some         → Some
Option::None         → None
Result::Ok           → Ok
Result::Err          → Err
```

### Comments (OK to keep)

```bash
Type::Function       → (OK in AST documentation)
Expression::Closure  → (OK in implementation docs)
```

---

## 📝 Manual Review Needed

**These are OK (Rust code in compiler):**

- `vex-ast/src/lib.rs` - `Type::Option`, `Type::Vec` (Rust enum variants)
- `vex-compiler/**/*.rs` - Any Rust code
- Implementation docs referring to Rust internals

**These MUST change (Vex code):**

- All `.md` files with Vex code examples
- All `examples/**/*.vx` files
- All `.vx` test files

---

## ✅ Verification

After changes, grep for remaining `::` in wrong places:

```bash
# Should find ZERO results:
grep -r "Vec::new" *.md
grep -r "Option::Some" *.md
grep -r "Map::new" *.md
```

---

## 🎯 Files to Update (Priority)

### High Priority (Vex code examples)

- [ ] BUILTIN_TYPES_ARCHITECTURE.md
- [ ] ITERATOR_SYSTEM_DESIGN.md
- [ ] VEX_RUNTIME_STDLIB_ROADMAP.md
- [ ] BUILTIN_TYPES_QUICKSTART.md
- [ ] NAMING_DECISIONS.md

### Medium Priority (Examples)

- [ ] examples/\*_/_.vx (check if any exist with ::)

### Low Priority (Old docs)

- [ ] TODO.md (only Vex examples, not AST references)
- [ ] Other documentation

---

**Status:** Created `MODULE_SYSTEM_SYNTAX_FIX.md` and `VEX_SYNTAX_GUIDE.md`  
**Next:** Manually update remaining :: usage in key documents
