# Constant Folding for If Statements - DX Enhancement

**Status:** 💡 Proposal  
**Date:** 22 Kasım 2025  
**Priority:** P2 - Developer Experience (DX) improvement

## Motivasyon

Geliştiriciler debug/test sırasında sıkça şöyle kod yazarlar:

```vex
let DEBUG = true;

if DEBUG {
    println("Debug mode active");
    // ... detaylı loglar
}

if false {
    // Geçici olarak devre dışı kod
    broken_function();
}
```

**Şu anda:** Her iki branch de LLVM IR'e dahil edilir, runtime'da check yapılır.

**İstenen:** Compile-time'da sabit değer varsa, dead code elimination yapılsın.

## Faydalar

### 1. **Temiz Debug Kodu**
```vex
const DEBUG_LEVEL = 2;

if DEBUG_LEVEL >= 2 {
    println("Detailed logs...");  // Sadece DEBUG_LEVEL >= 2 ise compile edilir
}
```

### 2. **Feature Flags**
```vex
const ENABLE_EXPERIMENTAL = false;

if ENABLE_EXPERIMENTAL {
    use_experimental_algorithm();  // Production'da tamamen kaldırılır
}
```

### 3. **Zero-cost Abstractions**
```vex
if true {
    optimized_path();  // LLVM her zaman bunu çağırır
} else {
    fallback_path();   // Hiç compile edilmez
}
```

### 4. **Kod Boyutu**
Dead code elimination → daha küçük binary

## Önerilen İmplementasyon

**Dosya:** `vex-compiler/src/codegen_ast/statements/loops/if_statement.rs`

```rust
fn compile_if_statement_impl(...) -> Result<(), String> {
    // ⭐ NEW: Constant folding optimization
    if let Some(const_val) = self.try_evaluate_const_expr(condition)? {
        // Compile-time constant condition!
        if const_val {
            // Always true → compile only then block
            return self.compile_block(then_block);
        } else {
            // Always false → compile only else/elif
            if let Some(else_blk) = else_block {
                return self.compile_block(else_blk);
            }
            // No else → skip entirely
            return Ok(());
        }
    }
    
    // Runtime condition → normal codegen
    let cond_val = self.compile_expression(condition)?;
    // ... mevcut kod
}
```

**Yardımcı fonksiyon:**
```rust
fn try_evaluate_const_expr(&self, expr: &Expression) -> Result<Option<bool>, String> {
    match expr {
        Expression::BoolLiteral(b) => Ok(Some(*b)),
        Expression::IntLiteral(n) => Ok(Some(*n != 0)),
        Expression::Ident(name) => {
            // Check if constant
            if let Some(const_val) = self.constants.get(name) {
                // Evaluate constant value
                Ok(Some(/* const_val != 0 */))
            } else {
                Ok(None) // Not constant
            }
        }
        Expression::Binary { left, op, right, .. } => {
            // Evaluate binary ops on constants
            // if DEBUG_LEVEL >= 2 gibi
            Ok(None) // Şimdilik skip
        }
        _ => Ok(None), // Not evaluatable at compile time
    }
}
```

## Kapsam

### Aşama 1 (Minimal - DX için yeterli)
- ✅ `if true { ... }` → sadece then bloğu compile edilir
- ✅ `if false { ... }` → sadece else bloğu compile edilir (varsa)
- ✅ `if 0 { ... }` → false olarak değerlendirilir
- ✅ `if 1 { ... }` → true olarak değerlendirilir

### Aşama 2 (Constant propagation)
- 🔵 `const DEBUG = true; if DEBUG { ... }`
- 🔵 `let x = 5; if x == 5 { ... }` (let değişkenleri için)

### Aşama 3 (Full constant folding)
- 🔵 `if 2 + 2 == 4 { ... }`
- 🔵 `if DEBUG && VERBOSE { ... }`
- 🔵 Binary ops on constants

## Riskler ve Dikkat Edilmesi Gerekenler

### 1. **Side Effects**
```vex
if get_value() == 0 {  // ❌ get_value() çağrılmamalı mı?
    // ...
}
```
**Çözüm:** Sadece literal'leri ve constant'ları evaluate et, fonksiyon çağrılarını değil.

### 2. **Debug Experience**
Dead code elimination yapılınca, debugger'da görünmez.
**Çözüm:** Debug build'de constant folding'i opsiyonel yap.

### 3. **Compiler Flags**
```rust
// cargo.toml veya compiler flag
const_folding_enabled: bool = !is_debug_build
```

## Alternatif Yaklaşımlar

### Yaklaşım A: LLVM'e bırak
LLVM zaten dead code elimination yapıyor. Bizim yapmamıza gerek var mı?

**Avantaj:** Daha az compiler complexity  
**Dezavantaj:** Compile error'lar runtime'a kadar görünmez

```vex
if false {
    this_function_doesnt_exist();  // LLVM kaldırır ama compile error yok!
}
```

### Yaklaşım B: AST-level optimization
Parser'dan sonra, codegen'den önce AST'yi optimize et.

**Avantaj:** Daha genel kullanım (sadece if değil, while vs.)  
**Dezavantaj:** Daha fazla iş

## Önerim

**Aşama 1'i implement edelim:**
- Sadece literal değerler (`true`, `false`, `0`, `1`)
- Minimal kod değişikliği (~20 satır)
- Büyük DX kazancı

**Karar:** Yapalım mı?

- ✅ **Evet** → Büyük DX benefit, minimal effort
- ❌ **Hayır** → LLVM zaten yapıyor, compiler basit kalsın
- 🔵 **Sonra** → P0 bugları önce fix edelim (overload resolution)

**Benim tavsiyem:** ✅ Yapalım ama önce overload bug'ını fix edelim (36 test failing).
