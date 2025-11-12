# Operator Overloading: Vex vs C++

## Temel Farklar

### C++ Yaklaşımı
```cpp
class Point {
    int x, y;
public:
    // Operator overloading
    Point operator+(const Point& other) const {
        return Point(x + other.x, y + other.y);
    }
    
    // Farklı tiplerle
    Point operator*(int scalar) const {
        return Point(x * scalar, y * scalar);
    }
};

// Kullanım
Point p3 = p1 + p2;      // operator+ çağrılır
Point p4 = p1 * 5;       // operator* çağrılır
```

### Vex Yaklaşımı
```vex
contract Add {
    fn add(other: Self): Self;
}

struct Point impl Add {
    x: i32,
    y: i32,
    
    fn add(other: Point): Point {
        return Point {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

// Kullanım
let p3 = p1 + p2;  // p1.add(p2) metoduna desugar edilir
```

## Karşılaştırma Tablosu

| Özellik | C++ | Vex | Açıklama |
|---------|-----|-----|----------|
| **Syntax** | `operator+` | `fn add()` | Vex daha açık method isimleri kullanır |
| **Aynı Tip** | ✅ `Point + Point` | ✅ `Point + Point` | Her ikisi de destekler |
| **Farklı Tipler** | ✅ `Point * int` | ⚠️ Manuel method | Vex'te `v.mul_scalar(5)` şeklinde |
| **Çoklu Contract** | ✅ Birden fazla operator | ✅ `impl Add, Display` | Her ikisi de destekler |
| **Desugaring** | Compiler magic | `a + b` → `a.add(b)` | Vex açıkça method call'a çevrilir |
| **Receiver** | `const Point&` | `self` (implicit) | Vex otomatik `&self` ekler |
| **Return Type** | Herhangi bir tip | `Self` veya named type | Vex type safety daha sıkı |

## Örnekler

### 1. Aynı Tip Operator Overloading ✅
**Hem C++ hem Vex destekler**

```vex
// Vex
struct Point impl Add {
    x: i32,
    y: i32,
    
    fn add(other: Point): Point {
        return Point {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}

let p3 = p1 + p2;  // ✅ Çalışır
```

```cpp
// C++ Equivalent
class Point {
    int x, y;
public:
    Point operator+(const Point& other) const {
        return Point(x + other.x, y + other.y);
    }
};

Point p3 = p1 + p2;  // ✅ Çalışır
```

### 2. Farklı Tiplerle Operator Overloading ⚠️
**C++ destekler, Vex manuel method gerektirir**

```cpp
// C++ - Farklı tipler
class Vector {
    int x, y;
public:
    Vector operator*(int scalar) const {  // ✅ Vector * int
        return Vector(x * scalar, y * scalar);
    }
};

Vector v2 = v1 * 5;  // ✅ Çalışır
```

```vex
// Vex - Özel contract gerekir
contract MulScalar {
    fn mul_scalar(scalar: i32): Self;
}

struct Vector impl MulScalar {
    x: i32,
    y: i32,
    
    fn mul_scalar(scalar: i32): Vector {
        return Vector {
            x: self.x * scalar,
            y: self.y * scalar,
        };
    }
}

// let v2 = v1 * 5;  // ❌ Çalışmaz (operator* contract yok)
let v2 = v1.mul_scalar(5);  // ✅ Manuel method call gerekir
```

**Neden?**
- C++ `operator*` hem `Vector * int` hem `int * Vector` için overload edilebilir
- Vex'te `*` operatörü sadece `Mul` contract'ına bakar
- `Mul` contract `fn mul(other: Self): Self` imzası istediği için farklı tip kabul etmez

### 3. Çoklu Contract (Multiple Traits) ✅
**Hem C++ hem Vex destekler**

```vex
// Vex - Birden fazla contract
struct Complex impl Add, Display {
    real: f64,
    imag: f64,
    
    fn add(other: Complex): Complex {
        return Complex {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        };
    }
    
    fn to_string(): string {
        return "Complex";
    }
}

let c3 = c1 + c2;  // ✅ Add contract
let s = c1.to_string();  // ✅ Display contract
```

```cpp
// C++ - Birden fazla operator
class Complex {
    double real, imag;
public:
    Complex operator+(const Complex& other) const {
        return Complex(real + other.real, imag + other.imag);
    }
    
    std::string to_string() const {
        return "Complex";
    }
};

Complex c3 = c1 + c2;  // ✅ operator+
auto s = c1.to_string();  // ✅ method
```

## Teknik Detaylar

### Vex Contract System
```vex
contract Add {
    fn add(other: Self): Self;  // Self = aynı tip
}

// Builtin implementations
i32 extends Add, Sub, Mul, Div, Rem;
f64 extends Add, Sub, Mul, Div, Rem;

// User implementations
struct Point impl Add {
    fn add(other: Point): Point { ... }
}
```

### Desugar Mekanizması
```vex
// Kaynak kod
let result = a + b;

// Compiler tarafından şuna çevrilir:
// 1. a'nın tipini kontrol et (Point)
// 2. Point'in Add contract'ı var mı? ✅
// 3. Desugar to: let result = a.add(b);
```

### Binary Ops Dispatch (binary_ops.rs)
```rust
// Vex compiler içinde
if let Type::Named(ref type_name) = left_type {
    let (contract_name, method_name) = self.binary_op_to_trait(op);
    // op=Add → ("Add", "add")
    
    if builtin_contracts::has_builtin_contract(type_name, contract_name) {
        // Builtin tip (i32, f64) → LLVM ops'a fallback
        return fallback_builtin_ops();
    }
    
    if self.has_operator_trait(type_name, contract_name) {
        // User-defined contract → method call
        return self.compile_method_call(left, method_name, &[right]);
    }
}
```

## Sınırlamalar ve Çözümler

### ❌ Şu An Desteklenmiyor
1. **Farklı tiplerle operator overloading**
   ```vex
   // ❌ let v2 = v1 * 5;  // Vector * i32
   ```
   **Çözüm**: Manuel method kullan
   ```vex
   // ✅ let v2 = v1.mul_scalar(5);
   ```

2. **Commutative operators (Değişmeli)**
   ```vex
   // ✅ let v2 = v1 * 5;  // Vector.mul_scalar(5)
   // ❌ let v2 = 5 * v1;  // i32 doesn't impl Mul<Vector>
   ```
   **Çözüm**: Gelecek versiyonda generic contract'lar
   ```vex
   // Future syntax (Phase 2)
   contract Mul<Rhs = Self> {
       type Output;
       fn mul(rhs: Rhs): Output;
   }
   
   struct Vector impl Mul<i32> {
       type Output = Vector;
       fn mul(scalar: i32): Vector { ... }
   }
   ```

### ✅ Şu An Destekleniyor
1. **Aynı tipte operator overloading**
   ```vex
   ✅ Point + Point
   ✅ Complex + Complex
   ```

2. **Birden fazla contract**
   ```vex
   ✅ struct X impl Add, Sub, Mul, Display
   ```

3. **Builtin tip operatörleri**
   ```vex
   ✅ i32 + i32 (builtin Add contract)
   ✅ f64 * f64 (builtin Mul contract)
   ```

## Gelecek Planları (Phase 2)

### Generic Contracts
```vex
// Rust-style associated types
contract Mul<Rhs = Self> {
    type Output;
    fn mul(rhs: Rhs): Output;
}

// Farklı tiplerle multiply
struct Vector impl Mul<i32> {
    type Output = Vector;
    fn mul(scalar: i32): Vector {
        return Vector {
            x: self.x * scalar,
            y: self.y * scalar,
        };
    }
}

// Kullanım
let v2 = v1 * 5;  // ✅ Vector.mul(5) → Vector
```

### Default Implementations
```vex
contract AddAssign {
    fn add_assign(other: Self) {
        // Default implementation
        self = self.add(other);
    }
}
```

## Özet

| Senaryo | Vex Desteği | Workaround |
|---------|-------------|------------|
| `Point + Point` | ✅ Tam destek | - |
| `Vector * 5` | ⚠️ Manuel method | `v.mul_scalar(5)` |
| `5 * Vector` | ❌ Desteklenmiyor | Gelecekte generic contracts |
| `impl Add, Sub` | ✅ Çoklu contract | - |
| Builtin ops | ✅ Otomatik | `i32 extends Add` |

**Sonuç**: Vex operator overloading şu an **aynı tipte** C++ gibi çalışıyor, ancak **farklı tiplerle** manuel method gerekiyor. Phase 2'de generic contract'lar ile C++ gibi esneklik gelecek.
