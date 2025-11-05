# Policy vs Trait - Syntax Clarification

**Date:** November 5, 2025  
**Issue:** Avoid confusion between `with Policy` and `impl Trait` syntax

---

## ❌ Problem: Ambiguous Syntax

### Current Design (Potential Confusion)

```vex
// Policy application
struct User with Serializable {
    id: i32,
}

// Trait implementation
struct User impl Serializable {
    fn (&self)serialize(): str { ... }
}
```

**Problem:** Both use similar names (Serializable), but one is metadata, one is behavior!

---

## ✅ Solution: Clear Syntax Separation

### Option A: Different Keywords (RECOMMENDED)

```vex
// Policy: with keyword
struct User with APIModel, ValidationRules {
    id: i32,
    name: str,
}

// Trait: impl keyword
struct User impl Drawable, Cloneable {
    fn (&self)draw() { ... }
    fn (&self)clone(): User { ... }
}

// Both together: Order matters for clarity
struct User
    with APIModel, ValidationRules    // Metadata first
    impl Drawable, Cloneable {        // Behavior second

    id: i32,
    name: str,

    fn (&self)draw() { ... }
    fn (&self)clone(): User { ... }
}
```

**Pros:**

- ✅ Clear separation: `with` = metadata, `impl` = behavior
- ✅ Different keywords prevent confusion
- ✅ Can use both on same struct
- ✅ Order is explicit (with before impl)

---

### Option B: Namespacing Convention

```vex
// Policy: Use *Policy suffix
policy APIModelPolicy {
    id  `json:"id"`,
}

// Trait: Use *able suffix
trait Drawable {
    fn (&self)draw();
}

struct User with APIModelPolicy impl Drawable {
    id: i32,
    fn (&self)draw() { ... }
}
```

**Pros:**

- ✅ Naming convention makes intent clear
- ✅ No syntax ambiguity

**Cons:**

- ❌ Relies on convention, not enforced

---

### Option C: Policy Keyword Instead of With

```vex
// Policy: policy keyword
struct User policy(APIModel, ValidationRules) {
    id: i32,
}

// Trait: impl keyword
struct User impl Drawable {
    fn (&self)draw() { ... }
}

// Both together
struct User
    policy(APIModel, ValidationRules)
    impl Drawable {

    id: i32,
    fn (&self)draw() { ... }
}
```

**Pros:**

- ✅ `policy()` is explicit
- ✅ No confusion with traits

**Cons:**

- ❌ More verbose
- ❌ Function-call syntax inconsistent with Vex style

---

## ✅ RECOMMENDED: Option A with Clear Rules

### Final Syntax Rules

#### Rule 1: `with` for Metadata (Policies)

```vex
struct User with APIModel {
    id: i32,
}
```

- Policies provide **metadata** (tags)
- No method implementations
- Just field annotations

---

#### Rule 2: `impl` for Behavior (Traits)

```vex
struct User impl Drawable {
    fn (&self)draw() {
        println("Drawing user");
    }
}
```

- Traits provide **behavior contracts**
- Must implement methods
- Can have trait fields (contracts)

---

#### Rule 3: Combining Both - Clear Order

```vex
struct User
    with APIModel          // Metadata first
    impl Drawable {        // Behavior second

    // Fields
    id: i32,
    name: str,

    // Methods (from trait)
    fn (&self)draw() { ... }
}
```

**Order:** `with` ALWAYS before `impl`  
**Compiler:** Error if reversed

---

#### Rule 4: Multiple Policies and Traits

```vex
struct User
    with APIModel, ValidationRules, TimestampPolicy    // All policies
    impl Drawable, Cloneable, Serializable {           // All traits

    id: i32,

    // Implement all trait methods
    fn (&self)draw() { ... }
    fn (&self)clone(): User { ... }
    fn (&self)serialize(): str { ... }
}
```

---

## Semantic Differences

### Policy (with)

- **Purpose:** Attach metadata to fields
- **Content:** Backtick strings with key:value pairs
- **Methods:** NONE (metadata only)
- **Example Use:** JSON serialization, database mapping, validation rules
- **Runtime:** Metadata read by codegen/reflection

### Trait (impl)

- **Purpose:** Define behavior contract
- **Content:** Method signatures (and optionally fields)
- **Methods:** REQUIRED implementations
- **Example Use:** Draw, Clone, Display, Iterator
- **Runtime:** Actual method calls

---

## Examples: Clear Intent

### Example 1: REST API Model

```vex
// Policy for metadata
policy RESTModel {
    id          `json:"id" db:"user_id"`,
    email       `json:"email" validate:"email"`,
    created_at  `json:"createdAt"`,
}

// Trait for behavior
trait Serializable {
    fn (&self)to_json(): str;
    fn from_json(json: str): Self;
}

// Struct with both
struct User
    with RESTModel                    // Metadata: how to serialize fields
    impl Serializable {               // Behavior: how to convert to/from JSON

    id: i32,
    email: str,
    created_at: i64,

    fn (&self)to_json(): str {
        // Use metadata from RESTModel to generate JSON
        format!(r#"{{"id":{},"email":"{}","createdAt":{}}}"#,
                self.id, self.email, self.created_at)
    }

    fn from_json(json: str): User {
        // Parse JSON using metadata
        // ...
    }
}
```

**Clear separation:**

- `RESTModel` policy → Field metadata (what names to use)
- `Serializable` trait → Conversion logic (how to do it)

---

### Example 2: Database ORM

```vex
policy DBModel {
    id      `db_primary:"true" auto_increment:"true"`,
    email   `db_column:"email_address" db_index:"true"`,
}

trait Persistable {
    fn (&self)save(): Result<(), DBError>;
    fn load(id: i32): Result<Self, DBError>;
}

struct User
    with DBModel              // Metadata: how to map to database
    impl Persistable {        // Behavior: how to save/load

    id: i32,
    email: str,

    fn (&self)save(): Result<(), DBError> {
        // Use DBModel metadata to generate SQL
        db.execute("INSERT INTO users ...")
    }

    fn load(id: i32): Result<User, DBError> {
        // Use DBModel metadata to map columns
        db.query("SELECT * FROM users WHERE id = ?", id)
    }
}
```

---

### Example 3: Validation Framework

```vex
policy ValidationRules {
    email    `validate:"required,email"`,
    age      `validate:"required,min=18,max=120"`,
    password `validate:"required,min=8"`,
}

trait Validatable {
    fn (&self)validate(): Result<(), ValidationError>;
}

struct UserRegistration
    with ValidationRules      // Metadata: what rules to apply
    impl Validatable {        // Behavior: how to validate

    email: str,
    age: i32,
    password: str,

    fn (&self)validate(): Result<(), ValidationError> {
        // Read ValidationRules metadata and apply
        for field in Self.fields() {
            let rules = field.metadata("validate");
            // Apply rules...
        }
        Ok(())
    }
}
```

---

## Compiler Rules

### Rule 1: Keyword Order Enforced

```vex
// ✅ CORRECT
struct User with Policy impl Trait { ... }

// ❌ ERROR: "Policy ('with') must come before trait ('impl')"
struct User impl Trait with Policy { ... }
```

---

### Rule 2: Policy Cannot Have Methods

```vex
// ❌ ERROR: "Policy cannot contain method implementations"
policy Invalid {
    id  `json:"id"`,
    fn some_method() { ... }  // Not allowed!
}
```

---

### Rule 3: Trait Must Have Method Signatures

```vex
// ✅ CORRECT
trait Valid {
    fn (&self)method();
}

// ❌ WARNING: "Trait has no methods (consider using policy instead)"
trait Empty {
    // No methods - probably should be a policy
}
```

---

### Rule 4: No Name Collision

```vex
policy Serializable {
    id  `json:"id"`,
}

trait Serializable {  // ❌ ERROR: "Name 'Serializable' already used by policy"
    fn (&self)serialize(): str;
}

// Solution: Use different names
policy SerializationPolicy { ... }
trait Serializable { ... }
```

---

## Best Practices

### ✅ DO: Use Clear Names

```vex
// Good: Clear intent
policy APIModel { ... }
policy DBSchema { ... }
policy ValidationRules { ... }

trait Drawable { ... }
trait Cloneable { ... }
trait Serializable { ... }
```

---

### ✅ DO: Separate Concerns

```vex
// Metadata in policy
policy UserMetadata {
    id  `json:"id" db:"user_id"`,
}

// Behavior in trait
trait UserBehavior {
    fn (&self)display();
}
```

---

### ❌ DON'T: Mix Metadata and Behavior Names

```vex
// Bad: Confusing
policy Serializable { ... }  // Metadata
trait Serializable { ... }   // Behavior - same name!

// Good: Different names
policy SerializationFormat { ... }
trait Serializable { ... }
```

---

## Summary Table

| Aspect          | Policy (`with`)         | Trait (`impl`)         |
| --------------- | ----------------------- | ---------------------- |
| **Purpose**     | Metadata                | Behavior               |
| **Contains**    | Backtick tags           | Method signatures      |
| **Requires**    | Nothing                 | Method implementations |
| **Example**     | `json:"id"`             | `fn draw()`            |
| **Use Case**    | Serialization format    | Drawing logic          |
| **Keyword**     | `with`                  | `impl`                 |
| **Order**       | First                   | Second                 |
| **Can Combine** | Yes (multiple policies) | Yes (multiple traits)  |

---

## ✅ FINAL DECISION

**Syntax:**

```vex
struct TypeName
    with Policy1, Policy2       // Optional: Metadata
    impl Trait1, Trait2 {       // Optional: Behavior

    // Fields
    field: type,

    // Methods (from traits)
    fn method() { ... }
}
```

**Rules:**

1. `with` for policies (metadata)
2. `impl` for traits (behavior)
3. `with` ALWAYS before `impl` (if both present)
4. Both are optional
5. No name collisions between policies and traits
6. Policies cannot have methods
7. Traits must have methods

**This design ensures:**

- ✅ Clear separation of concerns
- ✅ No syntax ambiguity
- ✅ Explicit > implicit
- ✅ Extensible for future features
