# Structs and Data Types

**Version:** 0.1.0 
**Last Updated:** November 3, 2025

This document defines struct types and related data structures in the Vex programming language.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1
7. \1
8. \1

---

## Struct Definitions

### Basic Syntax

**Syntax**: `struct Name { fields }`

[10 lines code: ```vex]

**Properties**:

- `struct` keyword
- Name in PascalCase (convention)
- Fields in braces with types
- Comma-separated fields
- Nominal typing (name-based, not structural)

### Field Declaration

Each field has name and type:

``````vex
struct Rectangle {
    width: i32,
    height: i32,
}
```

**Multiple Fields**:

``````vex
struct User {
    id: u64,
    username: string,
    email: string,
    age: i32,
    is_active: bool,
}
```

### Struct Tags (Go-style)

Vex supports Go-style struct tags for metadata:

``````vex
struct User {
    id: u64        `json:"id" db:"pk"`,
    username: string `json:"username" db:"username"`,
    email: string   `json:"email" db:"email"`,
    age: i32        `json:"age"`,
    is_active: bool `json:"is_active"`,
}
```

**Syntax**: Backtick-enclosed string literals after field declarations

**Use Cases**:

- JSON serialization/deserialization
- Database mapping
- Validation rules
- API documentation

**Properties**:

- Ignored by compiler (metadata only)
- Available at runtime via reflection
- Multiple tags separated by spaces
- Convention: `key:"value"` format

**Different Types**:

``````vex
struct Mixed {
    integer: i32,
    floating: f64,
    boolean: bool,
    text: string,
    array: [i32; 10],
    tuple: (i32, i32),
}
```

### Struct Tags (Go-style)

Vex supports Go-style backtick struct tags for metadata:

``````vex
struct User {
    id: u64        `json:"id" db:"pk"`,
    username: string `json:"username" db:"username"`,
    email: string   `json:"email" db:"email"`,
    age: i32        `json:"age"`,
    is_active: bool `json:"is_active"`,
}
```

**Syntax**: Backtick-enclosed string literals after field type

**Use Cases**:

- JSON serialization field mapping
- Database column mapping
- Validation rules
- API documentation
- Reflection metadata

**Implementation Status**: ✅ Fully implemented

- Struct tags ARE parsed and stored in AST (`Field.tag`)
- Metadata available in compiler
- **IMPORTANT**: Vex does NOT use Rust-style `#[attribute]` syntax
- Runtime reflection builtins: `typeof`, `type_id`, `type_size`, `type_align`, `is_*_type` functions
- Policy system provides rich metadata annotations

### Nested Structs

Structs can contain other structs:

[10 lines code: ```vex]

**Example Usage**:

[10 lines code: ```vex]

### Self-Referential Structs (Limited)

Structs cannot directly contain themselves:

``````vex
// ERROR: Infinite size
struct Node {
    value: i32,
    next: Node,  // ERROR
}
```

**Use references instead** (future):

``````vex
struct Node {
    value: i32,
    next: &Node!,  // OK: Pointer, fixed size
}
```

---

## Struct Instantiation

### Full Initialization

All fields must be provided:

``````vex
let point = Point {
    x: 10,
    y: 20,
};
```

**Order doesn't matter**:

``````vex
let p1 = Point { x: 10, y: 20 };
let p2 = Point { y: 20, x: 10 };  // Same as p1
```

### Missing Fields (Error)

``````vex
// ERROR: Missing field 'y'
let point = Point { x: 10 };
```

All fields must be initialized.

### Field Init Shorthand (Future)

``````vex
let x = 10;
let y = 20;
let point = Point { x, y };  // Shorthand for { x: x, y: y }
```

### Update Syntax (Future)

Copy existing struct with some fields changed:

``````vex
let p1 = Point { x: 10, y: 20 };
let p2 = Point { x: 30, ..p1 };  // y copied from p1
```

---

## Field Access

### Reading Fields

Use dot notation:

``````vex
let point = Point { x: 10, y: 20 };
let x_coord = point.x;  // 10
let y_coord = point.y;  // 20
```

**Nested Access**:

[10 lines code: ```vex]

### Writing Fields

Only possible with mutable variables:

``````vex
let! point = Point { x: 10, y: 20 };
point.x = 30;  // OK: point is mutable
point.y = 40;  // OK
```

**Immutable Structs**:

``````vex
let point = Point { x: 10, y: 20 };
// point.x = 30;  // ERROR: Cannot assign to immutable variable
```

### Field Access Through References

**Immutable Reference**:

``````vex
let point = Point { x: 10, y: 20 };
let ref_point: &Point = &point;
let x = ref_point.x;  // OK: Read through reference
```

**Mutable Reference**:

``````vex
let! point = Point { x: 10, y: 20 };
let ref_point: &Point! = &point;
ref_point.x = 30;  // OK: Write through mutable reference
```

**Note**: Auto-dereference for field access (future feature)

---

## Methods on Structs

Vex uses a hybrid model for method mutability. See `05_Functions_and_Methods.md` for the full specification.

### Inline Methods (in `struct` or `trait`)

- **Declaration**: `fn method_name()!` for mutable, `fn method_name()` for immutable.
- **Behavior**: A mutable method can modify `self`.
- **Call**: `object.method_name()` (no `!` at call site). The compiler ensures a mutable method is only called on a mutable (`let!`) variable.

[22 lines code: ```vex]

### External Methods (Golang-Style)

- **Declaration**: `fn (self: &MyType!) method_name()` for mutable, `fn (self: &MyType) method_name()` for immutable.
- **Behavior**: A mutable method can modify `self`.
- **Call**: `object.method_name()` (no `!` at call site).

[17 lines code: ```vex]

### Trait Methods vs Extra Methods

**Trait Methods**: MUST be in struct body

[24 lines code: ```vex]

**Extra Methods**: Can be external

``````vex
// ✅ OK: Extra methods can be external
fn (rect: &Rectangle) diagonal(): f64 {
    return sqrt(rect.width * rect.width + rect.height * rect.height);
}
```
