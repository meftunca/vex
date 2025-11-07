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

### Inline Methods

Define methods inside struct body:

```vex
struct Rectangle {
    width: i32,
    height: i32,

    fn (self: &Rectangle) area(): i32 {
        return self.width * self.height;
    }

    fn (self: &Rectangle) perimeter(): i32 {
        return 2 * (self.width + self.height);
    }

    fn (self: &Rectangle!) scale(factor: i32) {
        self.width = self.width * factor;
        self.height = self.height * factor;
    }
}
```

**Properties**:

- Methods defined in struct body
- Access fields via `self`
- Immutable receiver: `&Type`
- Mutable receiver: `&Type!`

### Golang-Style Methods

Define methods outside struct:

```vex
struct Circle {
    radius: f64,
}

fn (c: &Circle) area(): f64 {
    return 3.14159 * c.radius * c.radius;
}

fn (c: &Circle) circumference(): f64 {
    return 2.0 * 3.14159 * c.radius;
}

fn (c: &Circle!) set_radius(new_radius: f64) {
    c.radius = new_radius;
}
```

**Properties**:

- Methods defined separately
- Receiver is first parameter
- Can use any receiver name (`c`, `self`, etc.)
- Flexible organization

### Method Calls

```vex
let rect = Rectangle { width: 10, height: 20 };
let a = rect.area();        // 200
let p = rect.perimeter();   // 60

let! circle = Circle { radius: 5.0 };
circle.set_radius(10.0);
```

### Associated Functions (Future)

Functions without `self` parameter:

```vex
struct Point {
    x: i32,
    y: i32,

    fn new(x: i32, y: i32): Point {
        return Point { x, y };
    }

    fn origin(): Point {
        return Point { x: 0, y: 0 };
    }
}

let p1 = Point::new(10, 20);
let p2 = Point::origin();
```

---

## Generic Structs

### Single Type Parameter

```vex
struct Box<T> {
    value: T,
}

let int_box = Box<i32> { value: 42 };
let str_box = Box<string> { value: "hello" };
```

### Multiple Type Parameters

```vex
struct Pair<T, U> {
    first: T,
    second: U,
}

let pair1 = Pair<i32, string> {
    first: 42,
    second: "answer",
};

let pair2 = Pair<f64, bool> {
    first: 3.14,
    second: true,
};
```

### Generic Methods

```vex
struct Container<T> {
    value: T,

    fn (self: &Container<T>) get(): T {
        return self.value;
    }

    fn (self: &Container<T>!) set(new_value: T) {
        self.value = new_value;
    }
}
```

### Monomorphization

Each type instantiation generates specialized code:

```vex
let c1 = Container<i32> { value: 42 };
let c2 = Container<string> { value: "hello" };

// Compiler generates:
// struct Container_i32 { value: i32 }
// struct Container_string { value: string }
```

---

## Tuple Structs

### Definition

Structs with unnamed fields:

```vex
struct Point(i32, i32);
struct Color(u8, u8, u8);
struct Wrapper<T>(T);
```

**Properties**:

- Fields accessed by index (`.0`, `.1`, etc.)
- Lighter syntax than regular structs
- Still nominal types (name-based)

### Usage (Future)

```vex
let point = Point(10, 20);
let x = point.0;  // 10
let y = point.1;  // 20

let color = Color(255, 0, 0);  // Red
let red = color.0;    // 255
let green = color.1;  // 0
let blue = color.2;   // 0
```

### Destructuring (Future)

```vex
let Point(x, y) = point;
// x = 10, y = 20

let Color(r, g, b) = color;
// r = 255, g = 0, b = 0
```

---

## Unit Structs

### Definition

Structs with no fields:

```vex
struct Unit;
struct Marker;
```

**Properties**:

- Zero-sized types (no memory)
- Used for type-level programming
- Marker types for traits

### Usage (Future)

```vex
let unit = Unit;
let marker = Marker;

// Trait implementation
struct PhantomData<T>;

let phantom = PhantomData<i32>;
```

---

## Memory Layout

### Field Order

Fields are laid out in memory in declaration order:

```vex
struct Example {
    a: i8,    // Offset: 0
    b: i32,   // Offset: 4 (aligned)
    c: i16,   // Offset: 8
}
```

**Padding for Alignment**:

```
Memory Layout:
[a][___][bbbb][cc__]
 0  1-3  4-7  8-9,10-11 (padding)
```

### Size Calculation

Size = sum of field sizes + padding:

```vex
struct Small {
    x: i32,   // 4 bytes
    y: i32,   // 4 bytes
}
// Total: 8 bytes (no padding needed)

struct Padded {
    a: i8,    // 1 byte
    b: i32,   // 4 bytes (needs 4-byte alignment)
    c: i8,    // 1 byte
}
// Total: 12 bytes (with padding)
```

### Alignment

Struct alignment = largest field alignment:

```vex
struct Aligned {
    a: i8,    // Alignment: 1
    b: i64,   // Alignment: 8
    c: i16,   // Alignment: 2
}
// Struct alignment: 8 (from i64)
```

### Optimization (Future)

Compiler may reorder fields for optimal packing:

```vex
#[repr(C)]  // C-compatible layout (no reordering)
struct Fixed {
    a: i8,
    b: i32,
    c: i8,
}

// Without repr(C), compiler can optimize:
// Reordered: b (i32), a (i8), c (i8) -> 6 bytes + padding
```

---

## Examples

### Basic Struct

```vex
struct Point {
    x: i32,
    y: i32,
}

fn main(): i32 {
    let p = Point { x: 10, y: 20 };
    return p.x + p.y;  // 30
}
```

### Struct with Methods

```vex
struct Counter {
    value: i32,

    fn (self: &Counter) get(): i32 {
        return self.value;
    }

    fn (self: &Counter!) increment() {
        self.value = self.value + 1;
    }

    fn (self: &Counter!) add(amount: i32) {
        self.value = self.value + amount;
    }
}

fn main(): i32 {
    let! counter = Counter { value: 0 };
    counter.increment();
    counter.increment();
    counter.add(3);
    return counter.get();  // 5
}
```

### Nested Structs

```vex
struct Vector2D {
    x: f64,
    y: f64,
}

struct Particle {
    position: Vector2D,
    velocity: Vector2D,
}

fn main(): i32 {
    let particle = Particle {
        position: Vector2D { x: 0.0, y: 0.0 },
        velocity: Vector2D { x: 1.0, y: 2.0 },
    };

    let vx = particle.velocity.x;  // 1.0
    return 0;
}
```

### Generic Struct

```vex
struct Pair<T, U> {
    first: T,
    second: U,

    fn (self: &Pair<T, U>) get_first(): T {
        return self.first;
    }
}

fn main(): i32 {
    let pair = Pair<i32, string> {
        first: 42,
        second: "answer",
    };

    return pair.get_first();  // 42
}
```

### Mutable Fields

```vex
struct Player {
    health: i32,
    score: i32,
}

fn damage_player(p: &Player!, amount: i32) {
    p.health = p.health - amount;
}

fn add_score(p: &Player!, points: i32) {
    p.score = p.score + points;
}

fn main(): i32 {
    let! player = Player { health: 100, score: 0 };
    damage_player(&player, 20);
    add_score(&player, 50);
    return player.health + player.score;  // 80 + 50 = 130
}
```

---

## Best Practices

### 1. Use Descriptive Names

```vex
// Good: Clear purpose
struct UserAccount {
    username: string,
    email: string,
    created_at: i64,
}

// Bad: Vague
struct Data {
    s1: string,
    s2: string,
    n: i64,
}
```

### 2. Group Related Fields

```vex
// Good: Logical grouping
struct Config {
    // Server settings
    host: string,
    port: u16,

    // Security settings
    use_tls: bool,
    cert_path: string,
}
```

### 3. Keep Structs Focused

```vex
// Good: Single responsibility
struct User {
    id: u64,
    name: string,
    email: string,
}

struct Session {
    user_id: u64,
    token: string,
    expires_at: i64,
}

// Bad: Too many responsibilities
struct Everything {
    user_id: u64,
    user_name: string,
    session_token: string,
    cart_items: [Item],
    preferences: [Setting],
    // ...
}
```

### 4. Prefer Immutability

```vex
// Good: Immutable by default
let point = Point { x: 10, y: 20 };

// Only mutable when necessary
let! counter = Counter { value: 0 };
counter.increment();
```

### 5. Use Methods for Operations

```vex
// Good: Methods encapsulate logic
struct Rectangle {
    width: i32,
    height: i32,

    fn (self: &Rectangle) area(): i32 {
        return self.width * self.height;
    }
}

// Bad: External function
fn calculate_area(r: &Rectangle): i32 {
    return r.width * r.height;
}
```

---

## Struct Features Summary

| Feature         | Syntax                   | Status     | Example            |
| --------------- | ------------------------ | ---------- | ------------------ |
| Basic Struct    | `struct Name { fields }` | ✅ Working | `Point { x, y }`   |
| Field Access    | `obj.field`              | ✅ Working | `p.x`              |
| Inline Methods  | `fn (self: &T) { }`      | ✅ Working | Inside struct body |
| Golang Methods  | `fn (r: &T) { }`         | ✅ Working | Outside struct     |
| Generic Structs | `struct Name<T> { }`     | ✅ Working | `Box<i32>`         |
| Tuple Structs   | `struct Name(T, U)`      | 🚧 Parsed  | Access `.0`, `.1`  |
| Unit Structs    | `struct Name;`           | 🚧 Future  | Zero-sized         |
| Field Shorthand | `{ x, y }`               | ❌ Future  | Auto `x: x`        |
| Update Syntax   | `{ x, ..old }`           | ❌ Future  | Copy fields        |
| Associated Fn   | `Type::function()`       | ❌ Future  | No `self`          |

---

**Previous**: [06_Control_Flow.md](./06_Control_Flow.md)  
**Next**: [08_Enums.md](./08_Enums.md)

**Maintained by**: Vex Language Team
