# Structs and Data Types

**Version:** 0.9.0  
**Last Updated:** November 3, 2025

This document defines struct types and related data structures in the Vex programming language.

---

## Table of Contents

1. [Struct Definitions](#struct-definitions)
2. [Struct Instantiation](#struct-instantiation)
3. [Field Access](#field-access)
4. [Methods on Structs](#methods-on-structs)
5. [Generic Structs](#generic-structs)
6. [Tuple Structs](#tuple-structs)
7. [Unit Structs](#unit-structs)
8. [Memory Layout](#memory-layout)

---

## Struct Definitions

### Basic Syntax

**Syntax**: `struct Name { fields }`

```vex
struct Point {
    x: i32,
    y: i32,
}

struct Person {
    name: string,
    age: i32,
    email: string,
}
```

**Properties**:

- `struct` keyword
- Name in PascalCase (convention)
- Fields in braces with types
- Comma-separated fields
- Nominal typing (name-based, not structural)

### Field Declaration

Each field has name and type:

```vex
struct Rectangle {
    width: i32,
    height: i32,
}
```

**Multiple Fields**:

```vex
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

```vex
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

```vex
struct Mixed {
    integer: i32,
    floating: f64,
    boolean: bool,
    text: string,
    array: [i32; 10],
    tuple: (i32, i32),
}
```

### Struct Tags (Future - Go-style)

**Answer to TODO**: 🟡 Struct tags Medium Priority - Serialization/reflection için gerekli

```vex
// Future syntax proposal:
struct User {
    #[json = "user_id", db = "id"]
    id: u64,

    #[json = "username", validate = "length:3-20"]
    name: string,

    #[json = "email_address", validate = "email"]
    email: string,

    #[json = "-"]  // Skip this field in JSON
    password_hash: string,
}
```

**Use Cases**:

- JSON serialization field mapping
- Database column mapping
- Validation rules
- Documentation generation
- Code generation hints

**Implementation Status**: ❌ Not implemented yet

- Requires attribute syntax `#[...]`
- Needs reflection system
- Medium priority (after collections and iterators)

### Nested Structs

Structs can contain other structs:

```vex
struct Address {
    street: string,
    city: string,
    zip: i32,
}

struct Person {
    name: string,
    address: Address,
}
```

**Example Usage**:

```vex
let addr = Address {
    street: "123 Main St",
    city: "NYC",
    zip: 10001,
};

let person = Person {
    name: "Alice",
    address: addr,
};
```

### Self-Referential Structs (Limited)

Structs cannot directly contain themselves:

```vex
// ERROR: Infinite size
struct Node {
    value: i32,
    next: Node,  // ERROR
}
```

**Use references instead** (future):

```vex
struct Node {
    value: i32,
    next: &Node!,  // OK: Pointer, fixed size
}
```

---

## Struct Instantiation

### Full Initialization

All fields must be provided:

```vex
let point = Point {
    x: 10,
    y: 20,
};
```

**Order doesn't matter**:

```vex
let p1 = Point { x: 10, y: 20 };
let p2 = Point { y: 20, x: 10 };  // Same as p1
```

### Missing Fields (Error)

```vex
// ERROR: Missing field 'y'
let point = Point { x: 10 };
```

All fields must be initialized.

### Field Init Shorthand (Future)

```vex
let x = 10;
let y = 20;
let point = Point { x, y };  // Shorthand for { x: x, y: y }
```

### Update Syntax (Future)

Copy existing struct with some fields changed:

```vex
let p1 = Point { x: 10, y: 20 };
let p2 = Point { x: 30, ..p1 };  // y copied from p1
```

---

## Field Access

### Reading Fields

Use dot notation:

```vex
let point = Point { x: 10, y: 20 };
let x_coord = point.x;  // 10
let y_coord = point.y;  // 20
```

**Nested Access**:

```vex
let person = Person {
    name: "Alice",
    address: Address {
        street: "Main St",
        city: "NYC",
        zip: 10001,
    },
};

let city = person.address.city;  // "NYC"
```

### Writing Fields

Only possible with mutable variables:

```vex
let! point = Point { x: 10, y: 20 };
point.x = 30;  // OK: point is mutable
point.y = 40;  // OK
```

**Immutable Structs**:

```vex
let point = Point { x: 10, y: 20 };
// point.x = 30;  // ERROR: Cannot assign to immutable variable
```

### Field Access Through References

**Immutable Reference**:

```vex
let point = Point { x: 10, y: 20 };
let ref_point: &Point = &point;
let x = ref_point.x;  // OK: Read through reference
```

**Mutable Reference**:

```vex
let! point = Point { x: 10, y: 20 };
let ref_point: &Point! = &point;
ref_point.x = 30;  // OK: Write through mutable reference
```

**Note**: Auto-dereference for field access (future feature)

---

## Methods on Structs

### Method Mutability

**Immutable Methods** (default):

```vex
fn method_name(): return_type {
    // Read-only access: self.field ✅
    // Cannot mutate: self!.field ❌
}
```

**Mutable Methods** (with `!`):

```vex
fn method_name()!: return_type {
    // Read access: self.field ✅
    // Write access: self!.field ✅
}
```

### Inline Methods

```vex
struct Rectangle {
    width: i32,
    height: i32,

    // Immutable method (default)
    fn area(): i32 {
        return self.width * self.height;
    }

    // Immutable method (explicit)
    fn perimeter(): i32 {
        return 2 * (self.width + self.height);
    }

    // Mutable method (explicit !)
    fn scale(factor: i32)! {
        self!.width = self!.width * factor;
        self!.height = self!.height * factor;
    }
}
```

### Golang-Style Methods (Extra Methods Only)

```vex
struct Circle {
    radius: f64,
}

// Extra methods (not in trait) can be external
fn (c: &Circle) area(): f64 {
    return 3.14159 * c.radius * c.radius;
}

fn (c: &Circle) circumference(): f64 {
    return 2.0 * 3.14159 * c.radius;
}

// Mutable extra method
fn (c: &Circle!) set_radius(new_radius: f64)! {
    c!.radius = new_radius;
}
```

### Method Calls

**Immutable method calls**:

```vex
let rect = Rectangle { width: 10, height: 20 };
let a = rect.area();        // 200
let p = rect.perimeter();   // 60
```

**Mutable method calls**:

```vex
let! circle = Circle { radius: 5.0 };
circle.set_radius(10.0)!;   // ! required
```

### Trait Methods vs Extra Methods

**Trait Methods**: MUST be in struct body

```vex
trait Shape {
    fn area(): f64;
    fn scale(factor: f64)!;
}

struct Rectangle impl Shape {
    width: f64,
    height: f64,

    // Trait methods MUST be here
    fn area(): f64 {
        return self.width * self.height;
    }

    fn scale(factor: f64)! {
        self!.width = self!.width * factor;
        self!.height = self!.height * factor;
    }
}

// ❌ ERROR: Trait methods cannot be external
fn (r: &Rectangle) area(): f64 {
    return r.width * r.height;
}
```

**Extra Methods**: Can be external

```vex
// ✅ OK: Extra methods can be external
fn (rect: &Rectangle) diagonal(): f64 {
    return sqrt(rect.width * rect.width + rect.height * rect.height);
}
```
