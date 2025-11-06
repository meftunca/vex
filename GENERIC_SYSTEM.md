# Vex Generic System Referans Dokümanı

**Versiyon:** 0.2.0 (Syntax v0.9)  
**Son Güncelleme:** 6 Kasım 2025  
**Test Durumu:** 143/146 test geçiyor (%97.9)

Bu doküman, Vex dilinin generic (parametrik polimorfizm) sisteminin tam referansıdır. Mevcut implementasyon durumu ve planlanan özellikler detaylı olarak açıklanmıştır.

---

## 📑 İçindekiler

1. [Genel Bakış](#genel-bakış)
2. [✅ İmplemente Edilmiş Özellikler](#-i̇mplemente-edilmiş-özellikler)
3. [🚧 Planlanan Özellikler](#-planlanan-özellikler)
4. [Monomorphization (Derleme Zamanı Özelleştirme)](#monomorphization-derleme-zamanı-özelleştirme)
5. [İleri Düzey Patternler](#i̇leri-düzey-patternler)
6. [Örnekler ve Kullanım Senaryoları](#örnekler-ve-kullanım-senaryoları)

---

## Genel Bakış

Vex'in generic sistemi, **Rust'ın monomorphization yaklaşımından** ilham alarak tasarlanmıştır. Bu yaklaşım:

- ✅ **Sıfır runtime overhead**: Tüm generic kod derleme zamanında özelleştirilir
- ✅ **Tam tip güvenliği**: Compile-time'da tüm tip kontrolü yapılır
- ✅ **Yüksek performans**: Her instantiation için optimize kod üretilir
- ⚠️ **Binary boyutu artışı**: Her generic instantiation ayrı kod üretir

**Tasarım Felsefesi:** Type erasure yerine monomorphization kullanarak maksimum performans ve tip güvenliği sağlamak.

---

## ✅ İmplemente Edilmiş Özellikler

### 1. Generic Fonksiyonlar

**Durum:** ✅ Tam çalışıyor, 143/146 test geçiyor

Generic fonksiyonlar, farklı tipler üzerinde çalışabilen tekrar kullanılabilir kod yazmanızı sağlar.

**Temel Sözdizimi:**

```vex
fn identity<T>(x: T): T {
    return x;
}

// Kullanım
let num = identity<i32>(42);
let text = identity<string>("hello");
```

**Tip Çıkarımı (Type Inference):**

```vex
// Explicit tip belirtme
let x = identity<i32>(42);

// Tip çıkarımı - argümandan otomatik anlaşılır
let y = identity(42);  // T = i32
let z = identity("hi"); // T = string
```

**Çoklu Tip Parametreleri:**

```vex
fn pair<T, U>(first: T, second: U): (T, U) {
    return (first, second);
}

let p1 = pair<i32, string>(42, "answer");
let p2 = pair(3.14, true);  // Inferred: <f64, bool>
```

**Generic Return Types:**

```vex
fn create<T>(value: T): T {
    return value;
}

let x: i32 = create(42);
let y: string = create("text");
```

**Örnekler:**

```vex
// Swap fonksiyonu
fn swap<T>(a: T, b: T): (T, T) {
    return (b, a);
}

let (x, y) = swap(10, 20);        // x=20, y=10
let (s1, s2) = swap("hi", "bye"); // s1="bye", s2="hi"

// Double fonksiyonu
fn double<T>(x: T): T {
    return x + x;  // Numeric tipler için çalışır
}

let d1 = double(21);    // 42
let d2 = double(3.14);  // 6.28
```

**Çalışan Test Dosyaları:**

- `examples/05_generics/functions.vx` - Generic fonksiyonlar
- `examples/05_generics/nested_generics.vx` - İç içe generic kullanımı

---

### 2. Generic Struct'lar

**Durum:** ✅ Tam çalışıyor

Struct'lar, bir veya daha fazla tip parametresi ile tanımlanabilir.

**Tek Tip Parametresi:**

```vex
struct Box<T> {
    value: T,
}

// Kullanım
let int_box = Box<i32> { value: 42 };
let str_box = Box<string> { value: "hello" };

// Field access
let val = int_box.value;  // 42
```

**Çoklu Tip Parametreleri:**

```vex
struct Pair<T, U> {
    first: T,
    second: U,
}

let pair = Pair<i32, string> {
    first: 42,
    second: "answer",
};

let sum = pair.first;  // 42
```

**Generic Method'lar:**

```vex
struct Container<T> {
    value: T,
}

// Immutable method
fn (self: &Container<T>) get(): T {
    return self.value;
}

// Mutable method (v0.9 syntax: &Container<T>!)
fn (self: &Container<T>!) set(new_value: T) {
    self.value = new_value;
}

// Kullanım
let! container = Container<i32> { value: 42 };
let val = container.get();      // 42
container.set(100);
let new_val = container.get();  // 100
```

**İç İçe Generics (Nested Generics):**

```vex
struct Box<T> {
    value: T,
}

// Box içinde Box
let nested = Box<Box<i32>> {
    value: Box<i32> { value: 42 }
};

let inner_value = nested.value.value;  // 42
```

**Çalışan Test Dosyaları:**

- `examples/05_generics/structs.vx` - Generic struct'lar
- `examples/05_generics/nested_simple.vx` - İç içe generic'ler
- `examples/05_generics/nested_deep.vx` - Derin nested generic'ler

---

### 3. Generic Enum'lar

**Durum:** ✅ Parser hazır, temel kullanım çalışıyor

Generic enum'lar, farklı varyantlarda farklı tipler taşıyabilir.

**Option<T> - Temel Pattern:**

```vex
enum Option<T> {
    Some(T),
    None,
}

// Kullanım
let some_int = Option.Some(42);
let some_str = Option.Some("hello");
let nothing: Option<i32> = Option.None;
```

**Result<T, E> - İki Tip Parametresi:**

```vex
enum Result<T, E> {
    Ok(T),
    Err(E),
}

let success: Result<i32, string> = Result.Ok(42);
let failure: Result<i32, string> = Result.Err("error");
```

**Pattern Matching ile Kullanım:**

```vex
let result = Result.Ok(42);

match result {
    Result.Ok(value) => {
        // value: i32
        return value;
    }
    Result.Err(error) => {
        // error: string
        return 0;
    }
}
```

**Not:** Enum method implementasyonları henüz tam çalışmıyor, ancak temel kullanım ve pattern matching destekleniyor.

---

### 4. Generic Trait'ler

**Durum:** ✅ Temel trait bounds çalışıyor

Trait'ler generic parametreler alabilir ve generic fonksiyonlarda constraint olarak kullanılabilir.

**Trait Tanımı:**

```vex
trait Display {
    fn (self: &Self) display(): string;
}

trait Debug {
    fn (self: &Self) debug(): string;
}
```

**Generic Trait (Gelecek):**

```vex
// Henüz implemente edilmedi
trait Container<T> {
    fn (self: &Self!) get(): T;
    fn (self: &Self!) set(value: T);
}
```

---

### 5. Trait Bounds (Basit Kısıtlamalar)

**Durum:** ✅ Temel syntax parser'da, tek ve çoklu bounds çalışıyor

Trait bounds, generic tiplerin belirli trait'leri implement etmesini zorunlu kılar.

**Tek Bound:**

```vex
// T tipi Display trait'ini implement etmeli
fn print_value<T: Display>(value: T): i32 {
    // value.display() çağrılabilir
    return 42;
}
```

**Çoklu Bounds (Virgülle Ayrılmış):**

```vex
// T hem Clone hem Debug implement etmeli
fn process<T: Clone, Debug>(value: T): i32 {
    return 0;
}

// Alternatif syntax (+ ile)
fn process2<T: Clone + Debug>(value: T): i32 {
    return 0;
}
```

**Super-traits (Trait Inheritance):**

```vex
// Display, Debug ve Clone'u bir araya getiren trait
trait VerboseLoggable impl Display, Debug, Clone {}

// Temiz fonksiyon signature
fn log_verbose<T: VerboseLoggable>(item: T) {
    // T hem Display, hem Debug, hem Clone implement eder
}
```

**Struct'larda Trait Bounds:**

```vex
// Container sadece Display implement eden tipler için
struct Container<T: Display> {
    value: T,
}

fn (self: &Container<T>) show() {
    // self.value.display() çağrılabilir
}
```

**Çalışan Test Dosyaları:**

- `examples/09_trait/trait_bounds_basic.vx` - Temel trait bounds
- `examples/09_trait/trait_bounds_multiple.vx` - Çoklu bounds

---

### 6. Monomorphization (Compile-Time Specialization)

**Durum:** ✅ Tam çalışıyor, core özellik

Vex, her generic instantiation için özelleştirilmiş kod üretir. Bu, runtime overhead olmadan maksimum performans sağlar.

**Öncesi (Generic Kod):**

```vex
fn identity<T>(x: T): T {
    return x;
}

let a = identity(42);
let b = identity("hello");
```

**Sonrası (Compiler'ın Ürettiği Kod - Konsept):**

```vex
// Compiler her tip için ayrı fonksiyon üretir
fn identity_i32(x: i32): i32 {
    return x;
}

fn identity_string(x: string): string {
    return x;
}

let a = identity_i32(42);
let b = identity_string("hello");
```

**Avantajlar:**

1. ✅ **Sıfır Runtime Cost**: Runtime'da tip kontrolü yok
2. ✅ **Tam Tip Güvenliği**: Derleme zamanında tüm hatalar yakalanır
3. ✅ **Optimizasyon**: Her instantiation optimize edilebilir
4. ✅ **No Boxing**: Değerler wrap edilmez, direkt kullanılır

**Trade-offs:**

1. ⚠️ **Binary Boyutu**: Her instantiation binary boyutunu artırır
2. ⚠️ **Derleme Süresi**: Daha fazla kod üretilir
3. ⚠️ **Cache Pressure**: Büyük binary cache'i etkileyebilir

**Struct Örneği:**

```vex
struct Box<T> {
    value: T,
}

let int_box = Box<i32> { value: 42 };
let str_box = Box<string> { value: "hello" };

// Compiler üretir (konsept):
struct Box_i32 {
    value: i32,
}

struct Box_string {
    value: string,
}
```

---

## 🚧 Planlanan Özellikler

### 1. `where` Clauses

**Durum:** 🔴 Yüksek Öncelik (`TODO.md`'de)  
**Tahmini Süre:** ~1 gün

**Amaç:** Karmaşık trait bound'ları okunabilir hale getirmek ve associated type'ları kısıtlamak.

**Problem 1 - Okunaksız Signature'lar:**

```vex
// ❌ Önce: Okunması zor
fn complex_function<K: Hash + Eq + Debug, V: Clone + Debug, S: StateManager<K, V>>(manager: S) {}

// ✅ Sonra: Çok daha temiz
fn complex_function<K, V, S>(manager: S)
    where K: Hash + Eq + Debug,
          V: Clone + Debug,
          S: StateManager<K, V>
{}
```

**Problem 2 - Associated Type Constraints:**

```vex
// where clause olmadan MÜMKÜN DEĞİL
fn print_items<T>(iter: T)
    where T: Iterator,
          T.Item: Display  // Associated type constraint
{
    for item in iter {
        print(item);
    }
}
```

**Kullanım Senaryoları:**

- 3+ trait bound olan fonksiyonlar
- Associated type constraints
- Karmaşık generic ilişkiler

---

### 2. Associated Types

**Durum:** 🔴 Yüksek Öncelik (`TODO.md`'de)  
**Tahmini Süre:** ~2 gün

**Amaç:** Trait'lerde placeholder tipler tanımlamak, abstract API'ler oluşturmak.

**Iterator Trait Örneği:**

```vex
trait Iterator {
    // Associated type - implementasyon belirler
    type Item;

    fn (&self!) next(): Option<Self.Item>;
}

struct Counter {
    current: i32,
}

impl Iterator for Counter {
    type Item = i32;  // Item tipi i32 olarak belirtilir

    fn (&self!) next(): Option<i32> {
        self.current = self.current + 1;
        return Some(self.current);
    }
}
```

**Kullanım Senaryoları:**

- Iterator pattern (`Iterator.Item`)
- Collection API'leri (`Container.Element`)
- Async traits (`Future.Output`)
- Generic ilişkilerde tip belirleme

**Not:** `where` clause ile birlikte çalışır:

```vex
fn process<T>(iter: T)
    where T: Iterator,
          T.Item: Display
{
    // ...
}
```

---

### 3. Const Generics

**Durum:** 🟡 Orta Öncelik (Array<T,N> için planlı)  
**Tahmini Süre:** ~2-3 gün

**Amaç:** Compile-time sabitleri (sayılar) generic parametre olarak kullanmak.

**Fixed-Size Array Örneği:**

```vex
// Tip VE boyut üzerinden generic
struct Buffer<T, const SIZE: u64> {
    data: [T; SIZE];
}

// Kullanım
let buffer_1kb: Buffer<u8, 1024>;     // 1KB buffer
let buffer_4kb: Buffer<u8, 4096>;     // 4KB buffer
let buffer_floats: Buffer<f32, 256>;  // 256 float buffer
```

**Kullanım Senaryoları:**

- Fixed-size arrays: `[T; N]`
- Matrix işlemleri: `Matrix<T, ROWS, COLS>`
- SIMD vectors: `Vec<T, LANES>`
- Buffer allocation

**Rust Karşılaştırması:**

```rust
// Rust
struct Array<T, const N: usize> {
    data: [T; N],
}

// Vex (planlanan)
struct Array<T, const N: u64> {
    data: [T; N],
}
```

---

### 4. Otomatik Lifetime Analizi (Compiler Tarafından)

**Durum:** 🔴 Yüksek Öncelik (`TODO.md`'de) - Reference Lifetime Validation  
**Tahmini Süre:** ~2 gün

**⚠️ ÖNEMLİ TASARIM KARARI:** Vex, Rust'ın aksine explicit lifetime annotation syntax'ı (`'a`, `'b` gibi) **DESTEKLEMEZ**. Bunun yerine compiler, kendi analiz algoritması ile lifetime'ları otomatik tespit eder ve doğrular.

**Tasarım Felsefesi:**

- ✅ Kullanıcılar karmaşık `'a`, `'b`, `'static` gibi annotation'lar yazmak zorunda kalmaz
- ✅ Compiler, referans ilişkilerini otomatik analiz eder
- ✅ Borrow checker, lifetime hatalarını açık mesajlarla bildirir
- ✅ Kod daha temiz ve okunabilir kalır

**Vex'te Nasıl Yazılır:**

```vex
// ✅ Vex - Lifetime annotation YOK
fn longest(x: &str, y: &str): &str {
    if x.len() > y.len() {
        return x;
    } else {
        return y;
    }
}

// Compiler otomatik olarak analiz eder:
// - x ve y'nin lifetime'ları eşit olmalı
// - Return value, x ve y'nin minimum lifetime'ına sahip
```

**Rust ile Karşılaştırma:**

```rust
// ❌ Rust - Explicit lifetime annotation gerekli
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// ✅ Vex - Annotation yok, compiler halleder
fn longest(x: &str, y: &str): &str {
    if x.len() > y.len() { return x; } else { return y; }
}
```

**Struct'larda Referanslar:**

```vex
// ✅ Vex - Lifetime annotation YOK
struct Wrapper<T> {
    reference: &T,
}

fn create_wrapper<T>(value: &T): Wrapper<T> {
    return Wrapper { reference: value };
}

// Compiler otomatik analiz:
// - Wrapper.reference'ın lifetime'ı, value'nun lifetime'ına bağlı
// - Return edilen Wrapper, value'nun scope'undan çıkamaz
```

**Compiler'ın Yaptığı Analiz:**

1. **Referans İlişkileri**: Her referansın hangi değere bağlı olduğunu takip eder
2. **Scope Analizi**: Referansların geçerlilik sürelerini hesaplar
3. **Return Validation**: Döndürülen referansların güvenli olduğunu doğrular
4. **Error Reporting**: Lifetime hatalarını anlaşılır mesajlarla bildirir

**Örnek Hata Durumu:**

```vex
fn dangling_reference(): &i32 {
    let x = 42;
    return &x;  // ❌ HATA: x scope dışına çıkıyor
}

// Compiler mesajı:
// Error: Cannot return reference to local variable 'x'
// The referenced value will be dropped when the function returns
```

**Avantajlar:**

- 🎯 **Basitlik**: Kullanıcı complex annotation'lar öğrenmek zorunda değil
- 🎯 **Okunabilirlik**: Kod daha temiz, daha az syntax noise
- 🎯 **Güvenlik**: Compiler yine de tüm lifetime hatalarını yakalar
- 🎯 **Hata Mesajları**: Annotation yerine doğrudan problemi açıklar

**Kapsam:**

- ✅ Fonksiyon parametrelerinde referanslar
- ✅ Return type'larda referanslar
- ✅ Struct field'larında referanslar
- ✅ Method'larda `&self` ile ilişkiler
- ✅ Nested reference relationships

---

### 5. Gelişmiş Borrow Checker Features

**Durum:** 🔴 Yüksek Öncelik (`TODO.md`'de)

**Planlanan İyileştirmeler:**

- Non-Lexical Lifetimes (NLL) - Otomatik scope analizi
- Advanced lifetime inference - Karmaşık ilişkileri çıkarma
- Multiple mutable references (split borrows) - Struct field'ları için
- Conditional borrow checking - Control flow analizi

**Gelecek Özellikler:**

- Polonius borrow checker
- View types
- Linear types (affine types)

---

## Monomorphization (Derleme Zamanı Özelleştirme)

### Nasıl Çalışır?

1. **Parser Aşaması**: Generic tanımlar parse edilir
2. **Type Inference**: Kullanım yerlerinde tipler çıkarılır
3. **Instantiation**: Her farklı tip kombinasyonu için kod üretilir
4. **Optimization**: Her instantiation bağımsız optimize edilir

### Örnek: Identity Fonksiyonu

**Kaynak Kod:**

```vex
fn identity<T>(x: T): T {
    return x;
}

fn main(): i32 {
    let a = identity(42);
    let b = identity(3.14);
    let c = identity("hi");
    return a;
}
```

**Compiler Çıktısı (LLVM IR - Konsept):**

```llvm
; identity<i32>
define i32 @identity_i32(i32 %x) {
    ret i32 %x
}

; identity<f64>
define double @identity_f64(double %x) {
    ret double %x
}

; identity<string> (pointer)
define ptr @identity_string(ptr %x) {
    ret ptr %x
}
```

**Main Fonksiyonu:**

```llvm
define i32 @main() {
    %a = call i32 @identity_i32(i32 42)
    %b = call double @identity_f64(double 3.14)
    %c = call ptr @identity_string(ptr @str_hi)
    ret i32 %a
}
```

### Struct Monomorphization

**Kaynak Kod:**

```vex
struct Box<T> {
    value: T,
}

fn main(): i32 {
    let int_box = Box<i32> { value: 42 };
    let str_box = Box<string> { value: "hello" };

    return int_box.value;
}
```

**Üretilen Struct'lar (Konsept):**

```vex
// Box<i32>
struct Box_i32 {
    value: i32,
}

// Box<string>
struct Box_string {
    value: string,
}
```

**LLVM IR:**

```llvm
; Box<i32> struct type
%Box_i32 = type { i32 }

; Box<string> struct type
%Box_string = type { ptr }

define i32 @main() {
    ; Box<i32> instantiation
    %int_box = alloca %Box_i32
    %int_box_value_ptr = getelementptr %Box_i32, ptr %int_box, i32 0, i32 0
    store i32 42, ptr %int_box_value_ptr

    ; Box<string> instantiation
    %str_box = alloca %Box_string
    %str_box_value_ptr = getelementptr %Box_string, ptr %str_box, i32 0, i32 0
    store ptr @str_hello, ptr %str_box_value_ptr

    ; Return int_box.value
    %result = load i32, ptr %int_box_value_ptr
    ret i32 %result
}
```

---

## İleri Düzey Patternler

### 1. Generic Wrapper Pattern

```vex
struct Wrapper<T> {
    inner: T,
}

fn wrap<T>(value: T): Wrapper<T> {
    return Wrapper<T> { inner: value };
}

fn unwrap<T>(wrapper: Wrapper<T>): T {
    return wrapper.inner;
}

// Kullanım
let wrapped = wrap(42);
let value = unwrap(wrapped);  // 42
```

### 2. Generic Pair Operations

```vex
struct Pair<T, U> {
    first: T,
    second: U,
}

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

// Kullanım
let pair = Pair<i32, string> { first: 42, second: "answer" };
let swapped = pair.swap();  // Pair<string, i32>
```

### 3. Builder Pattern (Generic)

```vex
struct Builder<T> {
    items: Vec<T>,
}

fn Builder.new<T>(): Builder<T> {
    return Builder<T> { items: Vec.new() };
}

fn (self: &Builder<T>!) add(item: T): &Builder<T>! {
    self.items.push(item);
    return self;
}

fn (self: &Builder<T>) build(): Vec<T> {
    return self.items;
}

// Kullanım (method chaining)
let builder = Builder.new<i32>();
builder.add(1).add(2).add(3);
let result = builder.build();
```

### 4. Generic Result/Option Pattern

```vex
enum Option<T> {
    Some(T),
    None,
}

fn (self: &Option<T>) unwrap_or(default: T): T {
    match self {
        Option.Some(value) => return value,
        Option.None => return default,
    }
}

fn (self: &Option<T>) is_some(): bool {
    match self {
        Option.Some(_) => return true,
        Option.None => return false,
    }
}

// Kullanım
let maybe = Option.Some(42);
let value = maybe.unwrap_or(0);  // 42

let nothing: Option<i32> = Option.None;
let value2 = nothing.unwrap_or(0);  // 0
```

---

## Örnekler ve Kullanım Senaryoları

### Senaryo 1: Generic Container

```vex
struct Container<T> {
    data: Vec<T>,
    count: i32,
}

fn Container.new<T>(): Container<T> {
    return Container<T> {
        data: Vec.new(),
        count: 0,
    };
}

fn (self: &Container<T>!) push(item: T) {
    self.data.push(item);
    self.count = self.count + 1;
}

fn (self: &Container<T>) get(index: i32): Option<T> {
    if index >= 0 {
        if index < self.count {
            return Option.Some(self.data.get(index));
        }
    }
    return Option.None;
}

fn main(): i32 {
    let! container = Container.new<i32>();
    container.push(10);
    container.push(20);
    container.push(30);

    let result = container.get(1);
    match result {
        Option.Some(value) => return value,
        Option.None => return 0,
    }
}
```

### Senaryo 2: Generic Stack

```vex
struct Stack<T> {
    items: Vec<T>,
}

fn Stack.new<T>(): Stack<T> {
    return Stack<T> { items: Vec.new() };
}

fn (self: &Stack<T>!) push(item: T) {
    self.items.push(item);
}

fn (self: &Stack<T>!) pop(): Option<T> {
    let len = self.items.len();
    if len > 0 {
        return Option.Some(self.items.pop());
    }
    return Option.None;
}

fn (self: &Stack<T>) is_empty(): bool {
    return self.items.len() == 0;
}

// Kullanım
let! stack = Stack.new<i32>();
stack.push(1);
stack.push(2);
stack.push(3);

let top = stack.pop();  // Some(3)
let next = stack.pop(); // Some(2)
```

### Senaryo 3: Generic Iterator Pattern (Gelecek)

```vex
trait Iterator {
    type Item;  // Associated type

    fn (self: &Self!) next(): Option<Self.Item>;
}

struct RangeIterator {
    current: i32,
    end: i32,
}

impl Iterator for RangeIterator {
    type Item = i32;

    fn (self: &RangeIterator!) next(): Option<i32> {
        if self.current < self.end {
            let value = self.current;
            self.current = self.current + 1;
            return Option.Some(value);
        }
        return Option.None;
    }
}

// Generic fonksiyon iterator ile
fn sum<T>(iter: T): i32
    where T: Iterator,
          T.Item: i32
{
    let! total = 0;
    loop {
        match iter.next() {
            Option.Some(value) => total = total + value,
            Option.None => break,
        }
    }
    return total;
}
```

---

## Özet ve En İyi Pratikler

### ✅ Yapılması Gerekenler

1. **Generic'leri Sadelikle Kullan**: Sadece gerçekten gerektiğinde generic kullan
2. **Tip Çıkarımından Faydalın**: Explicit tip belirtmeyi minimumda tut
3. **Trait Bounds ile Kısıtla**: Generic'leri güvenli hale getirmek için trait bounds kullan
4. **Super-traits Kullan**: Çoklu constraint'leri grupla
5. **Monomorphization'ı Düşün**: Çok fazla instantiation binary boyutunu artırır

### ❌ Yapılmaması Gerekenler

1. **Gereksiz Generic Kullanma**: Her fonksiyonu generic yapma
2. **Lifetime Annotation Yazma**: Vex otomatik analiz eder, `'a` syntax'ı desteklenmez
3. **Aşırı Nested Generic**: `Box<Vec<Option<Result<T, E>>>>` gibi derinliklerden kaçın
4. **Type Erasure Bekleme**: Vex monomorphization kullanır, runtime overhead yok

### 🎯 Tasarım İlkeleri

- **Basitlik**: Kod temiz ve okunabilir olmalı
- **Güvenlik**: Compile-time'da tüm hatalar yakalanmalı
- **Performans**: Zero-cost abstractions
- **Esneklik**: Generic'ler farklı tipler için tekrar kullanılabilir olmalı

---

**Son Güncelleme:** 6 Kasım 2025  
**Kaynak:** Vex Language Documentation  
**İlgili Dokümanlar:**

- `TODO.md` - Planlanan özellikler
- `SYNTAX.md` - Sözdizimi referansı
- `Specifications/10_Generics.md` - Detaylı spesifikasyon
- `examples/05_generics/` - Çalışan örnekler

```

```
