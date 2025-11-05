# Vex Policy System - Design Document

**Date:** November 5, 2025  
**Version:** Vex v0.2.0  
**Status:** ✅ Design Finalized - Hybrid Approach (Inline + Policy)

---

## 🎯 Core Concept

**Policy** = Reusable metadata/constraint definitions that can be applied to structs, traits, and fields.

**Vex Approach: HYBRID**

- **Inline tags** (Go-style): For simple, one-off metadata
- **Policy declarations**: For reusable, shared metadata patterns

---

## 🚨 Critical: Policy vs Trait Syntax

**MUST READ:** [POLICY_TRAIT_SYNTAX_CLARIFICATION.md](./POLICY_TRAIT_SYNTAX_CLARIFICATION.md)

### Quick Summary

```vex
// Policy (with) = METADATA
struct User with APIModel {
    id: i32,  // Gets metadata from policy
}

// Trait (impl) = BEHAVIOR
struct User impl Drawable {
    fn (&self)draw() { ... }  // Must implement methods
}

// BOTH together - ORDER MATTERS
struct User
    with APIModel        // Metadata FIRST
    impl Drawable {      // Behavior SECOND

    id: i32,
    fn (&self)draw() { ... }
}
```

**Rules:**

1. `with` = metadata (policies)
2. `impl` = behavior (traits)
3. `with` ALWAYS before `impl`
4. No name collisions between policies and traits

---

## Design Philosophy

1. **Flexibility First**: Support both inline and policy-based metadata
2. **Start Simple, Scale Complex**: Inline for prototypes, policies for production
3. **Reusability**: Policies can be shared across structs/traits
4. **Composability**: Multiple policies can be combined (like traits)
5. **Type Safety**: All metadata validated at compile time
6. **Explicit > Implicit**: Clear which policies apply to which types
7. **Progressive Enhancement**: Start with inline, refactor to policies later
8. **Clear Separation**: Metadata (policy) ≠ Behavior (trait)

---

## ✅ Final Hybrid Syntax

Vex supports **THREE ways** to define metadata, providing flexibility for all use cases:

### 1. Inline Tags (Go-Style) - For Simple Cases

```vex
struct User {
    id: i32 `json:"id" db:"user_id"`,
    name: str `json:"name" validate:"required"`,
    email: str `json:"email" validate:"email"`,
}
```

**Use when:**

- ✅ Prototyping / Quick development
- ✅ Simple, one-off structs
- ✅ No reusability needed
- ✅ Small projects / scripts

---

### 2. Policy Declarations - For Reusable Patterns

```vex
policy APIModel {
    id      `json:"id" db:"user_id"`,
    name    `json:"name"`,
    email   `json:"email"`,
}

struct User with APIModel {
    id: i32,
    name: str,
    email: str,
}
```

**Use when:**

- ✅ Same metadata across multiple structs
- ✅ Team conventions / standards
- ✅ Large projects
- ✅ Need versioning (APIModel_v1, APIModel_v2)

---

### 3. Hybrid (Policy + Inline Override) - Best of Both Worlds

```vex
policy BaseModel {
    id          `json:"id" db_primary:"true"`,
    created_at  `json:"created_at"`,
}

struct User with BaseModel {
    id: i32,
    created_at: i64,
    name: str `json:"userName"`,  // Inline override/addition
    email: str `json:"email" validate:"email"`,  // New field with metadata
}
```

**Use when:**

- ✅ Want policy benefits but need customization
- ✅ Most fields follow pattern, some exceptions
- ✅ Incremental migration from inline to policy

**Rule:** Inline metadata **overrides** policy metadata for that field.

---

---

## Metadata Resolution Rules

### Rule 1: Name-Based Mapping (Default)

Policies map to struct fields by **field name**.

```vex
policy Serializable {
    id      `json:"id"`,
    name    `json:"name"`,
}

struct User with Serializable {
    id: i32,        // ✅ Maps to policy.id
    name: str,      // ✅ Maps to policy.name
    email: str,     // ⚠️  No mapping (not in policy)
}
```

---

### Rule 2: Policy Application with `with` Keyword

```vex
struct User with Serializable, Validatable {
    id: i32,
    name: str,
}
```

**Consistent with trait syntax:**

- `struct User impl Trait` - Implements behavior contract
- `struct User with Policy` - Applies metadata pattern

---

### Rule 3: Multiple Policies - Merge Left-to-Right

```vex
policy A {
    id  `json:"id" validate:"required"`,
}

policy B {
    id  `db:"user_id" validate:"min=1"`,
}

struct User with A, B {
    id: i32,
}

// Result: id has merged metadata
// `json:"id" db:"user_id" validate:"required" validate:"min=1"`
```

**Merge Rules:**

- Different keys → Combine (e.g., `json` + `db`)
- Same key, different values → **Last policy wins** + **WARNING**
- Same key, same value → Keep one (no duplicate)

---

### Rule 4: Inline Overrides Policy

```vex
policy DefaultJSON {
    id      `json:"id"`,
    name    `json:"name"`,
}

struct User with DefaultJSON {
    id: i32 `json:"userId"`,     // ✅ Inline overrides policy
    name: str,                    // Uses policy: json:"name"
    email: str `json:"email"`,   // New field with inline metadata
}

// Final metadata:
// id:    `json:"userId"`         (inline override)
// name:  `json:"name"`           (from policy)
// email: `json:"email"`          (inline only)
```

**Priority:** Inline > Policy > Default

---

### Rule 5: Policy Composition with `with`

```vex
policy Base {
    id      `json:"id"`,
    name    `json:"name"`,
}

policy Extended with Base {
    id      `db:"user_id" validate:"required"`,  // Merges with Base.id
    email   `json:"email" validate:"email"`,     // New field
}

// Result: Extended has:
// id:    `json:"id" db:"user_id" validate:"required"`
// name:  `json:"name"`
// email: `json:"email" validate:"email"`
```

**Composition Rules:**

1. Child policy **inherits** all parent fields
2. Child policy can **add** attributes to parent fields
3. Child policy can **override** parent attributes (with warning)
4. Child policy can **add** new fields

---

## Final Syntax Summary

---

## Complete Syntax Examples

### Example 1: Inline Only (Quick Development)

```vex
struct User {
    id: i32 `json:"id" db:"user_id"`,
    name: str `json:"name" validate:"required"`,
    email: str `json:"email" validate:"email"`,
}
```

**Use case:** Prototyping, small scripts, one-off structs

---

### Example 2: Policy Only (Reusable Patterns)

```vex
policy APIModel {
    id      `json:"id" db:"user_id"`,
    name    `json:"name"`,
    email   `json:"email"`,
}

policy Validatable {
    id      `validate:"required,min=1"`,
    name    `validate:"required,min=3,max=50"`,
    email   `validate:"required,email"`,
}

struct User with APIModel, Validatable {
    id: i32,
    name: str,
    email: str,
    age: i32,      // Not in policies (no metadata)
}
```

**Result:** User fields get merged metadata:

- `id`: `json:"id" db:"user_id" validate:"required,min=1"`
- `name`: `json:"name" validate:"required,min=3,max=50"`
- `email`: `json:"email" validate:"required,email"`
- `age`: (no metadata)

**Use case:** Large projects, team standards, shared conventions

---

### Example 3: Hybrid (Policy + Inline Override)

```vex
policy BaseModel {
    id          `json:"id" db_primary:"true"`,
    created_at  `json:"created_at"`,
    updated_at  `json:"updated_at"`,
}

struct User with BaseModel {
    id: i32,                             // Uses policy: json:"id" db_primary:"true"
    created_at: i64,                     // Uses policy: json:"created_at"
    updated_at: i64,                     // Uses policy: json:"updated_at"
    name: str `json:"userName"`,         // Inline only (not in policy)
    email: str `json:"email" validate:"email"`,  // Inline only
}

struct Admin with BaseModel {
    id: i32 `json:"admin_id"`,           // ✅ Override policy!
    created_at: i64,
    updated_at: i64,
    permissions: str `json:"permissions"`,
}
```

**Use case:** Most fields follow pattern, some need customization

---

### Example 4: Policy Composition (Inheritance)

```vex
policy Timestamped {
    created_at  `json:"created_at" db:"created_at"`,
    updated_at  `json:"updated_at" db:"updated_at"`,
}

policy Identifiable {
    id  `json:"id" db_primary:"true"`,
}

policy BaseModel with Timestamped, Identifiable {
    // Inherits all fields from both parents
}

policy UserModel with BaseModel {
    id      `db:"user_id"`,  // Override db column (keeps json:"id" and db_primary:"true")
    name    `json:"name" validate:"required"`,
    email   `json:"email" validate:"email"`,
}

struct User with UserModel {
    id: i32,
    created_at: i64,
    updated_at: i64,
    name: str,
    email: str,
}
```

**Result:** User gets all metadata from inheritance chain:

- `id`: `json:"id" db:"user_id" db_primary:"true"` (merged)
- `created_at`: `json:"created_at" db:"created_at"`
- `updated_at`: `json:"updated_at" db:"updated_at"`
- `name`: `json:"name" validate:"required"`
- `email`: `json:"email" validate:"email"`

**Use case:** Complex projects with layered metadata patterns

---

## Advanced Features

### Feature 1: Policy on Traits

```vex
trait Entity with Serializable {
    id: i32;
    name: str;
}

struct User impl Entity {
    id: i32,      // Gets Entity's Serializable policy
    name: str,
    email: str,   // No policy (not in Entity)
}
```

**Rule:** When struct implements trait, it inherits trait's policies for trait fields.

---

### Feature 2: Field-Level Policy Override

```vex
policy DefaultJSON {
    id      `json:"id"`,
    name    `json:"name"`,
}

struct User with DefaultJSON {
    id: i32 `json:"user_id"`,  // Override policy for this field
    name: str,                  // Uses policy default
}
```

**Rule:** Inline field attributes override policy attributes.

---

### Feature 3: Conditional Policies

```vex
policy DebugMode {
    #[cfg(debug)]
    id      `log:"true" trace:"true"`,

    #[cfg(release)]
    id      `log:"false"`,
}
```

**Rule:** Policies can have conditional attributes based on build config.

---

### Feature 4: Policy Inheritance

```vex
policy Timestamped {
    created_at  `json:"created_at" db:"created_at"`,
    updated_at  `json:"updated_at" db:"updated_at"`,
}

policy Identifiable {
    id  `json:"id" db_primary:"true"`,
}

policy BaseModel with Timestamped, Identifiable {
    // Inherits all fields from both
}

struct User with BaseModel {
    id: i32,
    created_at: i64,
    updated_at: i64,
}
```

---

---

## Advanced Features

### Feature 1: Policy on Traits

```vex
trait Entity with Serializable {
    id: i32;
    name: str;
}

struct User impl Entity {
    id: i32,      // Gets Entity's Serializable policy
    name: str,
    email: str,   // No policy (not in Entity)
}
```

**Status:** Phase 3  
**Rule:** When struct implements trait, it inherits trait's policies for trait fields.

---

### Feature 2: Conditional Policies (Future)

```vex
policy DebugMode {
    #[cfg(debug)]
    id      `log:"true" trace:"true"`,

    #[cfg(release)]
    id      `log:"false"`,
}
```

**Status:** Phase 4 (Optional)  
**Rule:** Policies can have conditional attributes based on build config.

---

## Implementation Phases

### Phase 1: Inline Tags (2-3 days) - Foundation

**Parser:**

- [ ] Parse backtick strings on struct fields
  ```vex
  struct User {
      id: i32 `json:"id" db:"user_id"`,
  }
  ```
- [ ] Store inline metadata in AST field definition
- [ ] Parse comma-separated key:"value" pairs

**AST:**

- [ ] Add `metadata: Option<String>` to `FieldDef`
- [ ] Parse backtick string into structured data

**Type Checker:**

- [ ] Validate metadata format (key:"value" pairs)
- [ ] Store parsed metadata in field

**Tests:**

- [ ] Single metadata: `json:"id"`
- [ ] Multiple metadata: `json:"id" db:"user_id"`
- [ ] Invalid format error

**Estimated:** 2-3 days  
**Priority:** 🔴 Critical (Foundation for everything)

---

### Phase 2: Policy Declarations (3-4 days) - Core

**Parser:**

- [ ] `policy Name { field `metadata`, ... }` declaration
- [ ] `struct Name with Policy1, Policy2 { ... }` syntax
- [ ] Store policy definitions in AST (new `Policy` item)

**AST:**

- [ ] Add `Policy` item type
  ```rust
  pub struct Policy {
      pub name: String,
      pub fields: Vec<(String, String)>,  // (field_name, metadata)
      pub parent_policies: Vec<String>,   // For composition
  }
  ```
- [ ] Add `policies: Vec<String>` to `StructDef`

**Type Checker:**

- [ ] Resolve policy names
- [ ] Map policy fields to struct fields (name-based)
- [ ] Merge multiple policies (left-to-right)
- [ ] Warn on conflicts (same key, different value)

**Metadata Storage:**

- [ ] Merge policy metadata into struct field metadata
- [ ] Track metadata origin (for debugging)

**Tests:**

- [ ] Basic policy application
- [ ] Multiple policies merge
- [ ] Policy field not in struct (warning)
- [ ] Struct field not in policy (OK)
- [ ] Conflict warning

**Estimated:** 3-4 days  
**Priority:** 🔴 Critical

---

### Phase 3: Hybrid (Inline Override) (1-2 days)

**Type Checker:**

- [ ] Apply policy metadata first
- [ ] Override with inline metadata (if present)
- [ ] Merge rules: inline > policy

**Tests:**

- [ ] Policy with inline override
- [ ] Inline adds to policy
- [ ] Inline completely replaces policy field

**Estimated:** 1-2 days  
**Priority:** 🟡 High

---

### Phase 4: Policy Composition (2-3 days)

**Parser:**

- [ ] `policy Child with Parent1, Parent2 { ... }` syntax

**Type Checker:**

- [ ] Resolve parent policies
- [ ] Merge parent fields into child
- [ ] Detect circular dependencies
- [ ] Handle override rules (child overrides parent)

**Tests:**

- [ ] Single parent
- [ ] Multiple parents
- [ ] Override parent field
- [ ] Circular dependency error

**Estimated:** 2-3 days  
**Priority:** 🟡 High

---

### Phase 5: Policy on Traits (1-2 days)

**Parser:**

- [ ] `trait Name with Policy { ... }` syntax

**Type Checker:**

- [ ] Apply trait policies to implementing structs
- [ ] Only apply to trait fields, not struct's own fields
- [ ] Merge trait policies with struct policies

**Tests:**

- [ ] Trait with policy
- [ ] Struct impl trait inherits policy
- [ ] Struct's own fields don't get trait policy

**Estimated:** 1-2 days  
**Priority:** 🟢 Medium

---

### Phase 6: Tooling & Ecosystem (3-4 days)

**Metadata Access API:**

- [ ] Runtime reflection (if needed)
  ```vex
  fn get_metadata(field: str): Map<str, str> {
      User::field_metadata(field)
  }
  ```
- [ ] Compile-time metadata for codegen

**Code Generation (vex-serde, vex-validator):**

- [ ] JSON serialization from `json:` tags
- [ ] Database mapping from `db:` tags
- [ ] Validation from `validate:` tags

**Documentation:**

- [ ] Policy usage guide
- [ ] Common patterns
- [ ] Best practices

**Estimated:** 3-4 days  
**Priority:** 🟢 Medium

---

### Phase 7: Advanced Features (Optional, Future)

**Conditional Policies:**

- [ ] `#[cfg(debug)]` support in policies

**Pattern Matching:**

- [ ] `*_id` pattern for field names
- [ ] Type-based policies

**Policy Parameters:**

- [ ] Generic policies: `policy Validated<min_age: i32>`

**Estimated:** 3-5 days  
**Priority:** 🔵 Low (Future)

---

## Total Estimated Effort

| Phase | Feature                  | Effort   | Priority        |
| ----- | ------------------------ | -------- | --------------- |
| 1     | Inline Tags (Foundation) | 2-3 days | 🔴 Critical     |
| 2     | Policy Declarations      | 3-4 days | 🔴 Critical     |
| 3     | Hybrid (Inline Override) | 1-2 days | 🟡 High         |
| 4     | Policy Composition       | 2-3 days | 🟡 High         |
| 5     | Policy on Traits         | 1-2 days | 🟢 Medium       |
| 6     | Tooling & Ecosystem      | 3-4 days | 🟢 Medium       |
| 7     | Advanced Features        | 3-5 days | 🔵 Low (Future) |

**Core System (Phases 1-4):** 8-12 days  
**Full Implementation (Phases 1-6):** 12-18 days  
**With Advanced (Phases 1-7):** 15-23 days

---

## Recommended Implementation Order

### Sprint 1: Foundation (2-3 days)

**Goal:** Get inline tags working

- Parse backtick strings on fields
- Store metadata in AST
- Basic metadata access

**Outcome:** Can write `id: i32 `json:"id"`` and it works

---

### Sprint 2: Core Policy System (3-4 days)

**Goal:** Policy declarations and application

- Parse policy definitions
- Apply policies to structs
- Merge multiple policies

**Outcome:** Can define policies and apply with `with` keyword

---

### Sprint 3: Hybrid Power (1-2 days)

**Goal:** Inline overrides policy

- Priority: inline > policy
- Override and merge rules

**Outcome:** Best of both worlds - policy + customization

---

### Sprint 4: Composition (2-3 days)

**Goal:** Policy inheritance

- `policy Child with Parent`
- Merge and override rules

**Outcome:** Reusable policy hierarchies

---

### Sprint 5: Ecosystem (4-6 days)

**Goal:** Make it useful

- Policy on traits
- vex-serde integration
- vex-validator integration

**Outcome:** Policies actually generate code

- [ ] Common policy patterns
- [ ] Best practices

**Estimated:** 2-3 days

---

## Total Estimated Effort

| Phase | Feature             | Effort   | Priority    |
| ----- | ------------------- | -------- | ----------- |
| 1     | Core Policy System  | 3-4 days | 🔴 Critical |
| 2     | Policy Composition  | 2-3 days | 🟡 High     |
| 3     | Advanced Features   | 3-4 days | 🟢 Medium   |
| 4     | Tooling & Ecosystem | 2-3 days | 🟢 Low      |

**Total:** 10-14 days for full implementation

---

## Example Use Cases

### Use Case 1: REST API Models

```vex
policy APIModel {
    id          `json:"id" db_primary:"true"`,
    created_at  `json:"created_at" db:"created_at"`,
    updated_at  `json:"updated_at" db:"updated_at"`,
}

policy UserValidation {
    email       `validate:"required,email"`,
    password    `validate:"required,min=8"`,
    age         `validate:"min=18,max=120"`,
}

struct User with APIModel, UserValidation {
    id: i32,
    email: str,
    password: str,
    age: i32,
    created_at: i64,
    updated_at: i64,
}

// Codegen can read metadata:
// user.id -> json:"id", db_primary:"true"
// user.email -> json:"email", validate:"required,email"
```

---

### Use Case 2: Database ORM

```vex
policy DBModel {
    id      `db_primary:"true" auto_increment:"true"`,
}

policy Indexed {
    email   `db_index:"true" unique:"true"`,
    name    `db_index:"true"`,
}

struct User with DBModel, Indexed {
    id: i32,
    email: str,
    name: str,
    password: str,  // Not indexed
}

// ORM can generate:
// CREATE TABLE users (
//   id INTEGER PRIMARY KEY AUTOINCREMENT,
//   email TEXT UNIQUE,
//   name TEXT,
//   password TEXT
// );
// CREATE INDEX idx_email ON users(email);
// CREATE INDEX idx_name ON users(name);
```

---

### Use Case 3: Validation Framework

```vex
policy StrictValidation {
    email       `validate:"required,email"`,
    password    `validate:"required,min=8,has_special_char"`,
    age         `validate:"required,min=18"`,
}

policy OptionalValidation {
    bio         `validate:"max=500"`,
    website     `validate:"url"`,
}

struct UserRegistration with StrictValidation, OptionalValidation {
    email: str,
    password: str,
    age: i32,
    bio: str?,
    website: str?,
}

// Validator can read metadata and apply rules:
fn validate(user: UserRegistration): Result<(), ValidationError> {
    // Read metadata from struct fields
    // Apply validation rules from policy
}
```

---

## ✅ Design Decisions Finalized

### Decision 1: Hybrid Approach ✅

- **Inline tags** for simple cases
- **Policy declarations** for reusable patterns
- **Both supported**, inline overrides policy

### Decision 2: Go-Style Backticks ✅

```vex
field_name  `json:"id" db:"user_id"`
```

- Familiar to Go developers
- Compact and readable
- Easy to parse

### Decision 3: Name-Based Mapping ✅

Policy fields map to struct fields by **field name**.

### Decision 4: `with` Keyword ✅

```vex
struct User with Policy1, Policy2 { ... }
```

Consistent with trait syntax.

### Decision 5: Policy Composition ✅

```vex
policy Child with Parent { ... }
```

Supports inheritance and merging.

---

## Open Questions (For Future Phases)

### Q1: Policy Scope

Should policies be module-scoped?

```vex
policy pub APIModel { ... }     // Public
policy internal DBModel { ... } // Module-private
```

**Decision needed:** Phase 2+

---

### Q2: Policy Parameters

Should policies support generics?

```vex
policy Validated<min_age: i32> {
    age  `validate:"min=${min_age}"`,
}
```

**Decision needed:** Phase 7 (Future)

---

### Q3: Runtime Access

Should metadata be available at runtime?

```vex
fn get_json_name(field: str): str? {
    User::field_metadata(field).get("json")
}
```

**Decision needed:** Phase 6 (Tooling)  
**Recommendation:** Yes, for reflection/serialization

---

### Q4: Key Validation

Free-form keys or predefined?

**Options:**

- A) **Free-form** (any key allowed) - Flexible but error-prone
- B) **Predefined** (json, db, validate only) - Safe but rigid
- C) **Extensible registry** (plugins register keys) - Best of both

**Decision needed:** Phase 1  
**Recommendation:** Start with free-form (A), add registry later (C)

---

## Quick Start Guide

### For Prototypes (Use Inline)

```vex
struct User {
    id: i32 `json:"id"`,
    name: str `json:"name"`,
}
```

### For Production (Use Policies)

```vex
policy APIModel {
    id    `json:"id" db:"user_id"`,
    name  `json:"name"`,
}

struct User with APIModel {
    id: i32,
    name: str,
}
```

### For Customization (Use Hybrid)

```vex
struct Admin with APIModel {
    id: i32 `json:"admin_id"`,  // Override
    name: str,                   // Use policy
}
```

---

## Next Steps

1. ✅ **Design finalized** - Hybrid approach with inline + policy
2. 🔄 **Start Phase 1** - Implement inline tags (2-3 days)
3. 🔄 **Then Phase 2** - Implement policy declarations (3-4 days)
4. 🔄 **Build vex-serde** - Use metadata for JSON serialization
5. 🔄 **Build vex-validator** - Use metadata for validation

**Ready to start implementation!** 🚀
