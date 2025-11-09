# Vex Sözdizimi Kılavuzu (Syntax Guide) v0.9

**Sürüm:** 0.2.0  
**Syntax Versiyonu:** v0.9  
**Son Güncelleme:** 3 Kasım 2025

Vex programlama dilinin tam sözdizimi referansı. Bu doküman, lexer, parser ve AST'den derlenen gerçek implementasyona dayanır.

---

## 📑 İçindekiler

1. [Temel Sözdizimi](#1-temel-sözdizimi)
2. [Değişkenler ve Sabitler](#2-değişkenler-ve-sabitler)
3. [Tipler (Types)](#3-tipler-types)
4. [Fonksiyonlar](#4-fonksiyonlar)
5. [Kontrol Akışı](#5-kontrol-akışı)
6. [Veri Yapıları](#6-veri-yapıları)
7. [Trait Sistemi](#7-trait-sistemi)
8. [İfadeler (Expressions)](#8-i̇fadeler-expressions)
9. [Pattern Matching](#9-pattern-matching)
10. [Modül Sistemi](#10-modül-sistemi)
11. [Async/Await](#11-asyncawait)
12. [Özel Özellikler](#12-özel-özellikler)

---

## 1. Temel Sözdizimi

### 1.1 Program Yapısı

```vex
// Import statements (optional)
import { io, log } from "std";

// Top-level items
fn main() : i32 {
    return 0;
}
```

### 1.2 Yorumlar (Comments)

```vex
// Tek satırlık yorum

/*
   Çok satırlı
   yorum bloğu
*/
```

### 1.3 Dosya Uzantısı

- Dosya uzantısı: `.vx`
- Encoding: UTF-8

---

## 2. Değişkenler ve Sabitler

### 2.1 `let` - Değişmez (Immutable) Değişken

**Varsayılan** değişken tanımlama yöntemi. Rust'ın `let` veya JavaScript'in `const` davranışı.

```vex
let x = 42;              // Tip çıkarımı (type inference)
let name = "Vex";        // string tipi
let pi = 3.14;           // f64 tipi

// Explicit type
let age: i32 = 25;
let height: f32 = 1.75;
let is_active: bool = true;

// ❌ HATA: Değişmez değişkene atama yapılamaz
// x = 50;  // Compile error
```

### 2.2 `let!` - Değişebilir (Mutable) Değişken

`!` (bang) operatörü ile açıkça belirtilir. Rust'ın `let mut` yerine geçer.

```vex
let! counter = 0;        // Mutable variable
counter = counter + 1;   // ✅ OK
counter += 1;            // ✅ OK (parsed, codegen pending)

let! name: string = "Alice";
name = "Bob";            // ✅ OK

// v0.9'da artık kullanılmaz:
// let mut x = 10;  // ❌ HATA: 'mut' keyword removed
```

### 2.3 `const` - Derleme Zamanı Sabiti

Değeri compile-time'da bilinmeli. Runtime fonksiyonları kullanılamaz.

```vex
const MAX_SIZE: i32 = 1000;
const PI: f64 = 3.14159;
const APP_NAME: string = "Vex Lang";

// ❌ HATA: Runtime değer
// const START_TIME = time::now();
```

---

## 3. Tipler (Types)

### 3.1 İlkel Tipler (Primitive Types)

#### İşaretli Tam Sayılar (Signed Integers)

```vex
i8      // -128 to 127
i16     // -32,768 to 32,767
i32     // -2,147,483,648 to 2,147,483,647
i64     // -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
```

#### İşaretsiz Tam Sayılar (Unsigned Integers)

```vex
u8      // 0 to 255 (byte alias)
u16     // 0 to 65,535
u32     // 0 to 4,294,967,295
u64     // 0 to 18,446,744,073,709,551,615
byte    // u8 alias
```

#### Kayan Nokta (Floating Point)

```vex
f32     // IEEE 754 single precision
f64     // IEEE 754 double precision
```

#### Diğer İlkel Tipler

```vex
bool    // true, false
string  // UTF-8 string (immutable)
error   // Error type (nil-able)
nil     // Null value
```

### 3.2 Bileşik Tipler (Composite Types)

#### Array (Sabit Boyutlu Dizi)

```vex
[i32; 5]           // 5 elemanlı i32 dizisi
[string; 10]       // 10 elemanlı string dizisi

// Kullanım
let arr: [i32; 3] = [1, 2, 3];
let first = arr[0];
```

#### Slice (Dinamik Görünüm)

```vex
&[T]               // Immutable slice
&[T]!              // Mutable slice

// Örnekler
fn process(data: &[i32]) { }
fn modify(data: &[i32]!) { }
```

#### Tuple (Çoklu Değer)

```vex
(i32, string)      // 2'li tuple
(i32, f64, bool)   // 3'lü tuple

// Kullanım
let point: (i32, i32) = (10, 20);
let (x, y) = point;  // Destructuring
```

#### Reference (Referans)

```vex
&T                 // Immutable reference (v0.9)
&T!                // Mutable reference (v0.9, NOT &mut T)

// Örnekler
fn read(data: &string) { }
fn write(data: &string!) { }

// Referans oluşturma
let x = 42;
let ref_x = &x;      // Immutable reference
let! y = 10;
let ref_y = &y!;     // Mutable reference
```

### 3.3 Generic Tipler

```vex
Vec<T>                    // Generic vector
Option<T>                 // Option type
Result<T, E>              // Result type
Box<T>                    // Heap allocated
Map<K, V>                 // Map type

// Kullanım
let numbers: Vec<i32> = Vec::new();
let maybe: Option<string> = Some("value");
```

### 3.4 Union ve Intersection (TypeScript-style)

```vex
// Union type (henüz tam implementasyon yok)
(i32 | f64)               // i32 veya f64
(string | error)          // string veya error

// Intersection type (henüz tam implementasyon yok)
(Reader & Writer)         // Hem Reader hem Writer
```

### 3.5 Type Alias

```vex
type UserID = u64;
type Point2D = (f32, f32);
type Result<T> = (T | error);

// Kullanım
let id: UserID = 12345;
let pos: Point2D = (1.5, 2.5);
```

---

## 4. Fonksiyonlar

### 4.1 Temel Fonksiyon Tanımı

```vex
// Parametresiz, dönüş değeri yok
fn hello() {
    // body
}

// Parametreli, dönüş değeri var
fn add(a: i32, b: i32) : i32 {
    return a + b;
}

// Tip çıkarımlı parametreler (henüz değil)
// fn process(x) { }  // ❌ Şu anda desteklenmiyor
```

### 4.2 Generic Fonksiyonlar

```vex
// Tek tip parametresi
fn identity<T>(x: T) : T {
    return x;
}

// Çoklu tip parametreleri
fn pair<T, U>(first: T, second: U) : (T, U) {
    return (first, second);
}

// Kullanım
let x = identity<i32>(42);
let y = identity<string>("hello");
```

### 4.3 Method Syntax (Receivers)

**v0.9 Güncellemesi:** Üç method tanımlama yöntemi destekleniyor:

#### 1. **Simplified Syntax (Struct İçinde - Önerilen)**

```vex
struct Point {
    x: i32,
    y: i32,

    // Receiver auto-generated (implicit &Point veya &Point!)
    fn distance_from_origin(): i32 {
        return self.x + self.y;
    }

    fn translate(dx: i32, dy: i32) {
        self.x = self.x + dx;  // Mutation çalışır
        self.y = self.y + dy;
    }
}
```

#### 2. **Golang-Style (Struct İçinde veya Dışında)**

```vex
// Explicit receiver - isim serbest
fn (p: &Point) distance_from_origin(): i32 {
    return p.x + p.y;
}

// Mutable receiver
fn (point: &Point!) translate(dx: i32, dy: i32) {
    point.x = point.x + dx;
    point.y = point.y + dy;
}

// ⚠️ Receiver parametresi herhangi bir isim olabilir:
fn (self: &Point) get_x(): i32 { return self.x; }
fn (this: &Point) get_y(): i32 { return this.y; }
fn (p: &Point) sum(): i32 { return p.x + p.y; }
```

#### 3. **Hybrid (İkisi Bir Arada)**

```vex
struct Calculator {
    value: i32,

    // Simplified (struct içinde)
    fn get_value(): i32 {
        return self.value;
    }
}

// Golang-style extension (struct dışında)
fn (calc: &Calculator!) add(x: i32) {
    calc.value = calc.value + x;
}
```

**Kullanım:**

```vex
let! point = Point { x: 10, y: 20 };
let dist = point.distance_from_origin();
point.translate(5, 5);
```

### 4.4 Recursive Functions

```vex
fn fibonacci(n: i32) : i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn factorial(n: i32) : i32 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}
```

### 4.5 Async Functions (Parser hazır, codegen bekliyor)

```vex
async fn fetch_data(url: string) : string {
    let response = await http::get(url);
    return response.body;
}

// Kullanım
let data = await fetch_data("https://api.example.com");
```

---

## 5. Kontrol Akışı

### 5.1 If-Else

```vex
// Basit if
if x > 10 {
    return 1;
}

// If-else
if x > 10 {
    return 1;
} else {
    return 0;
}

// If-else if-else
if score >= 90 {
    return 5;  // A
} else {
    if score >= 80 {
        return 4;  // B
    } else {
        return 3;  // C
    }
}

// Boolean expressions
if x > 0 {
    if x < 100 {
        return true;
    }
}
```

### 5.2 While Loop

```vex
let! i = 0;
while i < 10 {
    i = i + 1;
}

// Sonsuz döngü
while true {
    // break; (henüz implementasyon yok)
}
```

### 5.3 For Loop (C-style)

```vex
// Classic for loop
for let! i = 0; i < 10; i = i + 1 {
    // body
}

// Nested loops
for let! i = 0; i < 5; i = i + 1 {
    for let! j = 0; j < 5; j = j + 1 {
        // body
    }
}
```

### 5.4 For-In Loop (Parser hazır, codegen bekliyor)

```vex
// Iterate over range
for item in 0..10 {
    // body
}

// Iterate over array
for element in array {
    // body
}
```

### 5.5 Switch-Case

```vex
switch value {
    case 1:
        return 10;
    case 2, 3:
        return 20;
    case 4, 5, 6:
        return 30;
    default:
        return 0;
}

// Typeless switch (Go-style)
switch {
    case x > 10:
        return 1;
    case x > 5:
        return 2;
    default:
        return 0;
}
```

### 5.6 Match Expression (Parser hazır, codegen kısmi)

```vex
match value {
    0 => return "zero",
    1 => return "one",
    _ => return "other",
}

// With guard clause
match x {
    n if n > 0 => return "positive",
    0 => return "zero",
    _ => return "negative",
}

// Pattern matching
match point {
    (0, 0) => return "origin",
    (x, 0) => return "on x-axis",
    (0, y) => return "on y-axis",
    _ => return "other",
}
```

---

## 6. Veri Yapıları

### 6.1 Struct

```vex
// Basit struct tanımı
struct Point {
    x: i32,
    y: i32,
}

// Generic struct
struct Box<T> {
    value: T,
}

// Struct with tag (Go-style)
struct User {
    id: i32,
    name: string `json:"username" db:"user_name"`,
}

// Struct literal
let p = Point { x: 10, y: 20 };
let box_int = Box<i32> { value: 42 };

// Field access
let x_coord = p.x;
let y_coord = p.y;
```

### 6.2 Struct with Methods (Inline)

```vex
struct Point {
    x: i32,
    y: i32,

    fn (self: &Point) distance() : i32 {
        return self.x + self.y;
    }

    fn (self: &Point!) move(dx: i32, dy: i32) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}
```

### 6.3 Enum (C-style ve Data-carrying)

```vex
// C-style enum
enum Color {
    Red,
    Green,
    Blue,
}

// Data-carrying enum (parser hazır, codegen bekliyor)
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Kullanım
let color = Color::Red;
let maybe_value = Option::Some(42);
let result = Result::Ok("success");
```

### 6.4 Tuple

```vex
// Tuple literal
let point = (10, 20);
let person = ("Alice", 25, true);

// Destructuring
let (x, y) = point;
let (name, age, is_active) = person;

// Tuple type annotation
let coords: (i32, i32) = (100, 200);
```

---

## 7. Trait Sistemi

### 7.1 Trait Tanımı

**v0.9 Güncellemesi (Kasım 2025):** Trait method'ları artık **simplified syntax** kullanıyor:

- ✅ Yeni: `fn method(params): type;` (receiver yok)
- ❌ Eski: `fn method(params): type;`
- ⚠️ Trait method'ları **body içeremez** (sadece signature)

```vex
// Basit trait (sadece required methods - v0.9 syntax)
trait Logger {
    fn log(level: string, msg: string);  // Simplified syntax
    fn info(msg: string);
}

// ❌ HATA: Trait methods cannot have body
trait Display {
    fn show();  // ✅ Sadece signature

    // fn print() {  // ❌ Body not allowed in traits
    //     self.show();
    // }
}

// Generic trait
trait Converter<T> {
    fn to_type(): T;  // Simplified syntax
}

// Opsiyonel: Golang-style hala destekleniyor
trait Writer {
    fn write(data: &[byte]);  // ✅ Still valid
}
```

### 7.2 Trait Inheritance

```vex
// Trait inheritance
trait Writer: Formatter, Closer {
    fn write(data: &[byte]);
}

// Multiple inheritance
trait Logger: Display, Debug {
    fn log(msg: string);
}
```

### 7.3 Inline Trait Implementation (v1.3)

**v0.9 Güncellemesi:** Struct method'ları da **simplified syntax** kullanıyor:

- ✅ Yeni: `fn method(params) { ... }` (receiver yok - auto-generated)
- ✅ Alternatif: `fn (self: &T!) method(params) { ... }` (golang-style - opsiyonel)
- 📝 Not: Receiver parametresi **herhangi bir isim** olabilir: `(x: &T)`, `(this: &T)`, vb.

```vex
// Struct implements trait inline (v0.9 simplified syntax)
struct FileLogger impl Logger {
    path: string,

    // Simplified syntax - receiver auto-generated
    fn log(level: string, msg: string) {
        print("[", level, "] ", msg);
        return;
    }

    fn info(msg: string) {
        self.log("INFO", msg);  // 'self' kullanılabilir
        return;
    }
}

// Golang-style (opsiyonel - hala destekleniyor)
struct ConsoleLogger impl Logger {
    prefix: string,

    // Explicit receiver
    fn (self: &ConsoleLogger!) log(level: string, msg: string) {
        print(self.prefix, " [", level, "] ", msg);
    }

    // Receiver ismi serbest
    fn (logger: &ConsoleLogger) info(msg: string) {
        logger.log("INFO", msg);
    }
}

// Multiple traits (simplified)
struct File impl Reader, Writer, Closer {
    fd: i32,

    fn read(buf: &[byte]!): i32 {
        return 0;
    }

    fn write(data: &[byte]): i32 {
        return 0;
    }

    fn close() {
        // Close file
    }
}
```

### 7.4 Separate Trait Implementation (Parser hazır, codegen bekliyor)

```vex
impl Logger for FileLogger {
    fn (self: &FileLogger!) log(level: string, msg: string) {
        // Implementation
    }
}

// Generic impl
impl<T> Display for Box<T> {
    fn (self: &Box<T>!) show() {
        // Implementation
    }
}
```

---

## 8. İfadeler (Expressions)

### 8.1 Literal Expressions

```vex
// Integer literals
42
-100
0

// Float literals
3.14
-2.5
1.0

// String literals
"hello"
"world"

// F-string (interpolation parsing hazır, codegen kısmi)
f"Hello, {name}!"
f"x = {x}, y = {y}"

// Boolean literals
true
false

// Nil
nil
```

### 8.2 Binary Operations

```vex
// Arithmetic
x + y
x - y
x * y
x / y
x % y

// Comparison
x == y
x != y
x < y
x <= y
x > y
x >= y

// Logical
x && y
x || y
```

### 8.3 Unary Operations

```vex
// Negation
-x

// Logical NOT
!condition

// Reference
&value       // Immutable reference
&value!      // Mutable reference (v0.9)

// Dereference
*pointer
```

### 8.4 Postfix Operations (Parser hazır, codegen kısmi)

```vex
x++          // Increment
x--          // Decrement
```

### 8.5 Compound Assignment (Parser hazır, codegen bekliyor)

```vex
x += 5       // x = x + 5
x -= 3       // x = x - 3
x *= 2       // x = x * 2
x /= 4       // x = x / 4
```

### 8.6 Function Call

```vex
// Regular function call
add(10, 20)
fibonacci(10)

// Generic function call
identity<i32>(42)
max<f64>(1.5, 2.5)

// Method call
point.distance()
list.push(item)
```

### 8.7 Field Access & Indexing

```vex
// Field access
point.x
person.name
rect.width

// Array indexing
arr[0]
matrix[i][j]
```

### 8.8 Range (Parser hazır, codegen bekliyor)

```vex
0..10        // Range from 0 to 10 (exclusive)
1..=100      // Range from 1 to 100 (inclusive, future)
```

### 8.9 Type Cast

```vex
expr as i32
value as f64
number as string
```

---

## 9. Pattern Matching

### 9.1 Wildcard Pattern

```vex
match x {
    _ => return 0,  // Matches anything
}
```

### 9.2 Literal Pattern

```vex
match value {
    0 => return "zero",
    1 => return "one",
    42 => return "answer",
    _ => return "other",
}
```

### 9.3 Identifier Binding

```vex
match value {
    x => return x + 1,  // Binds value to x
}
```

### 9.4 Tuple Pattern

```vex
match point {
    (0, 0) => return "origin",
    (x, 0) => return "on x-axis",
    (0, y) => return "on y-axis",
    (x, y) => return "other",
}
```

### 9.5 Struct Pattern

```vex
match shape {
    Point { x, y } => return x + y,
    Rectangle { width, height } => return width * height,
    _ => return 0,
}
```

### 9.6 Enum Pattern (Parser hazır, codegen bekliyor)

```vex
match result {
    Ok(value) => return value,
    Err(e) => return default_value,
}

match option {
    Some(x) => return x,
    None => return 0,
}
```

---

## 10. Modül Sistemi

### 10.1 Import Statements

```vex
// Named imports
import { io, log } from "std";
import { Reader, Writer } from "std::io";

// Namespace import
import * as std from "std";
// Kullanım: std::io::read()

// Module import (entire module)
import "std/io";

// With alias (future)
import { read as read_file } from "std::io";
```

### 10.2 Export Statements

```vex
// Export list
export { io, net, http };

// Export function
export fn public_api() {
    // implementation
}

// Export constant
export const VERSION: string = "0.9.0";

// Export struct
export struct PublicStruct {
    field: i32,
}
```

### 10.3 Module Path Resolution

```
"std"           → vex-libs/std/mod.vx
"std::io"       → vex-libs/std/io/mod.vx
"std::net::tcp" → vex-libs/std/net/tcp.vx
```

---

## 11. Async/Await

### 11.1 Async Functions (Parser hazır, codegen bekliyor)

```vex
async fn fetch_user(id: u64) : User {
    let response = await http::get(f"/users/{id}");
    return response.json();
}

async fn process_data() {
    let user = await fetch_user(123);
    let posts = await fetch_posts(user.id);
    // Process...
}
```

### 11.2 Go Statement (Concurrency)

```vex
// Spawn goroutine/task
go expensive_computation();

// With function call
go process_item(item);

// Anonymous function (future)
go fn() {
    // Concurrent work
};
```

### 11.3 Select Statement (Parser hazır, codegen bekliyor)

```vex
select {
    result = await channel1.recv():
        // Handle result from channel1
    timeout = await time::sleep(1000):
        // Handle timeout
}
```

---

## 12. Özel Özellikler

### 12.1 Attributes

```vex
// Function attributes
#[inline]
fn fast_function() { }

#[cfg(target_os = "linux")]
fn linux_only() { }

#[vectorize]
fn simd_computation(data: &[f32]) { }

// Struct attributes
#[repr(C)]
struct CCompatible {
    field: i32,
}
```

### 12.2 Unsafe Blocks (Parser hazır)

```vex
unsafe {
    // Raw pointer operations
    let ptr = &raw x;
    *ptr = 42;
}
```

### 12.3 Extern Blocks (FFI)

```vex
extern "C" {
    fn printf(format: &byte, ...) : i32;
    fn malloc(size: u64) : *byte;
    fn free(ptr: *byte);
}

// Kullanım
extern fn c_function() {
    // Call C function
}
```

### 12.4 GPU/SIMD (Parser hazır, codegen bekliyor)

```vex
@gpu
fn matrix_multiply(a: &[f32], b: &[f32]) : [f32] {
    // GPU kernel
}

@vectorize
fn vector_add(a: &[f32], b: &[f32]) : [f32] {
    // SIMD optimized
}

// Launch syntax
launch matrix_multiply[32, 32](a, b);
```

### 12.5 Memory Management (Kısmi implementasyon)

```vex
// Heap allocation
let boxed = new(42);

// Slice creation
let slice = make([i32], 100);

// sizeof/alignof (built-in)
let size = @sizeof(i32);
let align = @alignof(Point);
```

---

## 13. Lexer Token Listesi

### Keywords

```
fn, let, struct, enum, interface, trait, impl, async, await,
go, gpu, launch, try, return, if, else, for, while, in,
import, export, from, as, true, false, nil, type, extends,
infer, const, unsafe, new, make, switch, case, default,
match, select, extern
```

### Primitive Types

```
i8, i16, i32, i64, u8, u16, u32, u64, f32, f64,
bool, string, byte, error, Map
```

### Operators

```
+, -, *, /, %, =, ==, !=, <, <=, >, >=,
&&, ||, !, &, |, ?
```

### Compound Assignment

```
+=, -=, *=, /=, ++, --
```

### Delimiters

```
(, ), {, }, [, ], ,, ;, :, ., .., ->, =>, _, #, ...
```

### Intrinsics

```
@vectorize, @gpu, @sizeof, @alignof, @intrinsic
```

---

## 14. Syntax Özeti: v0.9 Değişiklikleri

### ✅ Yeni Syntax (v0.9)

- `let!` - Mutable variables (replaces `let mut`)
- `&T!` - Mutable references (replaces `&mut T`)
- `struct Foo impl Trait` - Inline trait implementation

### ❌ Kaldırılan Syntax

- `mut` keyword - Use `let!` instead
- `:=` operator - Use `let` instead
- `&mut T` - Use `&T!` instead

### ⚠️ Deprecated

- `interface` keyword - Use `trait` instead

---

## 15. Çalışan Örnekler

Tüm syntax örnekleri için `examples/` dizinine bakın:

- `examples/01_basics/` - Variables, types
- `examples/02_functions/` - Functions, recursion, methods
- `examples/03_control_flow/` - If, switch, loops
- `examples/04_types/` - Structs, enums, tuples
- `examples/05_generics/` - Generic functions and types
- `examples/06_patterns/` - Pattern matching
- `examples/07_strings/` - String operations
- `examples/08_algorithms/` - Working algorithms (fibonacci, etc.)
- `examples/09_trait/` - Trait system examples

**Test Durumu:** 29/59 örnekler çalışıyor (71% success rate)

---

## 16. Kaynaklar

- **LANGUAGE_FEATURES.md** - Detaylı özellik listesi ve test durumları
- **TODO.md** - Aktif geliştirme görevleri
- **docs/VARIABLE_SYSTEM_V09.md** - v0.9 değişken sistemi detayları
- **TRAIT_SYSTEM_MIGRATION_STATUS.md** - Trait sistemi implementasyonu
- **examples/README.md** - Tüm örneklerin açıklaması

---

**Not:** Bu syntax guide, gerçek lexer (`vex-lexer/src/lib.rs`), parser (`vex-parser/src/parser/`), ve AST (`vex-ast/src/lib.rs`) implementasyonundan derlenmiştir. Tüm syntax kuralları çalışan kod tabanına dayanır.
