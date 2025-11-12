# Functions and Methods

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines functions, methods, and related concepts in the Vex programming language.

---

## Table of Contents

1. [Function Declarations](#function-declarations)
2. [Method Definitions](#method-definitions)
3. [Parameters and Arguments](#parameters-and-arguments)
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

**Amaç:** Kod tekrarını önlemek ve `struct`/`trait` tanımlarını temiz tutmak.

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
trait Display {
    fn show();        // Immutable contract
    fn update()!;     // Mutable contract
}

struct User impl Display {
    name: string,
    age: i32,

    // Contract methods MUST be here (in struct body)
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
