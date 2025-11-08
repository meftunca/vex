# Vex Const Keyword - Sadece Compile-Time Constants İçin

## 🎯 **Yeni Tasarım: Const Sadece Global Constants İçin**

Vex'te `const` keyword'ü **sadece compile-time constant global değişkenler** için kullanılacak. Diğer tüm immutability ihtiyaçları mevcut syntax ile karşılanacak.

## ✅ **Mevcut Immutability Sistemi (Zaten Yeterli)**

### 1. **Variables (Değişkenler)**

```vex
let x = 42;        // Immutable variable (varsayılan)
let! x = 42;       // Mutable variable (explicit)
```

### 2. **References (Referanslar)**

```vex
&T                 // Immutable reference
&T!                // Mutable reference
```

### 3. **Raw Pointers (Ham İşaretçiler)**

```vex
*T                 // Mutable raw pointer
*const T           // Const raw pointer (FFI için)
```

### 4. **Function Parameters (Fonksiyon Parametreleri)**

```vex
fn process(data: &u8) {    // Immutable parameter
fn modify(data: &u8!) {    // Mutable parameter
```

## 🚫 **Const Keyword Kullanılmayacak Yerler**

### ❌ **Variables'da Const**

```vex
// GEREKSİZ - let zaten immutable
const let x = 42;  // ❌ NO

// DOĞRU - let kullan
let x = 42;        // ✅ YES
```

### ❌ **Function Parameters'da Const**

```vex
// GEREKSİZ - type system zaten immutable garantisi veriyor
fn func(const param: i32) { }  // ❌ NO

// DOĞRU - reference immutability kullan
fn func(param: &i32) { }       // ✅ YES
```

### ❌ **Return Types'da Const**

```vex
// GEREKSİZ - return value semantics ile handle ediliyor
fn func(): const i32 { }  // ❌ NO

// DOĞRU - normal return type
fn func(): i32 { }        // ✅ YES
```

## ✅ **Const Sadece Global Compile-Time Constants İçin**

### **Syntax**

```vex
const NAME = value;           // Type inference
const NAME: Type = value;     // Explicit type
```

### **Örnekler**

```vex
// Global constants
const MAX_SIZE = 1000;
const PI: f64 = 3.14159;
const APP_NAME = "Vex Lang";
const DEBUG: bool = true;

// Arrays
const FIB_SEQUENCE = [0, 1, 1, 2, 3, 5, 8];

// Struct constants (gelecekte)
const DEFAULT_CONFIG = Config {
    host: "localhost",
    port: 8080,
};
```

### **Kısıtlamalar**

- **Compile-time evaluable** olmalı
- **Runtime functions** çağrılamaz
- **Global scope**'da olmalı
- **Immutable** (değiştirilemez)

## 🔧 **Implementation Detayları**

### **Parser (`vex-parser/src/parser/items/consts.rs`)**

```rust
pub(crate) fn parse_const(&mut self) -> Result<Item, ParseError> {
    self.consume(&Token::Const, "Expected 'const'")?;

    let name = self.consume_identifier()?;

    // Optional type annotation
    let ty = if self.match_token(&Token::Colon) {
        Some(self.parse_type()?)
    } else {
        None
    };

    self.consume(&Token::Eq, "Expected '=' after const name")?;
    let value = self.parse_expression()?;

    self.consume(&Token::Semicolon, "Expected ';' after const value")?;

    Ok(Item::Const(Const { name, ty, value }))
}
```

### **AST (`vex-ast/src/lib.rs`)**

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Const {
    pub name: String,
    pub ty: Option<Type>,    // Optional type annotation
    pub value: Expression,   // Must be compile-time evaluable
}
```

### **Codegen (TODO - henüz implement edilmemiş)**

```rust
// LLVM global constants olarak compile edilecek
// const MAX_SIZE = 1000; → @MAX_SIZE = constant i32 1000
```

## 📊 **Avantajlar**

### 1. **Minimal Syntax**

- Sadece 1 kullanım alanı: global constants
- Diğer immutability ihtiyaçları için ayrı syntax'lar yok

### 2. **Tutarlılık**

- `const` = compile-time constant
- `let` = runtime immutable variable
- `let!` = runtime mutable variable

### 3. **Kolay Öğrenme**

- Tek bir anlam: "bu değer compile-time'da hesaplanır"
- Semantic overload yok

### 4. **Diğer Dillerle Uyumluluk**

```vex
// C/C++
const int MAX_SIZE = 1000;

// Rust
const MAX_SIZE: i32 = 1000;

// Vex (yeni)
const MAX_SIZE = 1000;
```

## 🔄 **Migration (Eğer Eski Kod Varsa)**

### **Eski Const Kullanımları Çıkarılacak:**

```vex
// Eski (çıkarılacak)
const let x = 42;              // ❌
fn func(const param: i32) { }   // ❌

// Yeni (doğru)
let x = 42;                    // ✅
fn func(param: &i32) { }        // ✅
```

### **Sadece Global Constants Kalacak:**

```vex
const MAX_SIZE = 1000;         // ✅ (değişmez)
const PI = 3.14159;            // ✅ (değişmez)
```

## 🎯 **Sonuç**

**Const keyword'ü Vex'te sadece global compile-time constants için kullanılacak:**

- ✅ **Compile-time constants**: `const MAX_SIZE = 1000;`
- ✅ **Type inference**: `const PI = 3.14159;`
- ✅ **Optional type annotation**: `const PI: f64 = 3.14159;`

**Diğer tüm immutability ihtiyaçları mevcut syntax ile karşılanacak:**

- ✅ **Immutable variables**: `let x = 42;`
- ✅ **Mutable variables**: `let! x = 42;`
- ✅ **Immutable references**: `&T`
- ✅ **Mutable references**: `&T!`
- ✅ **Const pointers (FFI)**: `*const T`

Bu yaklaşım Vex'i **daha temiz, daha tutarlı** ve **daha öğrenmesi kolay** bir dil yapıyor! 🚀

---

**Tarih**: November 8, 2025
**Durum**: Tasarım Finalized
**Implementation**: Parser & AST hazır, Codegen TODO

# 🔄 **Güncellenmiş Tasarım: FFI Const Pointers da Otomatik**

## 🚫 **`*const T` Syntax'ı da Gereksiz - Otomatik Mapping**

Vex'te `*const T` syntax'ına bile gerek yok. Raw pointers otomatik olarak immutable/const semantics ile çalışacak.

### **Yeni Yaklaşım: Sadece `*T`**

```vex
*T        // Raw pointer (immutable by default - compiler const olarak handle eder)
*T!       // Mutable raw pointer (explicit ! ile)
```

### **FFI Otomatik Mapping**

```c
// C header
const char* strlen(const char* s);
char* strcpy(char* dest, const char* src);
```

```vex
// Vex import - compiler otomatik const mapping yapar
extern fn strlen(s: *u8): usize;      // *u8 → const char* (auto)
extern fn strcpy(dest: *u8!, src: *u8): *u8!;  // *u8! → char*, *u8 → const char*
```

### **Neden Bu Daha İyi?**

#### 1. **Syntax Simplicity**

```vex
// Eski (karmaşık)
*T                 // mutable raw pointer
*const T           // const raw pointer

// Yeni (basit)
*T                 // immutable raw pointer (default)
*T!                // mutable raw pointer (explicit)
```

#### 2. **Consistent with Vex Philosophy**

- `let` → immutable (default)
- `let!` → mutable (explicit)
- `*T` → immutable pointer (default)
- `*T!` → mutable pointer (explicit)

#### 3. **Automatic FFI Mapping**

Compiler C FFI declarations'ında otomatik olarak:

- `*T` → `const T*` (C'de)
- `*T!` → `T*` (C'de)

## 📝 **Güncellenmiş Raw Pointer Syntax**

### **Declaration**

```vex
let ptr: *i32 = get_raw_ptr();        // Immutable raw pointer
let! ptr: *i32! = get_raw_ptr();      // Mutable raw pointer
```

### **FFI Usage**

```vex
// C function: const char* get_version();
extern fn get_version(): *u8;         // Returns const pointer

// C function: void set_data(char* data);
extern fn set_data(data: *u8!);       // Takes mutable pointer

// C function: size_t strlen(const char* s);
extern fn strlen(s: *u8): usize;      // Takes const pointer (auto-mapped)
```

### **Pointer Operations**

```vex
let ptr: *i32 = get_ptr();            // Immutable
let value = *ptr;                     // Read OK
// *ptr = 42;                         // ❌ Error - immutable

let! ptr: *i32! = get_ptr();          // Mutable
let value = *ptr;                     // Read OK
*ptr = 42;                            // ✅ OK - mutable
```

## 🔧 **Implementation Changes**

### **Parser (`vex-parser/src/parser/types.rs`)**

```rust
// Eski - *const T parsing
if self.check(&Token::Star) {
    self.advance();
    let is_const = if self.check(&Token::Const) {
        self.advance();
        true
    } else {
        false
    };
    // ...
}

// Yeni - sadece *T ve *T! parsing
if self.check(&Token::Star) {
    self.advance();
    let is_mutable = self.match_token(&Token::Not);  // ! for mutable
    // is_const = !is_mutable (auto)
    // ...
}
```

### **AST (`vex-ast/src/lib.rs`)**

```rust
// Eski
RawPtr {
    inner: Box<Type>,
    is_const: bool,
},

// Yeni - sadece mutability flag
RawPtr {
    inner: Box<Type>,
    is_mutable: bool,  // false = const (default), true = mutable
},
```

### **Codegen (`vex-compiler/src/codegen_ast/types.rs`)**

```rust
Type::RawPtr { inner, is_mutable } => {
    let inner_ty = self.compile_type(inner)?;
    let ptr_ty = inner_ty.ptr_type(inkwell::AddressSpace::Generic);

    // FFI'da otomatik const mapping
    if in_ffi_context && !is_mutable {
        // LLVM const attribute ekle veya C const olarak export et
    }

    Ok(ptr_ty.into())
}
```

## ✅ **Avantajlar**

### 1. **Minimal Syntax**

- Sadece `*T` ve `*T!` - o kadar!
- `*const T` redundant syntax kalkıyor

### 2. **Automatic FFI**

- C `const char*` ↔ Vex `*u8` (auto mapping)
- C `char*` ↔ Vex `*u8!` (auto mapping)

### 3. **Consistent Immutability**

```vex
// Hepsi aynı pattern:
let x = 42;        // immutable
let! x = 42;       // mutable

&T                 // immutable ref
&T!                // mutable ref

*T                 // immutable ptr
*T!                // mutable ptr
```

### 4. **Less Cognitive Load**

- Developer sadece "mutable mı?" diye düşünür
- "const mı?" diye düşünmesine gerek yok

## 🔄 **Migration**

### **Eski Syntax (Çıkarılacak)**

```vex
*const T           // ❌
*T                 // ❌ (eski anlam)
```

### **Yeni Syntax**

```vex
*T                 // ✅ immutable raw pointer (default)
*T!                // ✅ mutable raw pointer (explicit)
```

## 🎯 **Final Result**

**Vex Raw Pointer Syntax:**

- `*T` → immutable (const) raw pointer
- `*T!` → mutable raw pointer

**FFI Auto-mapping:**

- Vex `*T` ↔ C `const T*`
- Vex `*T!` ↔ C `T*`

Bu yaklaşım Vex'i **ultra-minimal** yapıyor! 🚀
