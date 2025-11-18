# Vex Tasarım Notları: Constructor, Destructor, Channel Operatörleri & Prelude

Bu doküman, Vex dili için şu konularda aldığımız kararları özetler:

- **Constructor modeli**
- **Destructor / RAII modeli**
- **Channel & Iterator için `<-` operatörü**
- **Vex core prelude'de neler olmalı?**

Tüm örnekler “Vexçe” yazılmıştır; syntax bazı yerlerde pseudo olabilir ama tasarım fikrini taşır.

---

## 1. Struct Method Modeli & `self`

Vex’te `struct` içindeki fonksiyonlar **implicit `self`** ile çalışır:

- `fn foo()`  → `self: &Self` (immutable method)
- `fn foo()!` → `self: &mut Self` (mutating method)
- Gövde içinde `self` her zaman kullanılabilir; imzada yazılmaz.
- Gövdede `self` hiç kullanılmıyorsa fonksiyon **static / associated** fonksiyon sayılır.

```vex
struct Counter {
    current: i32,
    max: i32,

    // immutable method
    fn remaining(): i32 {
        return self.max - self.current;
    }

    // mutating method
    fn inc()! {
        self.current = self.current + 1;
    }

    // static / associated function (self yok)
    fn from_max(max: i32): Counter {
        return Counter { current: 0, max: max };
    }
}
```

---

## 2. Constructor: Type-level Call Operator (`op(...)` → `Foo(...)`)

### 2.1. Tanım

Bir `struct` için constructor, **type-level call operator** ile tanımlanır:

```vex
struct Foo {
    x: i32,

    // constructor
    op(x: i32): Foo {
        if x < 0 {
            panic("x must be >= 0");
        }
        return Foo { x: x };
    }
}
```

### 2.2. Çağrı Semantiği

```vex
let a = Foo(10);       // Foo::op(10)
let b = Foo { x: 10 }; // ham literal, doğrudan init
```

- `Foo(10)`  → **constructor çağrısı**, invariants / validation içerebilir.
- `Foo { x: 10 }` → ham literal, “saf layout” init.

Derleyici kuralı:

- Struct içinde `op(...)` tanımlıysa, `Foo(...)` çağrısı **`Foo::op(...)` olarak desugar** edilir.
- Eğer `op(...)` tanımlı değilse:
  - `Foo(...)` kullanımı **compile error**,
  - `Foo { ... }` literal her zaman serbest.

### 2.3. Overload & Generics

`op(...)` overload edilebilir:

```vex
struct Vec<T> {
    ptr: *T,
    len: usize,
    cap: usize,

    op(): Vec<T> {
        return Vec { ptr: null, len: 0, cap: 0 };
    }

    op(capacity: usize): Vec<T> {
        // capacity kadar allocate et
        ...
    }
}

fn main(): i32 {
    let v1 = Vec<i32>();     // Vec::<i32>::op()
    let v2 = Vec<i32>(16);   // Vec::<i32>::op(16)
    return 0;
}
```

Overload resolution, Vex’teki normal operator overloading kuralları ile aynıdır.

### 2.4. `new(...)` Sugar (Box / heap için opsiyonel tasarım)

Vex’te Rust’taki `Box::new` ergonomisini, Go’daki `new` hissiyle verebilirsin:

```vex
struct Box<T> {
    ptr: *T,

    fn _free()! {
        // T'nin destructor'ını çağır, sonra free
    }
}

fn box_new<T>(value: T): Box<T> {
    // allocate + move
}

fn new<T>(value: T): Box<T> {
    return box_new(value);
}

fn main(): i32 {
    let! f = new(File("log.txt"));  // Box<File>
    let! n = new(42);               // Box<i32>
    return 0;
}
```

(Bu sugar opsiyoneldir; dil seviyesinde `new Foo(...)` gibi ekstra syntax da tanımlanabilir.)

---

## 3. Destructor: `_free()!` ile RAII

Destructor, struct içinde tanımlanan **özel isimli** mutating method ile ifade edilir:

```vex
struct File {
    handle: OSHandle,

    // constructor
    op(path: &Str): File {
        let h = os_open(path)?;
        return File { handle: h };
    }

    // destructor
    fn _free()! {
        os_close(self.handle);
    }
}
```

### 3.1. Dil Kuralı

- Bir `struct` içinde **tam olarak** şu imzaya sahip method:

  ```vex
  fn _free()!
  ```

  **destructor** kabul edilir.

- Özellikler:
  - Parametre yok, dönüş tipi yok (implicit `void`).
  - Gövdede `self` → `&mut Self` (mutating method).
  - Derleyici, bu struct’ın her instance’ı için `_free()`’yi **en fazla 1 kez** çağırır:
    - Stack değişkeni scope’tan çıkarken,
    - Owner container’lar (`Box<T>`, `Vec<T>`, `Option<T>` vb.) drop olurken iç elementler için.

### 3.2. Kullanım Örneği (RAII)

```vex
fn main(): i32 {
    let! f = File("log.txt");   // File::op("log.txt")
    println("using file...");
    return 0;
} // scope sonu: f._free()! otomatik çağrılır → os_close
```

### 3.3. Manuel Çağrı

V1 önerisi (en güvenlisi):

- `_free()` user code’dan çağrılamaz:

  ```vex
  f._free(); // ❌ error: `_free` is a destructor and cannot be called manually
  ```

Gelecekte istenirse: `_free()` çağrısından sonra value “consumed” sayılıp tekrar kullanılamayacak şekilde semantik genişletilebilir.

### 3.4. Copy / Drop İlişkisi

- `Copy` olan tiplerin `_free()` tanımlamasına izin verilmez (compile error).
- Bir tip destructor’a sahipse:
  - `Option<T>`, `Vec<T>`, `Box<T>` gibi container’lar drop olurken elementler için `_free()` çağırır.

Derleyici içi implementasyon isterse gizli bir `Drop` contract’ı ile temsil edilebilir, ama kullanıcı bunu doğrudan görmek zorunda değildir.

---

## 4. `<-` Operatörü: Iterator & Channel Pop/Send/Recv

Vex’te `<-` operatörü, **operator overloading** ile tanımlanır, fakat dile özel bir miktar sugar verilir.

İki ana kullanım:

1. **Iterator / pop benzeri kaynaklar** için: `let x <- iter;`
2. **Channel** için (Go tarzı):
   - `let msg <- ch;` → recv
   - `ch <- msg;`     → send

### 4.1. Iterator için: `PopIter` + unary `op<-()!`

```vex
contract PopIter {
    type Item;

    // self'ten bir eleman çek, kendini ilerlet
    op<-()!: Option<Self::Item>;
}
```

Örnek implementasyon:

```vex
struct Counter impl PopIter {
    type Item = i32;
    current: i32,
    max: i32,

    op<-()!: Option<i32> {
        if self.current >= self.max {
            return None;
        }
        let v = self.current;
        self.current = self.current + 1;
        return Some(v);
    }
}
```

#### 4.1.1. Sugar: `let x <- it;`

```vex
let! it = Counter { current: 0, max: 3 };

let a <- it;
let b <- it;
let c <- it;
```

Desugar:

```vex
let a = it.op<-();
let b = it.op<-();
let c = it.op<-();
```

`it` tipi `PopIter` / `op<-()!` implemente etmiyorsa compile error alınır.

#### 4.1.2. `while let Some(v) <- it` Pattern’i

```vex
while let Some(v) <- it {
    println("v = {}", v);
}
```

Kabaca desugar:

```vex
while true {
    let tmp = it.op<-();      // Option<Item>
    match tmp {
        Some(v) => { println("v = {}", v); }
        None    => break;
    }
}
```

Sugar tamamen compile-time’dır; runtime cost yoktur.

---

### 4.2. Channel için: `ChanRecvCtx` / `ChanSendCtx` + `<-`

Context entegrasyonlu bir taslak (basit haliyle):

```vex
contract ChanRecvCtx {
    type Item;
    fn recv(ctx: &Ctx)!: Result<Option<Self::Item>, ChanError>;
}

contract ChanSendCtx<T> {
    fn send(ctx: &Ctx, value: T)!: Result<void, ChanError>;
}
```

Channel implementasyonu (pseudo):

```vex
struct Channel<T> impl ChanRecvCtx, ChanSendCtx<T> {
    // internal queue, lock vs.

    fn recv(ctx: &Ctx)!: Result<Option<T>, ChanError> {
        if ctx.is_cancelled() {
            return Err(ChanError::Cancelled);
        }
        // queue'dan eleman oku / bekle
    }

    fn send(ctx: &Ctx, v: T)!: Result<void, ChanError> {
        if ctx.is_cancelled() {
            return Err(ChanError::Cancelled);
        }
        // queue'ya push et
    }
}
```

#### 4.2.1. `<-` ile adapter (ctx ile birlikte)

Channel’ı context ile birlikte `PopIter` gibi kullanmak için:

```vex
struct ChanWithCtx<T> impl PopIter {
    chan: Channel<T>,
    ctx: &Ctx,

    op<-()!: Option<T> {
        let res = self.chan.recv(self.ctx)!;
        match res {
            Ok(Some(v)) => return Some(v),
            Ok(None)    => return None,   // channel kapalı
            Err(_)      => return None,   // cancel/timeout → None
        }
    }
}

impl<T> Channel<T> {
    fn with_ctx(ctx: &Ctx): ChanWithCtx<T> {
        return ChanWithCtx { chan: self, ctx: ctx };
    }
}
```

Kullanım:

```vex
while let Some(job) <- jobs.with_ctx(ctx) {
    handle_job(ctx, job);
}
```

`while let Some(v) <- ...` sugar’ı, iteratör veya channel uyumlu her türle birlikte çalışabilir.

---

### 4.3. `<-` Overload Resolution Kuralları

1. **Normal ifade:** `expr1 <- expr2`

   - Derleyici `expr1`’in tipine bakar.
   - Bu tip üzerinde `op<-(rhs_type)!` methodu varsa:

     ```vex
     expr1 <- expr2;
     // => expr1.op<-(expr2);
     ```

   - Overload yoksa compile error.

2. **`let pattern <- expr` formu (sadece let ile)**

   - `expr` tipinde `op<-()!` methodu aranır.
   - Varsa:

     ```vex
     let pattern <- expr;
     // => let pattern = expr.op<-();
     ```

   - Yoksa compile error.

3. **`while let Some(v) <- expr` formu**

   - Yukarıdaki `let pattern <- expr` kuralına ek olarak:
     - Sonuç `Option<T>` ise:
       - `Some(v)` eşleşirse loop iterasyonu devam eder,
       - `None` ise loop kırılır.

---

## 5. Vex Core Prelude Önerisi

Prelude = her dosyada otomatik gelen, “dil kullanmak için %99 ihtiyacın olan” tipler ve trait’ler olmalı.

### 5.1. Prelude’de Olması Mantıklı Olanlar

- **Temel sum types:**
  - `Option<T>`
  - `Result<T, E>`

- **Temel koleksiyonlar:**
  - `Vec<T>`
  - `Box<T>` (heap sahibi smart pointer)
  - `String`, `&Str` (veya Vex’teki string ikilisi)

- **Map / Set alias’ları:**
  - `Map<K, V> = HashMap<K, V>`
  - `Set<T> = HashSet<T>`
  - Asıl implementasyon `std::collections` altında kalır;
    prelude’de sadece kısa alias’lar bulunur.

- **Temel trait / contract’lar:**
  - `Iterator`
  - `Context` (iptal / deadline / metadata için)
  - `Error` (hata tipi base contract)
  - `Clone`, `Copy`, `Eq`, `Ord`, `Hash` benzeri temel trait’ler

Bu sayede:

```vex
fn main(): i32 {
    let mut v: Vec<i32> = Vec::new();
    let mut m: Map<Str, i32> = Map::new();
    let mut s: Set<i32> = Set::new();
    return 0;
}
```

gibi kodlar **hiç ekstra import yazmadan** çalışabilir.

### 5.2. Prelude’de Olmaması Daha Sağlıklı Olanlar

Bunlar daha “advanced / concurrency / interior mutability” tarafı; prelude’e konmayıp modül altından çağırılmaları daha temiz:

- `Mutex`, `RwLock` → `std::sync`
- `Arc`             → `std::sync`
- `Rc`              → `std::rc`
- `Cell`, `RefCell` → `std::cell`

Önerilen modül yapısı:

```text
std/
  collections/  (HashMap, HashSet, BTreeMap, BTreeSet, ...)
  sync/         (Mutex, RwLock, Arc, Condvar, ...)
  rc/           (Rc, Weak, ...)
  cell/         (Cell, RefCell, ...)
```

Ve opsiyonel “prelude paketleri”:

```vex
import { sync_prelude } from "std"; // Mutex, Arc, RwLock otomatik
import { rc_prelude }   from "std"; // Rc, RefCell otomatik
```

Böylece:

- “Sade script / CLI” yazan kullanıcı sadece core prelude ile mutlu olur.
- “Concurrency / GUI / framework” tarafında çalışan kullanıcı 1 import ile geniş prelude açabilir.

---

## 6. Genel Özet

- **Constructor**: `op(...)` → `Foo(...)` sugar’ı, invariants / policy ile özelleştirilebilir, ham literal (`Foo { ... }`) her zaman mevcut.
- **Destructor**: `fn _free()!` özel imzası, RAII + `_free` + `_free` çağrısı **otomatik ve deterministik**, user manual çağramaz (veya çağırırsa value consumed sayılır).
- **Channel & Iterator**: `<-` operatörü, tamamen `op<-()!` / `op<-(T)!` overloading ile tanımlı; dil sadece `let pattern <- expr` ve `while let Some(v) <- expr` sugar’larını sağlar.
- **Prelude**: sade ama güçlü:
  - `Option`, `Result`, `Vec`, `Box`, `String`, `Map`, `Set`, `Iterator`, `Context`, temel trait’ler;
  - concurrency & interior mutability tipleri (`Mutex`, `Arc`, `Rc`, `Cell`, `RefCell`) modül altında kalır.

Bu mimari, Vex’e:
- Rust seviyesinde **zero-cost abstraction** ve RAII gücü,
- Go seviyesinde **ergonomi** (özellikle `new(...)`, channel ve context kullanımı),
- TypeScript/Go tarzı **okunabilir syntax** kazandırmayı hedefler.
