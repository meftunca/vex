# SIMD Implementation Strategy for Vex

**Date:** November 7, 2025  
**Status:** Planning Phase  
**Priority:** HIGH - Performance Critical Feature

---

## 🎯 Goals

1. **4-8x Performance Boost** for vectorizable operations
2. **Cross-platform** support (x86 SSE/AVX, ARM NEON)
3. **Type-safe** SIMD operations with compile-time checks
4. **Auto-vectorization** hints for LLVM optimizer
5. **Explicit SIMD** intrinsics for performance-critical code

---

## 🏗️ Architecture Options

### Option 1: LLVM Auto-Vectorization (Easiest)

**Implementation:**

- Enable LLVM optimization passes (`-O2`, `-O3`)
- Add vectorization hints to loops
- Let LLVM decide when to vectorize

**Pros:**

- ✅ No language changes needed
- ✅ Works across platforms
- ✅ Compiler does the heavy lifting

**Cons:**

- ❌ Unpredictable performance (compiler may not vectorize)
- ❌ No explicit control
- ❌ Hard to debug why vectorization failed

**Estimated Time:** 2-3 hours

---

### Option 2: Explicit SIMD Types (Rust-style)

**Implementation:**

- Add SIMD types: `simd4<f32>`, `simd8<i32>`, etc.
- SIMD operations: `simd_add`, `simd_mul`, etc.
- Map to LLVM vector types

**Syntax:**

```vex
let v1 = simd4<f32>.new(1.0, 2.0, 3.0, 4.0);
let v2 = simd4<f32>.new(5.0, 6.0, 7.0, 8.0);
let result = v1 + v2;  // Vectorized addition (4 ops in parallel)
```

**Pros:**

- ✅ Full control over vectorization
- ✅ Predictable performance
- ✅ Type-safe at compile time

**Cons:**

- ❌ Requires AST changes (new types)
- ❌ Platform-specific code (SSE vs NEON)
- ❌ Developer must write SIMD explicitly

**Estimated Time:** 2-3 days

---

### Option 3: SIMD Intrinsics (Direct LLVM Mappings)

**Implementation:**

- Builtin functions: `simd_load`, `simd_add_f32x4`, etc.
- Direct LLVM IR generation
- Platform detection at compile time

**Syntax:**

```vex
let arr = [1.0, 2.0, 3.0, 4.0];
let v1 = simd_load_f32x4(&arr, 0);
let v2 = simd_load_f32x4(&arr, 4);
let result = simd_add_f32x4(v1, v2);
simd_store_f32x4(&output, 0, result);
```

**Pros:**

- ✅ Maximum performance
- ✅ No AST changes (just builtins)
- ✅ Direct LLVM mapping

**Cons:**

- ❌ Verbose syntax
- ❌ Easy to misuse (alignment issues)
- ❌ Not portable (x86 vs ARM differences)

**Estimated Time:** 1-2 days

---

### Option 4: Hybrid Approach (RECOMMENDED)

**Implementation:**
Combine auto-vectorization + explicit intrinsics:

1. LLVM auto-vectorization for loops (default)
2. SIMD intrinsics for hot paths (opt-in)
3. Platform abstraction layer (hide SSE/NEON)

**Syntax:**

```vex
// Auto-vectorization (compiler decides)
for i in 0..1000 {
    result[i] = a[i] + b[i];  // LLVM may vectorize
}

// Explicit SIMD (performance-critical)
@simd fn vector_add(a: &[f32], b: &[f32]): [f32] {
    let result = Vec.new();
    for i in 0..a.len() / 4 {
        let v1 = simd_load_f32x4(&a, i * 4);
        let v2 = simd_load_f32x4(&b, i * 4);
        let sum = simd_add_f32x4(v1, v2);
        simd_store_f32x4(&result, i * 4, sum);
    }
    return result;
}
```

**Pros:**

- ✅ Best of both worlds
- ✅ Easy for beginners (auto)
- ✅ Powerful for experts (explicit)
- ✅ Gradual adoption

**Cons:**

- ❌ More implementation work

**Estimated Time:** 3-4 days

---

## 🎨 Proposed Design: Hybrid SIMD

### Phase 1: Auto-Vectorization (2-3 hours)

**Goal:** Let LLVM vectorize simple loops

**Changes:**

1. Enable LLVM optimization level 2/3
2. Add loop vectorization hints
3. Test with benchmark

**Code:**

```rust
// vex-cli/src/main.rs
let passes = PassManager::create(&module);
passes.add_loop_vectorize_pass();
passes.add_slp_vectorize_pass();  // Superword-level parallelism
passes.run_on(&module);
```

**Expected Speedup:** 1.5-2x for simple array operations

---

### Phase 2: SIMD Intrinsics (1-2 days)

**Goal:** Add explicit SIMD builtins

**New Builtins:**

```vex
// Load/Store
fn simd_load_f32x4(ptr: &[f32], offset: i32): simd4<f32>;
fn simd_store_f32x4(ptr: &[f32], offset: i32, value: simd4<f32>);

// Arithmetic
fn simd_add_f32x4(a: simd4<f32>, b: simd4<f32>): simd4<f32>;
fn simd_sub_f32x4(a: simd4<f32>, b: simd4<f32>): simd4<f32>;
fn simd_mul_f32x4(a: simd4<f32>, b: simd4<f32>): simd4<f32>;
fn simd_div_f32x4(a: simd4<f32>, b: simd4<f32>): simd4<f32>;

// Horizontal operations
fn simd_sum_f32x4(a: simd4<f32>): f32;  // Sum all lanes
fn simd_dot_f32x4(a: simd4<f32>, b: simd4<f32>): f32;  // Dot product
```

**Implementation:**

```rust
// vex-compiler/src/codegen_ast/builtins/simd.rs
match builtin_name {
    "simd_add_f32x4" => {
        let v1 = args[0];  // LLVM <4 x float>
        let v2 = args[1];
        let result = builder.build_vector_add(v1, v2, "simd_add")?;
        Ok(result.into())
    }
    "simd_load_f32x4" => {
        let ptr = args[0];  // &[f32]
        let offset = args[1];  // i32

        // Cast to <4 x float>* and load
        let vec_ptr_type = context.f32_type().vec_type(4).ptr_type(AddressSpace::default());
        let vec_ptr = builder.build_pointer_cast(ptr, vec_ptr_type, "vec_cast")?;
        let vec_val = builder.build_load(vec_ptr, "simd_load")?;
        Ok(vec_val)
    }
}
```

**Expected Speedup:** 4-8x for explicit SIMD code

---

### Phase 3: Platform Abstraction (1 day)

**Goal:** Hide x86 vs ARM differences

**Approach:**

- CPU feature detection at runtime
- Compile-time platform selection
- Fallback to scalar code

**Code:**

```vex
// Platform-specific implementations
#[cfg(target_arch = "x86_64")]
fn simd_add_impl(a: simd4<f32>, b: simd4<f32>): simd4<f32> {
    // Use SSE/AVX
}

#[cfg(target_arch = "aarch64")]
fn simd_add_impl(a: simd4<f32>, b: simd4<f32>): simd4<f32> {
    // Use NEON
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn simd_add_impl(a: simd4<f32>, b: simd4<f32>): simd4<f32> {
    // Scalar fallback
}
```

---

## 📊 Benchmarks

### Baseline (No SIMD)

```vex
fn add_arrays(a: &[f32], b: &[f32]): [f32] {
    let result = Vec.new();
    for i in 0..a.len() {
        result.push(a[i] + b[i]);
    }
    return result;
}
```

**Performance:** ~1 GB/s

### Auto-Vectorization

```vex
// Same code, compiler optimizes
```

**Expected:** ~2-4 GB/s (2-4x speedup)

### Explicit SIMD

```vex
fn add_arrays_simd(a: &[f32], b: &[f32]): [f32] {
    let result = Vec.new();
    for i in 0..a.len() / 4 {
        let v1 = simd_load_f32x4(&a, i * 4);
        let v2 = simd_load_f32x4(&b, i * 4);
        let sum = simd_add_f32x4(v1, v2);
        simd_store_f32x4(&result, i * 4, sum);
    }
    return result;
}
```

**Expected:** ~8-16 GB/s (8-16x speedup)

---

## 🚧 Implementation Roadmap

### Week 1: Foundation (Phase 1 + 2)

- [x] Day 1: Auto-vectorization passes
- [ ] Day 2-3: SIMD intrinsics (load/store/arithmetic)
- [ ] Day 4: Basic benchmarks

### Week 2: Polish (Phase 3)

- [ ] Day 5: Platform abstraction
- [ ] Day 6: Documentation + examples
- [ ] Day 7: Performance tuning

---

## 🎯 Success Metrics

**Must Have:**

- ✅ 4x speedup on array operations (auto-vectorization)
- ✅ 8x speedup with explicit SIMD
- ✅ Works on x86_64 and aarch64

**Nice to Have:**

- ✅ Auto-vectorization hints in error messages
- ✅ SIMD code coverage analysis
- ✅ Runtime CPU feature detection

---

## 🔬 Technical Challenges

### Challenge 1: Alignment

**Problem:** SIMD requires 16-byte alignment  
**Solution:**

- Use `posix_memalign()` in allocator
- Add `@align(16)` attribute to types

### Challenge 2: Partial Vectors

**Problem:** Array length not divisible by 4  
**Solution:**

- Process main chunk with SIMD
- Handle remainder with scalar code

### Challenge 3: Platform Detection

**Problem:** Different instructions on x86 vs ARM  
**Solution:**

- LLVM target triple detection
- Runtime CPUID checks for x86
- ARM feature register for NEON

---

## 📖 Related Work

### Rust `std::simd`

- Portable SIMD types: `f32x4`, `i32x8`
- Platform-agnostic API
- Compile-time dispatch

### C++ `std::simd` (C++26)

- Similar to Rust approach
- Template-based SIMD types
- Auto-vectorization friendly

### Julia SIMD

- `@simd` macro for auto-vectorization
- LLVM-based backend
- Very high performance

### Our Approach

- Hybrid: auto + explicit
- LLVM vector types directly
- No external dependencies

---

## 🤔 Open Questions

1. **Type syntax:** `simd4<f32>` vs `f32x4` vs `Vector4f`?
2. **Alignment:** Auto-align all arrays or explicit `@align`?
3. **Naming:** `simd_add` vs `vadd` vs `vector_add`?
4. **Error handling:** What if SIMD not supported?
5. **Generic SIMD:** Should SIMD work with custom types?

---

## 💡 Recommendations

**Start with:** Option 4 (Hybrid Approach)

**Phase 1 (Today):**

- Enable LLVM optimization passes
- Test with simple array benchmark
- Measure baseline speedup

**Phase 2 (Tomorrow):**

- Implement `simd_load`, `simd_store`, `simd_add`
- Test on real workload
- Document API

**Phase 3 (Next Week):**

- Platform abstraction
- Error handling
- Production-ready

**Total Time:** 3-4 days  
**Expected Speedup:** 4-8x  
**Risk:** Low (LLVM does heavy lifting)

---

## 📝 Next Steps

1. [ ] Review this design with team
2. [ ] Prototype Phase 1 (auto-vectorization)
3. [ ] Benchmark baseline vs optimized
4. [ ] Decide on intrinsic syntax
5. [ ] Implement Phase 2 (explicit SIMD)

**Decision needed:** Which option to pursue?

- Option 1: Quick win (auto-vectorization only)
- Option 4: Full feature (hybrid approach)

---

**Authors:** AI Agent  
**Reviewers:** TBD  
**Status:** DRAFT - Needs Review

# Diğer Taslak

# Vex SIMD — **Direct LLVM Mapping** (Lowering Guide)

**Date:** 2025‑11‑07  
**Audience:** Compiler/Runtime engineers (and power users)  
**Goal:** Netleştirmek: _“Evet, doğrudan LLVM’e map ediyoruz.”_ Bu belge; **hangi Vex SIMD ilkelinin hangi LLVM yapısına** indiğini, **sabit** ve **scalable (VLA)** vektörlerdeki yolu, **mask/tail** ile **reduction** ve **multi‑versioning/dispatch** mimarisini anlatır.

---

## 1) Vektör Tipleri ve Maskeler

| Vex türü    | LLVM türü (sabit genişlik) | LLVM türü (scalable/VLA) |
| ----------- | -------------------------- | ------------------------ |
| `simd<T>`   | `<N x T>`                  | `<vscale x N x T>`       |
| `simd_mask` | `<N x i1>`                 | `<vscale x N x i1>`      |

- **x86/NEON/WASM**: `<N x T>` sabit genişlik; `N` hedefe göre (SSE=4 f32, AVX2=8 f32, AVX‑512=16 f32, NEON=4 f32, WASM=4 f32).
- **SVE/RVV**: `<vscale x N x T>` (VLA). _Gerçek `N·vscale` runtime’da belirlenir._

> **Not:** SVE/RVV için **VP (Vector Predication) intrinsics** yolu tercih edilir. Sabit genişlikte klasik maskeli intrinsics kullanılır.

---

## 2) Aritmetik & Karşılaştırma — IR Eşlemesi

| Vex işlemi                | LLVM sabit genişlik                        | LLVM scalable / VP yolu                      |
| ------------------------- | ------------------------------------------ | -------------------------------------------- |
| `a + b`, `a - b`, `a * b` | `fadd`, `fsub`, `fmul` / `add`,`sub`,`mul` | `llvm.vp.fadd/fsub/fmul` (+ `mask`, `evl`)   |
| `a / b` (float)           | `fdiv`                                     | `llvm.vp.fdiv`                               |
| `fma(a,b,c)`              | `llvm.fma.*` (veya `fmul`+`fadd`)          | `llvm.vp.fma`                                |
| Karşılaştırmalar          | `fcmp/icmp` → `<N x i1>`                   | `llvm.vp.fcmp/vp.icmp` → `<vscale x N x i1>` |
| `select(mask, t, f)`      | `select <N x i1>`                          | `llvm.vp.select`                             |
| `abs(x)` (float)          | `llvm.fabs.*` (veya bit trick)             | `llvm.vp.fabs`                               |
| `min/max` (float)         | `llvm.minnum/maxnum.*`                     | `vp` eşleniği (veya `fcmp` + `select`)       |

**Fast‑math bayrakları**: `nnan`, `ninf`, `nsz`, `contract`, `reassoc` vb. — yalnız **`~` benzerlik** operatörü bu bayraklardan **bağımsız**, doğruluk garantili tutulur.

---

## 3) Bellek İşlemleri — Load/Store/Gather/Scatter

| Vex                        | LLVM sabit genişlik                               | LLVM scalable / VP yolu       |
| -------------------------- | ------------------------------------------------- | ----------------------------- |
| `load<T>(p,i,m)`           | `llvm.masked.load.*` (mask varsa), yoksa `load`   | `llvm.vp.load` (mask + `evl`) |
| `store<T>(p,i,v,m)`        | `llvm.masked.store.*` (mask varsa), yoksa `store` | `llvm.vp.store`               |
| `gather<T>(base, idx, m)`  | `llvm.masked.gather.*`                            | `llvm.vp.gather`              |
| `scatter<T>(base,idx,v,m)` | `llvm.masked.scatter.*`                           | `llvm.vp.scatter`             |
| `prefetch(p,loc)`          | `llvm.prefetch(i8* ptr, rw, locality, cachetype)` | Aynı                          |
| `assume_aligned(p,N)`      | `llvm.assume` + `align` metadata / param attr     | Aynı                          |

- **Tail**: son blokta `m` maske **her zaman** üretilir; sabit genişlikte masked intrinsics, VLA’da `vp.*` ile `evl=aktif_lanes` kullanılır.
- **No‑alias**: parametre nitelikleri ve/veya `!alias.scope`/TBAA metadataları eklenir. Auto‑vectorizer’a doğrudan yardım eder.

---

## 4) Redüksiyonlar

| Vex                   | LLVM                                                                              |
| --------------------- | --------------------------------------------------------------------------------- |
| `reduce_add(x,m)`     | `llvm.experimental.vector.reduce.add.*` (+mask varsa `select`/`and`) veya VP yolu |
| `reduce_min/max(x,m)` | `llvm.experimental.vector.reduce.[s/u]min,[f]min/max.*`                           |
| `count(mask)`         | `zext mask → <N x i32>` + `vector.reduce.add`                                     |

> Scalable tipler için reduce intrinsics **nxv** türleri ile çağrılır; hedef derleyici sürümüne bağlı olarak pattern‑match edilmiş yatay toplama dizileri de üretilir.

---

## 5) Benzerlik Operatörü `~` (Sayısal)

**Vektör**: `va ~ vb → simd_mask`

1. `d = fabs(va - vb)` → `llvm.fabs/fsub` (veya VP eşleniği)
2. **Abs/Rel** politika: `thr = max(ε_abs, ε_rel * max(fabs(va), fabs(vb)))` → `llvm.maxnum` zinciri
3. `cmp = d <= thr` → `fcmp ole` → mask

**ULP modu** (opsiyonel): bit‑pattern ile ULP farkı:

- `as_i = bitcast(va)`/`bitcast(vb)`; negatifleri **signed‑magnitude order**’a dönüştür (işaret bit düzeltmesi)
- `udiff = abs(as_i - bs_i)`; `udiff <= ulps` → mask

**NaN**: önce `fcmp ord` ile ele; ord değilse sonuç **false**.

---

## 6) Döngü Sugar — `stripmine` Lowering

### 6.1. Sabit Genişlik (x86/NEON/WASM)

```
i = 0
step = VEC_WIDTH_T   ; (örn. f32 için 8/16/4)
while i < n:
  mask = mk_tail_mask(i, n, step)        ; <N x i1>
  v  = llvm.masked.load(base+i, mask)
  …                                       ; vektör işler
  llvm.masked.store(base+i, res, mask)
  i += step
```

### 6.2. Scalable (SVE/RVV) — VP/EVL Kalıbı

```
i = 0
while i < n:
  evl  = compute_active_lanes_T(n - i)    ; hedefe özgü: VLA uzunluğu (örn. CNTW)
  mask = mk_active_mask(evl)              ; <vscale x N x i1>
  v  = llvm.vp.load(base+i, mask, evl)
  …
  llvm.vp.store(base+i, res, mask, evl)
  i += evl
```

> Frontend, EVL/maske üretimini hedef‑bağımsız bir ara koda yazar; backend **SVE/RVV** için doğru intrinsic kombinasyonunu üretir.

---

## 7) Çoklu Sürüm + Runtime Dispatch

- Aynı fonksiyon için birden fazla hedef: **`default`**, `avx2`, `avx512f`, `sse4.2`, `neon`, `sve2`, `scalar`.
- **x86**: CPUID ile; **AArch64**: HWCAP/OS API; **WASM**: özellik biti.
- İlk çağrıda **en iyi varyant** seçilip fonksiyon pointer’ı **patch** edilir (veya ELF **`ifunc`**).
- SVE/RVV varsa **tek sürüm** VLA/VP ile yeterli (gerçek runtime width).

---

## 8) Teşhis ve İpuçları

- **Loop metadata**: `!llvm.loop.vectorize.enable`, `interleave.count`, `vectorize.width` (isteğe bağlı).
- **Remarks**: `-pass-remarks=vectorize -pass-remarks-missed=vectorize` → `-Rsimd` raporuna bağlanır.
- **Prefer width**: `simd.with_width(W)` kullanıldığında: sabit genişlikte **özel clone** üretilir, uygun `vectorize.width` ipuçları eklenir.

---

## 9) Örnek — Toplama (sabit ve scalable)

### 9.1. Vex (kullanıcı kodu)

```vex
for lanes (i, m) in simd.stripmine(n) {
    let va = load<f32>(a, i, m);
    let vb = load<f32>(b, i, m);
    store<f32>(c, i, va + vb, m);
}
```

### 9.2. LLVM (sabit genişlik, maskeli yol — şema)

```
%v  = call <N x float> @llvm.masked.load(..., <N x i1> %m, float* %passthru)
%w  = fadd <N x float> %v, %u
call void @llvm.masked.store(<N x float> %w, ..., <N x i1> %m)
```

### 9.3. LLVM (scalable/VP — şema)

```
%v  = call <vscale x N x float> @llvm.vp.load(..., <vscale x N x i1> %m, i32 %evl)
%w  = call <vscale x N x float> @llvm.vp.fadd(<vscale x N x float> %v, <vscale x N x float> %u, <vscale x N x i1> %m, i32 %evl)
call void @llvm.vp.store(<vscale x N x float> %w, ..., <vscale x N x i1> %m, i32 %evl)
```

---

## 10) Sürüm/Uyumluluk Notları

- **Öneri:** LLVM ≥ 17 (VP intrinsics ve scalable reduce desteği daha olgun). Daha eski derleyicilerde **maskeli intrinsics** + hedef‑özel SVE intrinsics (AArch64 ACLE) fallback’i.
- **WASM SIMD**: 128‑bit sabit; tüm mapping sabit genişlik dalını kullanır.
- **AVX‑512 downclock**: Dispatcher, küçük veri boylarında `avx2` patikasını tercih edebilir (heuristic).

---

## 11) Güvenlik & Doğruluk

- Maskeli load/store ile **out‑of‑bounds** tail erişimleri engellenir.
- `~` operatörü **fast‑math**’tan bağımsızdır, `NaN` tutarlı davranır (ord değilse false).
- No‑alias/tbaa metadataları yanlış verilmemeli; aksi UB/doğruluk kaybı.

---

## 12) Özet

Evet — Vex’in SIMD katmanı **doğrudan LLVM’e** map eder.

- Sabit genişlikte: **maskeli intrinsics** + autovec ipuçları.
- VLA mimarilerde: **VP intrinsics** ve **EVL/maske** ile tam runtime‑genişlik.
- Multi‑versioning + dispatch ile **x86/ARM/WASM**’da en iyi yol seçilir.
