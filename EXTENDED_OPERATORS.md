# Extended Operator Overloading Proposal
# Comprehensive operator set for Vex

## All Supported Operators

### 1. Arithmetic Operators
```vex
struct BigInt {
    value: i64,
}

impl BigInt {
    fn op+(other: BigInt): BigInt { ... }      // Addition
    fn op-(other: BigInt): BigInt { ... }      // Subtraction
    fn op*(other: BigInt): BigInt { ... }      // Multiplication
    fn op/(other: BigInt): BigInt { ... }      // Division
    fn op%(other: BigInt): BigInt { ... }      // Modulo
    fn op**(other: BigInt): BigInt { ... }     // Exponentiation (NEW!)
}

// Usage
let result = a + b;
let power = base ** exponent;  // 2 ** 8 = 256
```

### 2. Comparison Operators
```vex
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn op==(other: Point): bool { ... }        // Equality
    fn op!=(other: Point): bool { ... }        // Inequality
    fn op<(other: Point): bool { ... }         // Less than
    fn op>(other: Point): bool { ... }         // Greater than
    fn op<=(other: Point): bool { ... }        // Less or equal
    fn op>=(other: Point): bool { ... }        // Greater or equal
    fn op<=>(other: Point): i32 { ... }        // Three-way comparison (NEW!)
}

// Usage
if point1 == point2 { ... }
if point1 < point2 { ... }

// Three-way comparison returns: -1 (less), 0 (equal), 1 (greater)
let cmp = point1 <=> point2;
```

### 3. Bitwise Operators
```vex
struct Flags {
    bits: u32,
}

impl Flags {
    fn op&(other: Flags): Flags { ... }        // Bitwise AND
    fn op|(other: Flags): Flags { ... }        // Bitwise OR
    fn op^(other: Flags): Flags { ... }        // Bitwise XOR
    fn op~(): Flags { ... }                    // Bitwise NOT (unary)
    fn op<<(shift: i32): Flags { ... }         // Left shift
    fn op>>(shift: i32): Flags { ... }         // Right shift
}

// Usage
let combined = flag1 | flag2;
let masked = flags & mask;
let shifted = bits << 4;
```

### 4. Logical Operators
```vex
struct BoolWrapper {
    value: bool,
}

impl BoolWrapper {
    fn op&&(other: BoolWrapper): bool { ... }  // Logical AND
    fn op||(other: BoolWrapper): bool { ... }  // Logical OR
    fn op!(): bool { ... }                     // Logical NOT (unary)
}

// Usage
if condition1 && condition2 { ... }
let inverted = !flag;
```

### 5. Assignment Operators (Compound)
```vex
struct Counter {
    value: i32,
}

impl Counter {
    fn op+=(amount: i32) { ... }               // Add and assign
    fn op-=(amount: i32) { ... }               // Subtract and assign
    fn op*=(factor: i32) { ... }               // Multiply and assign
    fn op/=(divisor: i32) { ... }              // Divide and assign
    fn op%=(modulo: i32) { ... }               // Modulo and assign
    fn op&=(mask: i32) { ... }                 // AND and assign
    fn op|=(flags: i32) { ... }                // OR and assign
    fn op^=(bits: i32) { ... }                 // XOR and assign
    fn op<<=(shift: i32) { ... }               // Left shift and assign
    fn op>>=(shift: i32) { ... }               // Right shift and assign
}

// Usage
counter += 5;
flags |= NEW_FLAG;
bits <<= 2;
```

### 6. Index and Call Operators
```vex
struct Matrix {
    data: Vec<f64>,
    rows: i32,
    cols: i32,
}

impl Matrix {
    // Index operator (read)
    fn op[](row: i32, col: i32): f64 {
        return self.data[row * self.cols + col];
    }
    
    // Index operator (write) - returns mutable reference
    fn op[]=(row: i32, col: i32, value: f64) {
        self.data[row * self.cols + col] = value;
    }
    
    // Call operator (function call syntax)
    fn op()(x: f64, y: f64): f64 {
        // Callable matrix (like Python __call__)
        return self[0, 0] * x + self[0, 1] * y;
    }
}

// Usage
let value = matrix[2, 3];        // op[]
matrix[1, 1] = 5.0;              // op[]=
let result = matrix(10.0, 20.0); // op()
```

### 7. Range Operators (NEW!)
```vex
struct Range {
    start: i32,
    end: i32,
}

impl Range {
    // Range operator: 1..10
    fn op..(end: i32): Range {
        return Range { start: self, end: end };
    }
    
    // Inclusive range: 1..=10
    fn op..=(end: i32): RangeInclusive {
        return RangeInclusive { start: self, end: end };
    }
}

// Usage
for i in 0..10 { ... }           // 0 to 9
for i in 0..=10 { ... }          // 0 to 10 (inclusive)
let slice = array[2..5];         // Range indexing
```

### 8. Member Access Operators
```vex
struct SmartPointer<T> {
    ptr: *T,
}

impl<T> SmartPointer<T> {
    // Dereference operator
    fn op*(): &T {
        return self.ptr;
    }
    
    // Arrow operator (member access through pointer)
    fn op->(): &T {
        return self.ptr;
    }
}

// Usage
let value = *smart_ptr;          // Dereference
let field = smart_ptr->field;    // Member access
```

### 9. Increment/Decrement Operators
```vex
struct Iterator {
    index: i32,
}

impl Iterator {
    // Pre-increment: ++i
    fn op++(): &Iterator {
        self.index = self.index + 1;
        return self;
    }
    
    // Post-increment: i++
    fn op++(dummy: i32): Iterator {
        let old = *self;
        self.index = self.index + 1;
        return old;
    }
    
    // Pre-decrement: --i
    fn op--(): &Iterator {
        self.index = self.index - 1;
        return self;
    }
    
    // Post-decrement: i--
    fn op--(dummy: i32): Iterator {
        let old = *self;
        self.index = self.index - 1;
        return old;
    }
}

// Usage
++iterator;        // Pre-increment
let old = iter++;  // Post-increment
--counter;         // Pre-decrement
```

### 10. String Concatenation & Interpolation
```vex
struct String {
    data: Vec<u8>,
}

impl String {
    // String concatenation
    fn op+(other: String): String { ... }
    fn op+(other: &str): String { ... }
    
    // String repeat (like Python)
    fn op*(count: i32): String {
        let mut result = String.new();
        for i in 0..count {
            result += self;
        }
        return result;
    }
}

// Usage
let greeting = "Hello" + " " + "World";
let repeated = "abc" * 3;  // "abcabcabc"
```

### 11. Conversion Operators (NEW!)
```vex
struct Celsius {
    value: f64,
}

struct Fahrenheit {
    value: f64,
}

impl Celsius {
    // Explicit conversion operator
    fn op as(to: type Fahrenheit): Fahrenheit {
        return Fahrenheit { value: self.value * 9.0 / 5.0 + 32.0 };
    }
    
    // Implicit conversion operator (be careful!)
    fn op implicit as(to: type f64): f64 {
        return self.value;
    }
}

// Usage
let c = Celsius { value: 100.0 };
let f = c as Fahrenheit;         // Explicit conversion
let temp: f64 = c;               // Implicit conversion
```

### 12. Pipeline Operator (NEW! Inspired by F#/Elixir)
```vex
struct Pipeline<T> {
    value: T,
}

impl<T> Pipeline<T> {
    // Pipeline operator: value |> function
    fn op|>(func: fn(T): U): Pipeline<U> {
        return Pipeline { value: func(self.value) };
    }
}

// Usage
let result = value
    |> double
    |> add_ten
    |> to_string;

// Equivalent to: to_string(add_ten(double(value)))
```

### 13. Null Coalescing & Elvis Operators (NEW!)
```vex
struct Option<T> {
    value: T?,
}

impl<T> Option<T> {
    // Null coalescing: value ?? default
    fn op??(default: T): T {
        if self.value == null {
            return default;
        }
        return self.value;
    }
    
    // Elvis operator: value ?: default
    fn op?:(default: T): T {
        return self.value ?? default;
    }
    
    // Safe navigation: obj?.field
    fn op?.(field: string): Option<U> {
        if self.value == null {
            return None;
        }
        return Some(self.value.field);
    }
}

// Usage
let name = user?.name ?? "Anonymous";
let age = person?.age ?: 18;
```

### 14. Spaceship Operator (Three-way comparison)
```vex
struct Version {
    major: i32,
    minor: i32,
    patch: i32,
}

impl Version {
    // Spaceship operator: <=>
    fn op<=>(other: Version): i32 {
        if self.major != other.major {
            return self.major - other.major;
        }
        if self.minor != other.minor {
            return self.minor - other.minor;
        }
        return self.patch - other.patch;
    }
}

// Usage
let cmp = v1 <=> v2;
if cmp < 0 {
    println("v1 is older");
} else if cmp > 0 {
    println("v1 is newer");
} else {
    println("same version");
}
```

### 15. Pattern Matching Operator (NEW!)
```vex
struct Result<T, E> {
    value: T?,
    error: E?,
}

impl<T, E> Result<T, E> {
    // Pattern match operator: ~>
    fn op~>(pattern: Pattern): bool {
        match pattern {
            Ok(_) => return self.value != null,
            Err(_) => return self.error != null,
        }
    }
}

// Usage
if result ~> Ok(_) {
    println("Success!");
}
```

## Complete Operator Table

| Category | Operator | Syntax | Example | Priority |
|----------|----------|--------|---------|----------|
| **Arithmetic** | `+` | `fn op+(other: T): R` | `a + b` | High |
| | `-` | `fn op-(other: T): R` | `a - b` | High |
| | `*` | `fn op*(other: T): R` | `a * b` | High |
| | `/` | `fn op/(other: T): R` | `a / b` | High |
| | `%` | `fn op%(other: T): R` | `a % b` | High |
| | `**` | `fn op**(exp: T): R` | `a ** b` | Medium |
| **Comparison** | `==` | `fn op==(other: T): bool` | `a == b` | High |
| | `!=` | `fn op!=(other: T): bool` | `a != b` | High |
| | `<` | `fn op<(other: T): bool` | `a < b` | High |
| | `>` | `fn op>(other: T): bool` | `a > b` | High |
| | `<=` | `fn op<=(other: T): bool` | `a <= b` | High |
| | `>=` | `fn op>=(other: T): bool` | `a >= b` | High |
| | `<=>` | `fn op<=>(other: T): i32` | `a <=> b` | Medium |
| **Bitwise** | `&` | `fn op&(other: T): R` | `a & b` | High |
| | `\|` | `fn op\|(other: T): R` | `a \| b` | High |
| | `^` | `fn op^(other: T): R` | `a ^ b` | High |
| | `~` | `fn op~(): R` | `~a` | High |
| | `<<` | `fn op<<(n: i32): R` | `a << n` | High |
| | `>>` | `fn op>>(n: i32): R` | `a >> n` | High |
| **Logical** | `&&` | `fn op&&(other: T): bool` | `a && b` | High |
| | `\|\|` | `fn op\|\|(other: T): bool` | `a \|\| b` | High |
| | `!` | `fn op!(): bool` | `!a` | High |
| **Assignment** | `+=` | `fn op+=(other: T)` | `a += b` | High |
| | `-=` | `fn op-=(other: T)` | `a -= b` | High |
| | `*=` | `fn op*=(other: T)` | `a *= b` | High |
| | `/=` | `fn op/=(other: T)` | `a /= b` | High |
| | `%=` | `fn op%=(other: T)` | `a %= b` | High |
| | `&=` | `fn op&=(other: T)` | `a &= b` | Medium |
| | `\|=` | `fn op\|=(other: T)` | `a \|= b` | Medium |
| | `^=` | `fn op^=(other: T)` | `a ^= b` | Medium |
| | `<<=` | `fn op<<=(n: i32)` | `a <<= n` | Medium |
| | `>>=` | `fn op>>=(n: i32)` | `a >>= n` | Medium |
| **Index** | `[]` | `fn op[](idx: T): R` | `a[i]` | High |
| | `[]=` | `fn op[]=(idx: T, val: R)` | `a[i] = v` | High |
| **Call** | `()` | `fn op()(args...): R` | `f(x)` | High |
| **Range** | `..` | `fn op..(end: T): Range` | `0..10` | Medium |
| | `..=` | `fn op..=(end: T): Range` | `0..=10` | Medium |
| **Member** | `*` | `fn op*(): &T` | `*ptr` | High |
| | `->` | `fn op->(): &T` | `ptr->field` | High |
| **Inc/Dec** | `++` | `fn op++(): &Self` | `++i` | Medium |
| | `++` | `fn op++(i32): Self` | `i++` | Medium |
| | `--` | `fn op--(): &Self` | `--i` | Medium |
| | `--` | `fn op--(i32): Self` | `i--` | Medium |
| **Special** | `??` | `fn op??(def: T): T` | `a ?? b` | Low |
| | `?:` | `fn op?:(def: T): T` | `a ?: b` | Low |
| | `?.` | `fn op?.(field): Option<T>` | `obj?.field` | Low |
| | `\|>` | `fn op\|>(fn): R` | `v \|> f` | Low |
| | `~>` | `fn op~>(pat): bool` | `r ~> Ok(_)` | Low |
| **Conversion** | `as` | `fn op as(type T): T` | `x as f64` | Medium |

## Implementation Priority

### Phase 1 (Current - Completed)
- [x] Basic arithmetic: `+, -, *, /, %`
- [x] Comparison: `==, !=, <, >, <=, >=`

### Phase 2 (Next - High Priority)
- [ ] Operator syntax: `fn op+()`
- [ ] Generic contracts: `Add<Rhs=Self>`
- [ ] Overload resolution
- [ ] Compound assignment: `+=, -=, *=, /=`
- [ ] Index operators: `[]`, `[]=`

### Phase 3 (Medium Priority)
- [ ] Bitwise operators: `&, |, ^, ~, <<, >>`
- [ ] Logical operators: `&&, ||, !`
- [ ] Range operators: `..`, `..=`
- [ ] Increment/decrement: `++`, `--`

### Phase 4 (Low Priority - Advanced)
- [ ] Call operator: `()`
- [ ] Member access: `*`, `->`
- [ ] Null coalescing: `??`, `?:`
- [ ] Safe navigation: `?.`
- [ ] Pipeline: `|>`
- [ ] Spaceship: `<=>`
- [ ] Exponentiation: `**`
- [ ] Pattern match: `~>`

## Example: Comprehensive Vector Class

```vex
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector3 {
    // Arithmetic
    fn op+(other: Vector3): Vector3 {
        return Vector3 { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z };
    }
    
    fn op-(other: Vector3): Vector3 {
        return Vector3 { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z };
    }
    
    fn op*(scalar: f64): Vector3 {
        return Vector3 { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar };
    }
    
    fn op/(scalar: f64): Vector3 {
        return Vector3 { x: self.x / scalar, y: self.y / scalar, z: self.z / scalar };
    }
    
    // Comparison
    fn op==(other: Vector3): bool {
        return self.x == other.x && self.y == other.y && self.z == other.z;
    }
    
    // Compound assignment
    fn op+=(other: Vector3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
    
    fn op*=(scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
    
    // Index access
    fn op[](index: i32): f64 {
        if index == 0 { return self.x; }
        if index == 1 { return self.y; }
        return self.z;
    }
    
    fn op[]=(index: i32, value: f64) {
        if index == 0 { self.x = value; }
        else if index == 1 { self.y = value; }
        else { self.z = value; }
    }
    
    // Unary minus
    fn op-(): Vector3 {
        return Vector3 { x: -self.x, y: -self.y, z: -self.z };
    }
}

// Usage
fn test_vector(): i32 {
    let! v1 = Vector3 { x: 1.0, y: 2.0, z: 3.0 };
    let v2 = Vector3 { x: 4.0, y: 5.0, z: 6.0 };
    
    let v3 = v1 + v2;         // op+
    let v4 = v1 * 2.5;        // op*
    v1 += v2;                 // op+=
    let x = v1[0];            // op[]
    v1[1] = 10.0;             // op[]=
    let neg = -v1;            // op- (unary)
    
    return 0;
}
```

Bu öneriler Vex'i C++, Python ve modern dillerin en iyi özelliklerini birleştiren güçlü bir dil yapacak!
