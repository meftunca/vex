# Vex Çekirdek Sözdizimi: Değişkenler ve Fonksiyonlar (v0.9 Taslağı)

Bu döküman, Vex'in hibrit (Go + Rust) felsefesine dayanan temel tanımlama sözdizimini (syntax) belgeler.

---

## 1. Değişken Tanımlama (Variable Definition)

Vex, güvenliği (safety) ve niyeti (intent) ön planda tutar. Değişken tanımlamak için tek anahtar kelime `let`'tir. Değişebilirlik (mutability), `!` (bang) operatörü ile açıkça (explicitly) belirtilir.

### 1.1. `let` (Değişmez - Immutable) **[Varsayılan]**

Vex'te varsayılan, güvenli ve tercih edilen değişken tanımlama yoludur. `let` ile tanımlanan bir değişkenin değeri **ASLA** yeniden atanamaz (cannot be reassigned).

Bu, Rust'ın `let`'i veya JavaScript'in `const`'u ile aynı felsefeyi paylaşır.

```vex
// 1. Değişmez (Immutable) Değişken (Tip Çıkarımı)
// 'message' artık "Merhaba, Vex!" değerine sabitlenmiştir.
let message = "Merhaba, Vex!";

// HATA: message = "Yeni değer" // Derleme Hatası (Compile Error)!

// 2. Tip Belirterek (Explicit Type)
let pi: f64 = 3.14159;
let is_ready: bool = true;
```

---

### 1.2. `let!` (Değişebilir - Mutable) **[Açık Tercih]**

Bir değişkenin değerinin sonradan değişeceğini (mutable) açıkça belirtir. `!` (bang) sembolü, bu değişkenin "normal" (`let`) davranışının dışına çıktığını ve değişebileceğini gösterir.

Bu, Rust'ın `let mut` sözdiziminin Vex'teki yerini alır.

```vex
// 1. Değişebilir (Mutable) Değişken (Tip Çıkarımı)
// 'let!' kullanımı, derleyiciye "Bu değişken değişebilir" sinyali verir.
let! counter = 0;
counter += 1; // Tamamen geçerli

// 2. Tip Belirterek (Explicit Type)
let! name: string = "Alice";
name = "Bob"; // Tamamen geçerli
```

---

### 1.3. `const` (Derleme Zamanı Sabiti)

Bu bir değişken değildir. Değeri, program çalışmadan önce (compile-time) bilinmek zorunda olan, değiştirilemez bir sabittir. Rust/C++/Go'nun `const`'u ile aynıdır.

```vex
// 'MAX_CONNECTIONS' koddaki her yerde '1000' ile değiştirilir.
const MAX_CONNECTIONS: i32 = 1000;

// HATA: const START_TIME = time::now(); // 'time::now()' bir runtime fonksiyonudur.
```

---

### 1.4. Referanslar (References)

Vex, Rust'ın "ödünç alma" (borrowing) modelini benimser, ancak `!` (bang) sözdizimi ile basitleştirir.

- **`&T` (Değişmez Referans - Immutable Reference)**: Veriye "sadece okuma" (read-only) erişimi sağlar.
- **`&T!` (Değişebilir Referans - Mutable Reference)**: Veriye "özel yazma" (exclusive write) erişimi sağlar. Bu, Rust'ın `&mut T`'sinin yerini alır.

```vex
fn read_data(data: &[i32]) {
    // data[0] = 5; // HATA! 'data' değişmezdir.
}

fn write_data(data: &[i32]!) {
    data[0] = 5; // Tamamen geçerli.
}
```

**Kullanım Örneği:**

```vex
let x = 42;
let ref1: &i32 = &x;           // Değişmez referans
// *ref1 = 50;                 // HATA: değiştirilemez

let! y = 100;
let ref2: &i32! = &y;          // Değişebilir referans
*ref2 = 200;                   // ✅ Geçerli
```

---

### 1.5. Heap Allocation (Bellekten Ayırma)

Vex, `new()` fonksiyonu ile açık (explicit) heap allocation sağlar. `new()`, Rust'ın `Box::new()` yerine kullanılır (daha basit syntax).

```vex
// Stack (varsayılan)
let small_array = [0; 100];            // 400 bytes → Stack

// EXPLICIT (when needed)
big := new([0; 1000000]);            // Heap allocation (automatic thread-safe RC)

// Kullanım (auto-dereference)
big_array[0] = 42;                     // Compiler auto-deref
config.host = "localhost";             // Compiler auto-deref
```

---

### 1.6. Shared Ownership (Paylaşımlı Sahiplik)

Birden fazla sahibin (owner) aynı veriye erişmesi gerektiğinde, `new()` otomatik olarak thread-safe referans sayacı (reference counting) kullanır.

```vex
// Shared ownership (AUTOMATIC thread-safe!)
let config = new(Config{...});         // Heap + atomic refcount
let clone = config;                    // Clone reference (refcount++)

// Multi-thread kullanım (automatic!)
spawn(move || {
    use_config(config);                // Thread 1 - Safe!
});

spawn(move || {
    use_config(clone);                 // Thread 2 - Safe!
});
```

**Compiler Intelligence:**

- Single-thread kullanım → `Rc` optimize (faster)
- Multi-thread access → `Arc` automatic (thread-safe)
- Developer'ın seçim yapmasına gerek yok!

**Not:** `new()` her zaman thread-safe'dir. Compiler, kullanım pattern'ine göre optimize eder (zero-cost abstraction).

---

## 2. Fonksiyon Tanımlama (Function Definition)

Vex, fonksiyon tanımlamak için `fn` anahtar kelimesini kullanır. Sözdizimi, Go/TypeScript (dönüş tipi için `:`) felsefesini izler.

### 2.1. Temel Sözdizimi

**Syntax:** `fn name(arg1: Type, arg2: Type): ReturnType { ... }`

```vex
// Parametreler ve dönüş tipi
// (Rust'ın '->' ok operatörü yerine ':' kullanılır)
fn add(a: i32, b: i32): i32 {
    return a + b;
}
```

---

### 2.2. Dönüş Tipi Olmayan (Void)

Dönüş tipi belirtilmezse, void (boş) kabul edilir.

```vex
fn log_message(msg: string) {
    log::info(msg);
}
```

---

### 2.3. Hata ve Union Tipi Dönüşü

Vex, `|` (Union Tipi) destekler. Bu, Go-tarzı `(T, error)` yerine TypeScript-tarzı `T | error` bir hata yönetimi sağlar.

```vex
// (string | error) Union tipini döndürme
fn read_config(path: string): (string | error) {
    if path == "" {
        return error.new("Yol boş olamaz");
    }

    let content = try fs::read_file(path); // 'try' operatörü
    return content;
}

// Kullanım
fn main() {
    let result = read_config("config.toml");

    // Pattern matching ile hata yönetimi
    match result {
        content: string => log::info(content),
        err: error => log::error(err.message()),
    }
}
```

---

## 3. Metot Tanımlama (Method Definition)

Fonksiyonların struct'lara (veya enum'lara) bağlanması için Go-tarzı "receiver" (alıcı) sözdizimi kullanılır.

### 3.1. Immutable Methods

```vex
struct Server {
    port: i32,
}

// Değişmez (Immutable) metot (receiver: &Server)
// 'self' yerine 's' gibi kısa isimler tercih edilir (Go felsefesi)
fn (s: &Server) address(): string {
    return f"127.0.0.1:{s.port}";
}
```

---

### 3.2. Mutable Methods

```vex
// Değişebilir (Mutable) metot (receiver: &Server!)
// 's'nin tipi '&Server!' olduğu için 's.port' değiştirilebilir.
fn (s: &Server!) set_port(new_port: i32) {
    s.port = new_port;
}
```

---

### 3.3. Tam Örnek

```vex
struct Server {
    port: i32,
}

fn (s: &Server) address(): string {
    return f"127.0.0.1:{s.port}";
}

fn (s: &Server!) set_port(new_port: i32) {
    s.port = new_port;
}

// Kullanım
fn main() {
    // 'server' değişkenini 'let!' (mutable) olarak tanımlıyoruz
    let! server = Server{ port: 80 };

    log::info(server.address()); // "127.0.0.1:80"

    // 'server' değişebilir olduğu için 'set_port' çağrılabilir
    server.set_port(8080);

    log::info(server.address()); // "127.0.0.1:8080"
}
```

---

## 4. Eski Sözdizimi Karşılaştırması

### Değişken Tanımlama

| Eski (v0.2)      | Yeni (v0.9)         | Açıklama                               |
| ---------------- | ------------------- | -------------------------------------- |
| `let x = 42`     | `let x = 42`        | Immutable (değişmez)                   |
| `let mut x = 42` | `let! x = 42`       | Mutable (değişebilir)                  |
| `var x = 42`     | `let! x = 42`       | Mutable (kısa form kaldırıldı)         |
| `x := 42`        | `let x = 42`        | Type inference (short form kaldırıldı) |
| `const X = 42`   | `const X: i32 = 42` | Compile-time constant (tip zorunlu)    |

### Referans Tanımlama

| Eski (v0.2)     | Yeni (v0.9) | Açıklama                        |
| --------------- | ----------- | ------------------------------- |
| `&T`            | `&T`        | Immutable reference             |
| `&mut T`        | `&T!`       | Mutable reference (bang syntax) |
| `*T` (belirsiz) | `&T!`       | Mutable reference (açık)        |

### Heap Allocation

| Eski (v0.2)   | Yeni (v0.9)   | Açıklama                          |
| ------------- | ------------- | --------------------------------- |
| `Box::new(x)` | `new(x)`      | Heap allocation (basitleştirildi) |
| `Rc::new(x)`  | `Rc::new(x)`  | Reference counted (aynı)          |
| `Arc::new(x)` | `Arc::new(x)` | Thread-safe RC (aynı)             |

---

## 5. Borrow Checker Kuralları

Vex, Rust'ın borrow checker kurallarını uygular:

### Rule 1: Single Mutable or Multiple Immutable

```vex
let! x = 42;

// ✅ OK: Multiple immutable references
let r1: &i32 = &x;
let r2: &i32 = &x;

// ✅ OK: Single mutable reference
let r3: &i32! = &x;

// ❌ ERROR: Can't have mutable + immutable simultaneously
let r4: &i32 = &x;   // Immutable
let r5: &i32! = &x;  // Mutable - CONFLICT!
```

### Rule 2: References Must Not Outlive Data

```vex
fn broken(): &i32 {
    let x = 42;
    return &x;  // ❌ ERROR: x dies at end of function
}

fn fixed(): &i32 {
    let x = new(42);  // Heap allocated
    return x;         // ✅ OK: ownership transferred
}
```

### Rule 3: Immutable Cannot Be Modified

```vex
let x = 42;
let r: &i32 = &x;
// *r = 50;  // ❌ ERROR: cannot modify through immutable reference

let! y = 100;
let r: &i32! = &y;
*r = 200;  // ✅ OK: mutable reference
```

---

## 6. Escape Analysis (Otomatik Heap Promotion)

Compiler, bir değişkenin scope'undan "kaçtığını" (escapes) tespit ederse, otomatik olarak heap'e taşır.

```vex
// Case 1: Doesn't escape → Stack
fn compute(): i32 {
    let x = 42;        // Stack
    return x;          // Value copied
}

// Case 2: Reference escapes → Compiler ERROR (prevent dangling)
fn create_ref(): &i32 {
    let x = 42;
    return &x;         // ❌ ERROR: x dies, reference would dangle
}

// Case 3: Explicit heap → OK
fn create_heap(): &i32 {
    let x = new(42);   // Explicit heap
    return x;          // ✅ OK: ownership transferred
}

// Case 4: Return by value → Stack, copy out
fn create_struct(): Point {
    let p = Point{x: 10, y: 20};  // Stack
    return p;                      // Copied to caller's stack
}
```

---

## 7. Memory Layout Examples

### Example 1: Pure Stack (Zero Allocation)

```vex
fn fibonacci(n: i32): i32 {
    if n <= 1 { return n; }

    let! a = 0;
    let! b = 1;
    let! i = 2;

    while i <= n {
        let temp = a + b;
        a = b;
        b = temp;
        i += 1;
    }

    return b;
}
// Memory: 12 bytes stack (3 × i32)
// Allocation: 0
// Performance: Optimal
```

### Example 2: Borrowing (Zero Copy)

```vex
fn sum_array(arr: &[i32]): i32 {
    let! sum = 0;
    for x in arr {
        sum += x;
    }
    return sum;
}

fn main() {
    let data = [1, 2, 3, 4, 5];  // Stack array
    let result = sum_array(&data); // Borrow, no copy
    log::info(f"Sum: {result}");
}
// Memory: Array on stack, reference passed (8 bytes pointer)
// Allocation: 0
// Performance: Optimal
```

### Example 3: Explicit Heap

```vex
fn load_big_data(): &[i32] {
    let data = new([0; 1000000]);  // 4MB heap

    // Process data
    for i in 0..data.len() {
        data[i] = compute(i);
    }

    return data;  // Ownership transferred
}
// Memory: 4MB heap
// Allocation: 1 (malloc)
// Performance: Acceptable for large data
```

### Example 4: Shared Ownership (Automatic Thread-Safe)

```vex
fn create_shared_config(): Config {
    let config = new(Config{
        host: "localhost",
        port: 8080,
    });

    return config;
}

fn main() {
    let config1 = create_shared_config();
    let config2 = config1;          // Clone reference (refcount: 2)

    // Both can access, immutable by default
    log::info(config1.host);
    log::info(config2.port);

    // Multi-thread access (automatic thread-safe!)
    spawn(move || log::info(config1.port));
    spawn(move || log::info(config2.host));
}
// Memory: 1 Config on heap + atomic refcount metadata
// Allocation: 1 (automatic Arc allocation)
// Cleanup: Automatic when refcount → 0
```

---

## 8. Best Practices

### ✅ DO

```vex
// 1. Default to immutable
let x = 42;                    // Preferred

// 2. Use let! only when needed
let! counter = 0;              // Mutability explicit

// 3. Borrow instead of copy
fn process(data: &[i32]) { }   // Zero-copy

// 4. Explicit heap for large data
let big = new([0; 1000000]);   // Clear intent

// 5. Use Rc/Arc for shared ownership
let shared = Rc::new(data);    // Explicit sharing
```

### ❌ DON'T

```vex
// 1. Don't overuse let!
let! x = 42;                   // Unnecessary if x never changes

// 2. Don't pass large structs by value
fn process(data: [i32; 1000000]) { }  // Expensive copy!
// Better:
fn process(data: &[i32]) { }          // Zero-copy

// 3. Don't use new() for small data
let x = new(42);               // Unnecessary heap allocation
// Better:
let x = 42;                    // Stack is fine

// 4. Don't clone unnecessarily
let copy = data.clone();       // Expensive!
// Better:
let ref = &data;               // Borrow instead
```

---

## 9. Özet (Summary)

| Kavram                    | Syntax               | Mutability | Location | Örnek                   |
| ------------------------- | -------------------- | ---------- | -------- | ----------------------- |
| **Immutable variable**    | `let x = 42`         | ❌         | Stack    | `let x = 42;`           |
| **Mutable variable**      | `let! x = 42`        | ✅         | Stack    | `let! counter = 0;`     |
| **Compile-time constant** | `const X: T = V`     | ❌         | N/A      | `const MAX: i32 = 100;` |
| **Immutable reference**   | `&T`                 | ❌         | Stack    | `let r: &i32 = &x;`     |
| **Mutable reference**     | `&T!`                | ✅         | Stack    | `let r: &i32! = &x;`    |
| **Heap allocation**       | `new(V)`             | Depends    | Heap     | `let x = new([0; 1M]);` |
| **Shared ownership**      | `new(V)` (automatic) | ❌\*       | Heap     | `let s = new(data);`    |

\*`new()` ile yaratılan shared data varsayılan olarak immutable'dır. Mutable shared data için `Mutex` veya `RwLock` kullanın.

---

## 10. Migrasyon Rehberi (v0.2 → v0.9)

### Otomatik Değişiklikler (Formatter/Linter ile)

```vex
// Eski → Yeni
let mut x = 42;        →  let! x = 42;
var x = 42;            →  let! x = 42;
x := 42;               →  let x = 42;
&mut T                 →  &T!
Box::new(x)            →  new(x)
```

### Manuel İnceleme Gerektirenler

```vex
// Const tipi ekleme
const MAX = 100;       →  const MAX: i32 = 100;  // Type required

// Receiver syntax güncellemesi
fn (s: &mut Server)    →  fn (s: &Server!)
```

---

**Son Güncelleme:** 3 Kasım 2025  
**Versiyon:** v0.9 Taslak  
**Durum:** Önerilmiş, Henüz Uygulanmadı
