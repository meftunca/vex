# Policy System Implementation TODO

**Date:** November 8, 2025  
**Version:** Vex v0.2.0  
**Status:** 🚧 In Progress

---

## 📋 Implementation Order (Revised)

Based on discussion, implementation order optimized for dependencies:

1. **Policy Declarations (Phase 2)** → Core system foundation
2. **Anonymous Embedding** → Enables composition patterns
3. **Inline Tags (Phase 1)** → Built on policy infrastructure
4. **Trait Fields** → Complete struct composition

---

## 🎯 Sprint 1: Policy Declarations Foundation (3-4 days)

### Parser Changes

- [ ] **Parse policy declarations**

  ```vex
  policy APIModel {
      id      `json:"id" db:"user_id"`,
      name    `json:"name"`,
  }
  ```

  - [ ] Add `policy` keyword to lexer
  - [ ] Parse policy name
  - [ ] Parse policy body (field_name + backtick metadata)
  - [ ] Support multiple fields
  - [ ] Error handling for malformed policies

- [ ] **Parse backtick metadata strings**

  ```vex
  `json:"id" db:"user_id" validate:"required"`
  ```

  - [ ] Tokenize backtick strings (preserve content)
  - [ ] Store as raw string in AST (parsing in type checker)

- [ ] **Parse `with` keyword on structs**
  ```vex
  struct User with APIModel, ValidationRules {
      id: i32,
  }
  ```
  - [ ] Add `with` keyword to lexer
  - [ ] Parse comma-separated policy list
  - [ ] Store policy names in struct definition

**Files to modify:**

- `vex-lexer/src/lib.rs` - Add `policy`, `with` keywords
- `vex-parser/src/parser/items/mod.rs` - Add policy parsing
- `vex-parser/src/parser/items/policies.rs` (NEW) - Policy parsing logic
- `vex-parser/src/parser/items/structs.rs` - Add `with` clause parsing

---

### AST Changes

- [ ] **Add Policy item type**

  ```rust
  pub struct Policy {
      pub name: String,
      pub span: Span,
      pub fields: Vec<PolicyField>,
      pub parent_policies: Vec<String>,  // For composition later
  }

  pub struct PolicyField {
      pub name: String,
      pub metadata: String,  // Raw backtick content
      pub span: Span,
  }
  ```

- [ ] **Add Item::Policy variant**

  ```rust
  pub enum Item {
      Function(Function),
      Struct(StructDef),
      Enum(EnumDef),
      Trait(TraitDef),
      Policy(Policy),  // NEW
      // ...
  }
  ```

- [ ] **Update StructDef**

  ```rust
  pub struct StructDef {
      pub name: String,
      pub fields: Vec<FieldDef>,
      pub policies: Vec<String>,  // NEW: Policy names
      pub traits: Vec<String>,
      // ...
  }
  ```

- [ ] **Update FieldDef**
  ```rust
  pub struct FieldDef {
      pub name: String,
      pub ty: Type,
      pub metadata: Option<HashMap<String, String>>,  // NEW: Parsed metadata
      pub span: Span,
  }
  ```

**Files to modify:**

- `vex-ast/src/lib.rs` - Add Policy, PolicyField structs, update Item enum, update StructDef, update FieldDef

---

### Type Checker / Registry

- [ ] **Policy Registry**

  ```rust
  // In registry.rs
  pub struct TypeRegistry {
      // ... existing fields
      pub policies: HashMap<String, Policy>,  // NEW: policy_name -> Policy
  }
  ```

- [ ] **Register policies**

  - [ ] Iterate through `Item::Policy` in program
  - [ ] Store in `policies` HashMap
  - [ ] Detect duplicate policy names (compile error)

- [ ] **Policy name conflict detection**

  - [ ] Check if policy name exists as trait name
  - [ ] **Hard error**: `"Policy 'X' conflicts with trait 'X'"`
  - [ ] Check if trait name exists as policy name
  - [ ] **Hard error**: `"Trait 'X' conflicts with policy 'X'"`

- [ ] **Parse metadata strings**

  ```rust
  fn parse_metadata(raw: &str) -> HashMap<String, String> {
      // Input: `json:"id" db:"user_id" validate:"required"`
      // Output: {"json": "id", "db": "user_id", "validate": "required"}
  }
  ```

  - [ ] Split by whitespace (respecting quotes)
  - [ ] Parse `key:"value"` pairs
  - [ ] Handle escaped quotes
  - [ ] Error on malformed metadata
  - [ ] Store in FieldDef.metadata

- [ ] **Apply policies to structs**

  ```rust
  fn apply_policies_to_struct(struct_def: &StructDef, policies: &HashMap<String, Policy>) {
      // For each policy in struct.policies:
      //   1. Lookup policy in registry
      //   2. For each policy.field:
      //      a. Find matching struct field by name
      //      b. Merge policy metadata into field.metadata
      //   3. Handle field not found (warning: policy field not in struct)
  }
  ```

  - [ ] Lookup policy by name (error if not found)
  - [ ] Map policy fields to struct fields (by name)
  - [ ] Merge metadata (see merge rules below)
  - [ ] **Warning** if policy field not in struct
  - [ ] **Warning** if struct field not in any policy (OK, just no metadata)

- [ ] **Metadata merging rules**

  ```rust
  fn merge_metadata(
      existing: &HashMap<String, String>,
      new: &HashMap<String, String>
  ) -> HashMap<String, String> {
      // Merge strategy:
      // - Different keys → Combine
      // - Same key, different values → New value wins + WARNING
      // - Same key, same value → Keep one
  }
  ```

  - [ ] Merge different keys (union)
  - [ ] Same key, different values: **New value wins** + **WARNING**
  - [ ] Same key, same value: Keep one (no duplicate)

- [ ] **Multiple policies merge**
  ```vex
  struct User with PolicyA, PolicyB {
      id: i32,
  }
  ```
  - [ ] Apply policies left-to-right
  - [ ] PolicyA applied first, then PolicyB
  - [ ] PolicyB can override PolicyA (with warning)

**Files to modify:**

- `vex-compiler/src/codegen_ast/registry.rs` - Add policies HashMap, register_policy(), policy_trait_conflict_check()
- `vex-compiler/src/codegen_ast/mod.rs` - Call apply_policies_to_struct() after struct registration
- `vex-compiler/src/codegen_ast/metadata.rs` (NEW) - Metadata parsing and merging logic

---

### Codegen Integration

- [ ] **No codegen changes yet**
  - Metadata stored in AST/registry
  - Phase 6 will use metadata for serialization/validation
  - For now, just store and validate

---

### Testing

- [ ] **Parser tests** (`vex-parser/tests/`)

  - [ ] Parse policy declaration
  - [ ] Parse struct with `with` clause
  - [ ] Parse backtick metadata
  - [ ] Error: malformed policy
  - [ ] Error: malformed metadata

- [ ] **Type checker tests** (`examples/policy/`)

  - [ ] `01_basic_policy.vx` - Simple policy application

    ```vex
    policy APIModel {
        id  `json:"id"`,
    }

    struct User with APIModel {
        id: i32,
    }

    fn main(): i32 { return 0; }
    ```

  - [ ] `02_multiple_policies.vx` - Multiple policies

    ```vex
    policy A {
        id  `json:"id"`,
    }

    policy B {
        id  `db:"user_id"`,
    }

    struct User with A, B {
        id: i32,
    }
    // Metadata: {json: "id", db: "user_id"}

    fn main(): i32 { return 0; }
    ```

  - [ ] `03_policy_override_warning.vx` - Same key conflict

    ```vex
    policy A {
        id  `json:"id"`,
    }

    policy B {
        id  `json:"userId"`,  // Conflict!
    }

    struct User with A, B {
        id: i32,
    }
    // ⚠️ WARNING: PolicyB overrides PolicyA for key 'json'

    fn main(): i32 { return 0; }
    ```

  - [ ] `04_policy_not_found.vx` - Error case

    ```vex
    struct User with NonExistent {
        id: i32,
    }
    // ❌ ERROR: Policy 'NonExistent' not found

    fn main(): i32 { return 0; }
    ```

  - [ ] `05_policy_trait_conflict.vx` - Name collision error

    ```vex
    policy Serializable {
        id  `json:"id"`,
    }

    trait Serializable {
        fn serialize(): str;
    }
    // ❌ ERROR: Policy 'Serializable' conflicts with trait 'Serializable'

    fn main(): i32 { return 0; }
    ```

  - [ ] `06_field_not_in_policy.vx` - Warning case

    ```vex
    policy APIModel {
        id  `json:"id"`,
    }

    struct User with APIModel {
        id: i32,
        email: str,  // Not in policy (OK, no metadata)
    }

    fn main(): i32 { return 0; }
    ```

**Files to create:**

- `examples/policy/` directory
- Test files listed above
- Update `examples/README.md` with policy examples
- Update `test_all.sh` to include policy tests

---

### Error Messages

- [ ] **Policy not found**

  ```
  error[E0601]: Policy 'APIModel' not found
    --> test.vx:3:18
     |
   3 | struct User with APIModel {
     |                  ^^^^^^^^ no policy with this name exists
     |
     = help: Did you mean `APIModelV2`?
  ```

- [ ] **Policy-trait name conflict**

  ```
  error[E0602]: Policy 'Serializable' conflicts with trait 'Serializable'
    --> test.vx:1:8
     |
   1 | policy Serializable {
     |        ^^^^^^^^^^^^ policy name conflicts with existing trait
     |
     = note: Trait 'Serializable' defined at test.vx:5:7
     = help: Consider renaming to 'SerializablePolicy' or 'SerializationRules'
  ```

- [ ] **Metadata merge warning**
  ```
  warning[W0603]: Metadata key 'json' overridden by policy 'PolicyB'
    --> test.vx:10:5
     |
  10 | struct User with PolicyA, PolicyB {
     |                           ^^^^^^^ policy overrides previous value
     |
     = note: PolicyA sets json:"id", PolicyB sets json:"userId"
     = note: Using value from PolicyB: json:"userId"
  ```

**Files to modify:**

- `vex-diagnostics/src/lib.rs` - Add E0601, E0602, W0603 error codes

---

### Documentation

- [ ] Update `POLICY_SYSTEM_DESIGN.md` with implementation notes
- [ ] Update `TODO.md` with policy system completion
- [ ] Update `.github/copilot-instructions.md` with policy syntax

---

## 🎯 Sprint 2: Anonymous Embedding (2-3 days)

### Parser Changes

- [ ] **Parse anonymous embedding**
  ```vex
  struct User {
      Entity,    // Type without field name = anonymous
      email: str,
  }
  ```
  - [ ] Detect bare type name (no `:` separator)
  - [ ] Store as embedded field in AST
  - [ ] Distinguish from named field: `entity: Entity`

**Files to modify:**

- `vex-parser/src/parser/items/structs.rs` - Detect anonymous embedding syntax

---

### AST Changes

- [ ] **Add embedded flag to FieldDef**

  ```rust
  pub struct FieldDef {
      pub name: String,
      pub ty: Type,
      pub embedded: bool,  // NEW: true if anonymous embedding
      pub metadata: Option<HashMap<String, String>>,
      pub span: Span,
  }
  ```

- [ ] **Field naming convention**
  - If `embedded == true`, `name == type_name`
  - Example: `Entity` → `name: "Entity", ty: Type::Named("Entity"), embedded: true`

**Files to modify:**

- `vex-ast/src/lib.rs` - Add embedded flag to FieldDef

---

### Type Checker

- [ ] **Field promotion**

  ```rust
  fn promote_embedded_fields(struct_def: &StructDef, registry: &TypeRegistry) {
      // For each embedded field:
      //   1. Lookup embedded struct type
      //   2. Get all its fields
      //   3. Promote fields to parent struct (virtual access)
      //   4. Track field origin (for conflict detection)
  }
  ```

  - [ ] Identify embedded fields (`embedded == true`)
  - [ ] Lookup embedded struct type in registry
  - [ ] Track promoted fields (for access)
  - [ ] Create field access mapping: `user.id` → `user.Entity.id`

- [ ] **Field conflict detection**

  ```vex
  struct A { value: i32 }
  struct B { value: i32 }

  struct X {
      A,     // Has 'value'
      B,     // Has 'value' - CONFLICT!
  }
  ```

  - [ ] Detect field name conflicts between embedded structs
  - [ ] **Warning**: `"Field 'value' from B shadows field from A"`
  - [ ] Last embedding wins for promoted access
  - [ ] Explicit access still works: `x.A.value`, `x.B.value`

**Files to modify:**

- `vex-compiler/src/codegen_ast/registry.rs` - Add promoted_fields HashMap
- `vex-compiler/src/codegen_ast/mod.rs` - Call promote_embedded_fields()

---

### Codegen

- [ ] **Field access codegen**

  ```vex
  let user = User { Entity: entity_val, email: "test" };
  println(user.id);  // Promoted field access
  ```

  - [ ] Detect promoted field access
  - [ ] Translate to: `user.Entity.id` (access embedded struct field)
  - [ ] Generate LLVM GEP for nested struct access

- [ ] **Explicit struct access**
  ```vex
  println(user.Entity.id);  // Explicit access
  ```
  - [ ] Already works (normal field access)
  - [ ] No special handling needed

**Files to modify:**

- `vex-compiler/src/codegen_ast/expressions/access/field_access.rs` - Add promoted field resolution

---

### Testing

- [ ] **Basic tests** (`examples/embedding/`)

  - [ ] `01_simple_embedding.vx` - Basic anonymous embedding

    ```vex
    struct Entity {
        id: i32,
        name: str,
    }

    struct User {
        Entity,
        email: str,
    }

    fn main(): i32 {
        let! user = User {
            Entity: Entity { id: 1, name: "Alice" },
            email: "alice@example.com",
        };

        println(user.id);        // Promoted access
        println(user.Entity.id); // Explicit access
        return user.id;
    }
    ```

  - [ ] `02_multiple_embeddings.vx` - Multiple embedded structs
  - [ ] `03_embedding_conflict.vx` - Field name conflicts (warning)
  - [ ] `04_deep_embedding.vx` - Nested embeddings

**Files to create:**

- `examples/embedding/` directory
- Test files listed above

---

## 🎯 Sprint 3: Inline Tags (1-2 days)

### Parser Changes

- [ ] **Parse inline backtick metadata**
  ```vex
  struct User {
      id: i32 `json:"id" db:"user_id"`,
  }
  ```
  - [ ] Already done in Sprint 1 (backtick parsing)
  - [ ] Just enable on struct fields without policy

**Files to modify:**

- `vex-parser/src/parser/items/structs.rs` - Parse inline metadata on fields

---

### Type Checker

- [ ] **Inline metadata parsing**

  - [ ] Use same `parse_metadata()` function from Sprint 1
  - [ ] Store in `FieldDef.metadata`

- [ ] **Inline overrides policy (MERGE strategy)**

  ```vex
  policy APIModel {
      id  `json:"id" db:"user_id"`,
  }

  struct User with APIModel {
      id: i32 `json:"userId"`,  // Override 'json', keep 'db'
  }
  // Result: {json: "userId", db: "user_id"}
  ```

  - [ ] Apply policy metadata first
  - [ ] Merge inline metadata (key-by-key)
  - [ ] Inline value overrides policy value for same key
  - [ ] Keep policy keys not in inline metadata

**Files to modify:**

- `vex-compiler/src/codegen_ast/metadata.rs` - Update merge logic for inline override

---

### Testing

- [ ] **Tests** (`examples/policy/`)

  - [ ] `07_inline_only.vx` - No policy, just inline

    ```vex
    struct User {
        id: i32 `json:"id" db:"user_id"`,
    }
    ```

  - [ ] `08_inline_override_merge.vx` - Inline merges with policy

    ```vex
    policy APIModel {
        id  `json:"id" db:"user_id"`,
    }

    struct User with APIModel {
        id: i32 `json:"userId"`,  // Override json, keep db
    }
    // Result: {json: "userId", db: "user_id"}
    ```

  - [ ] `09_inline_add_to_policy.vx` - Inline adds new keys

    ```vex
    policy APIModel {
        id  `json:"id"`,
    }

    struct User with APIModel {
        id: i32 `db:"user_id" validate:"required"`,
    }
    // Result: {json: "id", db: "user_id", validate: "required"}
    ```

**Files to create:**

- Additional test files in `examples/policy/`

---

## 🎯 Sprint 4: Trait Fields (2-3 days)

### Parser Changes

- [ ] **Parse fields in traits**
  ```vex
  trait Identifiable {
      id: i32;        // Field (contract)
      name: str;      // Field (contract)
  }
  ```
  - [ ] Check if already supported (likely yes)
  - [ ] If not, add field parsing to trait body

**Files to modify:**

- `vex-parser/src/parser/items/traits.rs` - Ensure field parsing works

---

### AST Changes

- [ ] **Ensure TraitDef has fields**
  ```rust
  pub struct TraitDef {
      pub name: String,
      pub fields: Vec<FieldDef>,  // Should already exist
      pub methods: Vec<Function>,
      // ...
  }
  ```
  - [ ] Verify fields are stored in AST
  - [ ] If not, add fields to TraitDef

**Files to modify:**

- `vex-ast/src/lib.rs` - Verify TraitDef.fields exists

---

### Type Checker

- [ ] **Merge trait fields into implementing struct**

  ```rust
  fn apply_trait_fields(struct_def: &StructDef, traits: &[TraitDef]) {
      // For each trait:
      //   1. Get trait fields
      //   2. Add to struct fields (if not already present)
      //   3. Detect conflicts (see rules below)
  }
  ```

- [ ] **Conflict detection rules**

  **Rule 1: Trait + Trait field conflict**

  ```vex
  trait A { value: i32; }
  trait B { value: i32; }

  struct X impl A, B { }
  // ⚠️ WARNING: "Field 'value' from trait B overrides field from trait A"
  // Last trait wins
  ```

  - [ ] Detect same field name in multiple traits
  - [ ] **Warning** (not error)
  - [ ] Last trait wins

  **Rule 2: Trait field + Struct field conflict**

  ```vex
  trait T { id: i32; }

  struct User impl T {
      id: i32,  // Redundant!
  }
  // ⚠️ WARNING: "Field 'id' already defined by trait T"
  ```

  - [ ] Detect struct field with same name as trait field
  - [ ] **Warning**: Redundant definition
  - [ ] Struct field is ignored (trait field used)

  **Rule 3: Trait field + Embedded struct field conflict**

  ```vex
  trait Identifiable { id: i32; }
  struct Base { id: i32; }

  struct User impl Identifiable {
      Base,  // Conflict!
  }
  // ❌ ERROR: "Field 'id' conflict: trait Identifiable and embedded struct Base both define 'id'"
  ```

  - [ ] Detect trait field with same name as embedded struct field
  - [ ] **Hard error**: Ambiguous which 'id' is the contract

- [ ] **Struct literal validation**
  ```vex
  let user = User {
      id: 1,      // From trait
      name: "Alice",  // From trait
      email: "test",  // From struct
  };
  ```
  - [ ] Ensure all trait fields are initialized
  - [ ] Same rules as normal struct fields

**Files to modify:**

- `vex-compiler/src/codegen_ast/registry.rs` - Add apply_trait_fields()
- `vex-compiler/src/codegen_ast/traits.rs` - Trait field merging logic

---

### Codegen

- [ ] **Include trait fields in struct layout**
  - [ ] Trait fields are part of struct (no special handling)
  - [ ] Access like normal fields: `user.id`

**Files to modify:**

- `vex-compiler/src/codegen_ast/types.rs` - Ensure trait fields included in struct type

---

### Testing

- [ ] **Tests** (`examples/trait_fields/`)

  - [ ] `01_basic_trait_fields.vx` - Simple trait with fields

    ```vex
    trait Identifiable {
        id: i32;
        name: str;
    }

    struct User impl Identifiable {
        email: str,
    }

    fn main(): i32 {
        let user = User {
            id: 1,
            name: "Alice",
            email: "alice@example.com",
        };
        return user.id;
    }
    ```

  - [ ] `02_trait_field_conflict.vx` - Multiple traits with same field (warning)
  - [ ] `03_trait_struct_field_redundant.vx` - Struct redefines trait field (warning)
  - [ ] `04_trait_embedding_conflict.vx` - Trait + embedded struct conflict (error)

**Files to create:**

- `examples/trait_fields/` directory
- Test files listed above

---

## 🎯 Sprint 5: Policy Composition (2-3 days)

### Parser Changes

- [ ] **Parse policy inheritance**
  ```vex
  policy Child with Parent1, Parent2 {
      id  `validate:"required"`,
  }
  ```
  - [ ] Add `with` clause to policy declarations
  - [ ] Parse parent policy names

**Files to modify:**

- `vex-parser/src/parser/items/policies.rs` - Add `with` clause parsing

---

### Type Checker

- [ ] **Resolve parent policies**

  ```rust
  fn resolve_policy_inheritance(policy: &Policy, policies: &HashMap<String, Policy>) {
      // For each parent:
      //   1. Lookup parent policy
      //   2. Recursively resolve parent's parents
      //   3. Merge parent fields into child
      //   4. Detect circular dependencies
  }
  ```

  - [ ] Lookup parent policies
  - [ ] Recursively resolve grandparents
  - [ ] Merge parent fields (left-to-right)
  - [ ] Child can override parent (with warning)
  - [ ] Detect circular dependencies (error)

- [ ] **Circular dependency detection**
  ```vex
  policy A with B { }
  policy B with A { }
  // ❌ ERROR: Circular policy inheritance: A -> B -> A
  ```

**Files to modify:**

- `vex-compiler/src/codegen_ast/metadata.rs` - Add resolve_policy_inheritance()

---

### Testing

- [ ] **Tests** (`examples/policy/`)
  - [ ] `10_policy_composition.vx` - Basic inheritance
  - [ ] `11_policy_multilevel.vx` - Grandparent inheritance
  - [ ] `12_policy_circular.vx` - Circular dependency (error)

---

## 🎯 Sprint 6: Metadata Access API (1-2 days)

### Compile-Time Storage, Runtime Access (Zero-Cost)

**Architecture:** Zero-cost abstraction approach

- Metadata stored in AST as `HashMap<String, String>` during compilation
- At runtime, access via generated global constant tables in .rodata
- No reflection overhead - just direct HashMap lookups (compile-time generated)

**Decision:** Use `.` for metadata access (consistent with Vex, no `::`)

```vex
// Access field metadata at runtime (zero-cost)
fn get_json_name(field_name: str): str? {
    User.field_metadata(field_name).get("json")
}
```

- [ ] **Design metadata access syntax**

  - [ ] `Type.field_metadata(field_name)` returns `Map<str, str>`
  - [ ] Use `.` not `::` (Vex standard)
  - [ ] Metadata stored in AST HashMap (compile-time)
  - [ ] Accessed via global const at runtime (zero overhead)

- [ ] **Codegen for metadata access**
  - [ ] Generate global const HashMap in LLVM IR for each struct
    ```rust
    // Generated LLVM:
    @User_metadata = constant { field_name -> HashMap<key, value> }
    ```
  - [ ] Generate `field_metadata()` static method per struct
    ```vex
    // Generated for each struct:
    fn User.field_metadata(field: str): Map<str, str> {
        // Direct lookup in compile-time generated HashMap
        // Zero runtime overhead - just memory access
    }
    ```
  - [ ] Store metadata as global constants (immutable, in .rodata section)

**Files to modify:**

- `vex-compiler/src/codegen_ast/builtins/reflection.rs` - Add field_metadata() codegen
- `vex-compiler/src/codegen_ast/types.rs` - Generate metadata globals for structs

---

### Testing

- [ ] **Tests** (`examples/policy/`)

  - [ ] `13_metadata_access.vx` - Access metadata at runtime

    ```vex
    policy APIModel {
        id `json:"id" db:"user_id"`,
    }

    struct User with APIModel {
        id: i32,
    }

    fn main(): i32 {
        // Zero-cost: direct global const lookup
        let json_name = User.field_metadata("id").get("json");
        println(json_name);  // Prints: "id"

        let db_name = User.field_metadata("id").get("db");
        println(db_name);  // Prints: "user_id"

        return 0;
    }
    ```

  - [ ] `14_metadata_iteration.vx` - Iterate over all field metadata
    ```vex
    fn main(): i32 {
        // Access all fields (if we support this)
        let fields = User.fields();
        for field in fields {
            let metadata = User.field_metadata(field);
            println(format("Field: {}, JSON: {}", field, metadata.get("json")));
        }
        return 0;
    }
    ```

---

## 📊 Progress Tracking

### Sprint Status

| Sprint | Feature             | Status  | Days | Priority    |
| ------ | ------------------- | ------- | ---- | ----------- |
| 1      | Policy Declarations | ⏳ TODO | 3-4  | 🔴 Critical |
| 2      | Anonymous Embedding | ⏳ TODO | 2-3  | 🟡 High     |
| 3      | Inline Tags         | ⏳ TODO | 1-2  | 🟡 High     |
| 4      | Trait Fields        | ⏳ TODO | 2-3  | 🟡 High     |
| 5      | Policy Composition  | ⏳ TODO | 2-3  | 🟢 Medium   |
| 6      | Metadata Access API | ⏳ TODO | 1-2  | 🟢 Medium   |

**Total Estimated:** 11-17 days

---

## 📝 Future Considerations (Not in Current Plan)

### POLICY_v2: Tag Processing System

**Note:** This is for future implementation, not part of current policy system.

When we implement tag processing (POLICY_v2), we need to handle:

1. **System Built-in Tags** (Reserved)

   - `json:"..."` - JSON serialization mapping
   - `db:"..."` - Database column mapping
   - `validate:"..."` - Validation rules
   - `db_primary:"true"` - Primary key marker
   - `db_index:"true"` - Index marker
   - `db_unique:"true"` - Unique constraint

2. **Custom User Tags** (Extensible)

   - User-defined tags for custom purposes
   - Example: `api:"..."`, `cache:"..."`, `log:"..."`
   - No semantic meaning to compiler, just metadata storage

3. **Tag Validation** (Type-Aware)

   - Warn if tag doesn't make sense for field type
   - Example: `validate:"email"` on `i32` → warning
   - Example: `validate:"min=1"` on `i32` → OK

4. **Tag Processing Infrastructure**
   - Codegen hooks for tag-based code generation
   - Example: Generate `to_json()` from `json:` tags
   - Example: Generate SQL schema from `db:` tags
   - Example: Generate validators from `validate:` tags

**Status:** 📋 Design note only - not implementing now  
**Future Sprint:** POLICY_v2 (after core policy system stable)

---

## ❌ Rejected Features (Not Implementing)

### 1. Metadata Macros

```vex
// ❌ NOT implementing
policy RESTful {
    id `@primary @json`,  // Macro expansion
}
```

**Reason:** Adds complexity without clear benefit. Explicit is better.

### 2. Conditional Policies

```vex
// ❌ NOT implementing (maybe far future)
policy DebugMode {
    #[cfg(debug)]
    id `log:"true"`,
}
```

**Reason:** Low priority, niche use case. Not worth complexity now.

---

## 🎯 Immediate Next Steps

### Start Sprint 1: Policy Declarations

1. **Add keywords to lexer** (30 min)

   - [ ] Add `policy` keyword
   - [ ] Add `with` keyword

2. **Create policy parser** (2-3 hours)

   - [ ] Create `vex-parser/src/parser/items/policies.rs`
   - [ ] Parse policy declaration syntax
   - [ ] Parse backtick metadata

3. **Update AST** (1-2 hours)

   - [ ] Add Policy, PolicyField structs
   - [ ] Add Item::Policy variant
   - [ ] Update StructDef with policies field
   - [ ] Update FieldDef with metadata HashMap

4. **Test parsing** (1 hour)

   - [ ] Create basic test file
   - [ ] Verify policy AST structure

5. **Build registry** (2-3 hours)

   - [ ] Add policies HashMap to registry
   - [ ] Register policies from program
   - [ ] Policy-trait conflict detection

6. **Metadata parsing** (2-3 hours)

   - [ ] Create metadata.rs module
   - [ ] Implement parse_metadata() function
   - [ ] Implement merge_metadata() function

7. **Apply policies to structs** (3-4 hours)

   - [ ] Implement apply_policies_to_struct()
   - [ ] Field name mapping
   - [ ] Metadata merging

8. **Error messages** (1-2 hours)

   - [ ] Add diagnostic codes
   - [ ] Policy not found error
   - [ ] Policy-trait conflict error
   - [ ] Metadata merge warning

9. **Create tests** (2-3 hours)

   - [ ] Basic policy application
   - [ ] Multiple policies
   - [ ] Conflicts and warnings
   - [ ] Error cases

10. **Test and verify** (1-2 hours)
    - [ ] Run all tests
    - [ ] Fix bugs
    - [ ] Update documentation

---

## 📝 Design Decisions Summary

1. ✅ **Implementation Order:** Policy Declarations → Anonymous Embedding → Inline Tags → Trait Fields
2. ✅ **Metadata Storage:** `Option<HashMap<String, String>>` (parsed in type checker, stored in AST)
3. ✅ **Policy-Trait Conflict:** Hard error (no name collisions allowed)
4. ✅ **Metadata Access:** Compile-time HashMap, runtime zero-cost lookup via global const, use `.` not `::`
5. ✅ **Inline Override:** Merge strategy (inline key overrides policy key, keeps other keys)
6. ✅ **Export Keyword:** Use `export` not `pub` (Vex standard)
7. ✅ **Zero-Cost Principle:** No runtime reflection overhead - metadata in .rodata, direct access
8. ✅ **Tag Processing:** Deferred to POLICY_v2 (system tags + custom tags processing)

---

## 🚀 Ready to Start!

All design decisions finalized. Starting with Sprint 1: Policy Declarations.

**First file to create:** `vex-parser/src/parser/items/policies.rs`
