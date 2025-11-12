# Vex Operator Overloading - Complete Specification

**Version:** 0.2.0  
**Date:** November 12, 2025  
**Status:** In Progress

---

## 🎯 Core Principles

1. **Contract-based**: Operators MUST be declared in a contract first
2. **`op` prefix mandatory**: All operator methods use `op+`, `op-`, etc. (NOT method names like `add`)
3. **Multiple overloads**: One contract can define multiple operator overloads with different parameter types
4. **Explicit implementation**: No automatic derivation - you implement what you need

---

## 📋 Syntax Rules

### 1. Contract Declaration

```vex
contract Add {
    op+(rhs: Self): Self;                    // Same type addition
    op+(rhs: i32): Self;                     // Different type overload
    op+(rhs: f64): Self;                     // Another overload
}
```

**Rules:**
- Contract name describes the capability (`Add`, `Mul`, `Eq`)
- Methods MUST use `op` prefix: `op+`, `op-`, `op*`, etc.
- Multiple overloads allowed (different parameter types)
- Return type can vary per overload

### 2. Implementation

```vex
struct Vec2 impl Add {
    x: f64,
    y: f64,
}

// Vector + Vector
fn (self: Vec2) op+(other: Vec2): Vec2 {
    return Vec2 { x: self.x + other.x, y: self.y + other.y };
}

// Vector + scalar
fn (self: Vec2) op+(scalar: f64): Vec2 {
    return Vec2 { x: self.x + scalar, y: self.y + scalar };
}
```

**Rules:**
- Struct MUST declare `impl ContractName`
- Each overload is a separate method definition
- Method signature MUST match a contract declaration
- Compiler validates all contract methods are implemented

### 3. Usage

```vex
let v1 = Vec2 { x: 1.0, y: 2.0 };
let v2 = Vec2 { x: 3.0, y: 4.0 };

let v3 = v1 + v2;        // Calls op+(other: Vec2)
let v4 = v1 + 5.0;       // Calls op+(scalar: f64)
```

**Compiler Resolution:**
1. Check if left operand implements operator contract
2. Find matching overload based on right operand type
3. Call the method: `v1.op+(v2)`

---

## 📐 Phase 1: Arithmetic Operators

### Contracts

```vex
// stdlib/core/ops.vx

// Binary arithmetic operators
contract Add {
    op+(rhs: Self): Self;
}

contract Sub {
    op-(rhs: Self): Self;
}

contract Mul {
    op*(rhs: Self): Self;
}

contract Div {
    op/(rhs: Self): Self;
}

contract Rem {
    op%(rhs: Self): Self;
}

// Unary arithmetic operators
contract Neg {
    op-(): Self;  // Unary minus: -x
}
```

### Builtin Extensions

Compiler automatically provides these for primitives using `extends` keyword:

```vex
// Declared in stdlib/core/builtin_contracts.vx

i32 extends Add, Sub, Mul, Div, Rem, Neg;
i64 extends Add, Sub, Mul, Div, Rem, Neg;
f32 extends Add, Sub, Mul, Div, Rem, Neg;
f64 extends Add, Sub, Mul, Div, Rem, Neg;

// Compiler generates LLVM IR for these operations
// 'extends' keyword indicates builtin type extension (not user impl)
```

### User Example

```vex
contract Add {
    op+(rhs: Self): Self;
}

struct Complex impl Add {
    real: f64,
    imag: f64,
}

fn (self: Complex) op+(other: Complex): Complex {
    return Complex {
        real: self.real + other.real,
        imag: self.imag + other.imag,
    };
}

fn main(): i32 {
    let a = Complex { real: 1.0, imag: 2.0 };
    let b = Complex { real: 3.0, imag: 4.0 };
    let c = a + b;  // Works!
    return 0;
}
```

---

## 📐 Phase 1.5: Bitwise Operators

### Contracts

```vex
// stdlib/core/ops.vx

contract BitAnd {
    op&(rhs: Self): Self;
}

contract BitOr {
    op|(rhs: Self): Self;
}

contract BitXor {
    op^(rhs: Self): Self;
}

contract BitNot {
    op~(): Self;  // Unary operator
}

contract Shl {
    op<<(rhs: i32): Self;
}

contract Shr {
    op>>(rhs: i32): Self;
}
```

### Builtin Extensions

Compiler automatically provides these for integer types:

```vex
// Declared in stdlib/core/builtin_contracts.vx

i32 extends BitAnd, BitOr, BitXor, BitNot, Shl, Shr;
i64 extends BitAnd, BitOr, BitXor, BitNot, Shl, Shr;
u8 extends BitAnd, BitOr, BitXor, BitNot, Shl, Shr;
u16 extends BitAnd, BitOr, BitXor, BitNot, Shl, Shr;
u32 extends BitAnd, BitOr, BitXor, BitNot, Shl, Shr;
u64 extends BitAnd, BitOr, BitXor, BitNot, Shl, Shr;

// Compiler generates LLVM IR for these operations
```

### User Example

```vex
contract BitAnd {
    op&(rhs: Self): Self;
}

contract BitOr {
    op|(rhs: Self): Self;
}

contract Shl {
    op<<(rhs: i32): Self;
}

struct Flags impl BitAnd, BitOr, Shl {
    bits: u32,
}

fn (self: Flags) op&(other: Flags): Flags {
    return Flags { bits: self.bits & other.bits };
}

fn (self: Flags) op|(other: Flags): Flags {
    return Flags { bits: self.bits | other.bits };
}

fn (self: Flags) op<<(shift: i32): Flags {
    return Flags { bits: self.bits << shift };
}

fn main(): i32 {
    let flag1 = Flags { bits: 0b1010 };
    let flag2 = Flags { bits: 0b1100 };
    
    let and_result = flag1 & flag2;  // 0b1000
    let or_result = flag1 | flag2;   // 0b1110
    let shifted = flag1 << 2;        // 0b101000
    
    return 0;
}
```

---

## 📐 Phase 2: Comparison & Logical Operators

### Contracts

```vex
// stdlib/core/ops.vx

// Comparison operators
contract Eq {
    op==(rhs: Self): bool;
    op!=(rhs: Self): bool;  // Optional: auto-implemented as !op==
}

contract Ord {
    op<(rhs: Self): bool;
    op>(rhs: Self): bool;
    op<=(rhs: Self): bool;
    op>=(rhs: Self): bool;
}

// Logical unary operator
contract Not {
    op!(): bool;  // Logical NOT: !x
}
```

### Builtin Extensions

```vex
// Declared in stdlib/core/builtin_contracts.vx

i32 extends Eq, Ord;
i64 extends Eq, Ord;
f32 extends Eq, Ord;
f64 extends Eq, Ord;
bool extends Eq, Not;
string extends Eq, Ord;

// Compiler generates LLVM IR for these operations
```

### Special Rules

**Eq contract:**
- If only `op==` is implemented, compiler auto-generates `op!=` as `!self.op==(rhs)`
- If both are implemented, use user implementations

**Ord contract:**
- All four methods must be implemented (no auto-generation)
- Ensures consistent ordering behavior

### User Example

```vex
contract Eq {
    op==(rhs: Self): bool;
}

struct Point impl Eq {
    x: i32,
    y: i32,
}

fn (self: Point) op==(other: Point): bool {
    return self.x == other.x && self.y == other.y;
}

// Compiler auto-generates:
// fn (self: Point) op!=(other: Point): bool {
//     return !(self == other);
// }

fn main(): i32 {
    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 1, y: 2 };
    
    if p1 == p2 {  // Calls op==
        println("Equal!");
    }
    
    if p1 != p2 {  // Calls auto-generated op!=
        println("Not equal!");
    }
    
    return 0;
}
```

---

## 📐 Phase 3: Compound Assignment & Index Operators

### Contracts

```vex
// stdlib/core/ops.vx

contract AddAssign {
    op+=(rhs: Self);  // Mutates self
}

contract SubAssign {
    op-=(rhs: Self);
}

contract MulAssign {
    op*=(rhs: Self);
}

contract DivAssign {
    op/=(rhs: Self);
}

contract RemAssign {
    op%=(rhs: Self);
}

contract Index {
    type Output;
    op[](index: i32): Output;  // Read access
}

contract IndexMut {
    type Output;
    op[]=(index: i32, value: Output);  // Write access
}

contract BitAndAssign {
    op&=(rhs: Self);
}

contract BitOrAssign {
    op|=(rhs: Self);
}

contract BitXorAssign {
    op^=(rhs: Self);
}

contract ShlAssign {
    op<<=(rhs: i32);
}

contract ShrAssign {
    op>>=(rhs: i32);
}
```

### Builtin Extensions

```vex
// Declared in stdlib/core/builtin_contracts.vx

i32 extends AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign;
i64 extends AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign;
f32 extends AddAssign, SubAssign, MulAssign, DivAssign, RemAssign;
f64 extends AddAssign, SubAssign, MulAssign, DivAssign, RemAssign;
u32 extends AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign;
u64 extends AddAssign, SubAssign, MulAssign, DivAssign, RemAssign, BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign;

// Compiler generates LLVM IR for these operations
```

### Special Rules

**Compound Assignment:**
- Methods MUST be marked as mutable: `fn op+=()!`
- No return value (mutates in place)

**Index:**
- `op[]` for reading: `let x = arr[5]`
- `op[]=` for writing: `arr[5] = 10`
- Can have multiple overloads for different index types

### User Example

```vex
contract AddAssign {
    op+=(rhs: Self);
}

contract Index {
    type Output;
    op[](index: i32): Output;
}

contract IndexMut {
    type Output;
    op[]=(index: i32, value: Output);
}

struct Vec2 impl AddAssign, Index, IndexMut {
    x: f64,
    y: f64,
}

fn (self: Vec2) op+=(other: Vec2)! {
    self.x += other.x;
    self.y += other.y;
}

fn (self: Vec2) op[](index: i32): f64 {
    if index == 0 { return self.x; }
    if index == 1 { return self.y; }
    panic("Index out of bounds");
}

fn (self: Vec2) op[]=(index: i32, value: f64)! {
    if index == 0 { self.x = value; }
    elif index == 1 { self.y = value; }
    else { panic("Index out of bounds"); }
}

fn main(): i32 {
    let! v1 = Vec2 { x: 1.0, y: 2.0 };
    let v2 = Vec2 { x: 3.0, y: 4.0 };
    
    v1 += v2;           // Calls op+=
    
    let x = v1[0];      // Calls op[]
    v1[1] = 10.0;       // Calls op[]=
    
    return 0;
}
```

---

## 📐 Phase 4: Advanced Operators

### Contracts

```vex
// stdlib/core/ops.vx

contract Pow {
    op**(rhs: Self): Self;  // Exponentiation
}

contract PreInc {
    op++(): &Self;  // Pre-increment: ++i
}

contract PostInc {
    op++(dummy: i32): Self;  // Post-increment: i++
}

contract PreDec {
    op--(): &Self;  // Pre-decrement: --i
}

contract PostDec {
    op--(dummy: i32): Self;  // Post-decrement: i--
}

contract NullCoalesce {
    op??(default: Self): Self;  // Null coalescing: a ?? b
}

contract Range {
    type Output;
    op..(end: Self): Output;  // Exclusive range: 0..10
}

contract RangeInclusive {
    type Output;
    op..=(end: Self): Output;  // Inclusive range: 0..=10
}
```

### Special Rules

**Pre vs Post Increment:**
- Pre: Returns reference to self after increment
- Post: Returns old value, then increments (uses dummy i32 param to distinguish)

**Range Operators:**
- Return a Range object (iterator)
- Used in for-loops: `for i in 0..10`

### User Example

```vex
contract Pow {
    op**(rhs: i32): Self;
}

contract PostInc {
    op++(dummy: i32): Self;
}

struct BigInt impl Pow, PostInc {
    value: i64,
}

fn (self: BigInt) op**(exponent: i32): BigInt {
    let! result = 1;
    for i in 0..exponent {
        result *= self.value;
    }
    return BigInt { value: result };
}

fn (self: BigInt) op++(dummy: i32): BigInt! {
    let old = BigInt { value: self.value };
    self.value += 1;
    return old;
}

fn main(): i32 {
    let! n = BigInt { value: 2 };
    let power = n ** 8;  // 2^8 = 256
    
    let old = n++;       // Post-increment
    
    return 0;
}
```

---

## ⚠️ Operator Conflicts & Resolution

### Context-Based Operator Parsing

Some operator symbols have multiple meanings depending on context:

#### 1. `&` Operator
```vex
// Binary: Bitwise AND (overloadable)
let result = a & b;           // Calls op&(rhs)

// Unary: Reference/Address-of (NOT overloadable - builtin only)
let ptr = &value;             // Builtin borrow checker operation
let ref_param = &my_struct;   // Builtin reference
```

**Resolution:** Parser distinguishes based on operand count:
- Binary `&`: BitAnd contract
- Unary `&`: Builtin reference (borrow checker)

#### 2. `*` Operator
```vex
// Binary: Multiplication (overloadable)
let result = a * b;           // Calls op*(rhs)

// Unary: Dereference (NOT overloadable - builtin only)
let value = *ptr;             // Builtin dereference
```

**Resolution:** Parser distinguishes based on operand count:
- Binary `*`: Mul contract
- Unary `*`: Builtin dereference (borrow checker)

#### 3. `-` Operator
```vex
// Binary: Subtraction (overloadable)
let result = a - b;           // Calls op-(rhs)

// Unary: Negation (overloadable)
let neg = -value;             // Calls op-()
```

**Resolution:** Parser distinguishes based on operand count:
- Binary `-`: Sub contract
- Unary `-`: Neg contract

#### 4. `!` Operator
```vex
// Unary: Logical NOT (overloadable)
let inverted = !flag;         // Calls op!()

// Binary: Not equal (different operator)
let not_equal = a != b;       // Calls op!=(rhs)
```

**Resolution:** Different operators (`!` vs `!=`)

### Non-Overloadable Operators (Builtin Only)

These operators are NEVER overloadable for safety/performance:

```vex
// Memory operations (borrow checker)
&expr           // Reference (address-of)
*expr           // Dereference

// Short-circuit logical operators (lazy evaluation)
expr1 && expr2  // Logical AND - evaluates expr2 only if expr1 is true
expr1 || expr2  // Logical OR - evaluates expr2 only if expr1 is false

// Assignment (memory safety)
expr1 = expr2   // Assignment - NOT overloadable

// Member access
expr.field      // Dot operator - NOT overloadable
```

**Why not overloadable?**
- **`&` and `*`**: Tied to borrow checker, memory safety critical
- **`&&` and `||`**: Require lazy evaluation (right side may not execute)
- **`=`**: Overloading assignment is extremely dangerous (moves, copies)
- **`.`**: Member access is fundamental to type system

### Summary Table

| Symbol | Binary | Unary | Contract | Notes |
|--------|--------|-------|----------|-------|
| `+` | ✅ Add | ❌ | Add | Binary only |
| `-` | ✅ Sub | ✅ Neg | Sub, Neg | Both forms overloadable |
| `*` | ✅ Mul | ❌ Builtin | Mul | Unary is deref (builtin) |
| `/` | ✅ Div | ❌ | Div | Binary only |
| `%` | ✅ Rem | ❌ | Rem | Binary only |
| `&` | ✅ BitAnd | ❌ Builtin | BitAnd | Unary is ref (builtin) |
| `|` | ✅ BitOr | ❌ | BitOr | Binary only |
| `^` | ✅ BitXor | ❌ | BitXor | Binary only |
| `~` | ❌ | ✅ BitNot | BitNot | Unary only |
| `!` | ❌ | ✅ Not | Not | Unary only (binary is `!=`) |
| `<<` | ✅ Shl | ❌ | Shl | Binary only |
| `>>` | ✅ Shr | ❌ | Shr | Binary only |
| `==` | ✅ Eq | ❌ | Eq | Binary only |
| `!=` | ✅ Eq | ❌ | Eq | Auto-generated from `==` |
| `<` | ✅ Ord | ❌ | Ord | Binary only |
| `>` | ✅ Ord | ❌ | Ord | Binary only |
| `<=` | ✅ Ord | ❌ | Ord | Binary only |
| `>=` | ✅ Ord | ❌ | Ord | Binary only |
| `&&` | ❌ Builtin | ❌ | None | Short-circuit, not overloadable |
| `||` | ❌ Builtin | ❌ | None | Short-circuit, not overloadable |
| `=` | ❌ Builtin | ❌ | None | Assignment, not overloadable |
| `.` | ❌ Builtin | ❌ | None | Member access, not overloadable |

---

## 🏗️ Builtin Type Extension Architecture

### How `extends` Works for Primitive Types

Builtin primitive types (`i32`, `f64`, `bool`, `string`) cannot have method bodies written in Vex.
Instead, the `extends` keyword is a **compiler directive** that enables contract-based operator dispatch.

#### Declaration (Vex Code)

```vex
// stdlib/core/builtin_contracts.vx

// This is ONLY a declaration - no method bodies!
i32 extends Add, Sub, Mul, Div, Rem, Neg;
i32 extends BitAnd, BitOr, BitXor, BitNot, Shl, Shr;
i32 extends Eq, Ord;
i32 extends AddAssign, SubAssign, MulAssign, DivAssign, RemAssign;

f64 extends Add, Sub, Mul, Div, Rem, Neg;
f64 extends Eq, Ord;
f64 extends AddAssign, SubAssign, MulAssign, DivAssign, RemAssign;

bool extends Eq, Not;

string extends Add, Eq, Ord;  // + for concatenation
```

**Key Points:**
- No `fn` definitions for primitives
- `extends` is parsed as a special AST node: `BuiltinExtension`
- Compiler maintains a registry of builtin contracts

#### Parser (Rust Code)

```rust
// vex-parser/src/parser/items/mod.rs

// Parse: i32 extends Add, Sub, Mul;
if self.current_token_is_identifier() {
    let type_name = self.expect_identifier()?;
    
    if self.match_keyword("extends") {
        let contracts = self.parse_contract_list()?;  // Vec<String>
        
        return Ok(Item::BuiltinExtension {
            type_name,
            contracts,
        });
    }
}
```

#### AST Node

```rust
// vex-ast/src/lib.rs

pub enum Item {
    // ... existing variants
    
    /// Builtin type extension: `i32 extends Add, Sub;`
    /// Only for primitive types - compiler provides implementations
    BuiltinExtension {
        type_name: String,      // "i32", "f64", "bool", "string"
        contracts: Vec<String>, // ["Add", "Sub", "Mul", ...]
    },
}
```

#### Compiler Registry

```rust
// vex-compiler/src/builtin_contracts.rs

use std::collections::{HashMap, HashSet};

pub struct BuiltinContractRegistry {
    // Type -> Set of contracts
    extensions: HashMap<String, HashSet<String>>,
}

impl BuiltinContractRegistry {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }
    
    /// Register a builtin extension from AST
    pub fn register(&mut self, type_name: String, contracts: Vec<String>) {
        self.extensions
            .entry(type_name)
            .or_insert_with(HashSet::new)
            .extend(contracts);
    }
    
    /// Check if a type has a contract
    pub fn has_contract(&self, type_name: &str, contract: &str) -> bool {
        self.extensions
            .get(type_name)
            .map(|contracts| contracts.contains(contract))
            .unwrap_or(false)
    }
}
```

#### Operator Dispatch (Codegen)

```rust
// vex-compiler/src/codegen_ast/expressions/binary_ops.rs

fn compile_binary_op(
    &mut self,
    left: &Expression,
    op: &BinaryOp,
    right: &Expression,
) -> Result<BasicValueEnum<'ctx>, String> {
    let left_val = self.compile_expression(left)?;
    let right_val = self.compile_expression(right)?;
    let left_type = self.infer_expression_type(left)?;
    
    // Step 1: Check builtin contract (primitives)
    if let Type::Named(type_name) = left_type {
        let contract = op.to_contract_name();  // BinaryOp::Add -> "Add"
        
        if self.builtin_contracts.has_contract(&type_name, contract) {
            // Generate LLVM IR directly (no function call!)
            return self.compile_builtin_operator(&type_name, op, left_val, right_val);
        }
    }
    
    // Step 2: Check user-defined contract
    if let Some(impl) = self.user_contracts.get(&left_type) {
        if impl.has_operator(op) {
            return self.call_operator_method(&left_type, op, left_val, right_val);
        }
    }
    
    // Step 3: Error
    Err(format!("Type {} does not implement operator {}", left_type, op))
}

fn compile_builtin_operator(
    &self,
    type_name: &str,
    op: &BinaryOp,
    left: BasicValueEnum<'ctx>,
    right: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    use inkwell::{IntPredicate, FloatPredicate};
    
    match (type_name, op) {
        // Integer arithmetic
        ("i32" | "i64" | "u32" | "u64", BinaryOp::Add) => {
            Ok(self.builder.build_int_add(
                left.into_int_value(),
                right.into_int_value(),
                "add"
            ).into())
        }
        ("i32" | "i64" | "u32" | "u64", BinaryOp::Sub) => {
            Ok(self.builder.build_int_sub(
                left.into_int_value(),
                right.into_int_value(),
                "sub"
            ).into())
        }
        ("i32" | "i64" | "u32" | "u64", BinaryOp::Mul) => {
            Ok(self.builder.build_int_mul(
                left.into_int_value(),
                right.into_int_value(),
                "mul"
            ).into())
        }
        
        // Float arithmetic
        ("f32" | "f64", BinaryOp::Add) => {
            Ok(self.builder.build_float_add(
                left.into_float_value(),
                right.into_float_value(),
                "fadd"
            ).into())
        }
        ("f32" | "f64", BinaryOp::Sub) => {
            Ok(self.builder.build_float_sub(
                left.into_float_value(),
                right.into_float_value(),
                "fsub"
            ).into())
        }
        
        // Bitwise
        ("i32" | "i64" | "u32" | "u64", BinaryOp::BitAnd) => {
            Ok(self.builder.build_and(
                left.into_int_value(),
                right.into_int_value(),
                "and"
            ).into())
        }
        
        // Comparison
        ("i32" | "i64" | "u32" | "u64", BinaryOp::Eq) => {
            Ok(self.builder.build_int_compare(
                IntPredicate::EQ,
                left.into_int_value(),
                right.into_int_value(),
                "eq"
            ).into())
        }
        
        // String concatenation (runtime function)
        ("string", BinaryOp::Add) => {
            let concat_fn = self.get_runtime_function("vex_string_concat")?;
            Ok(self.builder.build_call(
                concat_fn,
                &[left.into(), right.into()],
                "strcat"
            ).try_as_basic_value().left().unwrap())
        }
        
        _ => Err(format!("Unsupported builtin operation: {} {:?}", type_name, op))
    }
}
```

### Performance Characteristics

| Type | Method | Code Generated | Performance |
|------|--------|----------------|-------------|
| `i32` | `op+()` | LLVM `add` instruction | **Zero overhead** |
| `f64` | `op*()` | LLVM `fmul` instruction | **Zero overhead** |
| `string` | `op+()` | C function call (`vex_string_concat`) | Function call overhead |
| User struct | `op+()` | Vex method call | Method call overhead |

**Why this approach?**
- ✅ **Zero overhead** for primitives (inline LLVM IR)
- ✅ **Type safety** via contract system
- ✅ **Clean syntax** - same as user types
- ✅ **No runtime penalty** for basic operations

---

## 🔧 Implementation Plan

### Step 1: Lexer Changes

**File:** `vex-lexer/src/lib.rs`

Add tokens:
- `OperatorMethod(String)` - For `op+`, `op-`, `op*`, etc.

**Pattern:** `op[+\-*/%=<>!&|^~[\]().]+(=)?`

```rust
// In tokenize loop
if chars.peek() == Some(&'o') && chars.peek_next() == Some(&'p') {
    chars.next(); // consume 'o'
    chars.next(); // consume 'p'
    
    let mut op = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_alphanumeric() || ch == '_' {
            break;
        }
        op.push(ch);
        chars.next();
    }
    
    tokens.push(Token::OperatorMethod(op));
}
```

### Step 2: AST Changes

**File:** `vex-ast/src/lib.rs`

```rust
// Add to FunctionSignature
pub struct FunctionSignature {
    pub name: String,
    pub is_operator: bool,  // NEW: true if name starts with "op"
    // ... existing fields
}

// Add helper
impl FunctionSignature {
    pub fn operator_symbol(&self) -> Option<&str> {
        if self.is_operator && self.name.starts_with("op") {
            Some(&self.name[2..])  // Strip "op" prefix
        } else {
            None
        }
    }
}
```

### Step 3: Parser Changes

**File:** `vex-parser/src/parser/items/functions.rs`

```rust
// Modify function signature parsing
fn parse_function_signature(&mut self) -> Result<FunctionSignature, ParseError> {
    // ... existing receiver parsing ...
    
    // Check for operator method
    let (name, is_operator) = if let Token::OperatorMethod(op) = self.peek() {
        self.advance();
        (format!("op{}", op), true)
    } else {
        (self.expect_identifier()?, false)
    };
    
    // ... rest of parsing ...
    
    Ok(FunctionSignature {
        name,
        is_operator,
        // ...
    })
}
```

### Step 4: Contract Validation

**File:** `vex-compiler/src/contract_validator.rs` (NEW)

```rust
pub struct ContractValidator {
    contracts: HashMap<String, ContractDef>,
    implementations: HashMap<String, Vec<String>>, // Type -> Contracts
}

impl ContractValidator {
    pub fn validate_operator_impl(
        &self,
        type_name: &str,
        contract: &str,
        method: &FunctionSignature,
    ) -> Result<(), String> {
        // 1. Check contract exists
        let contract_def = self.contracts.get(contract)
            .ok_or_else(|| format!("Contract {} not found", contract))?;
        
        // 2. Check operator method exists in contract
        let contract_method = contract_def.methods.iter()
            .find(|m| m.name == method.name)
            .ok_or_else(|| format!("Contract {} does not declare {}", contract, method.name))?;
        
        // 3. Check signatures match
        if !signatures_compatible(method, contract_method) {
            return Err(format!(
                "Method signature mismatch: expected {}, got {}",
                contract_method.signature(),
                method.signature()
            ));
        }
        
        Ok(())
    }
}
```

### Step 5: Codegen - Operator Dispatch

**File:** `vex-compiler/src/codegen_ast/expressions/binary_ops.rs`

```rust
fn compile_binary_op(
    &mut self,
    left: &Expression,
    op: &BinaryOp,
    right: &Expression,
) -> Result<Value, String> {
    let left_val = self.compile_expression(left)?;
    let right_val = self.compile_expression(right)?;
    
    let left_type = self.get_expression_type(left)?;
    
    // Check for operator contract implementation
    if let Some(contract) = self.get_operator_contract(op) {
        if self.contract_validator.has_impl(&left_type, contract) {
            // Call operator method
            let method_name = format!("op{}", op.symbol());
            return self.call_contract_method(
                &left_type,
                contract,
                &method_name,
                vec![left_val, right_val],
            );
        }
    }
    
    // Fallback to builtin operators (primitives)
    match (left_type.as_str(), op) {
        ("i32" | "i64" | "f32" | "f64", BinaryOp::Add) => {
            Ok(self.builder.build_add(left_val, right_val, "add"))
        }
        // ... other builtins
        _ => Err(format!(
            "Type {} does not implement {} operator",
            left_type,
            op.symbol()
        )),
    }
}

fn get_operator_contract(&self, op: &BinaryOp) -> Option<&str> {
    match op {
        BinaryOp::Add => Some("Add"),
        BinaryOp::Sub => Some("Sub"),
        BinaryOp::Mul => Some("Mul"),
        BinaryOp::Div => Some("Div"),
        BinaryOp::Mod => Some("Rem"),
        BinaryOp::Eq => Some("Eq"),
        // ...
        _ => None,
    }
}
```

### Step 6: Standard Library

**File:** `vex-libs/std/core/ops.vx` (NEW)

```vex
// Arithmetic operators (binary)
export contract Add {
    op+(rhs: Self): Self;
}

export contract Sub {
    op-(rhs: Self): Self;
}

export contract Mul {
    op*(rhs: Self): Self;
}

export contract Div {
    op/(rhs: Self): Self;
}

export contract Rem {
    op%(rhs: Self): Self;
}

// Arithmetic operators (unary)
export contract Neg {
    op-(): Self;
}

// Bitwise operators
export contract BitAnd {
    op&(rhs: Self): Self;
}

export contract BitOr {
    op|(rhs: Self): Self;
}

export contract BitXor {
    op^(rhs: Self): Self;
}

export contract BitNot {
    op~(): Self;
}

export contract Shl {
    op<<(rhs: i32): Self;
}

export contract Shr {
    op>>(rhs: i32): Self;
}

// Comparison operators
export contract Eq {
    op==(rhs: Self): bool;
    op!=(rhs: Self): bool;
}

export contract Ord {
    op<(rhs: Self): bool;
    op>(rhs: Self): bool;
    op<=(rhs: Self): bool;
    op>=(rhs: Self): bool;
}

// Logical operators (unary)
export contract Not {
    op!(): bool;
}

// Assignment operators
export contract AddAssign {
    op+=(rhs: Self);
}

export contract SubAssign {
    op-=(rhs: Self);
}

export contract MulAssign {
    op*=(rhs: Self);
}

export contract DivAssign {
    op/=(rhs: Self);
}

export contract RemAssign {
    op%=(rhs: Self);
}

export contract BitAndAssign {
    op&=(rhs: Self);
}

export contract BitOrAssign {
    op|=(rhs: Self);
}

export contract BitXorAssign {
    op^=(rhs: Self);
}

export contract ShlAssign {
    op<<=(rhs: i32);
}

export contract ShrAssign {
    op>>=(rhs: i32);
}

// Index operators
export contract Index {
    type Output;
    op[](index: i32): Output;
}

export contract IndexMut {
    type Output;
    op[]=(index: i32, value: Output);
}

// Advanced operators
export contract Pow {
    op**(rhs: Self): Self;
}

export contract PreInc {
    op++(): &Self;
}

export contract PostInc {
    op++(dummy: i32): Self;
}

export contract PreDec {
    op--(): &Self;
}

export contract PostDec {
    op--(dummy: i32): Self;
}

export contract NullCoalesce {
    op??(default: Self): Self;
}

export contract Range {
    type Output;
    op..(end: Self): Output;
}

export contract RangeInclusive {
    type Output;
    op..=(end: Self): Output;
}
```

---

## 🧪 Test Plan

### Phase 1 Tests: Arithmetic

**File:** `examples/operator/test_arithmetic.vx`

```vex
contract Add {
    op+(rhs: Self): Self;
}

contract Mul {
    op*(rhs: Self): Self;
}

struct Vec2 impl Add, Mul {
    x: f64,
    y: f64,
}

fn (self: Vec2) op+(other: Vec2): Vec2 {
    return Vec2 { x: self.x + other.x, y: self.y + other.y };
}

fn (self: Vec2) op*(scalar: f64): Vec2 {
    return Vec2 { x: self.x * scalar, y: self.y * scalar };
}

fn main(): i32 {
    let v1 = Vec2 { x: 1.0, y: 2.0 };
    let v2 = Vec2 { x: 3.0, y: 4.0 };
    
    let v3 = v1 + v2;
    assert(v3.x == 4.0 && v3.y == 6.0, "Addition failed");
    
    let v4 = v1 * 2.0;
    assert(v4.x == 2.0 && v4.y == 4.0, "Multiplication failed");
    
    return 0;
}
```

### Phase 2 Tests: Comparison

**File:** `examples/operator/test_comparison.vx`

```vex
contract Eq {
    op==(rhs: Self): bool;
}

contract Ord {
    op<(rhs: Self): bool;
    op>(rhs: Self): bool;
    op<=(rhs: Self): bool;
    op>=(rhs: Self): bool;
}

struct Point impl Eq, Ord {
    x: i32,
    y: i32,
}

fn (self: Point) op==(other: Point): bool {
    return self.x == other.x && self.y == other.y;
}

fn (self: Point) op<(other: Point): bool {
    if self.x != other.x {
        return self.x < other.x;
    }
    return self.y < other.y;
}

// ... implement other comparison operators

fn main(): i32 {
    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 1, y: 2 };
    let p3 = Point { x: 2, y: 3 };
    
    assert(p1 == p2, "Equality failed");
    assert(p1 < p3, "Less than failed");
    
    return 0;
}
```

### Phase 3 Tests: Compound & Index

**File:** `examples/operator/test_compound_index.vx`

```vex
contract AddAssign {
    op+=(rhs: Self);
}

contract Index {
    type Output;
    op[](index: i32): Output;
}

contract IndexMut {
    type Output;
    op[]=(index: i32, value: Output);
}

struct Counter impl AddAssign, Index, IndexMut {
    values: Vec<i32>,
}

fn (self: Counter) op+=(amount: i32)! {
    for i in 0..self.values.len() {
        self.values[i] += amount;
    }
}

fn (self: Counter) op[](index: i32): i32 {
    return self.values[index];
}

fn (self: Counter) op[]=(index: i32, value: i32)! {
    self.values[index] = value;
}

fn main(): i32 {
    let! counter = Counter { values: Vec.from([1, 2, 3]) };
    
    counter += 10;
    assert(counter[0] == 11, "Compound assign failed");
    
    counter[1] = 99;
    assert(counter[1] == 99, "Index assign failed");
    
    return 0;
}
```

---

## 📊 Success Criteria

### Phase 1 Complete:
- ✅ Arithmetic operators (`op+`, `op-`, `op*`, `op/`, `op%`) implemented
- ✅ Contract validation works
- ✅ User structs can implement Add, Sub, Mul, Div, Rem
- ✅ All Phase 1 tests pass

### Phase 1.5 Complete:
- ✅ Bitwise operators (`op&`, `op|`, `op^`, `op~`, `op<<`, `op>>`) implemented
- ✅ Builtin extensions for integer types work
- ✅ All Phase 1.5 tests pass

### Phase 2 Complete:
- ✅ Comparison operators (`op==`, `op!=`, `op<`, `op>`, `op<=`, `op>=`) implemented
- ✅ Auto-generation of `op!=` from `op==`
- ✅ All Phase 2 tests pass

### Phase 3 Complete:
- ✅ Compound assignment operators work
- ✅ Index operators (`op[]`, `op[]=`) work
- ✅ All Phase 3 tests pass

### Phase 4 Complete:
- ✅ Advanced operators (Pow, Inc/Dec, Range, NullCoalesce) implemented
- ✅ All Phase 4 tests pass
- ✅ Complete operator test suite passes (50+ tests)

---

## 🚀 Next Steps

1. ✅ **Spec complete** - All phases documented
2. ✅ **Builtin architecture defined** - Zero-overhead implementation
3. ✅ **Conflict resolution documented** - Context-based parsing
4. 🎯 **Ready to implement** - Start Phase 1: Arithmetic operators

---

**Status:** ✅ **Spec complete** - Ready for implementation
