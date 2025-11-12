# Operator Overloading Syntax Proposal: C++ Style

## Mevcut Durum (Contract-Based)

```vex
contract Add {
    fn add(other: Self): Self;  // ❌ Sadece aynı tip
}

struct Point impl Add {
    fn add(other: Point): Point { ... }
}

let p3 = p1 + p2;  // ✅ Çalışır
// let v2 = v1 * 5;  // ❌ Çalışmaz (farklı tip)
```

## Öneri 1: `fn op+()` Syntax (Kısa)

```vex
struct Vector {
    x: i32,
    y: i32,
}

impl Vector {
    // Vector + Vector
    fn op+(other: Vector): Vector {
        return Vector { x: self.x + other.x, y: self.y + other.y };
    }
    
    // Vector * i32 (farklı tip!)
    fn op*(scalar: i32): Vector {
        return Vector { x: self.x * scalar, y: self.y * scalar };
    }
    
    // Vector * f64 (başka overload!)
    fn op*(factor: f64): Vector {
        return Vector { 
            x: ((self.x as f64) * factor) as i32,
            y: ((self.y as f64) * factor) as i32,
        };
    }
    
    // Vector == Vector
    fn op==(other: Vector): bool {
        return self.x == other.x && self.y == other.y;
    }
    
    // Vector[i32] (index operator)
    fn op[](index: i32): i32 {
        if index == 0 { return self.x; }
        return self.y;
    }
}

// Kullanım
let v3 = v1 + v2;      // ✅ op+(Vector)
let v4 = v1 * 5;       // ✅ op*(i32)
let v5 = v1 * 2.5;     // ✅ op*(f64)
let eq = v1 == v2;     // ✅ op==(Vector)
let x = v1[0];         // ✅ op[](i32)
```

## Öneri 2: `operator +()` Syntax (Explicit)

```vex
impl Vector {
    operator +(other: Vector): Vector {
        return Vector { x: self.x + other.x, y: self.y + other.y };
    }
    
    operator *(scalar: i32): Vector {
        return Vector { x: self.x * scalar, y: self.y * scalar };
    }
    
    operator *(factor: f64): Vector {
        return Vector { 
            x: ((self.x as f64) * factor) as i32,
            y: ((self.y as f64) * factor) as i32,
        };
    }
}

// Kullanım aynı
let v3 = v1 + v2;
let v4 = v1 * 5;
```

## Öneri 3: Hybrid Approach (Contract + Operator Syntax)

```vex
// Generic contract (like Rust)
contract Add<Rhs = Self> {
    type Output;
    fn add(rhs: Rhs): Output;
}

struct Vector impl Add<i32>, Add<f64>, Add<Vector> {
    x: i32,
    y: i32,
    
    // Operator syntax → contract implementation
    fn op+(other: Vector): Vector {  // impl Add<Vector>
        return Vector { x: self.x + other.x, y: self.y + other.y };
    }
    
    fn op+(scalar: i32): Vector {  // impl Add<i32>
        return Vector { x: self.x + scalar, y: self.y + scalar };
    }
    
    fn op+(factor: f64): Vector {  // impl Add<f64>
        return Vector { 
            x: ((self.x as f64) + factor) as i32,
            y: ((self.y as f64) + factor) as i32,
        };
    }
}

// Compiler desugaring:
// v1 + v2 → v1.add(v2)  // Add<Vector>
// v1 + 5  → v1.add(5)   // Add<i32>
// v1 + 2.5 → v1.add(2.5) // Add<f64>
```

## Avantajlar

### ✅ C++ Gibi Esneklik
```vex
fn op+(other: Vector): Vector     // Vector + Vector
fn op+(scalar: i32): Vector       // Vector + i32
fn op+(factor: f64): Vector       // Vector + f64
fn op*(scalar: i32): Vector       // Vector * i32
```

**C++ Equivalent:**
```cpp
Vector operator+(const Vector& other);
Vector operator+(int scalar);
Vector operator+(double factor);
Vector operator*(int scalar);
```

### ✅ Overload Resolution
```vex
let v1 = Vector { x: 10, y: 20 };
let v2 = v1 + 5;       // Calls op+(i32)
let v3 = v1 + 2.5;     // Calls op+(f64)
let v4 = v1 + v1;      // Calls op+(Vector)
```

Compiler tip kontrolü yaparak doğru overload'ı seçer.

### ✅ Tüm Operatörler Desteklenir
```vex
fn op+()   // Addition
fn op-()   // Subtraction
fn op*()   // Multiplication
fn op/()   // Division
fn op%()   // Modulo
fn op==()  // Equality
fn op!=()  // Not equal
fn op<()   // Less than
fn op>()   // Greater than
fn op<=()  // Less or equal
fn op>=()  // Greater or equal
fn op[]()  // Index
fn op<<()  // Left shift
fn op>>()  // Right shift
fn op&()   // Bitwise AND
fn op|()   // Bitwise OR
fn op^()   // Bitwise XOR
```

## Dezavantajlar

### ❌ Contract Abstraction Kaybı
```vex
// Contract ile:
fn generic_add<T: Add>(a: T, b: T): T {
    return a + b;  // T'nin Add impl ettiğini garanti eder
}

// Operator syntax ile:
fn generic_add<T>(a: T, b: T): T {
    return a + b;  // ❌ T'nin + operatörünü desteklediğini garanti edemeyiz
}
```

### ❌ Trait Bound Kontrolü Zor
```vex
// Contract sistemi compile-time'da kontrol eder
contract Addable {
    fn add(other: Self): Self;
}

// Operator syntax'ta bu garanti yok
```

## Önerilen Çözüm: Hybrid System

### Syntax
```vex
// 1. Generic contract tanımla
contract Add<Rhs = Self> {
    type Output = Self;
    fn add(rhs: Rhs): Output;
}

// 2. Operator syntax ile implement et
struct Vector impl Add<i32>, Add<f64>, Add<Vector> {
    x: i32,
    y: i32,
    
    // Operator syntax → desugar to add() method
    fn op+(scalar: i32): Vector {
        return Vector { x: self.x + scalar, y: self.y + scalar };
    }
    
    fn op+(factor: f64): Vector {
        return Vector { 
            x: ((self.x as f64) + factor) as i32,
            y: ((self.y as f64) + factor) as i32,
        };
    }
    
    fn op+(other: Vector): Vector {
        return Vector { x: self.x + other.x, y: self.y + other.y };
    }
}

// 3. Generic fonksiyonlarda contract bound kullan
fn generic_sum<T: Add<i32>>(values: Vec<T>): T {
    let mut sum = values[0];
    for i in 1..values.len() {
        sum = sum + values[i];  // ✅ Add<i32> garantili
    }
    return sum;
}
```

### Compiler Desugaring
```vex
// Source code:
let v2 = v1 + 5;

// Compiler steps:
// 1. v1'nin tipi: Vector
// 2. + operatörü → Add contract
// 3. Parametre tipi: i32
// 4. Vector impl Add<i32> var mı? ✅
// 5. Desugar to: v1.add(5)
// 6. add() metodu → op+(i32) ile implement edilmiş
```

## Implementation Planı

### Phase 1: Parser (Lexer + AST)
```rust
// vex-lexer
pub enum Token {
    // ...existing...
    Operator,  // "operator" keyword
}

// vex-ast
pub struct OperatorImpl {
    pub op: BinaryOp,  // Add, Sub, Mul, etc.
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub body: Block,
}
```

### Phase 2: Type Checker
```rust
// vex-compiler/src/type_checker
impl TypeChecker {
    fn check_operator_overload(&mut self, op: &OperatorImpl) {
        // 1. Check parameter types
        // 2. Check return type
        // 3. Register in operator table
        // 4. Link to contract implementation if exists
    }
    
    fn resolve_operator_call(&self, left: &Type, op: BinaryOp, right: &Type) -> Type {
        // 1. Find matching operator overload
        // 2. Perform overload resolution
        // 3. Return result type
    }
}
```

### Phase 3: Codegen
```rust
// vex-compiler/src/codegen
impl ASTCodeGen {
    fn compile_binary_op(&mut self, left: &Expr, op: BinaryOp, right: &Expr) {
        // 1. Check for operator overload
        let operator_overload = self.find_operator_overload(left_type, op, right_type);
        
        if let Some(overload) = operator_overload {
            // 2. Desugar to method call
            return self.compile_method_call(left, &overload.method_name, &[right]);
        }
        
        // 3. Fallback to builtin ops
        self.compile_builtin_binary_op(left, op, right)
    }
}
```

## Karşılaştırma

| Özellik | Current (Contract) | Proposal (Operator) | Hybrid |
|---------|-------------------|---------------------|--------|
| **Syntax** | `fn add()` | `fn op+()` | `fn op+()` + contract |
| **Multiple types** | ❌ Self only | ✅ Any type | ✅ Any type |
| **Type safety** | ✅ Contract bounds | ⚠️ No bounds | ✅ Contract bounds |
| **Readability** | ⚠️ Verbose | ✅ Natural | ✅ Natural |
| **Generic constraints** | ✅ `T: Add` | ❌ No constraint | ✅ `T: Add<i32>` |
| **Overload resolution** | ❌ No overloads | ✅ Automatic | ✅ Automatic |
| **C++ compatibility** | ❌ Different | ✅ Similar | ✅ Similar |

## Örnek: Kompleks Sayılar

```vex
contract Add<Rhs = Self> {
    type Output = Self;
    fn add(rhs: Rhs): Output;
}

struct Complex impl Add<Complex>, Add<f64> {
    real: f64,
    imag: f64,
    
    // Complex + Complex
    fn op+(other: Complex): Complex {
        return Complex {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        };
    }
    
    // Complex + f64 (scalar addition)
    fn op+(scalar: f64): Complex {
        return Complex {
            real: self.real + scalar,
            imag: self.imag,
        };
    }
}

fn test(): i32 {
    let c1 = Complex { real: 1.0, imag: 2.0 };
    let c2 = Complex { real: 3.0, imag: 4.0 };
    
    let c3 = c1 + c2;      // ✅ op+(Complex)
    let c4 = c1 + 5.0;     // ✅ op+(f64)
    
    // Generic function with contract bound
    fn add_all<T: Add<T>>(values: Vec<T>): T {
        let mut sum = values[0];
        for i in 1..values.len() {
            sum = sum + values[i];
        }
        return sum;
    }
    
    let nums = vec![c1, c2];
    let total = add_all(nums);  // ✅ Complex impl Add<Complex>
    
    return 0;
}
```

## Karar

**Önerilen Yaklaşım**: **Hybrid System**

1. ✅ `fn op+()` syntax ekle (C++ uyumluluğu)
2. ✅ Generic contract'lar koru (type safety)
3. ✅ Overload resolution ekle (esneklik)
4. ✅ Contract bounds ile generic constraintler (güvenlik)

Bu sayede hem C++'taki gibi doğal syntax, hem de Rust'taki gibi type safety sağlanır.

## Implementation Roadmap

- [ ] Phase 1: Lexer/Parser - `fn op+()` syntax desteği
- [ ] Phase 2: Generic contract'lar - `contract Add<Rhs=Self>`
- [ ] Phase 3: Overload resolution - tip bazlı method seçimi
- [ ] Phase 4: Codegen - operator → method call desugaring
- [ ] Phase 5: Type checker - contract bound kontrolü

Estimated: 2-3 hafta (Phase 2 kapsamında)
