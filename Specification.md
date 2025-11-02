# Vex Dil Spesifikasyonu (Taslak v0.6)

**Felsefe:** Vex, modern donanım (çok çekirdeli CPU, SIMD, GPU, hızlı I/O) için yüksek performanslı kod yazmayı basitleştirmeyi amaçlayan, statik tipli ve derlenen bir sistem programlama dilidir. Go'nun basitliğini (arayüzler, struct etiketleri), Rust'ın bellek güvenliği garantilerini (daha basit bir modelle) ve TypeScript'in gelişmiş tip sistemini (Jenerikler, Union/Intersection, Koşullu Tipler) birleştirir.

**Dosya Uzantısı:** `.vx`

## 1. Sözdizimi Temelleri (Syntax Basics)

### 1.1. Merhaba, Vex!

Tüm yürütme `main` görevinden (task) başlar. Vex, `main` fonksiyonunu otomatik olarak bir ana coroutine (task) olarak çalıştırır.

```javascript
// 'std' modülünden 'io' ve 'log' alt modüllerini içe aktar
import { io, log } from "std";

// Giriş noktası.
// Dönüş tipi Go/TS tarzında: fn name(args): returnType
// 'error', nil olabilen (hata yok) yerleşik bir arayüzdür.
fn main(): error {
    log.info("Vex v0.6 çalışıyor.");

    // Formatlı çıktı (f-string).
    io.print(f"1 + 2 = {1 + 2}\n");

    // Hata yönetimi: 'main' sorunsuz biterse 'nil' döner.
    return nil;
}

```

### 1.2. Yorumlar (Comments)

Vex, C-tarzı yorumları kullanır.

```javascript
// Bu tek satırlık bir yorumdur.

/*
  Bu, birden fazla satıra
  yayılan bir yorum bloğudur.
*/
```

## 2. Tipler ve Değişkenler (Types & Variables)

Vex, statik ve güçlü tiplidir (static & strong-typed). Rust'ın değişmezlik (immutability) önceliğini ve Go/TS'nin esnek sözdizimini benimser.

### 2.1. Değişken Tanımlama (Variable Declaration)

Vex, C ve Go'nun basit ama güçlü değişken tanımlama sözdizimini benimser. Varsayılan olarak **değişebilir (mutable)** değişkenler oluşturulur.

#### 2.1.1. Değişebilir Değişkenler (Mutable - Varsayılan)

```javascript
// Tip çıkarımı ile (Go tarzı ':=' operatörü)
x := 10;              // 'x' tipi 'int' olarak çıkarsanır (mutable)
name := "Vex";        // 'name' tipi 'string' olarak çıkarsanır (mutable)

// Açık tip tanımı (C/Go tarzı)
i32 y = 20;         // Mutable, açık tip
f64 pi = 3.14159;     // Mutable, açık tip
[string] flags = ["simd", "gpu"];  // Mutable array

// Değiştirme (assignment)
x = 15;               // OK
y = 25;               // OK
flags[0] = "avx";     // OK
```

#### 2.1.2. Değişmez Değişkenler (Immutable - 'const')

Bir değişkenin değiştirilmesini engellemek için `const` anahtar kelimesi kullanılır.

```javascript
// Tip çıkarımı ile
const PI := 3.14159;           // Immutable, tip çıkarsanır
const VERSION := "0.6";        // Immutable

// Açık tip tanımı
const i32 MAX_SIZE = 1024;   // Immutable, açık tip
const string PROJECT = "Vex";  // Immutable

// PI = 3.14;  // HATA: 'PI' değişmezdir (immutable)
```

### 2.2. Temel Tipler (Primitive Types)

- **Integer:** `i8`, `i16`, `i32`, `i64` (işaretli)
- **Unsigned Integer:** `u8`, `u16`, `u32`, `u64`, `byte` (alias for `u8`), `uintptr`
- **Float:** `f32`, `f64`
- **Boolean:** `bool` (değerler: `true`, `false`)
- **String:** `string` (UTF-8, değişmez)
- **Nil:** `nil` (Go'dan ilhamla. `Option<T>` yerine `error` ve `interface` bağlamlarında kullanılır)

### 2.3. Bileşik Tipler (Composite Types)

#### 2.3.1. Array (Dizi) - Statik

Bellekte bitişik duran, sabit boyutlu koleksiyon. SIMD için temel yapı taşıdır.

```javascript
// 4 adet 32-bit float içeren bir dizi (stack üzerinde)
let simd_data: [f32; 4] = [1.0, 2.0, 3.0, 4.0];

```

#### 2.3.2. Slice (Kesit) - Dinamik

Bir dizinin (veya başka bir slice'ın) bir bölümüne bakan dinamik boyutlu bir "görünüm" (view). Go'dan ilham almıştır.

```javascript
let mut data: [f32; 4] = [1.0, 2.0, 3.0, 4.0];

// 'data' dizisinin tamamını gösteren bir slice
let s1: &[f32] = &data[..];

// İlk iki elemanı gösteren bir slice
let s2: &[f32] = &data[0..2];

```

#### 2.3.3. Struct (Yapı)

İlgili verileri bir araya toplayan nominal (nominal) bir tip. Jenerik (Generic) olabilir ve Go-tarzı etiketleri (Tags) destekler.

```javascript
// ---- Basit Struct ----
struct Vector2 {
    x: f64,
    y: f64,
}
Vector2 v1 = Vector2{ x: 1.0, y: 2.0 };
// veya tip çıkarımı ile:
v1 := Vector2{ x: 1.0, y: 2.0 };

// ---- Jenerik Struct (TS/Rust) ----
struct Response<T> {
    success: bool,
    data: T,
}
Response<string> ok = Response{ success: true, data: "Giriş yapıldı" };
// veya tip çıkarımı:
ok := Response<string>{ success: true, data: "Giriş yapıldı" };
err := Response<nil>{ success: false, data: nil };

// ---- Struct Etiketleri (Go) ----
// Etiketler, derleme zamanında kütüphaneler (örn: JSON, DB) için
// meta-veri sağlar.
struct User {
    id: u64     `json:"id" db:"user_id,pk"`,
    username: string `json:"username" validate:"required,min=4"`,
    email: string   `json:"email,omitempty" validate:"email"`,
}

```

### 2.4. Referanslar (References) - Güvenli Model

Vex, Rust'ın "lifetime" karmaşıklığı olmadan "aliasing" (takma ad) garantileri sağlayan basit bir referans modeli kullanır. Bu, kodun %99'u için tercih edilen yoldur.

- `&T`: **Paylaşılan Referans** (Immutable). Aynı veriye birden fazla `&T` olabilir.
- `*T`: **Özel Referans** (Mutable). Bir veriye aynı anda sadece bir adet `*T` olabilir. Bu referanslar _hem stack hem de heap_ verisi için geçerlidir.

```javascript
fn inspect(v: &Vector2) { /* ... */ }
fn move_vec(v: *Vector2, dx: f64) { v.x += dx; }

Vector2 v_stack = Vector2{x: 1.0, y: 0.0}; // Stack üzerinde (mutable)
inspect(&v_stack);      // Immutable reference
move_vec(&v_stack, 10.0);  // Mutable reference (& operatörü *T oluşturur)

```

**Not:** `*T` notasyonu, fonksiyon parametrelerinde "mutable reference" anlamına gelir. Raw pointer'lar için `unsafe` bölümüne bakınız (Bölüm 2.6).

### 2.5. Gelişmiş Tip Özellikleri (Advanced Type Features) (TS)

Vex, TypeScript'ten ilham alan güçlü tip özelliklerini destekler.

#### 2.5.1. Tip Alyasları (Type Aliases)

`type` anahtar kelimesi, mevcut bir tipe yeni bir isim (alyas) vermek için kullanılır.

```javascript
type UserID = u64;

// TypeScript-style Map ve Record tipleri (built-in generic types)
type StringMap = Map<string, string>;      // Hash map (mutable)
type Config = Record<string, string>;      // Object-like immutable map

fn get_user(id: UserID): User { /* ... */ }

```

**Not:** `Map<K,V>` ve `Record<K,V>` Vex'in built-in generic tipleridir:

- `Map<K,V>`: Mutable hash map (Rust's HashMap benzeri)
- `Record<K,V>`: Immutable key-value store (TypeScript's Record benzeri)

#### 2.5.2. Birleşim Tipleri (Union Types)

Bir tipin birden fazla olası tipten biri olmasını sağlar.

```javascript
type Value = string | int | f64;
type Result = string | error; // Hata yönetimi için yaygın kullanılır

fn process_value(v: Value) { /* ... */ }

```

#### 2.5.3. Kesişim Tipleri (Intersection Types)

Birden fazla tipi tek bir tipte "birleştirmek" veya "genişletmek" (extend) için kullanılır.

```javascript
// İki farklı arayüzü veya struct'ı birleştirmek
interface Reader { read(buf: &mut [byte]): int }
interface Writer { write(buf: &[byte]): int }

// 'ReadWriter', hem Reader hem de Writer özelliklerine sahiptir
type ReadWriter = Reader & Writer;

fn do_io(stream: ReadWriter) {
    stream.read(...);
    stream.write(...);
}

// Bir tipi diğeriyle "genişletmek"
type BaseConfig { host: string }
type DbConfig = BaseConfig & { database: string };

```

#### 2.5.4. Koşullu Tipler (Conditional Types)

Jenerik tipler içinde, tiplere dayalı mantık (if/else) kurmayı sağlar. Bu, Vex'e "tip seviyesinde programlama" (type-level programming) yeteneği kazandırır.

```javascript
// T tipi 'string' ise, 'string' tipini; değilse 'int' tipini döndür.
type GetStringType<T> = T extends string ? string : int;

let a: GetStringType<"hello">; // 'a'nın tipi 'string' olur
let b: GetStringType<123>;     // 'b'nin tipi 'int' olur

// Jeneriklerde kullanım:
// T, bir slice (dilim) ise, o slice'ın eleman tipini;
// değilse T'nin kendisini döndür.
type Unwrap<T> = T extends &[infer E] ? E : T;

type A = Unwrap<&[string]>; // A tipi 'string' olur
type B = Unwrap<i32>;       // B tipi 'i32' olur

```

### 2.6. Raw Pointer'lar ve 'unsafe' (Güvensiz) Kod

Vex, yüksek performanslı sistem programlama (FFI, `io_uring` buffer yönetimi) için "raw pointer" (ham pointer) kavramını destekler. Bu özellikler, Rust'tan ilhamla, **sadece&#32;`unsafe`&#32;blokları içinde** kullanılabilir.

**Önemli:** Güvenli kod için `&T` (immutable ref) ve `*T` (mutable ref) kullanın. Raw pointer'lar sadece unsafe kod için gereklidir.

- `*const T`: Değişmez (Immutable) Raw Pointer.
- `*mut T`: Değiştirilebilir (Mutable) Raw Pointer.

```javascript
// Güvenli referanslar (normal kullanım)
i32 x = 10;
&i32 ref_x = &x;        // Immutable reference
*i32 mut_ref = &x;      // Mutable reference

// Raw pointer'lar (unsafe kullanım)
*const i32 ptr_const = &x as *const i32;
*mut i32 ptr_mut = &x as *mut i32;

// 'unsafe' blok: Derleyiciye "Buradaki riskleri anlıyorum" dersiniz.
// Raw pointer'lar sadece 'unsafe' blok içinde dereference edilebilir.
unsafe {
    log.info(f"x'in değeri (pointer yoluyla): {*ptr_mut}");

    // Pointer'ı C-tarzında değiştirme
    *ptr_mut = 20;
}

```

### 2.7. Tip Dönüşümü (Type Conversion)

Vex, Go ve C'nin "implicit" (örtülü) dönüşümlerine izin vermez. Tüm dönüşümler "explicit" (açık) olmalıdır.

```javascript
let my_float: f64 = 3.14;

// 1. 'as' (Rust/TS) - Primitif tipler arası dönüşüm (Casting)
// Bu, potansiyel olarak veri kaybına (precision loss) neden olabilir.
let my_int: i32 = my_float as i32; // my_int = 3

// 2. `.into()` / `.from()` - Karmaşık (Complex) dönüşümler
// Bu, Vex'in 'From<T>' arayüzünü (Bölüm 4) uygulayan
// tipler için idiomatik yoldur.
let my_string = String::from("hello");
let user: User = data.into();

```

### 2.8. Enum'lar (Sum Types) (v0.6)

Vex, Rust/TS'den ilham alan güçlü "Sum Type" `enum`'larını destekler. Bu, bir tipin belirli durumlardan biri olmasını sağlar.

```javascript
// Veri taşıyabilen jenerik bir 'enum'
export enum Option<T> {
    Some(T), // Veri taşıyan durum
    None,    // Veri taşımayan durum
}

// Basit C-tarzı 'enum'
export enum HttpMethod {
    GET,
    POST,
    PUT,
}

// 'switch' (Bölüm 3.5.3) ile kullanım
fn handle_option(opt: Option<string>) {
    switch opt.(type) {
        case Option::Some(value):
            log.info(f"Değer: {value}");
        case Option::None:
            log.info("Değer yok.");
    }
}

```

### 2.9. Heap Tahsisi (new/make) (v0.6)

Vex, Go'dan ilham alan `new` ve `make` anahtar kelimelerini kullanarak _güvenli_ heap tahsisini destekler. Bu işlemler `unsafe` değildir.

- `new(T)`: `T` tipinde bir veriyi heap'te oluşturur ve ona _güvenli_ bir `*T` (mutable reference) döndürür.
- `make([T], size)`: Heap'te `T` tipinde elemanlar içeren, `size` boyutunda bir slice oluşturur ve `*[T]` döndürür.

```javascript
// Vector2'yi stack üzerinde oluştur
Vector2 v_stack = Vector2{x: 1.0, y: 0.0};

// Vector2'yi HEAP üzerinde oluştur (Go tarzı)
// 'v_heap' tipi: *Vector2 (mutable heap reference)
v_heap := new(Vector2{x: 1.0, y: 0.0});

// HEAP üzerinde 1024 byte'lık bir buffer oluştur (Go tarzı)
// 'buf' tipi: *[byte] (mutable heap slice)
buf := make([byte], 1024);

// Vex'in bellek modeli (RAII benzeri), bu referanslar
// scope dışına çıktığında belleği otomatik olarak 'free' eder.

```

## 3. Fonksiyonlar ve Hata Yönetimi

### 3.1. Fonksiyon Tanımlama

Go/TS tarzı sözdizimi kullanılır.

```javascript
// İki f64 alır, bir f64 döner
fn add(a: f64, b: f64): f64 {
    return a + b;
}

```

### 3.2. Jenerik Fonksiyonlar (Generic Functions) (TS/Rust)

Vex, `<T>` sözdizimini kullanarak jenerik fonksiyonları destekler.

```javascript
// T tipi için jenerik bir fonksiyon
fn get_first<T>(slice: &[T]): &T {
    // Vex, slice'ın boş olmamasını garanti etmelidir (veya Option<&T> döner)
    // Bu örnekte dolu olduğunu varsayalım.
    return &slice[0];
}

numbers := [1, 2, 3];
first_num := get_first<int>(&numbers);     // Açık tip
first_inferred := get_first(&numbers);     // Tip çıkarımı

```

### 3.3. Metodlar (Method Receivers)

Go/Rust tarzında, `struct`'lar için metodlar tanımlanabilir.

```javascript
struct Vector2 { x: f64, y: f64 }

// 'Vector2' için 'length' adında bir metod
fn (self: &Vector2) length(): f64 {
    return (self.x * self.x + self.y * self.y).sqrt();
}

let v = Vector2{x: 3.0, y: 4.0};
io.print(f"Uzunluk: {v.length()}"); // Çıktı: 5.0

```

### 3.4. Hata Yönetimi (Union Types + 'try')

Vex, Bölüm 2.5.2'de tanımlanan Union Tiplerini ve `try` operatörünü kullanır.

1. **Union Tipi:** Fonksiyon dönüşü `(string | error)` olarak tanımlanır.
2. **`error`&#32;Arayüzü:** `nil` olabilen yerleşik bir arayüz.
3. **`try`&#32;Operatörü:** Bir `error` alırsa, mevcut fonksiyondan o hatayı hemen döndürür.

```javascript
import { io } from "std";

// Dosya okuma 'string' veya 'error' döndürebilir
fn read_file(path: string): (string | error) {
    let content = try await io.file.read_text(path); // 'try' hatayı yönetir

    if content.len() == 0 {
        return error.new("Dosya boş."); // Yeni bir hata oluştur
    }
    return content; // Başarılı, string döner
}

fn main(): error {
    let data = try read_file("config.txt"); // 'try' hatayı 'main'den dışarı iletir
    io.print(data);
    return nil; // Hata yok
}

```

### 3.5. Kontrol Akışı (Control Flow)

Vex, Go'dan ilham alan temiz bir kontrol akışı seti kullanır.

#### 3.5.1. `if / else`

`if` blokları, Go gibi, koşuldan önce bir "init" (başlatma) ifadesi alabilir.

```javascript
let x = 10;
if x > 5 {
    log.info("x 5'ten büyük");
} else {
    log.info("x 5'ten küçük veya eşit");
}

// Go tarzı 'init' ifadesi
// 'data' değişkeni sadece 'if/else' bloğu içinde geçerlidir.
if let data = try read_file("config.txt") {
    // Başarılı (hata 'nil' idi)
    io.print(data);
} else {
    // Hata durumu ('data' burada 'error' tipindedir)
    log.error(f"Hata oluştu: {data}");
}

```

#### 3.5.2. `for` (Birleşik Döngü)

Vex, Go gibi, `while` döngüsüne sahip değildir. `for` döngüsü tüm döngü ihtiyaçlarını karşılar.

```javascript
// 1. C-tarzı 'for' (Klasik)
for let mut i = 0; i < 10; i++ {
    io.print(f"{i} ");
}

// 2. 'while' eşdeğeri ('for' koşul ile)
let mut running = true;
for running {
    if (try check_status()) == "done" {
        running = false;
    }
}

// 3. 'for' (Sonsuz döngü)
for {
    log.info("Sunucu çalışıyor...");
}

// 4. 'for...in' (İterasyon - Rust/TS tarzı)
let names = ["Vex", "Go", "Rust"];
for name in names {
    io.print(f"Merhaba, {name}");
}

```

#### 3.5.3. `switch` (Go Tarzı)

Vex, Go'nun güçlü `switch` ifadesini benimser. (Enum desteği v0.6'da eklendi).

- Otomatik `break` (fallthrough yoktur).
- `case`'ler birden fazla değer alabilir.
- "Type Switch", Vex'in Union (Bölüm 2.5.2) ve Enum (Bölüm 2.8) tipleri için mükemmeldir.

```javascript
let x = 2;
switch x {
    case 1:
        log.info("Bir");
    case 2, 3:
        log.info("İki veya Üç");
    default:
        log.info("Diğer");
}

// 'Type Switch' (Enum/Union tipleri için)
fn process_value(v: Value | Option<int>) {
    switch v.(type) {
        // Union Tipleri
        case string:
            log.info(f"Bu bir string: {v}");
        case int:
            log.info(f"Bu bir int: {v}");

        // Enum Tipleri (Bölüm 2.8)
        case Option::Some(val):
            log.info(f"Option<int> değeri: {val}");
        case Option::None:
            log.info("Option<int> değeri yok.");
    }
}

```

#### 3.5.4. `select` (Eşzamanlılık)

Vex, Bölüm 5'teki `async/await` görevlerini (task) yönetmek için Rust'ın `select!` makrosundan ilham alan bir `select` ifadesi sunar. Hangi `await`'in önce tamamlandığını yakalar.

```javascript
async fn task1(): string { /* ... */ }
async fn task2(): string { /* ... */ }

fn main(): error {
    // ...
    // Hangi görev (task) önce biterse
    select {
        res1 = await task1() => {
            log.info(f"Task 1 bitti: {res1}");
        },
        res2 = await task2() => {
            log.info(f"Task 2 bitti: {res2}");
        },
        await timer.sleep(5000) => {
            log.error("Zaman aşımı!");
        }
    }
    // ...
}

```

## 4. Arayüzler (Interfaces) - Go Tarzı

Vex, Go'nun yapısal (structural) ve dolaylı (implicit) arayüzlerini kullanır.

```javascript
// Bir 'Reader' arayüzü, 'read' metoduna sahip olmalıdır
interface Reader {
    read(buf: &mut [byte]): (int | error);
}

// 'File' struct'ı 'Reader'ı dolaylı olarak uygular
struct File { path: string }
fn (f: &File) read(buf: &mut [byte]): (int | error) {
    return (10, nil); // 10 byte okundu
}

// 'Reader' arayüzü ile çalışan genel bir fonksiyon
fn read_data(r: Reader) { /* ... */ }

let f = File{...};
read_data(f); // Geçerli

```

### 4.1. Jenerik Arayüzler (Generic Interfaces) (TS)

Arayüzler de jenerik olabilir.

```javascript
// Jenerik bir 'Cache' arayüzü
interface Cache<K, V> {
    get(key: K): (V | nil);
    set(key: K, value: V);
}

// Go-tarzı dolaylı implementasyon:
// 'RedisCache' struct'ı, 'get' ve 'set' metodlarını
// 'string' ve 'any' (veya spesifik tipler) için tanımlarsa,
// otomatik olarak 'Cache<string, any>' arayüzünü uygular.
struct RedisCache { /* ... */ }
fn (c: &RedisCache) get(key: string): (string | nil) { /* ... */ }
fn (c: &RedisCache) set(key: string, value: string) { /* ... */ }

// RedisCache, Cache<string, string> arayüzünü uygular.

```

## 4.2. Trait'ler (Traits) - Rust Tarzı (v0.6)

Vex, Rust'tan ilham alan **açık (explicit)** trait implementasyonunu da destekler. Interface'lerden farklı olarak, trait'ler **nominal typing** kullanır ve açıkça `impl` ile belirtilmelidir.

### 4.2.1. Trait Tanımlama

```javascript
// Bir trait tanımı - sadece method signature'ları içerir
trait Printable {
    fn print(self: &Self);
    fn debug(self: &Self);
}

trait Serializable {
    fn to_json(self: &Self): string;
    fn from_json(data: string): Self;
}

// Jenerik trait
trait Converter<T> {
    fn convert(self: &Self): T;
}
```

### 4.2.2. Trait Implementation

Trait'ler `impl TraitName for TypeName` sözdizimi ile açıkça implement edilir.

```javascript
struct Point {
    x: i32,
    y: i32,
}

// Point için Printable trait'ini implement et
impl Printable for Point {
    fn (self: &Point) print() {
        io.print(f"Point({self.x}, {self.y})");
    }

    fn (self: &Point) debug() {
        io.print(f"Point {{ x: {self.x}, y: {self.y} }}");
    }
}

// Point için Converter<string> trait'ini implement et
impl Converter<string> for Point {
    fn (self: &Point) convert(): string {
        return f"({self.x}, {self.y})";
    }
}

// Kullanım
let p = Point { x: 10, y: 20 };
p.print();           // "Point(10, 20)"
p.debug();           // "Point { x: 10, y: 20 }"
let s = p.convert(); // "(10, 20)"
```

### 4.2.3. Interface vs Trait - Ne Zaman Hangisi?

**Interface kullanın (Go-style):**

- Üçüncü parti kütüphanelerle çalışırken
- Duck typing istediğinizde
- Esneklik öncelikli olduğunda
- Bir tipin hangi interface'leri implement ettiğini bilmenize gerek olmadığında

**Trait kullanın (Rust-style):**

- Kendi kodunuzda, açık polymorphism istediğinizde
- Aynı isimli metodların farklı trait'lerden gelebileceği durumlarda
- Tip güvenliği ve açıklık (explicitness) öncelikli olduğunda
- Default implementation'lar istediğinizde (gelecek özellik)

```javascript
// Interface örneği (implicit)
interface Logger {
    log(msg: string);
}

struct FileLogger { path: string }
fn (f: &FileLogger) log(msg: string) { /* ... */ }
// FileLogger otomatik olarak Logger interface'ini implement eder

// Trait örneği (explicit)
trait Validator {
    fn validate(self: &Self): bool;
}

struct Email { address: string }
impl Validator for Email {
    fn (self: &Email) validate(): bool {
        return self.address.contains("@");
    }
}
// Email, açıkça Validator trait'ini implement eder
```

**Not:** Vex'te hem interface hem trait olabilir, ancak:

- Interface'ler dolaylı (implicit) ve yapısal (structural)
- Trait'ler açık (explicit) ve nominal

Bu, esneklik (interface) ve güvenlik (trait) arasında denge sağlar.

````

## 5. Eşzamanlılık (Concurrency) - Akıllı Context Switching

Vex'in runtime'ı, `io_uring` (Linux) veya benzeri (diğer OS'ler) asenkron I/O mekanizmaları üzerine kurulu bir "task" (coroutine) zamanlayıcısıdır.

- **Task:** Go'nun `goroutine`'i gibi çok hafif bir coroutine.
- **`go`&#32;(Go):** Yeni bir `task` başlatır (paralel çalıştırır).
- **`async`&#32;/&#32;`await`&#32;(Rust/TS):** Bir `task`'ın ne zaman bir I/O operasyonu için _askıya alınabileceğini_ (suspend) ve runtime'ın başka bir `task`'a geçebileceğini (context switch) belirtir.
- **`select`&#32;(Bölüm 3.5.4):** Birden fazla `await` işlemini yönetir.

```javascript
// 'async' bu fonksiyonun 'await' kullanabileceğini belirtir
async fn fetch_url(url: string): (string | error) {
    log.info(f"İstek atılıyor: {url}");

    // 'await' bu 'task'ı askıya alır.
    let resp = try await http.get(url);

    // I/O tamamlandığında, runtime bu 'task'ı uyandırır.
    return resp.text();
}

fn main(): error {
    go fetch_url("[https://api.site1.com](https://api.site1.com)");
    go fetch_url("[https://api.site2.com](https://api.site2.com)");

    await timer.sleep(5000); // Örn: 5 saniye bekle

    log.info("Tüm istekler gönderildi.");
    return nil;
}

````

## 6. Yüksek Performanslı Hesaplama (HPC) - Akıllı Derleme (v0.6)

Vex, HPC görevlerini (SIMD, GPU) _dilin sözdizimine_ (syntax) gömmek yerine, _derleyiciye ve runtime'a_ akıllıca devleder.

`@vectorize` (v0.5) ve `gpu fn` (v0.5) anahtar kelimeleri **kaldırıldı**. Kullanıcı, HPC için tasarlanmış normal Vex fonksiyonları yazar.

### 6.1. 'launch' Anahtar Kelimesi (HPC Niyeti)

Kullanıcı, bir fonksiyonun normal CPU akışından (sequential) çıkarılıp, yüksek performanslı bir "accelerator" (hızlandırıcı) üzerinde çalıştırılması _niyetini_ `launch` anahtar kelimesiyle belirtir.

```javascript
// Normal bir Vex fonksiyonu. Hiçbir özel 'gpu' veya 'simd'
// anahtar kelimesi içermez.
fn matrix_multiply(a: &[f32], b: &[f32], out: &mut [f32], size: u32) {
    // ... (v0.5'teki @gpu.global_id olmadan,
    //    derleyicinin bu döngüyü paralelleştireceğini varsayarak)
    // VEYA daha iyisi:
    // Derleyiciye paralelleştirme için ipuçları veren
    // 'std::simd' veya 'std::gpu' kütüphanesi kullanılır.

    // Şimdilik basit bir döngü bırakalım:
    for y in 0..size {
        for x in 0..size {
            let sum: f32 = 0.0;
            for k in 0..size {
                sum += a[y * size + k] * b[k * size + x];
            }
            out[y * size + x] = sum;
        }
    }
}

// CPU (Host) Kodu
fn main(): error {
    let N: u32 = 1024;
    // ... CPU'da 'a_data', 'b_data', 'out_data' slice'larını (heap'te, 'make' ile) oluştur ...

    log.info("HPC kernel'ı başlatılıyor...");

    // 'launch', bu fonksiyonu Vex Runtime'ının HPC kuyruğuna gönderir.
    // '[N, N]' (global boyut) bilgisi, runtime'a ne kadarlık
    // bir paralellik istediğimizi söyler.
    await launch matrix_multiply[N, N](a_data, b_data, &mut out_data, N);

    log.info(f"HPC işi bitti. İlk sonuç: {out_data[0]}");
    return nil;
}

```

### 6.2. Derleyici Bayrakları (HPC Yürütme)

`launch`'un _nasıl_ çalışacağı, `vex.json`'daki ayarlara veya derleyici bayrağına (`--accelerator`) göre belirlenir.

- `vex build --accelerator=cpu`:
  - `launch`, `matrix_multiply`'ı `std::sync` (thread pool) kullanarak CPU'da paralel olarak çalıştırır.
- `vex build --accelerator=cpu-simd`:
  - Derleyici, `matrix_multiply` fonksiyonunu (AOT veya JIT) analiz eder ve döngüleri _otomatik olarak vektörleştirmek_ (SIMD) için LLVM'i agresif bir şekilde kullanır.
- `vex build --accelerator=gpu-nvidia` (veya `gpu-amd`):
  - Derleyici, `matrix_multiply` fonksiyonunu (AOT veya JIT) **SPIR-V**'ye (veya PTX'e) çevirir.
  - `launch`, Vex Runtime'ının (Bölüm 5) Vulkan/CUDA aracılığıyla bu kernel'ı GPU'ya göndermesini tetikler.

## 7. Modüller ve Proje Yapısı

### 7.1. Proje Manifestosu (vex.json)

Her Vex projesi, Node.js'in `package.json`'undan ilham alan, yapısal (structured) bir `vex.json` dosyası ile tanımlanır.

```javascript
{
  "project": {
    "name": "my_app",
    "version": "0.1.0"
  },
  "dependencies": {
    "vex-json": "1.0.0"
  },
  "accelerator": {
    "default": "cpu-simd",
    "targets": ["cpu-simd", "gpu-nvidia"]
  }
}

```

### 7.2. Export Sözdizimi (Export Syntax)

Vex, TypeScript gibi açık (explicit) export kullanır.

- **Kural 1:** Bir dosyadaki her şey varsayılan olarak o dosyaya **özeldir (private)**.
- **Kural 2:** Bir `fn`, `struct`, `interface`, `type`, `enum` veya `const`'u public yapmak için `export` anahtar kelimesi kullanılır.
- **Kural 3:** Vex, `export default` mekanizmasını **desteklemez**.

```javascript
// Dosya: src/math/utils.vx
export fn add(a: int, b: int): int {
    return a + b;
}
export const PI = 3.14159;

```

### 7.3. Import Sözdizimi (Import Syntax)

TypeScript'in `import` sözdizimi kullanılır.

```javascript
// Dosya: src/main.vx
import { add, PI } from "./math/utils.vx";

fn main(): error {
    let val = add(1, 2);
    io.print(f"PI = {PI}");
    return nil;
}

```

#### 7.3.1. Tip Importu (import type)

TypeScript'ten ilhamla, Vex `import type` sözdizimini destekler.

```javascript
// Dosya: src/types.vx
export struct User { id: u64 }
export type Config = map[string]string;

// Dosya: src/main.vx

// 'User' ve 'Config' sadece tip olarak kullanılır.
import type { User, Config } from "./types.vx";

fn process_user(user: User) { /* ... */ }

```

## 8. FFI (Foreign Function Interface) (v0.6)

Vex, ekosistemi hızlıca başlatmak (bootstrap) ve mevcut C kütüphanelerini kullanmak için `unsafe` bir FFI mekanizması sağlar.

```javascript
// Vex'in 'unsafe' alt modülünü import et
import { unsafe } from "std";

// 'libc.so' (veya .dll/.dylib) içindeki 'printf'
// C fonksiyonunu Vex'e tanıtıyoruz.
extern "C" {
    fn printf(format: *const byte, ...) -> int;
}

fn main(): error {
    let message = "C'den Merhaba, Vex!\n";

    // Vex string'ini C-tarzı null-terminated string'e çevir
    let c_string = unsafe { unsafe::to_c_string(message) };

    // FFI çağrıları her zaman 'unsafe' blok içinde olmalıdır
    unsafe {
        printf(c_string);
    }
    return nil;
}

```

## 9. Standart Kütüphane (`std`) (v0.6)

Vex, Go'dan ilham alan, "pilleri dahil" (batteries-included) ancak modüler bir standart kütüphane (`std`) ile gelir.

- **`std::io`:** Dosya, ağ ve konsol I/O işlemleri. (Vex runtime'ının `io_uring`'ini kullanır).
- **`std::log`:** Basit, yapılandırılmış loglama.
- **`std::http`:** `std::io` üzerine kurulu, yüksek performanslı `async` HTTP sunucusu ve istemcisi.
- **`std::json`:** `struct` etiketlerini (Bölüm 2.3.3) anlayan hızlı JSON (de)serializasyonu.
- **`std::sync`:** CPU-paralelliği için temel araçlar (Mutex, WaitGroup, vb.).
- **`std::hpc`:** `launch` (Bölüm 6.1) ile etkileşime giren, `gpu` ve `simd` için düşük seviyeli "intrinsic"ler ve araçlar (örn: `hpc::global_id()`).
- **`std::testing`:** Dile entegre edilmiş basit bir test kütüphanesi (`vex test`).
- **`std::unsafe`:** `to_c_string()` gibi, FFI (Bölüm 8) ve pointer (Bölüm 2.6) işlemleri için yardımcılar.
