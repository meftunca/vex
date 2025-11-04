# Vex Language - Code Refactoring Plan

## Büyük Rust Dosyalarını Parçalama Stratejisi

**Tarih:** 4 Kasım 2025  
**Hedef:** 500+ satırlı Rust dosyalarını mantıklı modüllere ayırma

---

## 📊 Tespit Edilen Büyük Dosyalar (500+ satır)

| Dosya                | Satır | Öncelik   | Klasör                                      |
| -------------------- | ----- | --------- | ------------------------------------------- |
| `expressions/mod.rs` | 1401  | 🔴 Yüksek | `vex-compiler/src/codegen_ast/expressions/` |
| `functions.rs`       | 1159  | 🔴 Yüksek | `vex-compiler/src/codegen_ast/`             |
| `statements.rs`      | 1044  | 🔴 Yüksek | `vex-compiler/src/codegen_ast/`             |
| `expressions.rs`     | 772   | 🟡 Orta   | `vex-parser/src/parser/`                    |
| `items.rs`           | 747   | 🟡 Orta   | `vex-parser/src/parser/`                    |
| `lifetimes.rs`       | 665   | 🟡 Orta   | `vex-compiler/src/borrow_checker/`          |
| `borrows.rs`         | 603   | 🟡 Orta   | `vex-compiler/src/borrow_checker/`          |
| `moves.rs`           | 590   | 🟡 Orta   | `vex-compiler/src/borrow_checker/`          |
| `main.rs`            | 556   | 🟢 Düşük  | `vex-cli/src/`                              |

---

## 🎯 Parçalama Stratejisi

### 1. `expressions/mod.rs` (1401 satır) - 🔴 EN YÜKSEK ÖNCELİK

**Mevcut Yapı:**

- Ana dispatcher: `compile_expression()`
- Match expression: `compile_match_expression()` (~350 satır)
- Pattern matching: `compile_pattern_check()`, `compile_pattern_binding()`, `compile_pattern_match()` (~450 satır)
- Equality comparison: `compile_equality_comparison()` (~200 satır)
- F-string compilation: `compile_fstring()` (access.rs'de olabilir)

**Önerilen Yapı:**

```
expressions/
├── mod.rs                    # Dispatcher only (~200 satır)
├── access.rs                 # ✅ Zaten var (486 satır - kabul edilebilir)
├── binary_ops.rs             # ✅ Zaten var
├── calls.rs                  # ✅ Zaten var (413 satır - kabul edilebilir)
├── literals.rs               # ✅ Zaten var
├── special.rs                # ✅ Zaten var
├── match.rs                  # 🆕 Match expression (~600 satır)
│   ├── compile_match_expression()
│   ├── compile_pattern_check()
│   ├── compile_pattern_binding()
│   ├── compile_pattern_match()
│   └── compile_equality_comparison()
└── control.rs                # 🆕 Control flow expressions (~200 satır)
    ├── compile_block_expression()
    ├── compile_try_expression()
    └── compile_await_expression()
```

**Aksiyonlar:**

1. `match.rs` modülü oluştur (match ve pattern matching)
2. `control.rs` modülü oluştur (block, try, await)
3. `mod.rs` sadece dispatcher olarak kalır

---

### 2. `functions.rs` (1159 satır) - 🔴 YÜKSEK ÖNCELİK

**Mevcut Yapı:**

- Program compilation: `compile_program()` (~90 satır)
- Type registration: `register_*()` (~150 satır)
- Function declaration: `declare_function()`, `declare_trait_impl_method()`, `declare_struct_method()` (~250 satır)
- Function compilation: `compile_function()`, `compile_trait_impl_method()`, `compile_struct_method()` (~350 satır)
- Generic instantiation: `instantiate_generic_function()`, `instantiate_generic_struct()` (~200 satır)
- Type inference: `infer_type_args_from_call()`, `substitute_types_in_function()` (~100 satır)
- Async: `compile_async_function()` (~120 satır)

**Önerilen Yapı:**

```
codegen_ast/
├── functions.rs              # Ana dispatcher (~150 satır)
├── registration.rs           # 🆕 Type/function registration (~300 satır)
│   ├── register_type_alias()
│   ├── register_struct()
│   ├── register_enum()
│   ├── register_trait()
│   └── register_trait_impl()
├── declaration.rs            # 🆕 Function declarations (~300 satır)
│   ├── declare_function()
│   ├── declare_trait_impl_method()
│   ├── declare_struct_method()
│   └── generate_enum_constructors()
├── compilation.rs            # 🆕 Function body compilation (~400 satır)
│   ├── compile_function()
│   ├── compile_trait_impl_method()
│   ├── compile_struct_method()
│   └── compile_async_function()
└── generics.rs               # 🆕 Generic instantiation (~300 satır)
    ├── instantiate_generic_function()
    ├── instantiate_generic_struct()
    ├── infer_type_args_from_call()
    └── substitute_types_in_function()
```

**Aksiyonlar:**

1. `registration.rs` - Type registration logic
2. `declaration.rs` - Function declaration logic
3. `compilation.rs` - Function body compilation
4. `generics.rs` - Generic system
5. `functions.rs` - Sadece dispatcher ve `compile_program()`

---

### 3. `statements.rs` (1044 satır) - 🔴 YÜKSEK ÖNCELİK

**Mevcut Yapı:**

- Block compilation: `compile_block()` (~20 satır)
- Statement dispatcher: `compile_statement()` (~630 satır - çok büyük!)
- Control flow: `compile_if_statement()`, `compile_while_loop()`, `compile_for_loop()`, `compile_switch_statement()` (~400 satır)

**Önerilen Yapı:**

```
codegen_ast/
├── statements.rs             # Ana dispatcher (~150 satır)
├── variables.rs              # 🆕 Variable statements (~250 satır)
│   ├── compile_let_statement()
│   └── compile_assignment_statement()
├── control_flow.rs           # 🆕 Control flow (~400 satır)
│   ├── compile_if_statement()
│   ├── compile_while_loop()
│   ├── compile_for_loop()
│   └── compile_switch_statement()
├── expressions.rs            # ✅ Zaten var (block expressions)
└── defer.rs                   # 🆕 Defer statements (~200 satır)
    ├── execute_deferred_statements()
    └── clear_deferred_statements()
```

**Aksiyonlar:**

1. `variables.rs` - Let, assignment, return statements
2. `control_flow.rs` - If, while, for, switch
3. `defer.rs` - Defer logic (mod.rs'den taşınabilir)
4. `statements.rs` - Sadece dispatcher

---

### 4. `expressions.rs` (parser) (772 satır) - 🟡 ORTA ÖNCELİK

**Mevcut Yapı:**

- Expression parsing dispatcher: `parse_expression()` (~10 satır)
- Operator parsing: `parse_comparison()`, `parse_additive()`, `parse_multiplicative()`, `parse_unary()`, `parse_postfix()` (~300 satır)
- Primary parsing: `parse_primary()` (~200 satır)
- Match parsing: `parse_match_expression()`, `parse_pattern()`, `parse_single_pattern()` (~200 satır)
- Closure parsing: `parse_closure()` (~60 satır)

**Önerilen Yapı:**

```
parser/
├── expressions.rs            # Ana dispatcher (~150 satır)
├── operators.rs              # 🆕 Operator parsing (~300 satır)
│   ├── parse_comparison()
│   ├── parse_additive()
│   ├── parse_multiplicative()
│   ├── parse_unary()
│   └── parse_postfix()
├── primaries.rs              # 🆕 Primary expressions (~200 satır)
│   └── parse_primary()
└── patterns.rs               # 🆕 Pattern matching (~200 satır)
    ├── parse_match_expression()
    ├── parse_pattern()
    └── parse_single_pattern()
```

**Aksiyonlar:**

1. `operators.rs` - Operator precedence parsing
2. `primaries.rs` - Primary expressions (literals, identifiers, etc.)
3. `patterns.rs` - Match expressions and patterns
4. `expressions.rs` - Sadece dispatcher

---

### 5. `items.rs` (parser) (747 satır) - 🟡 ORTA ÖNCELİK

**Mevcut Yapı:**

- Import/export: `parse_import()`, `parse_export()` (~100 satır)
- Constants: `parse_const()` (~20 satır)
- Structs: `parse_struct()`, `parse_struct_method()` (~100 satır)
- Enums: `parse_enum()` (~50 satır)
- Type aliases: `parse_type_alias()` (~20 satır)
- Traits: `parse_interface_or_trait()`, `parse_trait_impl()`, `parse_trait_method_signature()` (~200 satır)
- Functions: `parse_function()`, `parse_parameters()` (~200 satır)
- Extern: `parse_extern_block()`, `parse_extern_function()` (~50 satır)

**Önerilen Yapı:**

```
parser/
├── items.rs                  # Ana dispatcher (~100 satır)
├── imports.rs                # 🆕 Import/export (~100 satır)
│   ├── parse_import()
│   └── parse_export()
├── types.rs                  # 🆕 Type definitions (~300 satır)
│   ├── parse_struct()
│   ├── parse_enum()
│   ├── parse_type_alias()
│   └── parse_trait()
├── functions.rs              # 🆕 Function parsing (~250 satır)
│   ├── parse_function()
│   ├── parse_parameters()
│   ├── parse_struct_method()
│   └── parse_trait_method_signature()
└── externs.rs                # 🆕 Extern blocks (~100 satır)
    ├── parse_extern_block()
    └── parse_extern_function()
```

**Aksiyonlar:**

1. `imports.rs` - Import/export parsing
2. `types.rs` - Struct, enum, type alias, trait parsing
3. `functions.rs` - Function and method parsing
4. `externs.rs` - Extern block parsing
5. `items.rs` - Sadece dispatcher

---

### 6. Borrow Checker Dosyaları (590-665 satır) - 🟡 ORTA ÖNCELİK

Bu dosyalar zaten mantıklı şekilde ayrılmış. Sadece internal helper fonksiyonları modüllere ayırabiliriz:

**`lifetimes.rs` (665 satır):**

```
borrow_checker/
├── lifetimes.rs              # Ana checker (~300 satır)
└── lifetimes_helpers.rs      # 🆕 Helper functions (~365 satır)
    ├── enter_scope()
    ├── exit_scope()
    ├── declare_variable()
    └── declare_pattern_bindings()
```

**`borrows.rs` (603 satır):**

- Zaten iyi organize edilmiş, sadece test fonksiyonları ayrılabilir

**`moves.rs` (590 satır):**

- Zaten iyi organize edilmiş, sadece test fonksiyonları ayrılabilir

---

### 7. `main.rs` (cli) (556 satır) - 🟢 DÜŞÜK ÖNCELİK

**Mevcut Yapı:**

- CLI argument parsing (~100 satır)
- Command handling (~200 satır)
- File operations (~100 satır)
- Error handling (~150 satır)

**Önerilen Yapı:**

```
cli/
├── main.rs                   # Entry point (~100 satır)
├── commands.rs               # 🆕 Command handlers (~200 satır)
│   ├── handle_compile()
│   ├── handle_run()
│   └── handle_test()
├── args.rs                   # 🆕 Argument parsing (~100 satır)
└── utils.rs                  # 🆕 File/error utilities (~150 satır)
```

**Aksiyonlar:**

1. `commands.rs` - Command execution
2. `args.rs` - CLI argument parsing
3. `utils.rs` - File operations, error formatting
4. `main.rs` - Sadece entry point

---

## 📋 Uygulama Sırası (Öncelik)

### Faz 1: Codegen AST (En Kritik) - 3-4 gün

1. ✅ `expressions/mod.rs` → `match.rs` + `control.rs`
2. ✅ `functions.rs` → `registration.rs` + `declaration.rs` + `compilation.rs` + `generics.rs`
3. ✅ `statements.rs` → `variables.rs` + `control_flow.rs` + `defer.rs`

### Faz 2: Parser - 2-3 gün

4. ✅ `expressions.rs` (parser) → `operators.rs` + `primaries.rs` + `patterns.rs`
5. ✅ `items.rs` → `imports.rs` + `types.rs` + `functions.rs` + `externs.rs`

### Faz 3: Borrow Checker - 1-2 gün

6. ✅ `lifetimes.rs` → `lifetimes_helpers.rs`

### Faz 4: CLI - 1 gün

7. ✅ `main.rs` → `commands.rs` + `args.rs` + `utils.rs`

---

## 🛠️ Refactoring Prensipleri

### 1. **Tek Sorumluluk Prensibi (SRP)**

Her modül sadece bir sorumluluğa sahip olmalı:

- `match.rs` → Sadece match expressions
- `variables.rs` → Sadece variable statements

### 2. **Bağımlılık Yönetimi**

- Modüller birbirine bağımlı olmamalı (cyclic dependency yok)
- Ortak utilities için `utils.rs` veya `common.rs`

### 3. **Test Korunması**

- Her modül için mevcut testler çalışmaya devam etmeli
- Test dosyaları da aynı şekilde organize edilmeli

### 4. **Public API Korunması**

- `pub(crate)` ve `pub` visibility'leri korunmalı
- Internal helper'lar `fn` olarak kalmalı

### 5. **Modül Boyutu**

- Her modül **200-400 satır** arası olmalı
- 500+ satırlı modüller tekrar parçalanmalı

---

## ✅ Başarı Kriterleri

1. **Hiçbir dosya 500+ satır olmamalı**
2. **Tüm testler geçmeli** (76/86 → 76/86)
3. **Derleme hatası olmamalı**
4. **Kod okunabilirliği artmalı**
5. **Yeni özellik eklemek daha kolay olmalı**

---

## 📝 Notlar

- Her refactoring adımından sonra **test çalıştırılmalı**
- Her modül için **kısa bir dokümantasyon** yorumu eklenmeli
- `mod.rs` dosyaları **sadece dispatcher** olarak kalmalı
- Helper fonksiyonlar **modül içinde private** olmalı

---

**Son Güncelleme:** 4 Kasım 2025  
**Durum:** Planlama Tamamlandı - Uygulamaya Hazır
