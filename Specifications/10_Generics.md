# Generics (Parametric Polymorphism)

**Version:** 0.1.0  
**Last Updated:** November 3, 2025

This document defines the generic type system in Vex, enabling code reuse through parametric polymorphism.

---

## Table of Contents

1. [Generic Functions](#generic-functions)
2. [Generic Structs](#generic-structs)
3. [Generic Enums](#generic-enums)
4. [Generic Traits](#generic-traits)
5. [Type Constraints](#type-constraints)
6. [Monomorphization](#monomorphization)

---

## Generic Functions

### Basic Syntax

**Syntax**: `fn name<T>(params): return_type`

```vex
fn identity<T>(x: T): T {
    return x;
}
```

**Usage**:

```vex
let num = identity<i32>(42);
let text = identity<string>("hello");
let flag = identity<bool>(true);
```

### Type Inference

Type parameters can be inferred from arguments:

```vex
fn identity<T>(x: T): T {
    return x;
}

let num = identity(42);        // T inferred as i32
let text = identity("hello");  // T inferred as string
```

**Explicit vs Inferred**:

```vex
// Explicit type argument
let result = identity<i32>(42);

// Inferred from argument
let result = identity(42);  // Same as above
```

### Multiple Type Parameters

```vex
fn pair<T, U>(first: T, second: U): (T, U) {
    return (first, second);
}

let p1 = pair<i32, string>(42, "answer");
let p2 = pair(3.14, true);  // Inferred: <f64, bool>
```

### Generic Return Types

```vex
fn create<T>(value: T): T {
    return value;
}

let x: i32 = create(42);
let y: string = create("text");
```

### Examples

**Swap Function**:

```vex
fn swap<T>(a: T, b: T): (T, T) {
    return (b, a);
}

let (x, y) = swap(10, 20);        // x=20, y=10
let (s1, s2) = swap("hi", "bye"); // s1="bye", s2="hi"
```

**Generic Comparison** (Future):

```vex
fn max<T: Ord>(a: T, b: T): T {
    if a > b {
        return a;
    }
    return b;
}
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

let pair = Pair<i32, string> {
    first: 42,
    second: "answer",
};
```

### Generic Methods

Methods on generic structs can operate on the generic types. Vex supports two styles for defining methods, both of which can be used with generic structs.

#### Kural 1: Inline Methods

Methods defined inside a `struct` or `trait` use the `!` suffix on the function signature to indicate mutability.

```vex
struct Container<T> {
    value: T,

    // Immutable method, returns a copy of the value.
    fn get(): T {
        return self.value;
    }

    // Mutable method, modifies the internal state.
    fn set(new_value: T)! {
        self.value = new_value;
    }
}
```

**Usage**:

```vex
let! container = Container<i32> { value: 42 };
let val = container.get();      // val is 42
container.set(100)!;            // State is mutated
// container.value is now 100
```

#### Kural 2: External Methods (Golang-style)

Methods can also be defined outside the struct body, using an explicit `self` parameter. Mutability is declared on the receiver's type.

```vex
// This style is also valid for generic structs.
fn (self: &Container<T>) get_external(): T {
    return self.value;
}

fn (self: &Container<T>!) set_external(new_value: T) {
    self.value = new_value;
}
```

**Usage**:

```vex
let! container = Container<i32> { value: 42 };
let val = container.get_external();      // val is 42
container.set_external(100);             // State is mutated
// container.value is now 100
```

### Nested Generics

```vex
struct Box<T> {
    value: T,
}

// Box containing Box
let nested = Box<Box<i32>> {
    value: Box<i32> { value: 42 }
};
```

### Examples

**Generic Stack** (Conceptual):

```vex
struct Stack<T> {
    items: [T],
    size: i32,

    fn (self: &Stack<T>!) push(item: T) {
        // Add item
    }

    fn (self: &Stack<T>!) pop(): T {
        // Remove and return item
    }
}
```

**Generic Point**:

```vex
struct Point<T> {
    x: T,
    y: T,
}

let int_point = Point<i32> { x: 10, y: 20 };
let float_point = Point<f64> { x: 1.5, y: 2.5 };
```

---

## Generic Enums

### Basic Generic Enum (Future)

```vex
enum Option<T> {
    Some(T),
    None,
}

let some_int = Option::Some(42);
let some_str = Option::Some("hello");
let nothing: Option<i32> = Option::None;
```

### Multiple Type Parameters (Future)

```vex
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let success: Result<i32, string> = Result::Ok(42);
let failure: Result<i32, string> = Result::Err("error");
```

### Pattern Matching (Future)

```vex
let result = Result::Ok(42);

match result {
    Result::Ok(value) => {
        // value: i32
    }
    Result::Err(error) => {
        // error: string
    }
}
```

### Examples

**Option Type** (Future):

```vex
enum Option<T> {
    Some(T),
    None,
}

fn find<T>(arr: [T], target: T): Option<i32> {
    for i in 0..arr.len() {
        if arr[i] == target {
            return Option::Some(i);
        }
    }
    return Option::None;
}
```

**Either Type** (Future):

```vex
enum Either<L, R> {
    Left(L),
    Right(R),
}

let left: Either<i32, string> = Either::Left(42);
let right: Either<i32, string> = Either::Right("text");
```

---

## Generic Traits

### Generic Trait Definition (Future)

```vex
trait Container<T> {
    fn get(): T;
    fn set(value: T);
}
```

### Implementation (Future)

```vex
struct Box<T> impl Container<T> {
    value: T,

    fn (self: &Box<T>!) get(): T {
        return self.value;
    }

    fn (self: &Box<T>!) set(value: T) {
        self.value = value;
    }
}
```

### Generic Methods in Traits (Future)

```vex
trait Converter {
    fn convert<T>(): T;
}

struct Value impl Converter {
    data: i32,

    fn (self: &Value!) convert<T>(): T {
        // Type-specific conversion
    }
}
```

---

## Type Constraints

### Trait Bounds (Future)

Restrict generic types to those implementing specific traits:

```vex
fn print_all<T: Display>(items: [T]) {
    for item in items {
        item.show();
    }
}
```

**Syntax**: `T: Trait` after type parameter

### Multiple Constraints (Future)

```vex
fn compare_and_show<T: Comparable & Display>(a: T, b: T): i32 {
    let result = a.compare(b);
    a.show();
    b.show();
    return result;
}
```

**Syntax**: `T: Trait1 & Trait2 & ...`

### Where Clauses ✅ COMPLETE (v0.1.2)

For complex constraints, use where clause for better readability:

```vex
fn print_both<T, U>(a: T, b: U): i32
where
    T: Display,
    U: Display
{
    print("T: ");
    print(a);
    print("U: ");
    print(b);
    return 0;
}

fn main(): i32 {
    let x: i32 = 42;
    let y: i32 = 100;
    print_both(x, y);
    return 0;
}
```

**Implementation**:

- Parser: `parse_where_clause()` in `vex-parser/src/parser/items/functions.rs:138`
- AST: `WhereClausePredicate { type_param, bounds }`
- Syntax: `where T: Trait1 & Trait2, U: Trait3`
- Test: `examples/test_where_clause.vx`
- Limitation: Struct inline methods don't support where clauses yet

### Bound on Structs (Future)

```vex
struct Container<T: Display> {
    value: T,

    fn (self: &Container<T>!) show() {
        self.value.show();  // OK: T implements Display
    }
}
```

### Conditional Methods (Future)

Methods available only when constraints met:

```vex
struct Wrapper<T> {
    value: T,
}

impl<T: Display> Wrapper<T> {
    fn (self: &Wrapper<T>!) show() {
        self.value.show();
    }
}

// show() only available for Wrapper<T> where T: Display
```

---

## Monomorphization

### Concept

Vex uses **monomorphization** for generics:

- Each generic instantiation generates specialized code
- No runtime overhead (unlike type erasure)
- Compile-time type checking
- Code size increases with instantiations

### Example

**Generic Code**:

```vex
fn identity<T>(x: T): T {
    return x;
}

let a = identity(42);
let b = identity("hello");
```

**Generated Code** (conceptual):

```vex
// Compiler generates specialized versions:
fn identity_i32(x: i32): i32 {
    return x;
}

fn identity_string(x: string): string {
    return x;
}

let a = identity_i32(42);
let b = identity_string("hello");
```

### Benefits

1. **Zero Runtime Cost**: No type checking at runtime
2. **Type Safety**: Full compile-time verification
3. **Optimization**: Compiler can optimize each instantiation
4. **No Boxing**: Values aren't boxed/wrapped

### Trade-offs

1. **Code Size**: Each instantiation increases binary size
2. **Compile Time**: More code to generate
3. **Cache Pressure**: Larger code can affect cache

### Example with Structs

**Generic Struct**:

```vex
struct Box<T> {
    value: T,
}

let int_box = Box<i32> { value: 42 };
let str_box = Box<string> { value: "hello" };
```

**Generated Structs**:

```vex
struct Box_i32 {
    value: i32,
}

struct Box_string {
    value: string,
}
```

---

## Advanced Patterns

### Generic Wrapper

```vex
struct Wrapper<T> {
    inner: T,
}

fn wrap<T>(value: T): Wrapper<T> {
    return Wrapper<T> { inner: value };
}

let wrapped_int = wrap(42);
let wrapped_str = wrap("text");
```

### Generic Pair Operations

```vex
struct Pair<T, U> {
    first: T,
    second: U,

    fn (self: &Pair<T, U>) get_first(): T {
        return self.first;
    }

    fn (self: &Pair<T, U>) get_second(): U {
        return self.second;
    }

    fn (self: &Pair<T, U>) swap(): Pair<U, T> {
        return Pair<U, T> {
            first: self.second,
            second: self.first,
        };
    }
}
```

### Phantom Types (Future)

Type parameters not stored but used for compile-time checks:

```vex
struct PhantomData<T>;

struct Marker<T> {
    data: i32,
    _phantom: PhantomData<T>,
}

let m1: Marker<i32> = Marker { data: 42, _phantom: PhantomData };
let m2: Marker<string> = Marker { data: 42, _phantom: PhantomData };
// m1 and m2 are different types despite same data
```

---

## Examples

### Identity Function

```vex
fn identity<T>(x: T): T {
    return x;
}

fn main(): i32 {
    let a = identity(42);        // i32
    let b = identity("hello");   // string
    return a;
}
```

### Generic Box

```vex
struct Box<T> {
    value: T,

    fn (self: &Box<T>) get(): T {
        return self.value;
    }
}

fn main(): i32 {
    let box = Box<i32> { value: 42 };
    return box.get();
}
```

### Swap Function

```vex
fn swap<T>(a: T, b: T): (T, T) {
    return (b, a);
}

fn main(): i32 {
    let (x, y) = swap<i32>(10, 20);
    return x;  // 20
}
```

### Generic Pair

```vex
struct Pair<T, U> {
    first: T,
    second: U,
}

fn make_pair<T, U>(a: T, b: U): Pair<T, U> {
    return Pair<T, U> { first: a, second: b };
}

fn main(): i32 {
    let pair = make_pair<i32, string>(42, "answer");
    return pair.first;  // 42
}
```

### Generic Methods

```vex
struct Container<T> {
    value: T,
    count: i32,

    fn (self: &Container<T>) get_value(): T {
        return self.value;
    }

    fn (self: &Container<T>) get_count(): i32 {
        return self.count;
    }

    fn (self: &Container<T>!) increment() {
        self.count = self.count + 1;
    }
}

fn main(): i32 {
    let! container = Container<i32> {
        value: 42,
        count: 0,
    };

    container.increment();
    return container.get_count();  // 1
}
```

---

## Best Practices

### 1. Use Descriptive Type Parameter Names

```vex
// Good: Descriptive single letters or words
fn map<T, U>(value: T, func: fn(T): U): U { }
fn process<Input, Output>(data: Input): Output { }

// Bad: Unclear abbreviations
fn process<X, Y>(data: X): Y { }
```

**Common Conventions**:

- `T` - Generic type
- `U`, `V`, `W` - Additional types
- `E` - Error type
- `K` - Key type
- `V` - Value type
- `R` - Result/Return type

### 2. Prefer Specific Types When Possible

```vex
// Good: When type is always the same
fn add_integers(a: i32, b: i32): i32 {
    return a + b;
}

// Bad: Unnecessary generic
fn add<T>(a: T, b: T): T {
    return a + b;  // Assumes T supports +
}
```

### 3. Use Constraints for Safety (Future)

```vex
// Good: Explicit constraint
fn compare<T: Comparable>(a: T, b: T): bool {
    return a > b;  // Safe: T implements Comparable
}

// Bad: Unconstrained
fn compare<T>(a: T, b: T): bool {
    return a > b;  // Error: T might not support >
}
```

### 4. Avoid Over-Genericization

```vex
// Good: Reasonable generality
struct Pair<T, U> {
    first: T,
    second: U,
}

// Bad: Unnecessarily complex
struct Pair<T, U, F, G>
where
    F: Fn(T): U,
    G: Fn(U): T
{
    // Too generic for common use
}
```

### 5. Document Generic Constraints

```vex
/// Creates a new container with the given value.
/// Type T can be any type that implements Clone.
fn create<T: Clone>(value: T): Container<T> {
    // Implementation
}
```

---

## Generics Summary

| Feature             | Syntax             | Status     | Example                        |
| ------------------- | ------------------ | ---------- | ------------------------------ |
| Generic Functions   | `fn name<T>()`     | ✅ Working | `identity<T>(x: T)`            |
| Generic Structs     | `struct S<T> { }`  | ✅ Working | `Box<i32>`                     |
| Multiple Parameters | `<T, U, V>`        | ✅ Working | `Pair<T, U>`                   |
| Type Inference      | Omit type args     | ✅ Working | `identity(42)`                 |
| Generic Methods     | `fn (self: &S<T>)` | ✅ Working | Methods on generic types       |
| Monomorphization    | Automatic          | ✅ Working | Zero runtime cost              |
| Generic Enums       | `enum E<T> { }`    | ✅ Working | `Option<T>`, `Result<T,E>`     |
| Trait Bounds        | `<T: Trait>`       | ✅ Working | Constrained types              |
| Where Clauses       | `where T: Trait`   | ✅ v0.1.2  | Complex constraints            |
| Associated Types    | `type Item;`       | ✅ Working | Trait associated types working |
| Higher-Kinded       | `F<T>`             | ❌ Future  | Generic over generics          |
| Const Generics      | `[T; N]`           | ❌ Future  | Array size parameter           |

---

## Compilation Model

### Instantiation Process

1. **Parse**: Generic definition → AST with type parameters
2. **Type Check**: Verify generic constraints
3. **Instantiate**: Generate concrete types for each usage
4. **Monomorphize**: Create specialized code for each type
5. **Optimize**: Optimize each instantiation independently
6. **Link**: Combine all instantiations into binary

### Example Flow

```vex
fn identity<T>(x: T): T { return x; }

let a = identity(42);       // Instantiate identity<i32>
let b = identity("hi");     // Instantiate identity<string>
```

**Compilation**:

1. Parse `identity<T>` as generic template
2. Encounter `identity(42)` → infer T = i32
3. Generate `identity_i32(x: i32): i32`
4. Encounter `identity("hi")` → infer T = string
5. Generate `identity_string(x: string): string`
6. Link both specialized versions

---

**Previous**: [09_Traits.md](./09_Traits.md)  
**Next**: [11_Pattern_Matching.md](./11_Pattern_Matching.md)

**Maintained by**: Vex Language Team
