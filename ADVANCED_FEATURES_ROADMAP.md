# Vex Language - Advanced Features Roadmap

**Created:** November 10, 2025  
**Version:** v0.2.0 Planning  
**Status:** Implementation Plan

---

## 🎯 Executive Summary

Bu dokuman, Vex dilinin ileri seviye özelliklerinin implementasyon planını içerir. Her özellik, dependency'lere göre sıralanmış ve implementation adımları detaylandırılmıştır.

**Toplam Özellik Sayısı:** 8 major features  
**Tahmini Süre:** 4-6 hafta (full-time)  
**Öncelik Kategorileri:** CRITICAL (2), HIGH (3), MEDIUM (3)

---

## 📊 Feature Priority Matrix

| Feature                          | Priority | Est. Time | Dependencies     | Impact                         | Status      |
| -------------------------------- | -------- | --------- | ---------------- | ------------------------------ | ----------- |
| **1. Static Methods (new/free)** | CRITICAL | 3-4 days  | None             | Constructor/Destructor pattern | ✅ COMPLETE |
| **2. Iterator Trait**            | CRITICAL | 5-7 days  | Associated Types | For-in loop for collections    |             |
| **3. Trait Bounds Enforcement**  | HIGH     | 4-5 days  | None             | Generic type safety            | ✅ COMPLETE |
| **4. Display Trait**             | HIGH     | 2-3 days  | None             | Debug/print standardization    | 🚧 NEXT     |
| **5. Associated Types Codegen**  | HIGH     | 5-6 days  | None             | Advanced trait usage           | ✅ PARSER   |
| **6. Drop Trait (Auto-cleanup)** | MEDIUM   | 6-8 days  | Static Methods   | RAII destructors               |             |
| **7. Clone Trait**               | MEDIUM   | 2-3 days  | None             | Explicit copying               |             |
| **8. Eq/Ord Traits**             | MEDIUM   | 3-4 days  | None             | Generic comparison             |             |

**Total Estimated Time:** 30-44 days

---

## 🔥 PHASE 1: Foundation (Week 1-2)

### Feature 1: Static Methods & Constructor Pattern ⭐ CRITICAL

**Priority:** CRITICAL  
**Estimated Time:** 3-4 days  
**Dependencies:** None

#### Problem Statement

Vex'te şu anda static method syntax'ı yok. `Vec.new()`, `HashMap.new()` gibi constructor pattern'lar parse edilmiyor ve codegen desteklenmiyor.

**Current State:**

```vex
// ❌ NOT SUPPORTED - Parse error
let v = Vec.new();
let map = HashMap.new();

// ✅ WORKAROUND - Using global functions
let v = vec_new();  // Manual function call
```

**Desired State:**

```vex
// ✅ Static method syntax
let v = Vec.new();
let map = HashMap.new();

// ✅ Type-associated functions
struct Point {
    x: i32,
    y: i32,

    // Static method (no receiver)
    fn new(x: i32, y: i32): Point {
        return Point { x: x, y: y };
    }

    // Static method for origin
    fn origin(): Point {
        return Point { x: 0, y: 0 };
    }
}

// Usage
let p1 = Point.new(10, 20);
let p2 = Point.origin();
```

#### Implementation Steps

**Step 1: AST Extension** (0.5 days)

- File: `vex-ast/src/lib.rs`
- Add `is_static: bool` field to `Function` struct
- Add `receiver: Option<Receiver>` distinction:
  - `None` + `is_static=true` → Static method
  - `None` + `is_static=false` → Free function
  - `Some(Receiver)` → Instance method

**Step 2: Parser Support** (1 day)

- File: `vex-parser/src/parser/items/functions.rs`
- Parse static method syntax:
  ```vex
  struct S {
      fn static_method() { }     // is_static=true, receiver=None
      fn (self: &S) method() { } // is_static=false, receiver=Some
  }
  ```
- Detect static method: No receiver + inside struct body
- Validate: Static methods cannot use `self`

**Step 3: Lexer Token** (0.5 days)

- File: `vex-lexer/src/lib.rs`
- Add `Expression::StaticMethodCall`:
  ```rust
  StaticMethodCall {
      type_name: String,      // "Vec", "HashMap"
      method_name: String,    // "new", "with_capacity"
      type_args: Vec<Type>,   // <i32>, <String, i32>
      args: Vec<Expression>,
  }
  ```

**Step 4: Parser Expression** (1 day)

- File: `vex-parser/src/parser/expressions.rs`
- Parse `Type.method()` syntax:
  - Detect: Identifier + `.` + Identifier + `(` → Could be static call
  - Lookahead: Check if first identifier is a type name
  - Construct `StaticMethodCall` expression

**Step 5: Codegen** (1-1.5 days)

- File: `vex-compiler/src/codegen_ast/expressions/mod.rs`
- Compile static method calls:

  ```rust
  Expression::StaticMethodCall { type_name, method_name, type_args, args } => {
      // Mangle name: Vec_new_i32 (for Vec<i32>::new)
      let mangled = format!("{}_{}", type_name, method_name);

      // Add type args to mangling
      if !type_args.is_empty() {
          mangled += &mangle_type_args(type_args);
      }

      // Call as regular function
      self.compile_call(&mangled, args)
  }
  ```

**Step 6: Standard Library Integration** (0.5 days)

- Update `vex-libs/std/collections/src/lib.vx`:

  ```vex
  struct Vec<T> {
      data: *T,
      len: i64,
      cap: i64,

      // Static constructor
      fn new(): Vec<T> {
          let v: Vec<T>;
          extern "C" {
              fn vex_vec_new(): *u8;
          }
          // ... implementation
          return v;
      }
  }
  ```

**Testing:**

- Create `examples/test_static_methods.vx`:

  ```vex
  struct Counter {
      count: i32,

      fn new(): Counter {
          return Counter { count: 0 };
      }

      fn with_start(n: i32): Counter {
          return Counter { count: n };
      }
  }

  fn main(): i32 {
      let c1 = Counter.new();
      let c2 = Counter.with_start(10);
      return c1.count + c2.count;  // 0 + 10 = 10
  }
  ```

**Success Criteria:**

- ✅ Parse `Type.method()` syntax without errors ← **COMPLETE**
- ✅ Static methods compile to mangled function names ← **COMPLETE**
- ✅ Generic static methods work: `Vec<i32>.new()` ← **COMPLETE**
- ✅ All standard library types use `.new()` constructor ← **COMPLETE**

**Implementation Status:** ✅ **COMPLETE** (Nov 11, 2025)

- AST: Added `type_args: Vec<Type>` to `MethodCall`
- Parser: Generic type arguments parsed for static methods
- Codegen: Type mangling for `Vec<i32>.new()` → `vec_i32_new`
- Tests: 225/231 passing (97.4%), `test_static_methods_complete.vx` working
- ✅ All standard library types use `.new()` constructor

---

### Feature 2: Trait Bounds Enforcement ✅ COMPLETE

**Priority:** HIGH  
**Estimated Time:** 4-5 days  
**Status:** COMPLETE (Nov 11, 2025)

#### ✅ COMPLETED:

- [x] Trait bounds checker validates trait implementations at compile time
- [x] check_type_bounds() correctly identifies missing trait implementations
- [x] Multiple trait bounds work (`<T: Display + Clone>`)
- [x] Struct impl syntax supports multiple traits (`impl Display, Clone`)
- [x] Clear error messages when bounds not satisfied
- [x] Integration with generic function instantiation

#### Problem Statement

Trait bounds (`<T: Display>`) parse edilip AST'ye konuyor ama compile-time'da enforce edilmiyor. ✅ **FIXED**

**Current State:**

```vex
fn print_value<T: Display>(x: T) { }

struct Point { x: i32, y: i32 }  // Display implement ETMİYOR!

fn main(): i32 {
    let p = Point { x: 10, y: 20 };
    print_value(p);  // ❌ HATA VERMELİ ama vermiyor!
    return 0;
}
```

**Log Output:**

```
✅ Trait bounds validated for print_value::<Point>
```

→ Aslında validation YOK, sadece log var!

#### Implementation Steps

**Step 1: Enhance Trait Bounds Checker** (2 days)

- File: `vex-compiler/src/trait_bounds_checker.rs`
- Current state: Basic skeleton exists, no real checking
- Implement `check_type_bounds()` properly:

  ```rust
  fn check_type_bounds(&mut self, type_param: &TypeParam, concrete_type: &Type) -> Result<(), String> {
      for required_trait in &type_param.bounds {
          match required_trait {
              TraitBound::Simple(trait_name) => {
                  // CURRENT: Just logs success
                  // NEW: Actually check implementation

                  let type_name = self.extract_type_name(concrete_type);

                  if !self.type_implements_trait(&type_name, trait_name) {
                      return Err(format!(
                          "Type `{}` does not implement trait `{}`\n\
                           Required by type parameter `{}` with bound `{}`",
                          type_name, trait_name, type_param.name, trait_name
                      ));
                  }
              }
          }
      }
      Ok(())
  }
  ```

**Step 2: Collect Trait Implementations** (1 day)

- File: `vex-compiler/src/trait_bounds_checker.rs`
- Enhance `initialize()` method:

  ```rust
  pub fn initialize(&mut self, program: &Program) {
      // 1. Collect trait definitions
      for item in &program.items {
          if let Item::Trait(trait_def) = item {
              self.traits.insert(trait_def.name.clone(), trait_def.clone());
          }
      }

      // 2. Collect inline trait implementations
      for item in &program.items {
          if let Item::Struct(struct_def) = item {
              for trait_name in &struct_def.impl_traits {
                  self.type_impls
                      .entry(struct_def.name.clone())
                      .or_insert_with(Vec::new)
                      .push(trait_name.clone());
              }
          }
      }

      // 3. Validate trait methods are implemented
      for item in &program.items {
          if let Item::Struct(struct_def) = item {
              for trait_name in &struct_def.impl_traits {
                  if let Some(trait_def) = self.traits.get(trait_name) {
                      self.validate_trait_implementation(struct_def, trait_def)?;
                  }
              }
          }
      }
  }
  ```

**Step 3: Method Signature Validation** (1 day)

- File: `vex-compiler/src/trait_bounds_checker.rs`
- Add `validate_trait_implementation()`:

  ```rust
  fn validate_trait_implementation(&self, struct_def: &Struct, trait_def: &Trait) -> Result<(), String> {
      for trait_method in &trait_def.methods {
          // Required methods (no default body) MUST be implemented
          if trait_method.body.is_none() {
              let found = struct_def.methods.iter().any(|m| {
                  m.name == trait_method.name &&
                  self.signatures_match(m, trait_method)
              });

              if !found {
                  return Err(format!(
                      "Struct `{}` does not implement required method `{}` from trait `{}`",
                      struct_def.name, trait_method.name, trait_def.name
                  ));
              }
          }
      }
      Ok(())
  }
  ```

**Step 4: Integration with Codegen** (0.5 days)

- File: `vex-compiler/src/codegen_ast/mod.rs`
- Call trait bounds checker on every generic function instantiation:

  ```rust
  pub fn compile_generic_function_instance(&mut self, ...) -> Result<(), String> {
      // Check trait bounds BEFORE monomorphization
      self.trait_bounds_checker.check_function_bounds(func, type_args)?;

      // Then proceed with monomorphization
      ...
  }
  ```

**Step 5: Multiple Trait Bounds** (0.5 days)

- File: `vex-compiler/src/trait_bounds_checker.rs`
- Support `<T: Display + Clone>` syntax:
  ```rust
  fn check_type_bounds(&mut self, type_param: &TypeParam, concrete_type: &Type) -> Result<(), String> {
      // type_param.bounds is Vec<TraitBound>, iterate all
      for bound in &type_param.bounds {
          // Check each bound independently
          self.check_single_bound(bound, concrete_type)?;
      }
      Ok(())
  }
  ```

**Step 6: Where Clause Support** (0.5 days)

- File: `vex-compiler/src/trait_bounds_checker.rs`
- Support `where T: Display, U: Clone` syntax:

  ```rust
  pub fn check_where_clause(&mut self, where_clause: &[WhereClausePredicate], type_args: &[Type]) -> Result<(), String> {
      for predicate in where_clause {
          // Find concrete type for this type parameter
          let concrete_type = find_type_arg(&predicate.type_param, type_args)?;

          // Check all bounds
          for bound in &predicate.bounds {
              self.check_single_bound(bound, &concrete_type)?;
          }
      }
      Ok(())
  }
  ```

**Testing:**

- Create `examples/test_trait_bounds_enforcement.vx`:

  ```vex
  trait Display {
      fn to_string(): string;
  }

  struct Point impl Display {
      x: i32,
      y: i32,
      fn to_string(): string { return "Point"; }
  }

  struct Vector {  // NO Display!
      x: f64,
      y: f64,
  }

  fn print<T: Display>(value: T) {
      // Should compile only if T implements Display
  }

  fn main(): i32 {
      let p = Point { x: 1, y: 2 };
      print(p);  // ✅ OK - Point implements Display

      let v = Vector { x: 1.0, y: 2.0 };
      // print(v);  // ❌ COMPILE ERROR - Vector doesn't implement Display

      return 0;
  }
  ```

**Success Criteria:**

- ✅ Compile error when trait bound not satisfied
- ✅ Multiple trait bounds work: `<T: A + B>`
- ✅ Where clauses validated: `where T: Display`
- ✅ Clear error messages with trait and type names

---

### Feature 3: Display Trait ⭐ HIGH

**Priority:** HIGH  
**Estimated Time:** 2-3 days  
**Dependencies:** None (but benefits from Trait Bounds Enforcement)

#### Problem Statement

Her struct kendi debug/print formatını manuel implement ediyor. Standart bir `Display` trait'i yok.

**Current State:**

```vex
struct Point {
    x: i32,
    y: i32,

    // Everyone implements their own version
    fn to_string(): string {
        return "Point";
    }
}
```

**Desired State:**

```vex
trait Display {
    fn to_string(): string;
}

struct Point impl Display {
    x: i32,
    y: i32,

    fn to_string(): string {
        return "Point(" + i32_to_string(self.x) + ", " + i32_to_string(self.y) + ")";
    }
}

// Generic print function
fn debug<T: Display>(value: T) {
    print(value.to_string());
}
```

#### Implementation Steps

**Step 1: Define Display Trait** (0.5 days)

- File: `vex-libs/std/traits/src/lib.vx` (new file)
- Create standard traits module:

  ```vex
  // std/traits/src/lib.vx

  // Display trait - for user-facing string formatting
  export trait Display {
      fn to_string(): string;
  }

  // Debug trait - for debugging output (more verbose)
  export trait Debug {
      fn debug_string(): string;
  }
  ```

**Step 2: Builtin Type Implementations** (1 day)

- File: `vex-compiler/src/codegen_ast/builtin_types/primitives.rs` (new)
- Auto-implement Display for primitive types:

  ```rust
  // Compiler provides built-in Display implementations
  // i32, i64, f32, f64, bool, string → all have to_string()

  impl Display for i32 {
      fn to_string(): string {
          extern "C" { fn i32_to_string(i32): string; }
          return i32_to_string(self);
      }
  }

  impl Display for string {
      fn to_string(): string {
          return self;  // Already a string
      }
  }
  ```

**Step 3: Runtime Functions** (0.5 days)

- File: `vex-runtime/c/vex_display.c` (new)
- Implement conversion functions:

  ```c
  VexString* i32_to_string(int32_t value) {
      char buffer[32];
      snprintf(buffer, sizeof(buffer), "%d", value);
      return vex_string_from_cstr(buffer);
  }

  VexString* f64_to_string(double value) {
      char buffer[64];
      snprintf(buffer, sizeof(buffer), "%g", value);
      return vex_string_from_cstr(buffer);
  }

  VexString* bool_to_string(bool value) {
      return vex_string_from_cstr(value ? "true" : "false");
  }
  ```

**Step 4: Formatter Support** (0.5 days)

- File: `vex-formatter/src/visitor.rs`
- Format trait implementations:

  ```rust
  fn visit_struct(&mut self, struct_def: &Struct) {
      writeln!(self.output, "struct {} impl {} {{",
          struct_def.name,
          struct_def.impl_traits.join(", ")
      );

      // ... format methods including Display
  }
  ```

**Step 5: Documentation** (0.5 days)

- Update `Specifications/09_Traits.md`:
  - Add Display trait as standard trait
  - Document to_string() convention
  - Show primitive type implementations

**Testing:**

- Create `examples/test_display_trait.vx`:

  ```vex
  import { Display } from "std/traits";

  struct Point impl Display {
      x: i32,
      y: i32,

      fn to_string(): string {
          return "Point(" + i32_to_string(self.x) + ", " + i32_to_string(self.y) + ")";
      }
  }

  fn debug<T: Display>(value: T) {
      print(value.to_string());
  }

  fn main(): i32 {
      let p = Point { x: 10, y: 20 };
      debug(p);  // Output: "Point(10, 20)"

      debug(42);      // Output: "42"
      debug(3.14);    // Output: "3.14"
      debug(true);    // Output: "true"
      debug("hello"); // Output: "hello"

      return 0;
  }
  ```

**Success Criteria:**

- ✅ Display trait defined in std/traits
- ✅ All primitive types implement Display
- ✅ Custom structs can implement Display
- ✅ Generic functions use Display bound

---

## 🚀 PHASE 2: Advanced Traits (Week 3-4)

### Feature 4: Associated Types Codegen ✅ PARSER COMPLETE

**Priority:** HIGH  
**Estimated Time:** 5-6 days  
**Status:** Parser 100% complete, codegen pending (20%)

#### ✅ COMPLETED (Nov 11, 2025):

- [x] Type::SelfType and Type::AssociatedType AST variants (vex-ast/src/lib.rs)
- [x] Parser support for `Self.Item` syntax (vex-parser/src/parser/types.rs)
  - **Note:** Vex uses `.` not `::` (consistent with method call syntax)
- [x] Pattern match updates in borrow_checker and trait_bounds_checker
- [x] Basic ast_type_to_llvm() handling (returns opaque pointer)
- [x] Parser test with Iterator trait example (examples/test_associated_types_parser.vx)
- [x] **Comprehensive edge case testing - 8 test files created:**
  - ✅ Multiple associated types (`Self.Key`, `Self.Value`)
  - ✅ Generic containers (`Vec<Self.Item>`, `Result<Self.Output, Self.Error>`)
  - ✅ Function signatures with associated types
  - ✅ Complex nested generics
  - ✅ Self keyword standalone usage
  - ❌ Invalid `Self::Item` syntax (correctly rejected)
  - ❌ Incomplete `Self.` syntax (correctly rejected)

#### ⏳ PENDING:

- [ ] Type resolution: resolve_associated_type() implementation
- [ ] Trait impl context tracking for Self.Item binding
- [ ] Integration with Iterator trait
- [ ] Full Iterator trait implementation with associated types
- [ ] Generic Iterator<Item=T> support
- [ ] Range type integration

#### Problem Statement

Associated types (`type Item;`) parse ediliyor ama codegen'de `Self.Item` kullanımı çalışmıyor.

**Current State:**

```vex
trait Iterator {
    type Item;  // ✅ Parses

    fn next(): Option<Self.Item> {  // ⚠️ Self.Item parses but doesn't resolve yet
        // Compiler returns opaque pointer, needs resolution
    }
}
```

**Desired State:**

```vex
trait Iterator {
    type Item;

    fn next(): Option<Self.Item> {  // ✅ Works
        // Self.Item resolves to concrete type
    }
}

struct Counter impl Iterator {
    type Item = i32;  // Bind associated type
    current: i32,

    fn next(): Option<i32> {  // Self.Item → i32
        let val = self.current;
        self.current = self.current + 1;
        return Some(val);
    }
}
```

#### Implementation Steps

**Step 1: AST Enhancement** ✅ COMPLETE (0.5 days)

- File: `vex-ast/src/lib.rs`
- Already has `associated_types: Vec<String>` in `Trait`
- Already has `associated_type_bindings: Vec<(String, Type)>` in `Struct`
- ✅ Added `Type::SelfType` and `Type::AssociatedType`:

  ```rust
  pub enum Type {
      // ... existing variants

      /// Self keyword (used in trait/impl contexts)
      SelfType,

      /// Associated type reference: Self.Item, Self.Output
      /// NOTE: Vex uses . not :: (consistent with method calls)
      AssociatedType {
          self_type: Box<Type>,  // Self or concrete type
          name: String,          // "Item", "Output"
      },
  }
  ```

**Step 2: Parser Support** ✅ COMPLETE (1 day)

- File: `vex-parser/src/parser/types.rs` (lines 236-252)
- ✅ Parse `Self.Item` syntax (NOTE: uses `.` not `::`):

  ```rust
  // Check for Self keyword
  if let Token::Identifier(ref id) = self.current_token() {
      if id == "Self" {
          self.advance(); // consume 'Self'

          // Check for Self.Item (associated type)
          if self.current_token() == &Token::Dot {
              self.advance(); // consume '.'

              let name = self.expect_identifier()?;

              return Ok(Type::AssociatedType {
                  self_type: Box::new(Type::SelfType),
                  name,
              });
          }

          return Ok(Type::SelfType);
      }
  }
  ```

**Step 3: Type Resolution** 🔄 IN PROGRESS (2 days)

- File: `vex-compiler/src/codegen_ast/types.rs`
- Add `resolve_associated_type()` method:

  ```rust
  impl<'ctx> ASTCodeGen<'ctx> {
      fn resolve_associated_type(&self, self_type: &Type, assoc_name: &str) -> Result<Type, String> {
          // 1. Get concrete type name
          let type_name = match self_type {
              Type::SelfType => {
                  // In method context, Self = current struct
                  self.current_struct_name.as_ref()
                      .ok_or("Self outside of struct context")?
              }
              Type::Named(name) => name,
              _ => return Err(format!("Invalid self type for associated type: {:?}", self_type)),
          };

          // 2. Look up associated type binding in struct
          if let Some(struct_def) = self.program.get_struct(type_name) {
              for (name, bound_type) in &struct_def.associated_type_bindings {
                  if name == assoc_name {
                      return Ok(bound_type.clone());
                  }
              }
          }

          Err(format!("Associated type `{}.{}` not found", type_name, assoc_name))
      }
  }
  ```

**Step 4: Codegen Integration** ⏳ PENDING (1.5 days)

- File: `vex-compiler/src/codegen_ast/types.rs` (lines 395-410)
- ⚠️ Currently returns opaque pointer, needs resolution:
  ```rust
  Type::AssociatedType { self_type, name } => {
      // TODO: Resolve to concrete type
      eprintln!(
          "⚠️  Associated type {}.{} encountered, needs resolution",
          self.type_to_string(self_type),
          name
      );
      // Currently: opaque pointer
      // Future: Resolve and convert
      let concrete = self.resolve_associated_type(self_type, name)?;
      self.ast_type_to_llvm(&concrete)
  }
  ```

**Step 5: Method Signature Substitution** ⏳ PENDING (1 day)

- File: `vex-compiler/src/codegen_ast/items/traits.rs`
- Substitute associated types in method signatures:

  ```rust
  fn compile_trait_method_with_substitution(
      &mut self,
      trait_method: &TraitMethod,
      assoc_type_bindings: &[(String, Type)],
  ) -> Result<(), String> {
      // Clone method signature
      let mut method = trait_method.clone();

      // Substitute all Self::Item → i32 (or bound type)
      for param in &mut method.params {
          param.ty = self.substitute_associated_types(&param.ty, assoc_type_bindings)?;
      }

      if let Some(ref mut ret_ty) = method.return_type {
          *ret_ty = self.substitute_associated_types(ret_ty, assoc_type_bindings)?;
      }

      // Now compile with substituted types
      self.compile_method(&method)
  }
  ```

**Testing:**

- Create `examples/test_associated_types.vx`:

  ```vex
  trait Container {
      type Item;

      fn get(): Option<Self::Item>;
      fn add(item: Self::Item);
  }

  struct IntBox impl Container {
      type Item = i32;
      value: i32,

      fn get(): Option<i32> {
          return Some(self.value);
      }

      fn add(item: i32) {
          self.value = item;
      }
  }

  fn main(): i32 {
      let! box = IntBox { value: 42 };
      let result = box.get();  // Option<i32>

      match result {
          Some(v) => return v,
          None => return -1,
      }
  }
  ```

**Success Criteria:**

- ✅ `Self.Item` syntax parses correctly (COMPLETE)
- ✅ Parser rejects invalid syntax (`Self::Item`, `Self.`) (COMPLETE)
- ✅ Edge cases tested: multiple associated types, generics, nesting (COMPLETE)
- ⏳ Associated type bindings resolve to concrete types (PENDING)
- ⏳ Method signatures use resolved types (PENDING)
- ⏳ Generic traits with associated types work end-to-end (PENDING)

**Test Files Created:**

- `examples/test_associated_types_parser.vx` - Basic Iterator test
- `examples/test_associated_types_edge_cases.vx` - Comprehensive edge cases
- `examples/test_associated_types_invalid1.vx` - Invalid `::` syntax
- `examples/test_associated_types_invalid2.vx` - Incomplete `Self.`
- `examples/test_self_keyword.vx` - Self keyword standalone
- `examples/test_associated_types_summary.vx` - Test summary
- `examples/ASSOCIATED_TYPES_TEST_RESULTS.md` - Detailed test results

---

### Feature 5: Iterator Trait ⭐ CRITICAL

**Priority:** CRITICAL  
**Estimated Time:** 5-7 days  
**Dependencies:** Associated Types Codegen

#### Problem Statement

For-in loop sadece Range/RangeInclusive için hardcoded. Vec, Map, Set, Array için iteration yok.

**Current State:**

```vex
for i in 0..10 {  // ✅ Works - hardcoded Range support
    print(i);
}

let v = vec(1, 2, 3);
// for x in v {  // ❌ PARSE ERROR - Vec iteration not supported
//     print(x);
// }
```

**Desired State:**

```vex
trait Iterator {
    type Item;
    fn next(): Option<Self::Item>;
}

// Vec implements Iterator
struct Vec<T> impl Iterator {
    type Item = T;

    fn iter(): VecIterator<T> {
        return VecIterator { vec: self, index: 0 };
    }
}

struct VecIterator<T> impl Iterator {
    type Item = T;
    vec: &Vec<T>,
    index: i64,

    fn next(): Option<T> {
        if self.index < self.vec.len {
            let value = self.vec.data[self.index];
            self.index = self.index + 1;
            return Some(value);
        }
        return None;
    }
}

// For-in loop desugaring
for x in collection {
    // Desugars to:
    let! iter = collection.iter();
    while true {
        match iter.next() {
            Some(x) => { /* body */ },
            None => break,
        }
    }
}
```

#### Implementation Steps

**Step 1: Define Iterator Trait** (0.5 days)

- File: `vex-libs/std/traits/src/lib.vx`
- Add Iterator trait:

  ```vex
  export trait Iterator {
      type Item;
      fn next(): Option<Self::Item>;
  }

  export trait IntoIterator {
      type Item;
      type IntoIter: Iterator;  // Requires trait bounds!

      fn into_iter(): Self::IntoIter;
  }
  ```

**Step 2: Vec Iterator Implementation** (1.5 days)

- File: `vex-libs/std/collections/src/vec.vx`
- Implement iterator for Vec:

  ```vex
  struct VecIterator<T> impl Iterator {
      type Item = T;

      vec: &Vec<T>,
      index: i64,

      fn next(): Option<T> {
          if self.index < self.vec.len {
              let value = self.vec.get(self.index);
              self.index = self.index + 1;
              return Some(value);
          }
          return None;
      }
  }

  struct Vec<T> impl IntoIterator {
      type Item = T;
      type IntoIter = VecIterator<T>;

      fn into_iter(): VecIterator<T> {
          return VecIterator { vec: &self, index: 0 };
      }
  }
  ```

**Step 3: Parser Enhancement** (1 day)

- File: `vex-parser/src/parser/statements/loops.rs`
- Already parses `for x in expr { }` syntax ✅
- Add type checking: `expr` must implement `IntoIterator`

**Step 4: Codegen Desugaring** (2-3 days)

- File: `vex-compiler/src/codegen_ast/statements/loops.rs`
- Rewrite `compile_for_in_loop()`:

  ```rust
  pub(crate) fn compile_for_in_loop(
      &mut self,
      variable: &str,
      iterable: &Expression,
      body: &Block,
  ) -> Result<(), String> {
      // OLD: Hardcoded for Range only
      // NEW: Generic implementation

      // 1. Call iterable.into_iter() → iterator
      let iter_expr = Expression::MethodCall {
          receiver: Box::new(iterable.clone()),
          method: "into_iter".to_string(),
          type_args: vec![],
          args: vec![],
      };

      // 2. Allocate mutable iterator variable
      let iter_var = format!("__iter_{}", self.unique_id());
      self.compile_let_statement(&iter_var, Some(iter_type), iter_expr, true)?;

      // 3. Create loop: while true { match iter.next() { ... } }
      let loop_cond = self.context.append_basic_block(fn_val, "for.cond");
      let loop_body = self.context.append_basic_block(fn_val, "for.body");
      let loop_end = self.context.append_basic_block(fn_val, "for.end");

      // 4. Condition: call iter.next()
      self.builder.position_at_end(loop_cond);
      let next_call = Expression::MethodCall {
          receiver: Box::new(Expression::Variable(iter_var.clone())),
          method: "next".to_string(),
          type_args: vec![],
          args: vec![],
      };
      let option_value = self.compile_expression(&next_call)?;

      // 5. Match on Option<T>
      // match option_value {
      //     Some(x) => { body },
      //     None => break,
      // }

      // Implementation using LLVM phi nodes or runtime option_is_some()

      Ok(())
  }
  ```

**Step 5: Runtime Support** (1 day)

- File: `vex-runtime/c/vex_iterator.c` (new)
- Helper functions for Option unwrapping:

  ```c
  // Check if Option<T> is Some
  bool vex_option_is_some(void* option_ptr) {
      // Option<T> layout: { bool is_some, T value }
      return *(bool*)option_ptr;
  }

  // Extract value from Some(T)
  void* vex_option_unwrap(void* option_ptr, size_t value_size) {
      bool is_some = *(bool*)option_ptr;
      if (!is_some) {
          vex_panic("Unwrap on None");
      }
      return (char*)option_ptr + sizeof(bool);
  }
  ```

**Step 6: Standard Library Integration** (0.5 days)

- Implement Iterator for:
  - `Vec<T>` → `VecIterator<T>`
  - `Range` → Already has `next()`, just add trait impl
  - `RangeInclusive` → Same
  - `Array<T, N>` → `ArrayIterator<T>`

**Testing:**

- Create `examples/test_iterator_trait.vx`:

  ```vex
  import { Iterator, IntoIterator } from "std/traits";

  fn main(): i32 {
      // Test 1: Range (already works)
      let! sum = 0;
      for i in 0..5 {
          sum = sum + i;
      }
      // sum = 0 + 1 + 2 + 3 + 4 = 10

      // Test 2: Vec (NEW!)
      let v = vec(10, 20, 30);
      let! total = 0;
      for x in v {
          total = total + x;
      }
      // total = 10 + 20 + 30 = 60

      return sum + total;  // 10 + 60 = 70
  }
  ```

**Success Criteria:**

- ✅ Iterator trait defined with associated type
- ✅ Vec<T> implements Iterator
- ✅ For-in loop desugars to while + match
- ✅ All collections support iteration

---

## 🎨 PHASE 3: Nice-to-Have Features (Week 5-6)

### Feature 6: Drop Trait (Auto-cleanup) 🔧 MEDIUM

**Priority:** MEDIUM  
**Estimated Time:** 6-8 days  
**Dependencies:** Static Methods

#### Problem Statement

Defer statement manuel cleanup sağlıyor ama RAII pattern (automatic cleanup) yok. Her scope exit'te manuel `defer` yazmak gerekiyor.

**Current State:**

```vex
fn process_file(path: string): i32 {
    let file = open_file(path);
    defer close_file(file);  // ✅ Manual cleanup

    // Use file...

    // defer runs here automatically
    return 0;
}
```

**Desired State:**

```vex
trait Drop {
    fn drop();  // Called automatically on scope exit
}

struct File impl Drop {
    fd: i32,

    fn drop() {
        // Automatic cleanup - no defer needed!
        close_file(self.fd);
    }
}

fn process_file(path: string): i32 {
    let file = File.open(path);  // No defer needed!

    // Use file...

    // file.drop() called automatically here
    return 0;
}
```

#### Implementation Steps

**Step 1: Define Drop Trait** (0.5 days)

- File: `vex-libs/std/traits/src/lib.vx`
- Add Drop trait:
  ```vex
  export trait Drop {
      fn drop();  // No parameters, operates on self
  }
  ```

**Step 2: AST Tracking** (1 day)

- File: `vex-compiler/src/codegen_ast/mod.rs`
- Track variables that need cleanup:

  ```rust
  pub struct ASTCodeGen<'ctx> {
      // ... existing fields

      /// Variables that implement Drop, need cleanup on scope exit
      drop_variables: Vec<String>,

      /// Scope depth for tracking variable lifetimes
      scope_depth: usize,
  }
  ```

**Step 3: Variable Allocation Tracking** (1.5 days)

- File: `vex-compiler/src/codegen_ast/statements/let_statement.rs`
- When allocating variable, check if type implements Drop:

  ```rust
  fn compile_let_statement(&mut self, name: &str, ty: Option<&Type>, value: &Expression, is_mut: bool) -> Result<(), String> {
      // ... allocate variable

      // Check if type implements Drop trait
      let type_name = self.extract_type_name_from_expr(value)?;
      if self.type_implements_drop(&type_name) {
          // Register for automatic cleanup
          self.drop_variables.push(name.to_string());
      }

      Ok(())
  }
  ```

**Step 4: Scope Exit Cleanup** (2-3 days)

- File: `vex-compiler/src/codegen_ast/statements/mod.rs`
- Insert cleanup calls at scope exits:

  ```rust
  fn compile_block(&mut self, block: &Block) -> Result<(), String> {
      self.scope_depth += 1;
      let scope_start = self.drop_variables.len();

      for stmt in &block.statements {
          self.compile_statement(stmt)?;
      }

      // Cleanup: call drop() on all variables from this scope
      let scope_vars = self.drop_variables.split_off(scope_start);
      for var in scope_vars.iter().rev() {  // LIFO order
          self.compile_drop_call(var)?;
      }

      self.scope_depth -= 1;
      Ok(())
  }
  ```

**Step 5: Return Statement Cleanup** (1 day)

- File: `vex-compiler/src/codegen_ast/statements/mod.rs`
- Insert cleanup before return:

  ```rust
  fn compile_return(&mut self, value: Option<&Expression>) -> Result<(), String> {
      // Compile return value first
      let ret_val = if let Some(expr) = value {
          Some(self.compile_expression(expr)?)
      } else {
          None
      };

      // Call drop() on all variables in current scope (LIFO)
      for var in self.drop_variables.iter().rev() {
          self.compile_drop_call(var)?;
      }

      // Now return
      if let Some(val) = ret_val {
          self.builder.build_return(Some(&val));
      } else {
          self.builder.build_return(None);
      }

      Ok(())
  }
  ```

**Step 6: Break/Continue Cleanup** (1 day)

- File: `vex-compiler/src/codegen_ast/statements/loops.rs`
- Insert cleanup before break/continue:

  ```rust
  fn compile_break(&mut self) -> Result<(), String> {
      // Get loop end block
      let (_, loop_end) = self.loop_context_stack.last()
          .ok_or("Break outside loop")?;

      // Cleanup variables in loop scope
      for var in self.drop_variables.iter().rev() {
          self.compile_drop_call(var)?;
      }

      // Branch to loop end
      self.builder.build_unconditional_branch(*loop_end)?;
      Ok(())
  }
  ```

**Step 7: Standard Library Integration** (0.5 days)

- Implement Drop for resource types:

  ```vex
  // File
  struct File impl Drop {
      fd: i32,
      fn drop() {
          extern "C" { fn close(fd: i32); }
          close(self.fd);
      }
  }

  // Vec
  struct Vec<T> impl Drop {
      data: *T,
      len: i64,
      cap: i64,

      fn drop() {
          extern "C" { fn vex_vec_free(vec: *u8); }
          vex_vec_free(self.data);
      }
  }
  ```

**Testing:**

- Create `examples/test_drop_trait.vx`:

  ```vex
  struct Resource impl Drop {
      id: i32,

      fn drop() {
          print("Dropping resource ");
          print_i32(self.id);
      }
  }

  fn test_scope() {
      let r1 = Resource { id: 1 };
      {
          let r2 = Resource { id: 2 };
          // r2.drop() called here
      }
      // r1.drop() called here
  }

  fn test_early_return(): i32 {
      let r = Resource { id: 3 };
      if true {
          return 42;  // r.drop() called before return
      }
      return 0;
  }

  fn main(): i32 {
      test_scope();
      return test_early_return();
  }

  // Expected output:
  // Dropping resource 2
  // Dropping resource 1
  // Dropping resource 3
  ```

**Success Criteria:**

- ✅ Drop trait defined in std/traits
- ✅ Variables implementing Drop cleaned up automatically
- ✅ LIFO order maintained (reverse allocation order)
- ✅ Cleanup on return, break, continue, scope exit
- ✅ Compatible with existing defer statement

---

### Feature 7: Clone Trait 🔧 MEDIUM

**Priority:** MEDIUM  
**Estimated Time:** 2-3 days  
**Dependencies:** None

#### Problem Statement

Explicit kopyalama için standart bir yöntem yok. Her struct kendi `clone()` method'unu implement ediyor.

**Desired State:**

```vex
trait Clone {
    fn clone(): Self;
}

struct Point impl Clone {
    x: i32,
    y: i32,

    fn clone(): Point {
        return Point { x: self.x, y: self.y };
    }
}

fn duplicate<T: Clone>(value: T): (T, T) {
    return (value, value.clone());
}
```

#### Implementation Steps

**Step 1: Define Clone Trait** (0.5 days)

- File: `vex-libs/std/traits/src/lib.vx`

**Step 2: Derive Clone Macro (Future)** (1.5 days)

- Auto-implement Clone for simple structs
- `#[derive(Clone)]` attribute support

**Step 3: Standard Library** (0.5 days)

- Implement Clone for Box, Vec, Option, Result

**Step 4: Testing** (0.5 days)

- Create `examples/test_clone_trait.vx`

---

### Feature 8: Eq/Ord Traits 🔧 MEDIUM

**Priority:** MEDIUM  
**Estimated Time:** 3-4 days  
**Dependencies:** None

#### Problem Statement

Generic karşılaştırma fonksiyonları yazılamıyor. Her tip için manuel == ve < operatörleri var.

**Desired State:**

```vex
trait Eq {
    fn equals(other: &Self): bool;
}

trait Ord {
    fn compare(other: &Self): i32;  // -1, 0, 1
}

fn max<T: Ord>(a: T, b: T): T {
    if a.compare(&b) > 0 {
        return a;
    }
    return b;
}
```

#### Implementation Steps

**Step 1: Define Eq/Ord Traits** (0.5 days)
**Step 2: Primitive Implementations** (1.5 days)
**Step 3: Generic Sort Function** (1 day)
**Step 4: Testing** (0.5 days)

---

## 📅 Implementation Timeline

### Week 1: Foundation

- **Day 1-2:** Static Methods (Parser, Codegen)
- **Day 3-4:** Static Methods (Testing, Stdlib integration)
- **Day 5:** Trait Bounds Enforcement (Part 1)

### Week 2: Trait System

- **Day 1-2:** Trait Bounds Enforcement (Part 2, Testing)
- **Day 3-4:** Display Trait (Definition, Primitives, Testing)
- **Day 5:** Associated Types (AST, Parser)

### Week 3: Advanced Traits

- **Day 1-3:** Associated Types (Type Resolution, Codegen)
- **Day 4-5:** Iterator Trait (Part 1: Definition, Vec impl)

### Week 4: Iterator & Drop

- **Day 1-2:** Iterator Trait (Part 2: Codegen desugaring)
- **Day 3-4:** Iterator Trait (Part 3: Testing, Stdlib)
- **Day 5:** Drop Trait (Part 1: AST tracking)

### Week 5: Drop & Clone

- **Day 1-3:** Drop Trait (Part 2: Scope cleanup, Testing)
- **Day 4-5:** Clone Trait (Definition, Implementations)

### Week 6: Eq/Ord & Polish

- **Day 1-2:** Eq/Ord Traits
- **Day 3-4:** Integration testing, Bug fixes
- **Day 5:** Documentation updates

---

## ✅ Success Metrics

### Code Quality

- [ ] All new features have >= 95% test coverage
- [ ] No regressions in existing 289 passing tests
- [ ] LLVM IR optimizations verified (no performance loss)

### Documentation

- [ ] All features documented in Specifications/
- [ ] REFERENCE.md updated with new syntax
- [ ] Examples created for each feature

### Performance

- [ ] Static methods: Zero overhead (same as free functions)
- [ ] Iterator trait: Zero-cost abstraction (LLVM inlines)
- [ ] Drop trait: Minimal overhead (single call per variable)

---

## 🔗 Related Documents

- `INCOMPLETE_FEATURES_AUDIT.md` - Feature gap analysis
- `docs/REFERENCE.md` - Language reference
- `Specifications/09_Traits.md` - Trait system spec
- `TODO.md` - Current development priorities
- `CORE_FEATURES_STATUS.md` - Feature implementation status

---

**Maintained by:** Vex Language Team  
**Last Updated:** November 10, 2025  
**Next Review:** After Phase 1 completion
