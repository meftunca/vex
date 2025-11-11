# Functions and Methods

**Version:** 0.1.0 
**Last Updated:** November 3, 2025

This document defines functions, methods, and related concepts in the Vex programming language.

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

## Function Declarations

### Basic Syntax

**Syntax**: `fn name(parameters): return_type { body }`

[11 lines code: ```vex]

**Components**:

- `fn` keyword
- Function name (identifier)
- Parameter list in parentheses
- Optional return type after colon
- Function body in braces

### Simple Functions

**No Parameters**:

``````vex
fn hello(): i32 {
    return 42;
}
```

**No Return Value** (returns nil):

``````vex
fn print_message() {
    // Implicit return nil
}
```

**Single Expression** (explicit return required):

``````vex
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

``````vex
fn calculate_total(items: [i32; 10]): i32 { }
fn is_prime(n: i32): bool { }
fn get_user_name(): string { }
fn validate_input(data: string): bool { }
```

### Entry Point

The `main` function is the program entry point:

``````vex
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

### Kural 1: Inline Metodlar (Struct ve Trait İçinde)

**Amaç:** Kod tekrarını önlemek ve `struct`/`trait` tanımlarını temiz tutmak.

- **Tanımlama:** Metodun `mutable` olduğu, imzanın sonuna eklenen `!` işareti ile belirtilir. Receiver (`self`) bu stilde implisittir ve yazılmaz.
 - `fn method_name()!`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
 - `self.field = new_value`
- **Çağrı:** `Mutable` metodlar, çağrı anında `!` **kullanılmadan** çağrılır. Derleyici, metodun sadece `let!` ile tanımlanmış `mutable` bir nesne üzerinde çağrıldığını compile-time'da kontrol eder.
 - `object.method_name()`

**Örnek:**

[22 lines code: ```vex]

### Kural 2: External Metodlar (Golang-Style)

**Amaç:** Metodun hangi veri tipi üzerinde çalıştığını ve `mutable` olup olmadığını receiver tanımında açıkça belirtmek.

- **Tanımlama:** Metodun `mutable` olduğu, receiver tanımındaki `&Type!` ifadesi ile belirtilir. Metod imzasının sonunda `!` **kullanılmaz**.
 - `fn (self: &MyType!) method_name()`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
 - `self.field = new_value`
- **Çağrı:** Çağrı sırasında `!` işareti **kullanılmaz**.
 - `object.method_name()`

**Örnek:**

[22 lines code: ```vex]

### Trait Method Implementation

[18 lines code: ```vex]

**Error**: Trait methods cannot be external

``````vex
// ❌ COMPILE ERROR: Trait method cannot be external
fn (u: &User) show() {
    print(u.name);
}
```
