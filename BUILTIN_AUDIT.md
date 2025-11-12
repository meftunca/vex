# Vex Builtin Functions Audit

## Kategoriler ve Mimari Düzenleme Planı

### ✅ Gerçek Builtin (Compiler'da kalmalı)

#### Core I/O ve Runtime
- `print()`, `println()` - Console output (LLVM direct)
- `panic()` - Runtime panic with stack trace
- `assert()` - Debug assertion
- `unreachable()` - Unreachable code marker

#### Memory Management (Low-level LLVM)
- `alloc()`, `free()`, `realloc()` - Raw memory allocation
- `sizeof()`, `alignof()` - Type introspection

#### Memory Operations (LLVM intrinsics)
- `memcpy()`, `memset()`, `memcmp()`, `memmove()` - Direct LLVM mapping
- Bu fonksiyonlar LLVM intrinsic'leri, compiler'da kalmalı

#### Compiler Hints (LLVM intrinsics)
- `assume()` - Optimizer'a hint
- `likely()`, `unlikely()` - Branch prediction hints
- `prefetch()` - Cache prefetch hint

#### Bit Manipulation (LLVM intrinsics)
- `ctlz()`, `cttz()`, `ctpop()` - Count leading/trailing zeros, popcount
- `bswap()`, `bitreverse()` - Byte swap, bit reverse
- `sadd_overflow()`, `ssub_overflow()`, `smul_overflow()` - Overflow detection

#### Reflection (Compile-time)
- `typeof()`, `type_size()`, `type_align()` - Type metadata
- `is_int_type()`, `is_float_type()`, `is_pointer_type()` - Type checks
- `type_id()`, `field_metadata()` - Advanced reflection

---

### 🔄 Stdlib'e Taşınmalı (Vex koduyla yazılabilir)

#### Collections (Vec, HashMap, Set)

**Vec:**
- ❌ **C Runtime'da kalmalı**: `vex_vec_new()`, `vex_vec_push()`, `vex_vec_get()`, `vex_vec_len()`, `vex_vec_free()`
- ✅ **Stdlib'e taşınmalı**: 
  ```vex
  // stdlib/collections/vec.vx
  struct Vec<T> impl Iterator {
      _internal_ptr: ptr  // C runtime pointer
      
      type Item = T
      
      fn new(): Vec<T> { ... }
      fn push( value: T)! { ... }
      fn len(): i64 { ... }
      fn get( index: i64): T { ... }
      fn iter(): VecIterator<T> { ... }
      fn next()!: Option<T> { ... }
  }
  ```

**HashMap:**
- ❌ **C Runtime**: `vex_map_new()`, `vex_map_insert()`, `vex_map_get()`, etc.
- ✅ **Stdlib wrapper**: HashMap<K,V> struct + methods

**Set:**
- ❌ **C Runtime**: `vex_set_new()`, `vex_set_insert()`, etc.
- ✅ **Stdlib wrapper**: Set<T> struct + methods

#### Smart Pointers

**Box:**
- ❌ **C Runtime**: `vex_box_new()`, `vex_box_free()`
- ✅ **Stdlib wrapper**: 
  ```vex
  // stdlib/box.vx
  struct Box<T> impl Drop {
      _ptr: ptr
      
      fn new(value: T): Box<T> { ... }
      fn get(): T { ... }
      fn drop()! { vex_box_free(self._ptr) }
  }
  ```

#### String

**Mevcut Durum:**
- ❌ **Compiler builtin**: `builtin_strlen()`, `builtin_strcmp()`, `builtin_strcpy()`, etc.
- ❌ **C Runtime**: `vex_string_new()`, `vex_string_from()`, etc.

**Yeni Mimari:**
- ✅ **Stdlib String type**:
  ```vex
  // stdlib/string.vx
  struct String impl Display {
      _ptr: ptr  // C runtime pointer
      
      fn new(): String { ... }
      fn from(s: str): String { ... }
      fn len(): i64 { ... }
      fn as_cstr(): ptr { ... }
      fn concat( other: String): String { ... }
      fn fmt(): String { ... }
  }
  ```

#### Slice (View Type)

**Mevcut:**
- ❌ **Compiler builtin**: `builtin_slice_new()`, `builtin_slice_from_vec()`, etc.

**Yeni:**
- ✅ **Stdlib**:
  ```vex
  // stdlib/slice.vx
  struct Slice<T> {
      data: ptr
      len: i64
      
      fn new(data: ptr, len: i64): Slice<T> { ... }
      fn from_vec(v: Vec<T>): Slice<T> { ... }
      fn len(): i64 { ... }
      fn get( index: i64): T { ... }
  }
  ```

#### Option/Result (Enum Types)

**Mevcut:**
- ❌ **Compiler builtin**: `builtin_option_some()`, `builtin_option_none()`, `builtin_result_ok()`, `builtin_result_err()`

**Yeni:**
- ✅ **Stdlib enum**:
  ```vex
  // stdlib/option.vx
  enum Option<T> {
      Some(T)
      None
      
      fn is_some(): bool { ... }
      fn is_none(): bool { ... }
      fn unwrap(): T { ... }
      fn unwrap_or( default: T): T { ... }
  }
  
  // stdlib/result.vx
  enum Result<T, E> {
      Ok(T)
      Err(E)
      
      fn is_ok(): bool { ... }
      fn is_err(): bool { ... }
      fn unwrap(): T { ... }
  }
  ```

#### Type Conversions

**Mevcut:**
- ❌ **Compiler builtin**: `builtin_vex_i32_to_string()`, `builtin_vex_i64_to_string()`, etc.

**Yeni:**
- ✅ **Stdlib trait**:
  ```vex
  // stdlib/convert.vx
  trait Display {
      fn fmt(): String
  }
  
  // Primitive types implement Display inline
  // i32, i64, f32, f64, bool implement Display via compiler builtin
  ```

#### UTF-8 Operations

**Mevcut:**
- ❌ **Compiler builtin**: `builtin_utf8_valid()`, `builtin_utf8_char_count()`, `builtin_utf8_char_at()`

**Yeni:**
- ✅ **String methodları**:
  ```vex
  struct String impl Display {
      _ptr: ptr
      
      fn is_valid_utf8(): bool { ... }
      fn char_count(): i64 { ... }
      fn char_at( index: i64): char { ... }
  }
  ```

#### Array Operations

**Mevcut:**
- ❌ **Compiler builtin**: `builtin_array_len()`, `builtin_array_get()`, `builtin_array_set()`, `builtin_array_append()`

**Yeni:**
- ✅ **Syntax sugar** (compiler'da array için özel handling):
  ```vex
  let arr: [i32; 5] = [1, 2, 3, 4, 5];
  let len = arr.len();     // Compiler knows array size at compile-time
  let elem = arr[2];       // Index operator
  arr[3] = 100;           // Index assignment
  ```

#### Channel (Concurrency)

**Mevcut:**
- ❌ **Compiler builtin**: `builtin_channel_new()`

**Yeni:**
- ❌ **C Runtime kalmalı**: Async runtime dependency
- ✅ **Stdlib wrapper**:
  ```vex
  // stdlib/sync/channel.vx
  struct Channel<T> {
      _ptr: ptr
      
      fn new(): Channel<T> { ... }
      async fn send( value: T)! { ... }
      async fn recv()!: T { ... }
  }
  ```

---

## Mimari Öneri

### 1. Compiler Builtins (Minimal Set)
```
vex-compiler/src/codegen_ast/builtins/
├── core.rs          # print, println, panic, assert, unreachable
├── memory.rs        # alloc, free, realloc, sizeof, alignof
├── memory_ops.rs    # memcpy, memset, memcmp, memmove
├── intrinsics.rs    # LLVM bit ops (ctlz, bswap, overflow checks)
├── hints.rs         # assume, likely, unlikely, prefetch
└── reflection.rs    # typeof, type_id, field_metadata
```

**Toplam: ~30 builtin function** (şu anda ~88)

### 2. C Runtime (FFI Layer)
```
vex-runtime/c/
├── vex.h            # Main header
├── vex_vec.c        # Vec low-level ops
├── vex_box.c        # Box allocation
├── vex_string.c     # String heap ops
├── vex_hashmap.c    # HashMap (SwissTable)
├── vex_set.c        # Set wrapper
├── vex_channel.c    # Async channel
└── vex_slice.c      # Slice operations
```

**Bu katman değişmez** - FFI boundary, performance-critical

### 3. Vex Stdlib (High-level Wrappers)
```
stdlib/
├── prelude.vx       # Auto-imported types (Option, Result, Vec, String)
├── collections/
│   ├── vec.vx       # Vec<T> wrapper + Iterator
│   ├── hashmap.vx   # HashMap<K,V> wrapper
│   └── set.vx       # Set<T> wrapper
├── string.vx        # String methods + Display
├── option.vx        # Option<T> enum + methods
├── result.vx        # Result<T,E> enum + methods
├── convert.vx       # Display, ToString, From, Into traits
├── iter.vx          # Iterator trait
├── box.vx           # Box<T> smart pointer
├── slice.vx         # Slice<T> view type
└── sync/
    └── channel.vx   # Channel<T> async wrapper
```

---

## Aksiyon Planı

### Faz 1: Trait System Tamamlama (Gerekli)
- [ ] Iterator trait implementation
- [ ] Display trait implementation
- [ ] Drop trait implementation
- [ ] Associated type support (type Item)

### Faz 2: Stdlib Foundation
- [ ] `stdlib/prelude.vx` oluştur
- [ ] `stdlib/iter.vx` - Iterator trait tanımla
- [ ] `stdlib/convert.vx` - Display, ToString traits

### Faz 3: Core Types Migration
- [ ] `stdlib/option.vx` - Option enum + methods
- [ ] `stdlib/result.vx` - Result enum + methods
- [ ] `stdlib/string.vx` - String wrapper + Display impl

### Faz 4: Collections Migration
- [ ] `stdlib/collections/vec.vx` - Vec wrapper + Iterator impl
- [ ] `stdlib/collections/hashmap.vx` - HashMap wrapper
- [ ] `stdlib/collections/set.vx` - Set wrapper
- [ ] `stdlib/slice.vx` - Slice wrapper

### Faz 5: Smart Pointers
- [ ] `stdlib/box.vx` - Box wrapper + Drop impl

### Faz 6: Compiler Cleanup
- [ ] Builtin registry'den stdlib'e taşınan fonksiyonları kaldır
- [ ] Geriye sadece ~30 gerçek builtin kalmalı
- [ ] Method call resolution stdlib'i otomatik import etmeli

---

## Örnek: Vec Migration

### Şu Anki Durum
```rust
// Compiler builtin
pub fn builtin_vec_new<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String> {
    // LLVM codegen directly calls vex_vec_new()
}
```

### Hedef Durum
```vex
// stdlib/collections/vec.vx
struct Vec<T> impl Iterator, Drop {
    _ptr: ptr  // Opaque C runtime pointer
    
    type Item = T
    
    fn new(): Vec<T> {
        return Vec { _ptr: vex_vec_new(sizeof(T)) }
    }
    
    fn push( value: T)! {
        vex_vec_push(self._ptr, &value)
    }
    
    fn len(): i64 {
        return vex_vec_len(self._ptr)
    }
    
    fn get( index: i64): T {
        let ptr = vex_vec_get(self._ptr, index)
        return *ptr as T
    }
    
    fn next()!: Option<T> {
        // Iterator implementation
    }
    
    fn drop()! {
        vex_vec_free(self._ptr)
    }
}
```

### FFI Declarations (Compiler'da)
```rust
// vex-compiler: Sadece FFI declaration
declare_runtime_fn("vex_vec_new", [i64], ptr);
declare_runtime_fn("vex_vec_push", [ptr, ptr], void);
declare_runtime_fn("vex_vec_get", [ptr, i64], ptr);
declare_runtime_fn("vex_vec_len", [ptr], i64);
declare_runtime_fn("vex_vec_free", [ptr], void);
```

---

## Avantajlar

1. **Daha Temiz Compiler**: Compiler sadece language primitives ile ilgilenir
2. **Daha Kolay Bakım**: Stdlib Vex koduyla yazılır, anlaşılır
3. **Kullanıcı Extensibility**: Kullanıcılar kendi collection'larını yazabilir
4. **Rust-like**: Rust'ın stdlib yapısına benzer, familiar
5. **Trait System Kullanımı**: Iterator, Display gibi trait'ler pratik yapılır
6. **Type Safety**: Stdlib generic types kullanır

## Dezavantajlar ve Çözümler

1. **Bootstrap Problem**: Stdlib compiler olmadan compile edilemez
   - **Çözüm**: Compiler stdlib'i pre-compile edip embed eder
   
2. **Performance**: Stdlib overhead ekleyebilir
   - **Çözüm**: Inline optimization, stdlib fonksiyonları inline olur
   
3. **Circular Dependency**: Stdlib compiler'a, compiler stdlib'e bağımlı
   - **Çözüm**: Compiler stdlib olmadan da çalışabilir (bare-metal mode)

---

## Sonuç

**Hedef**: Builtin sayısını 88'den ~30'a düşürmek, geri kalanını Vex stdlib'de implement etmek.

**Zaman Çizelgesi**:
- Faz 1-2: 1 hafta (Trait system)
- Faz 3-4: 2 hafta (Core types + Collections)
- Faz 5-6: 1 hafta (Smart pointers + Cleanup)

**Toplam**: ~4 hafta tam migration
