# Vex Language Syntax Reference

**Version:** 0.2.0 (Syntax v0.9)  
**Test Coverage:** 110/112 (98.2%)

## Variables

```vex
let x = 42;              // Immutable
let! counter = 0;        // Mutable
const MAX = 100;         // Compile-time constant
```

## Types

### Primitives

```vex
i8 i16 i32 i64 i128      // Signed integers
u8 u16 u32 u64 u128      // Unsigned integers
f32 f64                  // Floating point
bool                     // Boolean
char                     // Character
str                      // String slice
()                       // Unit type
```

### Compound Types

```vex
[i32]                    // Array type
(i32, f64, bool)         // Tuple type
fn(i32, i32): i32        // Function type
&T                       // Immutable reference
&T!                      // Mutable reference
```

## Functions

```vex
// Basic function
fn add(x: i32, y: i32): i32 {
    return x + y;
}

// No return type (unit)
fn print_hello() {
    println("Hello");
}

// Generic function
fn identity<T>(x: T): T {
    return x;
}

// Generic with trait bounds
fn process<T: Display>(value: T): str {
    return value.to_string();
}

// Multiple type parameters
fn pair<T, U>(first: T, second: U): (T, U) {
    return (first, second);
}
```

## Closures

```vex
// Basic closure
let add = |x: i32, y: i32| x + y;

// With explicit return type
let multiply = |x: i32, y: i32|: i32 { x * y };

// Capturing environment
let factor = 10;
let scale = |x: i32| x * factor;

// Generic closure bounds
fn map<T, U, F: Callable(T): U>(arr: [T], f: F): [U] {
    // ...
}
```

## Structs

```vex
// Basic struct
struct Point {
    x: i32,
    y: i32,
}

// Generic struct
struct Box<T> {
    value: T,
}

// Nested generics
struct Pair<T, U> {
    first: T,
    second: U,
}

// Struct literal
let p = Point { x: 10, y: 20 };
let b = Box { value: 42 };          // Type inferred from annotation
let b: Box<i32> = Box { value: 42 };  // Explicit annotation
```

## Struct Methods

```vex
impl Point {
    fn new(x: i32, y: i32): Point {
        return Point { x: x, y: y };
    }

    fn distance(&self): f64 {
        return sqrt((self.x * self.x + self.y * self.y) as f64);
    }

    fn move!(&self!, dx: i32, dy: i32) {
        self!.x = self!.x + dx;
        self!.y = self!.y + dy;
    }
}

// Usage
let p = Point.new(10, 20);
let d = p.distance();
p.move!(5, 5);
```

## Enums

```vex
// Simple enum
enum Status {
    Ok,
    Error,
    Pending,
}

// Enum with data
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Enum literals
let s = Status.Ok;
let opt = Option.Some(42);
let res = Result.Err("failed");
```

## Pattern Matching

```vex
// Match expression
let result = match status {
    Status.Ok => 1,
    Status.Error => 0,
    Status.Pending => -1,
};

// Match with data
match option {
    Option.Some(x) => println(x),
    Option.None => println("None"),
}

// Match with block
match value {
    0 => {
        println("zero");
        return 0;
    },
    x => {
        println("non-zero");
        return x;
    },
}

// Tuple pattern
match tuple {
    (0, 0) => "origin",
    (0, _) => "y-axis",
    (_, 0) => "x-axis",
    (x, y) => "other",
}
```

## Control Flow

```vex
// If expression
let result = if x > 0 { "positive" } else { "non-positive" };

// If statement
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// While loop
while counter < 10 {
    counter = counter + 1;
}

// For loop
for i in 0..10 {
    println(i);
}

// For with array
for item in array {
    println(item);
}

// Loop with break/continue
loop {
    if done {
        break;
    }
    if skip {
        continue;
    }
}
```

## Traits

```vex
// Trait definition
trait Display {
    fn to_string(&self): str;
}

// Trait with default method
trait Printable {
    fn to_string(&self): str;

    fn print(&self) {
        println(self.to_string());
    }
}

// Trait implementation (separate block)
struct Point impl Display {
    fn to_string(&self): str {
        return format("({}, {})", self.x, self.y);
    }
}

// Inline trait implementation
struct Point impl Display {}

// Multiple traits
struct Value impl Display {}
struct Value impl Clone {}

// Trait bounds
fn show<T: Display>(value: T) {
    println(value.to_string());
}

// Multiple bounds
fn process<T: Display + Clone>(value: T) {
    // ...
}

// Generic trait implementation
struct Box<T> impl Display {
    fn to_string(&self): str {
        return self.value.to_string();
    }
}
```

## Operators

### Arithmetic

```vex
x + y    // Addition
x - y    // Subtraction
x * y    // Multiplication
x / y    // Division
x % y    // Modulo
-x       // Negation
```

### Comparison

```vex
x == y   // Equal
x != y   // Not equal
x < y    // Less than
x <= y   // Less or equal
x > y    // Greater than
x >= y   // Greater or equal
```

### Logical

```vex
x && y   // Logical AND
x || y   // Logical OR
!x       // Logical NOT
```

### Bitwise

```vex
x & y    // Bitwise AND
x | y    // Bitwise OR
x ^ y    // Bitwise XOR
x << y   // Left shift
x >> y   // Right shift
```

### Assignment

```vex
x = y    // Assign
x += y   // Add assign
x -= y   // Subtract assign
x *= y   // Multiply assign
x /= y   // Divide assign
```

### Postfix

```vex
x++      // Post-increment (mutable variables only)
x--      // Post-decrement (mutable variables only)
```

## Array Operations

```vex
// Array literal
let arr = [1, 2, 3, 4, 5];

// Array indexing
let x = arr[0];
arr[1] = 10;

// Array type annotation
let arr: [i32] = [1, 2, 3];

// Multi-dimensional
let matrix = [[1, 2], [3, 4]];
```

## Tuple Operations

```vex
// Tuple literal
let t = (1, "hello", true);

// Tuple indexing
let x = t.0;
let s = t.1;

// Tuple destructuring
let (a, b, c) = t;

// Nested tuples
let nested = ((1, 2), (3, 4));
```

## Field Access

```vex
// Struct field
point.x
point.y

// Nested field
box.value.x

// Chained access
b3.value.value.value  // Deeply nested generics
```

## Error Handling

```vex
// Try operator
let value = operation()?;  // Propagates error

// Match on Result
match result {
    Result.Ok(v) => v,
    Result.Err(e) => return Result.Err(e),
}
```

## Defer

```vex
fn process() {
    let file = open("data.txt");
    defer close(file);  // Executes at function end

    // Work with file...
}  // close(file) called here

// Multiple defers (LIFO order)
defer cleanup_a();
defer cleanup_b();  // Executes first
```

## Built-in Functions

### Memory

```vex
@size_of<T>()        // Size of type
@align_of<T>()       // Alignment of type
@malloc(size)        // Allocate memory
@free(ptr)           // Free memory
@memcpy(dst, src, n) // Copy memory
@memset(ptr, val, n) // Set memory
```

### Type Introspection

```vex
@type_of(expr)       // Get type as string
@type_name<T>()      // Type name
@type_id<T>()        // Type ID
```

### Assertions

```vex
@assert(condition)   // Runtime assert
@unreachable()       // Mark unreachable code
```

### Overflow Checks

```vex
@add_overflow(a, b)  // Checked addition
@sub_overflow(a, b)  // Checked subtraction
@mul_overflow(a, b)  // Checked multiplication
```

### Hints

```vex
@likely(condition)   // Branch prediction hint
@unlikely(condition) // Branch prediction hint
```

## Comments

```vex
// Single-line comment

/*
 * Multi-line comment
 */
```

## Module System

```vex
// Import
import std.io;
import std.collections.{HashMap, HashSet};

// Use imported items
let map = HashMap.new();
println("Hello");
```

## Deprecated Syntax (Will Error)

```vex
❌ mut x = 42;           // Use: let! x = 42;
❌ fn(): i32 -> { }      // Use: fn(): i32 { }
❌ interface Foo {}      // Use: trait Foo {}
❌ x := 42;              // Use: let x = 42;
❌ &mut T                // Use: &T!
❌ impl Trait for Type   // Use: struct Type impl Trait
```

## Grammar Summary

```
Program       := Item*
Item          := Function | Struct | Enum | Trait | Impl | Const
Function      := 'fn' Ident TypeParams? '(' Params ')' (':' Type)? Block
Struct        := 'struct' Ident TypeParams? '{' Fields '}'
Enum          := 'enum' Ident TypeParams? '{' Variants '}'
Trait         := 'trait' Ident '{' TraitItems '}'
Impl          := 'struct' Ident TypeParams? 'impl' Trait '{' ImplItems '}'

Statement     := Let | Assign | If | While | For | Loop | Return | Defer | Expression
Let           := 'let' '!'? Ident (':' Type)? '=' Expression
Assign        := Expression '=' Expression
Defer         := 'defer' Expression

Expression    := Match | If | Block | Binary | Unary | Call | Index | Field | Literal
Match         := 'match' Expression '{' MatchArms '}'
Binary        := Expression BinOp Expression
Call          := Expression '(' Arguments ')'
Closure       := '|' Params '|' (':' Type)? (Expression | Block)

Type          := PrimitiveType | NamedType | GenericType | TupleType | ArrayType | FnType | RefType
GenericType   := Ident '<' TypeArgs '>'
RefType       := '&' Type '!'?
FnType        := 'fn' '(' TypeList ')' ':' Type
```

## Notes

- **Function return types:** Use `:` not `->`
- **Mutability:** Use `let!` not `mut`, `&T!` not `&mut T`
- **Traits:** Use `trait` not `interface`
- **Generics:** Fully supported with depth limit of 64
- **Borrow checker:** 4-phase analysis (immutability, moves, borrows, lifetimes)
- **Closure capture:** Automatic environment detection and binding
- **Test status:** 110/112 passing (98.2%)
