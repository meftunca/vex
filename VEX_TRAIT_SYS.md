# Vex Trait ve Metot Sistemi (v1.4 - Resmi Spesifikasyon)

## 1. Felsefe: Satır-içi Güvenlik (Inline Safety)

Bu döküman, Vex'in `interface` (Go/TS-tarzı) ve `trait` (Rust-tarzı) felsefeleri arasındaki "kimlik krizini" çözer.

Vex, `interface` anahtar kelimesini _tamamen kaldırır_ ve **"Satır-içi Implementasyon"** (Inline Implementation) adı verilen, Vex'e özgü (unique) bir `trait` sistemini benimser.

Bu sistemin hedefleri:

1. **Güvenlik (Safety):** `struct` tanımına `impl Trait` bildirimini _zorunlu_ kılarak "kazara implementasyonu" (accidental implementation) engeller (Nominal Tipleme). (Standart 7)
2. **Sadelik (Simplicity):** Rust'ın ayrı `impl Trait for Struct { ... }` "kalıp kodunu" (boilerplate) kaldırır.
3. **Bütünlük (Cohesion):** Go/Rust'ın "Sorumlulukların Ayrılması" (SoC) felsefesinden _ödün vererek_, C++/Java/TypeScript `class`'larına benzer şekilde, veri (data) ve davranışı (behavior) _tek bir blokta_ toplar.
4. **Anlaşılırlık (Clarity):** Bir `struct`'ın hangi davranışları (`trait`'leri) desteklediği, `struct` tanımına bakılarak _anında_ anlaşılır olmalıdır. (Standart 3)

## 2. Trait Tanımlama (Trait Definition)

`trait`'ler (arayüzler), bir tipin (type) sahip olması gereken _davranışları_ (metot imzalarını) tanımlar.

- `interface` anahtar kelimesi Vex'te _tanımlı değildir_.
- `trait` blokları _sadece_ metot imzalarını (signatures) içerir.
- **Kritik:** `trait` imzaları, metodun "alıcı" (receiver) tipini (`self`) belirtmek için `Self` (büyük S ile) anahtar kelimesini _kullanmak zorundadır_. Bu, derleyicinin imzayı doğru eşleştirmesi (matching) için gereklidir.

```javascript
// 'vex_std_packages.md' (Bölüm 3.1) için temel 'io' trait'leri

// Yazma davranışı
export trait Writer {
    // Vex (v0.9) 'mutability' (değişebilirlik) felsefesine
    // uygun olarak '&Self!' (mutable borrow) kullanılır.
    fn (self: &Self!) write(data: &[byte]): (i32 | error);
}

// Okuma davranışı
export trait Reader {
    fn (self: &Self!) read(data: &[byte]!): (i32 | error);
}

// Jenerik (Generic) Trait
export trait From<T> {
    // 'Self' (dönüş tipi), 'From<T>' trait'ini
    // uygulayan tipin kendisidir.
    fn from(value: T): Self;
}

```

### 2.1. Trait'lerde "Default Metotlar"

Rust gibi, Vex `trait`'leri de varsayılan (default) implementasyonlar sağlayabilir.

```javascript
export trait Logger {
    // Gerekli (Required) Metot
    fn (self: &Self!) log(level: LogLevel, msg: string);

    // Varsayılan (Default) Metot
    // 'log' metodunu kullanarak 'info' için bir kısayol sağlar.
    fn (self: &Self!) info(msg: string) {
        self.log(LogLevel::Info, msg);
    }
}

```

### 2.2. Trait Kalıtımı (Trait Inheritance)

Bir `trait`, başka `trait`'leri "gerektirebilir".

```javascript
// 'ReadWriter' olabilmek için, önce 'Reader' VE 'Writer'
// olmalısın.
export trait ReadWriter: Reader, Writer {
    // Bu trait, ek metotlar tanımlayabilir
    fn flush(): error;
}

```

## 3. Struct ve Implementasyon (Sistem v1.4)

`struct` tanımı, `impl` anahtar kelimesiyle hangi `trait`'leri uygulayacağını _bildirir_ (declares) ve bu `trait`'lerin metotlarını _doğrudan_ `struct` bloğunun _içinde_ tanımlar.

### 3.1. Çoklu Trait Implementasyonu (Multiple Trait Implementation)

Vex, bir `struct`'ın aynı anda _birden fazla_ (multiple) `trait`'i implemente etmesini tam olarak destekler.

Bu, `impl` anahtar kelimesinden sonra `trait` isimlerinin **virgülle (comma)** ayrılmasıyla yapılır.

```javascript
// 1. Veri (Data) + Bildirim (Declaration) + Davranış (Behavior)
// 'File' struct'ının 'Reader' VE 'Writer' trait'lerini
// uygulayacağını (implemente edeceğini) bildiriyoruz.
export struct File impl Reader, Writer {
    // --- Alanlar (Fields) ---
    fd: i32,
    path: string,

    // --- Implementasyonlar (Struct bloğunun içinde) ---

    // 'Reader' trait'inin implementasyonu
    // 'self: &Self!' (trait'ten) -> 'self: &File!' (struct'a)
    fn (self: &File!) read(data: &[byte]!): (i32 | error) {
        // 'read' implementasyonu...
        // Derleyici, bu imzanın 'Reader' trait'i (bildirildiği için)
        // ile eşleştiğini DOĞRULAR (VERIFY).
        return 0;
    }

    // 'Writer' trait'inin implementasyonu
    fn (self: &File!) write(data: &[byte]): (i32 | error) {
        // 'write' implementasyonu...
        return 0;
    }

    // --- 'File'a özel (trait'te olmayan) metot ---
    fn (self: &File!) close(): error {
        // 'close' implementasyonu...
        // Bu metot 'Reader' veya 'Writer' trait'inde olmadığı için,
        // derleyici bunu sadece 'File'a ait özel bir metot olarak kabul eder.
        return nil;
    }
}

```

## 4. Jenerik (Generic) Trait Implementasyonu

`struct` tanımı, _belirli_ (concrete) tipler için jenerik `trait`'leri de implemente edebilir.

```javascript
// 1. 'Display' trait'i
trait Display {
    fn (self: &Self) to_string(): string;
}

// 2. 'i32' için 'Display' implementasyonu
// (Bu, 'i32' tipine 'metot ekleme' (method extension) yapar)
impl Display for i32 {
    fn (self: &i32) to_string(): string {
        // ... C FFI (itoa) kullanarak implementasyon ...
    }
}

// 3. Jenerik 'From<T>' trait'i (Bölüm 2)
// 'i32'den 'string'e dönüşümü implemente et
impl From<i32> for string {
    fn from(value: i32): string {
        return value.to_string(); // 'Display' implementasyonunu kullanır
    }
}

// 4. Jenerik 'struct' üzerinde jenerik 'trait'
struct MyBox<T> {
    value: T,
}

// 'Display' trait'ini, 'MyBox<T>' için UYGULAMA,
// *sadece* T'nin kendisi 'Display' ise.
impl<T: Display> Display for MyBox<T> {
    fn (self: &MyBox<T>) to_string(): string {
        return f"[Box: {self.value.to_string()}]";
    }
}

```

_Not: Vex'in&#32;`impl Trait for Type`&#32;sözdiziminin (Rust gibi),&#32;`struct ... impl Trait`&#32;(C++/Java gibi) sözdizimiyle nasıl birleşeceği, v1.4'ün en karmaşık felsefi noktasıdır. Bu döküman, ikincisini (Satır-içi) ana model olarak kabul eder._

## 5. Trait Kullanımı (Statik vs. Dinamik)

`trait`'ler iki şekilde kullanılır:

### 5.1. Statik Dağıtım (Static Dispatch) - Jenerikler

Bu, Vex'in "sıfır maliyetli soyutlama" (zero-cost abstraction) ve Rust-seviyesi performans elde etme yoludur.

```javascript
// Derleyici, 'T'nin 'Reader' trait'ini uyguladığını BİLİR.
// Çağrı, derleme zamanında 'File::read'e dönüştürülür.
fn read_all<T: Reader>(r: &T!): (string | error) {
    let! buf = make([byte], 1024);
    try r.read(buf!);
    // ...
}

// Kullanım
let! f = File{...};
read_all(&f!); // Performanslı (sıfır maliyet)

```

### 5.2. Dinamik Dağıtım (Dynamic Dispatch) - "dyn"

Bir listede _farklı_ tipleri (ama _aynı_ davranışı) bir arada tutmak için "dinamik" (çalışma zamanı) `trait` nesneleri kullanılır.

```javascript
// 'dyn' (dynamic) anahtar kelimesi, bunun bir 'trait object'
// olduğunu belirtir.
fn log_data(w: &dyn Writer!) {
    // Çağrının 'File::write' mi yoksa 'Socket::write' mi
    // olduğu çalışma zamanında (runtime) çözülür.
    // Bu, küçük bir 'vtable' maliyeti (overhead) ekler.
    w.write(f"Log time: {time::now()}".to_bytes());
}

// Kullanım
let! file_logger = File{...};
let! socket_logger = Socket{...}; // 'Socket' de 'Writer' uygular

log_data(&file_logger!);
log_data(&socket_logger!);

```

## 6. Derleyici Davranışı ve Güvenlik (Resmi Kurallar)

Derleyici, `struct File impl Reader` bildirimini gördüğünde aşağıdaki kuralları _zorunlu_ (enforce) kılar (Standart 3, 5, 7):

1. **Güvenlik (Safety ✅):** `impl` bildirimi zorunludur. `File`'ı `Writer` bekleyen bir fonksiyona gönderebilirsiniz:fn log_data(w: &Writer!) { /\* ... \*/ }

let! f = File{...};
log_data(&f!); // GEÇERLİ (VALID)

2. **Hata Önleme (Error Prevention ✅):** Eğer `struct File impl Reader` yazıp, `read` metodunu `struct` bloğunun _içine_ eklemeyi _unutursanız_ (veya yanlış tanımlarsanız), derleyici hata verir:// HATA: 'File' struct'ı 'Reader' trait'ini
   // implemente edeceğini bildirdi, ancak
   // 'fn read(self: &Self!, data: &[byte]!): (i32 | error)' metodu
   // 'File' bloğu içinde bulunamadı veya imzası eşleşmiyor!

3. **Ergonomi (Ergonomics ✅):** Rust'ın ayrı `impl Writer for File { ... }` "kalıp kodu" (boilerplate) _yoktur_. Tüm mantık (veri ve davranış) tek bir `struct` bloğu içinde toplanmıştır (C++/Java/TS `class`'larına benzer).

## 7. Özet: Vex (v1.4) Modeli

| Özellik            | Rust (Nominal)                              | Go (Structural)            | Vex v1.4 (Hibrit)                                              |
| ------------------ | ------------------------------------------- | -------------------------- | -------------------------------------------------------------- |
| **Arayüz Tanımı**  | `trait`                                     | `interface`                | `trait`                                                        |
| **Implementasyon** | `impl Trait for Struct { ... }` (Ayrı Blok) | Otomatik (Implicit)        | `struct Struct impl Trait1, Trait2 { ... fn ... }` (Satır-içi) |
| **Güvenlik**       | ✅ Yüksek (Açık)                            | ⚠️ Düşük (Kazara olabilir) | ✅ Yüksek (Açık Bildirim)                                      |
| **Ergonomi**       | ⚠️ Düşük (Kalıp kod)                        | ✅ Yüksek (Kalıp kod yok)  | ✅ Yüksek (Kalıp kod yok)                                      |
| **Veri/Davranış**  | Ayrı (Separate)                             | Ayrı (Separate)            | **Bütünleşik (Cohesive)**                                      |
| **Kalıtım**        | ✅ (`trait A: B`)                           | ❌ (Yok)                   | ✅ (`trait A: B`)                                              |
| **Dispatch**       | Statik (`T: Trait`) & Dinamik (`dyn Trait`) | Sadece Dinamik             | Statik (`T: Trait`) & Dinamik (`dyn Trait`)                    |

| |
