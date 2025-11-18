# Vex Primitive Type Upcasting Support

## ✅ Desteklenen Upcast Türleri

### 1. **Let Statement'larda Implicit Upcast**

```vex
let a: i8 = 10;
let b: i32 = a;  // ✅ i8 -> i32 (sign extension)

let c: i32 = 100;
let d: i64 = c;  // ✅ i32 -> i64 (sign extension)

let e: f32 = 3.14;
let f: f64 = e;  // ✅ f32 -> f64 (float extension)
```

**Lokasyon:** `vex-compiler/src/codegen_ast/statements/let_statement/variable_registration.rs`

- `cast_integer_if_needed()` (satır 83-117)
- `cast_float_if_needed()` (satır 121-137)

**Nasıl Çalışır:**

- Target type annotation varsa (`let x: i64 = ...`)
- Değer daha küçük width'te compile edilmişse
- Otomatik olarak sign/zero extension veya float extension yapılır

**Integer Upcast Stratejisi:**

- Signed types (i8, i16, i32, i64): **Sign extension** (`build_int_s_extend`)
- Unsigned types (u8, u16, u32, u64): **Zero extension** (`build_int_z_extend`)

---

### 2. **Function Call'larda Implicit Upcast**

```vex
fn takes_i64(x: i64) {
    print("Received i64\n");
}

let small: i32 = 42;
takes_i64(small);  // ✅ i32 -> i64 (automatic casting)
```

**Lokasyon:** `vex-compiler/src/codegen_ast/expressions/calls/function_calls.rs`

- Satır 183-221

**Nasıl Çalışır:**

- Function parameter type biliniyorsa
- Argument compile edildikten sonra
- Parameter type'a match etmek için otomatik cast yapılır

**ÖNEMLİ NOT:** Overloaded fonksiyonlar için casting devre dışı (overload resolution doğru variant'ı seçsin diye).

---

### 3. **Explicit Cast (as operatörü)**

```vex
let a: i32 = 100;
let b: i64 = a as i64;  // ✅ Explicit cast

let c: f64 = 3.14;
let d: i32 = c as i32;  // ✅ Float -> Int (truncates)

let e: i64 = 1000;
let f: i32 = e as i32;  // ✅ Downcast (truncates)
```

**Lokasyon:** `vex-compiler/src/codegen_ast/expressions/special/casts.rs`

- Satır 1-150

**Desteklenen Cast Türleri:**

- **Int -> Int:** Widening (extension) veya Narrowing (truncation)
- **Float -> Float:** f32 ↔ f64
- **Int -> Float:** Signed/unsigned aware conversion
- **Float -> Int:** Truncation (kesme, yuvarlama yok)
- **Pointer casts:** `*T -> *U`
- **Int -> Pointer:** Null pointer için (`0 as *u8`)

---

## 🔄 Downcast (Daraltma)

Downcast de destekleniyor ama **veri kaybı** olabilir:

```vex
let big: i64 = 1000000000000;
let small: i32 = big as i32;  // ⚠️ Truncates - veri kaybı!

let precise: f64 = 3.14159265;
let rough: f32 = precise as f32;  // ⚠️ Precision loss
```

**LLVM İnstructions:**

- `build_int_truncate` - Integer daraltma
- `build_float_trunc` - Float daraltma

---

## ⚙️ Type Casting Implementasyonu

### Integer Casting Logic

```rust
if current_width < target_width {
    // UPCAST: Widening
    if is_unsigned {
        builder.build_int_z_extend(val, target, "zext")
    } else {
        builder.build_int_s_extend(val, target, "sext")
    }
} else if current_width > target_width {
    // DOWNCAST: Narrowing
    builder.build_int_truncate(val, target, "trunc")
} else {
    // Same width (i32 -> u32): Bitcast (no operation)
    val
}
```

### Float Casting Logic

```rust
if source == f32 && target == f64 {
    builder.build_float_ext(val, f64, "fext")
} else if source == f64 && target == f32 {
    builder.build_float_trunc(val, f32, "ftrunc")
}
```

---

## 📋 Özet

| Kaynak | Hedef | Yöntem         | Otomatik?        | Veri Kaybı?        |
| ------ | ----- | -------------- | ---------------- | ------------------ |
| i8     | i32   | Sign extend    | ✅ Yes           | ❌ No              |
| i32    | i64   | Sign extend    | ✅ Yes           | ❌ No              |
| u8     | u32   | Zero extend    | ✅ Yes           | ❌ No              |
| i64    | i32   | Truncate       | ⚠️ Explicit only | ⚠️ Yes             |
| f32    | f64   | Float extend   | ✅ Yes           | ❌ No              |
| f64    | f32   | Float truncate | ⚠️ Explicit only | ⚠️ Yes (precision) |
| i32    | f64   | Int to float   | ✅ Yes           | ⚠️ Minimal         |
| f64    | i32   | Float to int   | ⚠️ Explicit only | ⚠️ Yes (truncates) |

**✅ Otomatik Upcast:** Safe conversions (widening)
**⚠️ Manual Cast Gerekir:** Lossy conversions (narrowing, precision loss)

---

## 🎯 Sonuç

**EVET**, Vex'te primitive tipler için **kapsamlı upcast desteği** var:

1. ✅ Let statements'ta implicit upcast
2. ✅ Function parameters'ta implicit upcast
3. ✅ Explicit cast operatörü (`as`)
4. ✅ Signed/unsigned aware extension
5. ✅ Float precision handling

**Güvenlik:** Widening (upcast) otomatik, narrowing (downcast) explicit gerektirir.
