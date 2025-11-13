# RUST_NOTICE.md

Bu dosya, Vex Lang projesindeki Rust kodlarındaki önemli TODO, CRITICAL ve benzeri mesajları toplar. Her mesaj için dosya, satır bilgisi ve neden girildiğine dair açıklama eklenmiştir.

## CRITICAL FIXES

### File: vex-compiler/src/codegen_ast/expressions/literals/structs_tuples.rs:59

**Message:** CRITICAL FIX: If field value is a StructLiteral with empty type_args but field_ty is Generic, inject type_args recursively

**Reason:** Generic struct'ların doğru type arguments ile instantiate edilmesi için gerekli. Eğer bir field value StructLiteral ise ve type_args boşsa ama field_ty Generic ise, expected type'dan type_args'ı recursive olarak derive edip inject eder. Bu, type system'ın tutarlılığı için kritik.

### File: vex-compiler/src/codegen_ast/expressions/literals/structs_tuples.rs:103

**Message:** ⭐ CRITICAL: Cast integer literals to match field type width

**Reason:** Struct field'larında integer literals'ların field type'ının bit width'ine cast edilmesi gerekiyor. Farklı bit width'lerde (örneğin i32'den i64'e) otomatik cast yapılmazsa type mismatch hataları oluşur. Bu, compile-time type safety için kritik.

### File: vex-compiler/src/codegen_ast/expressions/literals/structs_tuples.rs:142

**Message:** CRITICAL FIX: If field is a struct type and casted_field_val is a pointer, we need to load the struct value (memcpy) instead of storing the pointer

**Reason:** Eğer field bir struct type ise ve compiled value bir pointer ise, pointer yerine actual struct value'yu load etmek gerekir. Bu, memory management ve struct semantics'in doğru çalışması için gerekli. Pointer store etmek yerine memcpy ile value store edilir.

### File: vex-compiler/src/codegen_ast/types/conversion.rs:100

**Message:** ⚠️ CRITICAL FIX: "str" is a special type alias for String

**Reason:** Parser, `: str` syntax'ını Type::Named("str") olarak parse eder, ancak LLVM'da "str" bir pointer type olmalıdır (String için). Bu fix olmadan string handling hatalı olur.

### File: vex-compiler/src/codegen_ast/types/conversion.rs:121

**Message:** ⚠️ CRITICAL FIX: Return the struct TYPE, not a pointer!

**Reason:** Functions struct return ederken by value return etmelidir, pointer değil. Variables ise pointer type kullanır. Bu ayrım, proper struct return semantics için kritik. Yanlış olursa function returns hatalı olur.

### File: vex-compiler/src/codegen_ast/expressions/calls/builtins/mod.rs:78

**Message:** ⚠️ CRITICAL: Check if this is a user-defined struct vs builtin type

**Reason:** User-defined structs, aynı isimdeki builtin types'lardan precedence alır. Bu check olmadan name shadowing hatalı handle edilir ve builtin type methods yanlış çağrılır.

## TODO ITEMS

### File: vex-compiler/src/codegen_ast/expressions/calls/builtins/ranges_arrays.rs:164

**Message:** TODO: type-specific loading

**Reason:** Şu anda array element loading için i32 assume ediliyor. Farklı type'lar için type-specific loading implement edilmemiş, bu eksiklik runtime hatalarına yol açabilir.

### File: vex-compiler/src/codegen_ast/expressions/calls/builtins/ranges_arrays.rs:284

**Message:** TODO: proper Option return

**Reason:** Array access şu anda element'i directly return ediyor, ancak proper Option<T> return implement edilmemiş. Bu, bounds checking ve None return için gerekli.

### File: vex-compiler/src/codegen_ast/expressions/mod.rs:267

**Message:** TODO: poll

**Reason:** Async expressions için poll mechanism implement edilmemiş, assume always ready. Gerçek async semantics için poll implementasyonu gerekiyor.

### File: vex-compiler/src/codegen_ast/expressions/mod.rs:404

**Message:** TODO: Infer from Result<T, E>

**Reason:** Result type'ından data_type infer edilmemiş, hardcoded i32 kullanılıyor. Generic Result handling için type inference gerekli.

### File: vex-compiler/src/codegen_ast/expressions/mod.rs:497

**Message:** TODO: Add proper AST type tracking for variables

**Reason:** Variables için proper AST type tracking eksik. Bu, type checking ve codegen accuracy için önemli.

### File: vex-compiler/src/codegen_ast/expressions/mod.rs:555

**Message:** TODO: full implementation

**Reason:** Enum data-carrying variants için full implementation eksik, sadece tag var. Data-carrying için struct + tag implementasyonu gerekiyor.

### File: vex-compiler/src/codegen_ast/types/conversion.rs:215

**Message:** TODO: Better error handling

**Reason:** Type conversion'da error handling improve edilmesi gerekiyor. Şu anda basic error handling var, daha robust olması lazım.

### File: vex-compiler/src/codegen_ast/types/conversion.rs:318

**Message:** TODO: Implement proper intersection semantics

**Reason:** Intersection types için proper semantics implement edilmemiş. Type system'ın completeness'i için gerekli.

### File: vex-compiler/src/codegen_ast/types/conversion.rs:420

**Message:** TODO: Resolve to actual type from impl context

**Reason:** Impl blocks'dan actual type resolve edilmemiş. Generic resolution için kritik.

### File: vex-compiler/src/codegen_ast/core/utilities.rs:97

**Message:** TODO: Add LLVM metadata to mark as readonly/constant

**Reason:** LLVM metadata ile readonly/constant marking eksik. Optimization için önemli.

### File: vex-compiler/src/codegen_ast/core/utilities.rs:283

**Message:** TODO: Add LLVM metadata to mark as readonly/constant

**Reason:** Aynı, başka location'da. LLVM metadata eksikliği.
