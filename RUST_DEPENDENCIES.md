# Rust Dependency Analysis for Vex Language

## Tam Bağımlılık Ağacı

### 1. Compiler (vex-compiler)

#### A. CRITICAL Dependencies (Compiler çalışması için şart)

```toml
inkwell = "0.4.0"
```

- **Ne yapar**: LLVM Rust bindings
- **Neden gerekli**: IR generation, code optimization, machine code
- **Alternatif**: YOK (LLVM C API direct FFI ile değiştirilebilir)
- **Self-hosting'te**: Vex FFI ile LLVM C API'ye bağlanacağız

```toml
vex-ast = { path = "../vex-ast" }
vex-lexer = { path = "../vex-lexer" }
vex-parser = { path = "../vex-parser" }
```

- **Ne yapar**: AST, lexer, parser implementation
- **Neden gerekli**: Source code → AST transformation
- **Alternatif**: YOK (core compiler logic)
- **Self-hosting'te**: Vex'te yeniden yazılacak (vexc/lexer.vx)

#### B. ERROR HANDLING

```toml
anyhow = "1.0"
thiserror = "1.0"
```

- **Ne yapar**: Error handling utilities
- **Neden gerekli**: ? operator, error propagation
- **Alternatif**: Std Result<T, E> (Rust std)
- **Self-hosting'te**: Vex'in kendi Result/Error sistemi
- **Kolay değiştirilebilir**: ✅ (sadece convenience)

#### C. GPU/SPIRV (Optional)

```toml
rspirv = "0.12.0"
spirv = "0.3.0"
```

- **Ne yapar**: SPIR-V generation (GPU shaders)
- **Neden gerekli**: GPU kernel support
- **Alternatif**: Direkt SPIR-V FFI
- **Self-hosting'te**: Vex FFI ile SPIR-V tools
- **Optional**: ✅ (sadece GPU features için)

---

### 2. Runtime (vex-runtime)

#### A. ERROR HANDLING

```toml
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"
```

- **Ne yapar**: Error handling + logging
- **Neden gerekli**: Runtime error reporting
- **Alternatif**: Custom error types
- **Self-hosting'te**: Vex error handling
- **Kolay değiştirilebilir**: ✅

#### B. ASYNC (Removed - tokio bağımlılığı kaldırıldı!)

```toml
# tokio = "1.0" -- REMOVED ✅
# tokio-uring = "0.4" -- REMOVED ✅
```

**Başarı**: Native runtime'a geçtik, tokio dependency yok!

---

### 3. AST/Lexer/Parser

#### A. SERIALIZATION

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

- **Ne yapar**: AST serialization (JSON)
- **Neden gerekli**: AST export/import, debugging
- **Alternatif**: Custom serialization
- **Self-hosting'te**: Vex serialize/deserialize traits
- **Kolay değiştirilebilir**: ✅

#### B. LALRPOP (Parser Generator - Optional)

```toml
lalrpop = "0.20"
lalrpop-util = "0.20"
```

- **Ne yapar**: Parser generation
- **Neden gerekli**: Grammar-based parsing
- **Alternatif**: Hand-written parser (zaten var!)
- **Self-hosting'te**: Vex'te hand-written parser
- **Kolay değiştirilebilir**: ✅ (zaten manual parser var)

---

## Kategorize Bağımlılıklar

### 🔴 CRITICAL (Zor değiştirilir)

1. **inkwell** (LLVM bindings)
   - Replacement: LLVM C API via FFI
   - Effort: 🔴🔴🔴🔴🔴 (çok zor)
   - Timeline: 6+ ay

### 🟡 IMPORTANT (Orta zorlukta değiştirilir)

2. **AST/Lexer/Parser** (internal crates)
   - Replacement: Vex'te yeniden yaz
   - Effort: 🟡🟡🟡🟡⚪ (4/5 zorluk)
   - Timeline: 4-6 ay

### 🟢 CONVENIENCE (Kolay değiştirilir)

3. **anyhow/thiserror** (error handling)

   - Replacement: Vex Result/Error types
   - Effort: 🟢🟢⚪⚪⚪ (2/5 zorluk)
   - Timeline: 1-2 hafta

4. **serde** (serialization)

   - Replacement: Custom serialize
   - Effort: 🟢🟢⚪⚪⚪ (2/5 zorluk)
   - Timeline: 2-3 hafta

5. **lalrpop** (parser generator)
   - Replacement: Yok (zaten hand-written parser var)
   - Effort: 🟢⚪⚪⚪⚪ (1/5 - already done!)
   - Timeline: ✅ Complete

---

## Rust'tan Tam Bağımsızlık için Roadmap

### Phase 1: Remove Convenience Dependencies (2-3 ay)

- [ ] Replace anyhow/thiserror with custom errors
- [ ] Replace serde with custom serialization
- [ ] Replace log with custom logging
- **Result**: Sadece critical deps kalır

### Phase 2: Rewrite Compiler in Vex (6-12 ay)

- [ ] vexc/lexer.vx
- [ ] vexc/parser.vx
- [ ] vexc/typeck.vx
- [ ] vexc/codegen.vx (LLVM C API FFI)
- **Result**: Vex compiler Vex'te yazılmış

### Phase 3: LLVM FFI (3-6 ay)

- [ ] LLVM C API bindings (Vex FFI)
- [ ] Replace inkwell with native Vex bindings
- **Result**: Zero Rust dependency!

### Phase 4: Bootstrap (1-2 ay)

- [ ] Compile vexc.vx with Rust compiler
- [ ] Compile vexc.vx with Vex compiler
- [ ] Verify both produce same output
- **Result**: Self-hosted compiler! 🎉

---

## Özet Tablo

| Dependency     | Category  | Criticality    | Replacement Effort | Timeline |
| -------------- | --------- | -------------- | ------------------ | -------- |
| **inkwell**    | LLVM      | 🔴 Critical    | Very Hard          | 6-12 mo  |
| **AST/Parser** | Compiler  | 🟡 Important   | Hard               | 4-6 mo   |
| **anyhow**     | Error     | 🟢 Convenience | Easy               | 1-2 wk   |
| **thiserror**  | Error     | 🟢 Convenience | Easy               | 1-2 wk   |
| **serde**      | Serialize | 🟢 Convenience | Easy               | 2-3 wk   |
| **log**        | Logging   | 🟢 Convenience | Easy               | 1 wk     |
| **lalrpop**    | Parser    | 🟢 Optional    | Done               | ✅       |
| **rspirv**     | GPU       | 🟡 Optional    | Medium             | 2-3 mo   |

---

## Şu Anki Rust Bağımlılık Yüzdesi

### Code Lines

```
Total Project: ~15,000 lines
- Vex Code (.vx): ~2,000 lines (13%)
- Rust Code (.rs): ~13,000 lines (87%)
```

### Critical Dependencies

```
Total Dependencies: 15
- Critical (LLVM): 1 (inkwell)
- Important (Compiler): 3 (ast/lexer/parser)
- Convenience: 7 (error/serde/log)
- Optional: 4 (GPU/async libs)
```

**Rust Dependency Score: 87%**
**Self-hosting Readiness: ~30%**

---

## Sonuç ve Öneri

### Kısa Vadeli (3-6 ay)

1. ✅ Remove tokio (Done!)
2. ⏳ Implement Vex std library (collections, I/O)
3. ⏳ Add closure/macro support

### Orta Vadeli (6-12 ay)

1. ⏳ Start rewriting compiler in Vex
2. ⏳ LLVM C API bindings via FFI
3. ⏳ Remove convenience deps

### Uzun Vadeli (12-24 ay)

1. ⏳ Complete self-hosted compiler
2. ⏳ Bootstrap verification
3. 🎯 **ZERO Rust dependency**

**Tahmin: 2 yıl içinde tamamen Rust'tan bağımsız olabiliriz.**
