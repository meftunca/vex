# Vex Standard Library - Kapsamlı Analiz ve İlerleme Planı

**Tarih:** 13 Kasım 2025  
**Versiyon:** 0.2.0 (Syntax v0.1.2)  
**Test Durumu:** 407/407 test geçiyor (%100) ✅

---

## 📊 Mevcut Durum - Genel Bakış

### Kod Metrikleri

| Kategori                       | Dosya Sayısı | Satır Sayısı   | Durum               |
| ------------------------------ | ------------ | -------------- | ------------------- |
| **Vex Stdlib (stdlib/)**       | 8 dosya      | 757 satır      | 🟡 Temel seviye     |
| **C Runtime (vex-runtime/c/)** | 30 dosya     | 13,258 satır   | ✅ Production-ready |
| **Runtime API (vex.h)**        | 1 dosya      | ~240 fonksiyon | ✅ Kapsamlı         |
| **Builtin Contracts**          | 1 dosya      | 366 satır      | ✅ Çalışıyor        |
| **Toplam Test**                | 418 dosya    | -              | ✅ %100 başarılı    |

### Stdlib Paket Yapısı (Mevcut)

```
stdlib/
├── core/src/
│   ├── lib.vx              # 18 satır - Display, Clone, Debug contracts
│   ├── ops.vx              # 249 satır - Operator contracts (Add, Sub, Mul, etc.)
│   └── builtin_contracts.vx # Placeholder
├── fmt/src/
│   └── lib.vx              # 29 satır - Display, Default, Format contracts
├── fmt_builtin/src/
│   └── lib.vx              # Placeholder
└── vex/
    ├── beta_option.vx      # 71 satır - Option<T> implementasyonu
    ├── beta_result.vx      # 77 satır - Result<T,E> implementasyonu
    └── beta_vec.vx         # 313 satır - Vec<T> implementasyonu
```

---

## 🔍 Detaylı Analiz

### 1. **C Runtime (vex-runtime/c/) - ✅ ÇOK GÜÇLÜ**

#### Güçlü Yönler:

- **240+ fonksiyon** - Kapsamlı runtime API
- **13,258 satır** production-ready C kodu
- **Sıfır overhead** - Direkt LLVM IR entegrasyonu
- **Cross-platform** - Linux/macOS/Windows desteği

#### Temel Kategoriler:

##### String Operations (UTF-8 Desteği)

```c
✅ vex_strlen, vex_strcmp, vex_strcat
✅ vex_utf8_valid, vex_utf8_char_count
✅ vex_utf8_to_utf16, vex_utf8_to_utf32
✅ vex_i32_to_string, vex_f64_to_string
```

##### Memory Management

```c
✅ vex_malloc, vex_calloc, vex_realloc, vex_free
✅ vex_memcpy, vex_memmove, vex_memset, vex_memcmp
```

##### I/O Operations

```c
✅ vex_print, vex_println, vex_printf
✅ vex_file_open, vex_file_read, vex_file_write
✅ vex_file_seek, vex_file_tell, vex_file_size
```

##### Data Structures (C-level)

```c
✅ VexArray - Dinamik array (len, capacity, slice, append)
✅ VexMap - SwissTable HashMap implementation
✅ VexSlice - Zero-copy slice (&[T])
✅ VexRange - Iterator support (0..10, 0..=10)
```

##### Advanced Runtime

```c
✅ async_runtime/ - Coroutine runtime (Rust-style async/await)
✅ vex_net/ - Epoll/kqueue/IOCP event loop
✅ vex_time/ - Time operations, timers, timezone
✅ vex_channel - CSP-style channels
✅ vex_sync - Mutex, RwLock, Semaphore
```

#### Eksiklikler (C Runtime):

- ❌ Regex API henüz Vex'e expose edilmedi
- ❌ Network sockets için high-level Vex API yok
- ❌ Async/await Vex syntax desteği yok (sadece C runtime var)

---

### 2. **Vex Stdlib Packages - 🟡 TEMEL SEVİYE**

#### Mevcut Paketler:

##### `core` - Builtin Contracts ✅

```vex
// stdlib/core/src/ops.vx (249 satır)
contract Add { op+(rhs: Self): Self; }
contract Sub { op-(rhs: Self): Self; }
contract Mul, Div, Mod, BitAnd, BitOr, BitXor, Eq, Ord
contract Index { op[](index: T): Output; }
contract Drop { drop(); }
```

**Durum:** ✅ Tamam - Operator overloading çalışıyor  
**Eksik:**

- Associated types tam desteği (Index.Output)
- Compound assignment operators (op+=, op-=)

##### `fmt` - Formatting ⚠️

```vex
// stdlib/fmt/src/lib.vx (29 satır)
contract Display { display(): str; }
contract Format { format(spec: str): str; }
```

**Durum:** ⚠️ Sadece contract tanımları var  
**Eksik:**

- Gerçek implementasyon yok
- `fmt.printf`, `fmt.sprintf` yok
- Format specifiers (%d, %s, %f) yok

##### `vex` - Beta Types 🟡

```vex
// stdlib/vex/beta_option.vx (71 satır)
enum BetaOption<T> { Some(T), None }
fn IsSome<T>, IsNone<T>, Unwrap<T>, UnwrapOr<T>

// stdlib/vex/beta_result.vx (77 satır)
enum BetaResult<T,E> { Ok(T), Err(E) }

// stdlib/vex/beta_vec.vx (313 satır)
struct BetaVec<T> { data: *T, len: i64, cap: i64 }
```

**Durum:** 🟡 Beta - Kısıtlı API  
**Eksik:**

- Iterator support yok
- Method chaining yok (map, filter, reduce)
- Error propagation (? operator) yok

---

## 🆚 Rust/Go Karşılaştırması

### Rust Standard Library Kapsamı

#### Rust `std::` Modülleri (Temel):

```rust
✅ std::vec::Vec<T>        - Dinamik array (700+ satır API)
✅ std::string::String      - UTF-8 string (500+ satır)
✅ std::collections::      - HashMap, BTreeMap, HashSet
✅ std::option::Option<T>  - Null safety
✅ std::result::Result<T,E> - Error handling
✅ std::iter::Iterator     - Lazy iteration
✅ std::io::                - Read, Write, BufReader
✅ std::fs::                - File system operations
✅ std::path::Path         - Path manipulation
✅ std::sync::             - Mutex, RwLock, Arc, Barrier
✅ std::thread::           - Thread spawning
✅ std::time::             - Duration, Instant, SystemTime
✅ std::fmt::              - Display, Debug, format!
✅ std::env::              - Environment variables
✅ std::process::          - Command execution
```

**Rust Toplam:** ~50 modül, ~1000 public type, ~5000+ public fonksiyon

### Go Standard Library Kapsamı

#### Go `std` Packages (Temel):

```go
✅ fmt         - Printf, Sprintf, Fprintf (150+ fonksiyon)
✅ strings     - Contains, Split, Join, ToUpper
✅ strconv     - Atoi, Itoa, ParseInt, FormatFloat
✅ io          - Reader, Writer, Copy, ReadAll
✅ os          - File, Open, Create, ReadDir
✅ path/filepath - Join, Clean, Abs, Walk
✅ sync        - Mutex, RWMutex, WaitGroup, Once
✅ time        - Now, Parse, Format, Sleep, Timer
✅ net         - Listen, Dial, HTTP client/server
✅ http        - Server, Client, Request, Response
✅ encoding    - json, xml, base64, hex
✅ errors      - New, Is, As, Wrap
✅ context     - Context, WithCancel, WithTimeout
```

**Go Toplam:** ~150 package, ~5000+ public function

### Vex Mevcut Durum

| Kategori           | Rust                        | Go                       | Vex              | Eksiklik               |
| ------------------ | --------------------------- | ------------------------ | ---------------- | ---------------------- |
| **Collections**    | Vec, HashMap, BTreeMap      | slice, map               | BetaVec          | ❌ HashMap, Set, Queue |
| **Strings**        | String, &str, format!       | strings, fmt             | Sadece C runtime | ❌ Vex API yok         |
| **I/O**            | Read, Write, BufReader      | io.Reader, io.Writer     | Sadece C runtime | ❌ Vex API yok         |
| **Error Handling** | Result<T,E>, ? operator     | error interface          | BetaResult       | ❌ ? operator yok      |
| **Iterators**      | Iterator trait, map, filter | for range                | Yok              | ❌ Tamamen eksik       |
| **Time**           | Duration, Instant           | time.Time, time.Duration | C runtime        | ❌ Vex API yok         |
| **Concurrency**    | thread, Arc, Mutex          | goroutine, channel       | C runtime        | ❌ Vex syntax yok      |
| **Formatting**     | fmt::Display, format!       | fmt.Printf               | Sadece contract  | ❌ İmplementasyon yok  |

---

## 🎯 Öncelikli Eksiklikler

### Kritik (P0) - Temel İşlevsellik İçin Gerekli

1. **String Manipulation API**

   - ❌ `string.len()`, `string.contains()`, `string.split()`
   - ❌ `string.to_upper()`, `string.to_lower()`
   - ❌ `string.trim()`, `string.replace()`
   - **C Runtime:** ✅ Var (`vex_strlen`, `vex_strcmp`)
   - **Vex API:** ❌ Yok

2. **Formatting (`fmt` package)**

   - ❌ `fmt.printf(format, ...args)`
   - ❌ `fmt.sprintf(format, ...args): string`
   - ❌ Format specifiers: `%d`, `%s`, `%f`, `%v`
   - **C Runtime:** ✅ Var (`vex_printf`, `vex_sprintf`)
   - **Vex API:** ❌ Yok

3. **Collections (`collections` package)**

   - ❌ `HashMap<K,V>` - Key-value storage
   - ❌ `HashSet<T>` - Unique values
   - ❌ `LinkedList<T>`, `Queue<T>`, `Stack<T>`
   - **C Runtime:** ✅ VexMap var (SwissTable)
   - **Vex API:** ❌ Yok

4. **Iterator Trait**
   - ❌ `contract Iterator<T> { next(): Option<T>; }`
   - ❌ `map<U>(fn(T): U): Iterator<U>`
   - ❌ `filter(fn(&T): bool): Iterator<T>`
   - ❌ `collect<C>(): C`
   - **Eksiklik:** Tamamen yok

### Yüksek Öncelik (P1) - Pratik Kullanım İçin

5. **I/O Abstractions (`io` package)**

   - ❌ `contract Read { read(&!buf): Result<usize, Error>; }`
   - ❌ `contract Write { write(&buf): Result<usize, Error>; }`
   - ❌ `struct BufReader<R: Read>`
   - **C Runtime:** ✅ `vex_file_read`, `vex_file_write`
   - **Vex API:** ❌ Yok

6. **File System (`fs` package)**

   - ❌ `fs.read_file(path): Result<string, Error>`
   - ❌ `fs.write_file(path, content): Result<(), Error>`
   - ❌ `fs.read_dir(path): Result<Vec<DirEntry>, Error>`
   - **C Runtime:** ✅ `vex_file_open`, `vex_file_read_all`
   - **Vex API:** ❌ Yok

7. **Path Manipulation (`path` package)**

   - ❌ `path.join(parts): string`
   - ❌ `path.clean(path): string`
   - ❌ `path.ext(path): string`
   - **C Runtime:** ✅ `vex_path_join`, `vex_path_clean`
   - **Vex API:** ❌ Yok

8. **Error Types**
   - ❌ `struct Error { message: string, code: i32 }`
   - ❌ `? operator` - Error propagation
   - ❌ Error wrapping/unwrapping
   - **Eksiklik:** Sadece BetaResult var

### Orta Öncelik (P2) - İleri Seviye Özellikler

9. **Time (`time` package)**

   - ❌ `time.now(): Time`
   - ❌ `time.sleep(duration)`
   - ❌ `struct Duration`
   - **C Runtime:** ✅ Tam implementasyon var
   - **Vex API:** ❌ Yok

10. **Concurrency (`sync` package)**

    - ❌ `struct Mutex<T>`
    - ❌ `struct Channel<T>`
    - ❌ `spawn()` - Thread/goroutine
    - **C Runtime:** ✅ `vex_sync.c`, `vex_channel.c`
    - **Vex API:** ❌ Yok

11. **Network (`net` package)**

    - ❌ `net.listen(address): Result<Listener, Error>`
    - ❌ `net.dial(address): Result<Connection, Error>`
    - **C Runtime:** ✅ `vex_net/` - Epoll/kqueue event loop
    - **Vex API:** ❌ Yok

12. **Async/Await**
    - ❌ `async fn`, `await` keywords
    - ❌ `Future<T>` trait
    - **C Runtime:** ✅ Coroutine runtime var
    - **Vex Syntax:** ❌ Yok

---

## 📋 İlerleme Planı - Önerilen Yol Haritası

### Phase 1: Temel Kullanılabilirlik (2-3 hafta)

#### 1.1 String API (`std::string`)

**Hedef:** Rust/Go string paritesi

```vex
// stdlib/std/string/src/lib.vx
export struct String {
    data: *u8,
    len: i64,
    cap: i64,
}

export impl String {
    // Constructors
    fn new(): String;
    fn from_str(s: &str): String;
    fn with_capacity(cap: i64): String;

    // Core methods
    fn len(&self): i64;
    fn is_empty(&self): bool;
    fn push(&!self, s: &str);
    fn push_char(&!self, c: char);

    // Search
    fn contains(&self, needle: &str): bool;
    fn starts_with(&self, prefix: &str): bool;
    fn ends_with(&self, suffix: &str): bool;
    fn index_of(&self, needle: &str): Option<i64>;

    // Transform
    fn to_upper(&self): String;
    fn to_lower(&self): String;
    fn trim(&self): String;
    fn trim_start(&self): String;
    fn trim_end(&self): String;

    // Split
    fn split(&self, delimiter: &str): Vec<String>;
    fn lines(&self): Vec<String>;
    fn words(&self): Vec<String>;

    // Replace
    fn replace(&self, from: &str, to: &str): String;
    fn replace_all(&self, from: &str, to: &str): String;

    // Substring
    fn substring(&self, start: i64, end: i64): String;
    fn chars(&self): Vec<char>;
}

// String formatting
export fn format(fmt: &str, ...args: any): String;
```

**C Runtime Mapping:**

```c
✅ vex_strlen          → String.len()
✅ vex_strcmp          → String == operator
✅ vex_strcat          → String.push()
✅ vex_strdup          → String.from_str()
✅ vex_utf8_char_count → String.chars().len()
```

#### 1.2 Formatting (`std::fmt`)

**Hedef:** Go `fmt.Printf` paritesi

```vex
// stdlib/std/fmt/src/lib.vx
export fn printf(format: &str, ...args: any): i64;
export fn sprintf(format: &str, ...args: any): String;
export fn fprintf(writer: &!Write, format: &str, ...args: any): Result<i64, Error>;

export fn println(...args: any);
export fn print(...args: any);
export fn eprintln(...args: any);

// Format specifiers:
// %d, %i - Signed integer
// %u - Unsigned integer
// %f - Floating point
// %s - String
// %v - Default format (Display trait)
// %p - Pointer
// %x, %X - Hexadecimal
// %o - Octal
// %b - Binary
```

**C Runtime Mapping:**

```c
✅ vex_printf  → fmt.printf()
✅ vex_sprintf → fmt.sprintf()
✅ vex_print   → fmt.print()
✅ vex_println → fmt.println()
```

#### 1.3 Collections - HashMap (`std::collections`)

**Hedef:** Rust HashMap paritesi

```vex
// stdlib/std/collections/src/hashmap.vx
export struct HashMap<K, V> {
    // Internal: VexMap C runtime
}

export impl<K, V> HashMap<K, V> {
    fn new(): HashMap<K, V>;
    fn with_capacity(capacity: i64): HashMap<K, V>;

    fn insert(&!self, key: K, value: V): Option<V>;
    fn get(&self, key: &K): Option<&V>;
    fn get_mut(&!self, key: &K): Option<&!V>;
    fn remove(&!self, key: &K): Option<V>;

    fn contains_key(&self, key: &K): bool;
    fn len(&self): i64;
    fn is_empty(&self): bool;
    fn clear(&!self);

    // Iterators
    fn keys(&self): Iterator<&K>;
    fn values(&self): Iterator<&V>;
    fn iter(&self): Iterator<(&K, &V)>;
}
```

**C Runtime Mapping:**

```c
✅ VexMap (SwissTable) → HashMap<K,V>
✅ vex_map_new         → HashMap.new()
✅ vex_map_insert      → HashMap.insert()
✅ vex_map_get         → HashMap.get()
```

#### 1.4 Error Handling (`std::error`)

**Hedef:** Rust Result + ? operator

```vex
// stdlib/std/error/src/lib.vx
export struct Error {
    message: String,
    code: i32,
    source: Option<Box<Error>>,
}

export impl Error {
    fn new(message: &str): Error;
    fn with_code(message: &str, code: i32): Error;
    fn wrap(err: Error, message: &str): Error;

    fn message(&self): &str;
    fn code(&self): i32;
    fn source(&self): Option<&Error>;
}

// Result type improvements
export type Result<T> = Result<T, Error>;

// ? operator (syntax sugar for early return on Err)
// fn read_config(): Result<Config> {
//     let file = fs.read_file("config.toml")?; // Auto unwrap or return Err
//     let config = toml.parse(file)?;
//     return Ok(config);
// }
```

### Phase 2: I/O & File System (1-2 hafta)

#### 2.1 I/O Traits (`std::io`)

```vex
// stdlib/std/io/src/lib.vx
export contract Read {
    read(&!self, buf: &![u8]): Result<i64, Error>;
}

export contract Write {
    write(&!self, buf: &[u8]): Result<i64, Error>;
    flush(&!self): Result<(), Error>;
}

export struct BufReader<R: Read> {
    inner: R,
    buffer: Vec<u8>,
}

export impl<R: Read> BufReader<R> {
    fn new(reader: R): BufReader<R>;
    fn read_line(&!self): Result<String, Error>;
    fn lines(&!self): Iterator<Result<String, Error>>;
}
```

#### 2.2 File System (`std::fs`)

```vex
// stdlib/std/fs/src/lib.vx
export struct File {
    // Internal: VexFile C runtime
}

export impl File {
    fn open(path: &str): Result<File, Error>;
    fn create(path: &str): Result<File, Error>;
    fn read(&!self, buf: &![u8]): Result<i64, Error>;
    fn write(&!self, buf: &[u8]): Result<i64, Error>;
}

export fn read_file(path: &str): Result<String, Error>;
export fn write_file(path: &str, content: &str): Result<(), Error>;
export fn read_dir(path: &str): Result<Vec<DirEntry>, Error>;
```

**C Runtime Mapping:**

```c
✅ vex_file_open     → fs.File.open()
✅ vex_file_read     → fs.File.read()
✅ vex_file_write    → fs.File.write()
✅ vex_file_read_all → fs.read_file()
```

### Phase 3: Iterators & Advanced Collections (2 hafta)

#### 3.1 Iterator Trait

```vex
// stdlib/std/iter/src/lib.vx
export contract Iterator<T> {
    type Item;
    fn next(&!self): Option<Item>;
}

export impl<T> Iterator<T> {
    fn map<U>(self, f: fn(T): U): Map<Self, U>;
    fn filter(self, predicate: fn(&T): bool): Filter<Self>;
    fn fold<B>(self, init: B, f: fn(B, T): B): B;
    fn collect<C: FromIterator<T>>(self): C;
    fn count(self): i64;
    fn sum(self): T where T: Add;
}

// Example usage:
// let numbers = vec![1, 2, 3, 4, 5];
// let sum = numbers.iter()
//     .filter(|x| x % 2 == 0)
//     .map(|x| x * 2)
//     .sum();  // 12 (2*2 + 4*2)
```

#### 3.2 Vec Improvements

```vex
// stdlib/std/vec/src/lib.vx
export impl<T> Vec<T> {
    // Iterators
    fn iter(&self): Iter<T>;
    fn iter_mut(&!self): IterMut<T>;
    fn into_iter(self): IntoIter<T>;

    // Functional methods
    fn map<U>(self, f: fn(T): U): Vec<U>;
    fn filter(self, predicate: fn(&T): bool): Vec<T>;
    fn filter_map<U>(self, f: fn(T): Option<U>): Vec<U>;

    // Additional collections
    fn extend(&!self, other: Vec<T>);
    fn append(&!self, other: &!Vec<T>);
    fn splice(&!self, range: Range, replace_with: Vec<T>);
}
```

### Phase 4: Concurrency & Async (3-4 hafta)

#### 4.1 Sync Primitives (`std::sync`)

```vex
// stdlib/std/sync/src/lib.vx
export struct Mutex<T> {
    // Internal: vex_sync.c
}

export impl<T> Mutex<T> {
    fn new(value: T): Mutex<T>;
    fn lock(&!self): MutexGuard<T>;
    fn try_lock(&!self): Option<MutexGuard<T>>;
}

export struct Channel<T> {
    // Internal: vex_channel.c
}

export impl<T> Channel<T> {
    fn new(): Channel<T>;
    fn send(&!self, value: T): Result<(), Error>;
    fn recv(&!self): Result<T, Error>;
}
```

#### 4.2 Async/Await Syntax

```vex
// New syntax support needed
async fn fetch_data(url: &str): Result<String, Error> {
    let response = http.get(url).await?;
    let body = response.body().await?;
    return Ok(body);
}

fn main() {
    let data = fetch_data("https://api.example.com").await;
    println(data);
}
```

**C Runtime Mapping:**

```c
✅ async_runtime/ → async fn runtime
✅ vex_net/       → async I/O
✅ poller_vexnet  → Event loop backend
```

### Phase 5: Advanced Features (2-3 hafta)

#### 5.1 Time (`std::time`)

```vex
// stdlib/std/time/src/lib.vx
export struct Duration {
    nanos: i64,
}

export struct Instant {
    // Internal: vex_time
}

export impl Instant {
    fn now(): Instant;
    fn elapsed(&self): Duration;
}

export fn sleep(duration: Duration);
export fn sleep_ms(milliseconds: i64);
```

#### 5.2 Network (`std::net`)

```vex
// stdlib/std/net/src/lib.vx
export struct TcpListener {
    // Internal: vex_net
}

export impl TcpListener {
    fn bind(addr: &str): Result<TcpListener, Error>;
    fn accept(&!self): Result<TcpStream, Error>;
}

export struct TcpStream {
    // Internal: vex_net
}

export impl TcpStream {
    fn connect(addr: &str): Result<TcpStream, Error>;
}

impl Read for TcpStream { ... }
impl Write for TcpStream { ... }
```

---

## 🚀 Hızlı Başlangıç Planı (1 Hafta Sprint)

### Sprint 1: String + fmt (Minimum Viable Stdlib)

**Hedef:** Go `fmt.Printf` + Rust `String` paritesi

**Çıktılar:**

1. `std::string::String` - 20 metod ✅
2. `std::fmt` - printf, sprintf, println ✅
3. Test coverage: %80+ ✅

**Kod Tahmini:**

- String API: ~400 satır Vex + 200 satır C wrapper
- fmt API: ~200 satır Vex + 100 satır C wrapper
- Tests: ~300 satır

**Toplam:** ~1200 satır (1 hafta içinde yapılabilir)

### Sprint 2: Collections + Error (Temel Veri Yapıları)

**Hedef:** HashMap + Result + ? operator

**Çıktılar:**

1. `std::collections::HashMap<K,V>` ✅
2. `std::error::Error` ✅
3. `? operator` syntax support ✅

**Kod Tahmini:** ~800 satır

### Sprint 3: I/O + fs (Dosya İşlemleri)

**Hedef:** Rust `std::fs` paritesi

**Çıktılar:**

1. `std::io::Read`, `std::io::Write` ✅
2. `std::fs::File`, `read_file`, `write_file` ✅

**Kod Tahmini:** ~600 satır

---

## 📊 Başarı Metrikleri

### Kısa Vade (1 ay)

- ✅ String API: 20+ metod
- ✅ fmt paritesi: printf, sprintf, println
- ✅ HashMap<K,V> functional
- ✅ Error + ? operator

### Orta Vade (3 ay)

- ✅ Iterator trait + map/filter/collect
- ✅ I/O abstractions (Read, Write)
- ✅ File system (fs::File, read_file)
- ✅ 50+ stdlib tests

### Uzun Vade (6 ay)

- ✅ Async/await syntax
- ✅ Network (TcpListener, TcpStream)
- ✅ Concurrency (Mutex, Channel)
- ✅ 100+ stdlib tests
- ✅ Rust/Go paritesi: %70+

---

## ✅ Sonuç ve Öneriler

### Mevcut Durum Özeti

**Güçlü Yönler:**

1. ✅ **C Runtime son derece güçlü** - 240+ fonksiyon, production-ready
2. ✅ **Operator overloading çalışıyor** - Add, Sub, Mul, Index, etc.
3. ✅ **Test coverage mükemmel** - %100 pass rate
4. ✅ **Async runtime hazır** - Sadece Vex syntax bekleniyor

**Zayıf Yönler:**

1. ❌ **Vex stdlib çok minimal** - Sadece 757 satır
2. ❌ **C runtime ile Vex arasında köprü yok** - String, HashMap, I/O için API yok
3. ❌ **Iterator support yok** - map/filter/collect eksik
4. ❌ **Error handling primitive** - ? operator yok

### Önerilen Strateji

#### **Bottom-Up Yaklaşım (Önerilen)** ✅

1. **C Runtime'ı Vex'e Expose Et**

   - Mevcut 240 fonksiyonu Vex API ile sar
   - Zero-cost abstractions (inline wrapper'lar)
   - Örnek: `vex_strlen` → `String.len()`

2. **Temel Paketlerle Başla**

   - `std::string` (1 hafta)
   - `std::fmt` (1 hafta)
   - `std::collections::HashMap` (1 hafta)

3. **Iterator Trait'i Implement Et**

   - Vec, HashMap, String için Iterator
   - map, filter, collect metodları

4. **Syntax Enhancements**
   - `? operator` for error propagation
   - `async/await` keywords

#### **Top-Down Yaklaşım (Alternatif)** ⚠️

1. Rust/Go stdlib'i kopyala (5000+ fonksiyon)
2. Her fonksiyonu tek tek implement et
3. **Problem:** Çok uzun sürer (6+ ay), verimsiz

### Hızlı Kazanımlar (Quick Wins)

**1 Hafta İçinde Yapılabilir:**

```vex
// String API (C runtime wrapper)
export impl String {
    fn len(&self): i64 { return vex_strlen(self.data); }
    fn contains(&self, needle: &str): bool { ... }
    fn split(&self, delimiter: &str): Vec<String> { ... }
}

// fmt.printf (C runtime wrapper)
export fn printf(format: &str, ...args: any): i64 {
    return vex_printf(format.data, args);
}

// HashMap (C VexMap wrapper)
export impl<K,V> HashMap<K,V> {
    fn new(): HashMap<K,V> { return HashMap { map: vex_map_new() }; }
    fn insert(&!self, key: K, value: V) { vex_map_insert(&self.map, key, value); }
}
```

**Sonuç:** 1200 satır Vex kodu ile Rust/Go'nun %30'u tamamlanır.

### Final Tavsiye

**ÖNCELİK:**

1. ✅ String API (C runtime wrapper) - 1 hafta
2. ✅ fmt.printf/sprintf - 1 hafta
3. ✅ HashMap<K,V> - 1 hafta
4. ✅ Iterator trait - 2 hafta

**4 hafta sonunda:**

- ✅ Pratik kod yazılabilir
- ✅ Rust/Go'nun %40'ı tamamlanmış olur
- ✅ Test coverage %80+

**Devam:**

- I/O (Read, Write, File)
- Error handling (? operator)
- Async/await syntax

**Zaman çizelgesi:** 3 ay içinde production-ready stdlib.
