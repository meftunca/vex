# Functions and Methods

**Version:** 0.2.0  
**Last Updated:** November 12, 2025

This document defines functions, methods, and related concepts in the Vex programming language.

---

## Table of Contents

1. [Function Declarations](#function-declarations)
2. [Method Definitions](#method-definitions)
3. [Parameters and Arguments](#parameters-and-arguments)
   - [Go-Style Parameter Grouping](#go-style-parameter-grouping) ⭐ NEW
4. [Return Values](#return-values)
5. [Generic Functions](#generic-functions)
6. [Function Overloading](#function-overloading)
7. [Higher-Order Functions](#higher-order-functions)
8. [Special Function Types](#special-function-types)

---

## Function Declarations

### Basic Syntax

**Syntax**: `fn name(parameters): return_type { body }`

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn greet(name: string) {
    // No return type = returns nil (unit)
}

fn main(): i32 {
    return 0;  // Entry point
}
```

**Components**:

- `fn` keyword
- Function name (identifier)
- Parameter list in parentheses
- Optional return type after colon
- Function body in braces

### Simple Functions

**No Parameters**:

```vex
fn hello(): i32 {
    return 42;
}
```

**No Return Value** (returns nil):

```vex
fn print_message() {
    // Implicit return nil
}
```

**Single Expression** (explicit return required):

```vex
fn double(x: i32): i32 {
    return x * 2;
}
```

### Function Naming

**Conventions**:

- `snake_case` for function names
- Descriptive names preferred
- Verbs for actions: `calculate_sum`, `print_result`
- Predicates start with `is_`, `has_`, `can_`: `is_valid`, `has_error`

**Examples**:

```vex
fn calculate_total(items: [i32; 10]): i32 { }
fn is_prime(n: i32): bool { }
fn get_user_name(): string { }
fn validate_input(data: string): bool { }
```

### Entry Point

The `main` function is the program entry point:

```vex
fn main(): i32 {
    return 0;  // Exit code
}
```

**Properties**:

- Must return `i32` (exit code)
- No parameters (command-line args future feature)
- Program execution starts here
- Return 0 for success, non-zero for error

---

## Method Definitions

Vex, metodun tanımlandığı bağlama göre değişen, esnek ve pragmatik bir mutasyon sözdizimi kullanır. Bu "Hibrit Model" olarak adlandırılır.

### Kural 1: Inline Metodlar (Struct ve Contract İçinde)

**Amaç:** Kod tekrarını önlemek ve `struct`/`contract` tanımlarını temiz tutmak.

- **Tanımlama:** Metodun `mutable` olduğu, imzanın sonuna eklenen `!` işareti ile belirtilir. Receiver (`self`) bu stilde implisittir ve yazılmaz.
  - `fn method_name()!`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
  - `self.field = new_value`
- **Çağrı:** `Mutable` metodlar, çağrı anında `!` **kullanılmadan** çağrılır. Derleyici, metodun sadece `let!` ile tanımlanmış `mutable` bir nesne üzerinde çağrıldığını compile-time'da kontrol eder.
  - `object.method_name()`

**Örnek:**

```vex
struct Point {
    x: i32,
    y: i32,

    // Immutable method (implicit self)
    fn distance(): f64 {
        return sqrt(self.x * self.x + self.y * self.y);
    }

    // Mutable method (implicit self)
    fn move_to(new_x: i32, new_y: i32)! {
        self.x = new_x;
        self.y = new_y;
    }
}

// --- Çağrılar ---
let p = Point { x: 10, y: 20 };
let dist = p.distance();

let! p_mut = Point { x: 0, y: 0 };
p_mut.move_to(30, 40); // '!' yok
```

### Kural 2: External Metodlar (Golang-Style)

**Amaç:** Metodun hangi veri tipi üzerinde çalıştığını ve `mutable` olup olmadığını receiver tanımında açıkça belirtmek.

- **Tanımlama:** Metodun `mutable` olduğu, receiver tanımındaki `&Type!` ifadesi ile belirtilir. Metod imzasının sonunda `!` **kullanılmaz**.
  - `fn (self: &MyType!) method_name()`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
  - `self.field = new_value`
- **Çağrı:** Çağrı sırasında `!` işareti **kullanılmaz**.
  - `object.method_name()`

**Örnek:**

```vex
struct Rectangle {
    width: i32,
    height: i32,
}

// Immutable external method
fn (r: &Rectangle) area(): i32 {
    return r.width * r.height;
}

// Mutable external method
fn (r: &Rectangle!) scale(factor: i32) {
    r.width = r.width * factor;
    r.height = r.height * factor;
}

// --- Çağrılar ---
let rect = Rectangle { width: 10, height: 5 };
let a = rect.area();

let! rect_mut = Rectangle { width: 10, height: 5};
rect_mut.scale(2); // '!' yok
```

### Contract Method Implementation

```vex
contract Display {
    show();        // ✅ No 'fn' prefix in contract declarations
    update()!;     // Mutable contract method
}

struct User impl Display {
    name: string,
    age: i32,

    // Contract methods MUST be implemented here (in struct body)
    fn show() {
        print(self.name, " - ", self.age);
    }

    fn update()! {
        self.age = self.age + 1;
    }
}
```

**Error**: Contract methods cannot be external

```vex
// ❌ COMPILE ERROR: Contract method cannot be external
fn (u: &User) show() {
    print(u.name);
}
```

---

## Parameters and Arguments

### Basic Parameter Syntax

Parameters are declared with a name and type, separated by colon:

```vex
fn add(x: i32, y: i32): i32 {
    return x + y;
}

fn greet(name: string, age: i32) {
    print("Hello ", name, ", age ", age);
}
```

### Go-Style Parameter Grouping

⭐ **NEW in v0.2.0**: Consecutive parameters of the same type can be grouped together.

**Syntax**: `(name1, name2, name3: type)`

```vex
// Traditional syntax (still supported)
fn add(a: i32, b: i32, c: i32): i32 {
    return a + b + c;
}

// Go-style grouping (new!)
fn add(a, b, c: i32): i32 {
    return a + b + c;
}
```

Both syntaxes are equivalent and produce identical AST nodes.

**Multiple Groups**:

```vex
fn process(x, y, z: f64, name, tag: string): void {
    let sum = x + y + z;
    println(name, ": ", tag, " = ", sum);
}
```

**Mixed Parameters**:

```vex
fn compute(a, b: i32, factor: f64, c, d: i32): f64 {
    let sum = a + b + c + d;
    return (sum as f64) * factor;
}
```

**In Methods**:

```vex
struct Point {
    x: f64,
    y: f64,

    // Grouping works in methods
    distance_to(x1, y1: f64): f64 {
        let dx = self.x - x1;
        let dy = self.y - y1;
        return sqrt(dx * dx + dy * dy);
    }
}

// Also in external methods
fn (p: &Point!) translate(dx, dy: f64) {
    p.x = p.x + dx;
    p.y = p.y + dy;
}
```

**In Contracts**:

```vex
contract Geometry {
    distance(x1, y1, x2, y2: f64): f64;
    translate(dx, dy: f64)!;
}
```

**Benefits**:

- ✅ Reduces repetition for same-typed parameters
- ✅ Cleaner, more readable function signatures
- ✅ Familiar to Go developers
- ✅ Purely syntactic sugar (no runtime overhead)
- ✅ Optional - traditional syntax still supported

**Implementation Note**: The parser automatically expands grouped parameters to individual `Param` AST nodes during parsing, so the rest of the compiler sees fully expanded parameters.

### Parameter Passing

Vex uses **pass-by-value** semantics by default:

```vex
fn modify(x: i32) {
    x = 10;  // Only modifies local copy
}

let y = 5;
modify(y);
// y is still 5
```

For reference semantics, use pointers or references (see [21_Mutability_and_Pointers.md](21_Mutability_and_Pointers.md)).

### Default Parameter Values

⭐ **NEW in v0.2.0**: Parameters can have default values.

**Syntax**: `parameter: type = default_expression`

```vex
// Simple default value
fn greet(name: string = "World") {
    print("Hello, ", name, "!");
}

// Multiple defaults
fn create_point(x: i32 = 0, y: i32 = 0): Point {
    return Point { x: x, y: y };
}

// Mixed: required and optional parameters
fn add_numbers(a: i32, b: i32 = 10, c: i32 = 20): i32 {
    return a + b + c;
}

// With parameter grouping
fn process(x, y: f64 = 1.0): f64 {
    return x * y;
}
```

**Calling with defaults**:

```vex
// Use all defaults
greet();  // "Hello, World!"

// Override some defaults
create_point(5);  // Point { x: 5, y: 0 }

// Override all
create_point(5, 10);  // Point { x: 5, y: 10 }

// Mixed parameters
add_numbers(1);        // 1 + 10 + 20 = 31
add_numbers(1, 2);     // 1 + 2 + 20 = 23
add_numbers(1, 2, 3);  // 1 + 2 + 3 = 6
```

**Rules**:

- Default values can be any compile-time constant expression
- Parameters with defaults must come after required parameters
- When calling, you can omit trailing parameters with defaults
- You cannot skip a parameter in the middle (no named arguments yet)

**Examples**:

```vex
// ✅ Valid
fn foo(a: i32, b: i32 = 10) { }
fn bar(x: i32, y: i32 = 5, z: i32 = 3) { }

// ❌ Invalid: default before required
fn baz(a: i32 = 10, b: i32) { }  // Compile error

// Calling
foo(1);     // OK: a=1, b=10
foo(1, 2);  // OK: a=1, b=2

bar(1);        // OK: x=1, y=5, z=3
bar(1, 2);     // OK: x=1, y=2, z=3
bar(1, 2, 3);  // OK: x=1, y=2, z=3
```

**Implementation**: The compiler automatically fills in missing arguments with their default expressions during code generation. This is a zero-cost abstraction - no runtime overhead.

### Variadic Parameters

✅ **Implemented in v0.2.0**: Functions can accept variable number of arguments.

**Syntax**: `parameter_name: ...Type`

```vex
// Simple variadic
fn sum(base: i32, numbers: ...i32): i32 {
    // numbers is variadic - can accept 0 or more i32 values
    return base;  // TODO: iterate over numbers when runtime support added
}

// Variadic with defaults
fn greet_many(prefix: string = "Hello", names: ...string) {
    print(prefix, " to everyone!");
}

// Only variadic parameter
fn count_all(items: ...i32): i32 {
    // Would return count of items
    return 0;
}
```

**Calling variadic functions**:

```vex
// Pass multiple arguments
sum(10, 1, 2, 3, 4, 5);

// Combine defaults and variadic
greet_many("Hi", "Alice", "Bob", "Charlie");

// Use default for regular param
greet_many("World");  // Uses "Hello" default

// Pass many variadic args
count_all(1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
```

**Rules**:

- ✅ Variadic parameter must be the LAST parameter
- ✅ Only ONE variadic parameter per function
- ✅ Can combine with default parameters
- ✅ Variadic parameters can accept zero or more arguments
- ⚠️ Runtime iteration over variadic args not yet implemented
- ⚠️ Currently used mainly for FFI (C variadic functions)

**Examples**:

```vex
// ✅ Valid
fn foo(a: i32, items: ...string) { }
fn bar(prefix: string = "default", args: ...i32) { }

// ❌ Invalid: variadic not last
fn baz(items: ...i32, suffix: string) { }  // Compile error

// ❌ Invalid: multiple variadic
fn qux(items1: ...i32, items2: ...string) { }  // Compile error
```

**Current Status**:

- ✅ Parser support: `name: ...Type` syntax
- ✅ Type checking: variadic type validation
- ✅ Codegen: accepts variable argument count
- ⏳ Runtime: iteration over variadic args (future feature)

**Future**: Access variadic arguments via slice or iterator:

```vex
// Future syntax (not yet implemented)
fn sum(numbers: ...i32): i32 {
    let! total = 0;
    for num in numbers {  // Iterate over variadic args
        total = total + num;
    }
    return total;
}
```

---

## Return Values
