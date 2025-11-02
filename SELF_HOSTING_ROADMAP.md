# Vex Self-Hosting (Bootstrapping) Roadmap

## Hedef: Vex compiler'ı Vex ile yazmak (vexc.vx)

## Şu Anki Durum:

```
Vex Source Code (examples/*.vx)
    ↓
Rust Compiler (vex-compiler/)
    ↓
LLVM IR
    ↓
Binary Executable
```

## Hedef Durum (Self-hosted):

```
Vex Compiler Source (vexc.vx)
    ↓
Vex Compiler (written in Vex)
    ↓
LLVM IR
    ↓
New Vex Compiler Binary
```

---

## Faz 1: Temel Dil Özellikleri (Şu Anda ~%70 Tamamlandı)

- [x] Basic types, functions, control flow
- [x] Structs, enums, tuples
- [x] Generics
- [x] Pattern matching
- [x] Trait system
- [x] Module system
- [ ] **Eksikler:**
  - [ ] Closure/Lambda support
  - [ ] Advanced memory management (ownership/borrow)
  - [ ] Macro system
  - [ ] Compile-time execution (const eval)

## Faz 2: Compiler Infrastructure (Vex ile yazılması gereken)

### 2.1. Lexer/Parser

```vex
// vexc/lexer.vx
struct Token {
    type: TokenType,
    value: string,
    position: Position,
}

fn tokenize(source: string) : Vec<Token> {
    // Lexical analysis
}
```

### 2.2. AST

```vex
// vexc/ast.vx
enum Expr {
    Binary(BinaryExpr),
    Call(CallExpr),
    Literal(LiteralExpr),
}

enum Stmt {
    Let(LetStmt),
    Return(ReturnStmt),
    Expr(ExprStmt),
}
```

### 2.3. Type Checker

```vex
// vexc/typeck.vx
fn infer_type(expr: &Expr, ctx: &Context) : (Type | TypeError) {
    // Type inference
}
```

### 2.4. LLVM Codegen

```vex
// vexc/codegen.vx
// Bu kısım FFI gerektirir - LLVM C API'ye bağlanmalı

import "llvm/ffi" as llvm;

fn compile_function(func: &Function) : Result<(), Error> {
    builder := llvm.create_builder();
    // LLVM IR generation
}
```

## Faz 3: Gerekli Sistem Özellikleri

### 3.1. FFI Support (C interop)

```vex
// C fonksiyonlarını çağırabilme
extern "C" fn malloc(size: usize) : *mut u8;
extern "C" fn printf(format: *const i8, ...);

// Callback support
type Callback = fn(i32) : i32;
```

### 3.2. Raw Pointers & Unsafe

```vex
unsafe {
    let ptr = malloc(1024);
    *ptr = 42;
    free(ptr);
}
```

### 3.3. Build System

```vex
// build.vx - Vex build script
fn main() {
    config := BuildConfig {
        target: "x86_64-unknown-linux-gnu",
        opt_level: OptLevel.Release,
    };

    compile("src/main.vx", config)?;
}
```

## Faz 4: Standard Library (Vex'te yazılmış)

### 4.1. Core Collections

```vex
// std/collections/vec.vx
struct Vec<T> {
    data: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> Vec<T> {
    fn new() : Vec<T> { ... }
    fn push(mut self, item: T) { ... }
    fn pop(mut self) : Option<T> { ... }
}
```

### 4.2. String

```vex
// std/string.vx
struct String {
    bytes: Vec<u8>,
}

impl String {
    fn from_str(s: &str) : String { ... }
    fn push_str(mut self, s: &str) { ... }
}
```

### 4.3. I/O

```vex
// std/io.vx
trait Read {
    fn read(mut self, buf: &mut [u8]) : (usize | Error);
}

trait Write {
    fn write(mut self, buf: &[u8]) : (usize | Error);
}
```

## Faz 5: Bootstrapping Process

### Adım 1: Rust Compiler ile Vex Compiler v1 yaz (şu anki durum)

```
vex-compiler/ (Rust) → vexc v1.0 (binary)
```

### Adım 2: Vex Compiler v2'yi Vex ile yaz

```vex
// vexc/main.vx
fn main() {
    args := std.env.args();
    source := std.fs.read_to_string(args[1])?;

    tokens := lexer.tokenize(source);
    ast := parser.parse(tokens)?;
    typed_ast := typeck.check(ast)?;

    codegen.compile(typed_ast, "output.o")?;
}
```

### Adım 3: Rust Compiler ile Vex Compiler v2'yi compile et

```bash
vexc v1.0 (Rust-based) + vexc/main.vx → vexc v2.0 (Vex-based)
```

### Adım 4: Vex Compiler v2 ile kendini compile et (self-hosting!)

```bash
vexc v2.0 + vexc/main.vx → vexc v2.0 (verified)
```

## Zaman Çizelgesi (Tahmini)

| Faz        | Süre         | Açıklama                       |
| ---------- | ------------ | ------------------------------ |
| Faz 1      | 2-3 ay       | Closure, macro, ownership ekle |
| Faz 2      | 6-8 ay       | Vex'te lexer/parser/typeck yaz |
| Faz 3      | 3-4 ay       | FFI, unsafe, build system      |
| Faz 4      | 4-6 ay       | Std library (collections, I/O) |
| Faz 5      | 2-3 ay       | Bootstrap ve test              |
| **TOPLAM** | **17-24 ay** | ~2 yıl                         |

## Kritik Bağımlılıklar (Rust'tan kopuş için)

1. **LLVM C API** - Vex FFI ile kullanılacak
2. **Memory allocator** - jemalloc/mimalloc gibi
3. **Platform syscalls** - libc veya direkt syscall
4. **Build toolchain** - linker (lld), ar, etc.

## Öncelikli Eksikler (Self-hosting için şart):

1. ❌ Closure/Lambda (compiler'da yoğun kullanılıyor)
2. ❌ Macro system (kod generation için)
3. ❌ Compile-time execution (performance için)
4. ❌ Advanced error handling (Result/Option tam desteği)
5. ❌ File I/O (source code okuma için)
6. ❌ Memory management (allocation/deallocation)
7. ❌ FFI system (LLVM API için)

## Sonuç:

**Vex şu anda %30-40 self-hosting'e hazır.**
Eksik özellikler tamamlanırsa 2 yıl içinde tamamen self-hosted olabilir.
