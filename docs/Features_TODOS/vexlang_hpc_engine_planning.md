# VexLang HPC Engine Planning (Zero-Cost & Zero-Alloc)

**Status:** Draft v0.1  
**Author:** ChatGPT (with @buraks requirements)  
**Scope:** VexLang HPC yürütme motoru için ayrıntılı tasarım. Amaç, *zero-cost abstraction* ve sıcak yolda *zero-alloc* garanti etmek; compile-time planning ile en yüksek performans.

---

## İçindekiler
1. [Hedefler ve Garantiler](#hedefler-ve-garantiler)
2. [Mimari Genel Bakış](#mimari-genel-bakış)
3. [IR (Vex IR, VIR) Tasarımı](#ir-vex-ir-vir-tasarımı)
4. [Compile-Time Planning (MetaSchedule)](#compile-time-planning-metaschedule)
5. [Runtime Planning (Minimal Overhead)](#runtime-planning-minimal-overhead)
6. [Bellek Modeli ve Zero-Alloc](#bellek-modeli-ve-zero-alloc)
7. [CPU Backend](#cpu-backend)
8. [GPU Backend](#gpu-backend)
9. [Kernel Önbelleği (JIT/AOT Cache)](#kernel-önbelleği-jitaot-cache)
10. [Numerik Politika & Determinizm](#numerik-politika--determinism)
11. [API Yüzeyi (Kullanıcı & İç Arayüz)](#api-yüzeyi-kullanıcı--iç-arayüz)
12. [Hata Yönetimi, Telemetri ve Profiling](#hata-yönetimi-telemetri-ve-profiling)
13. [Test Planı & Doğrulama](#test-planı--doğrulama)
14. [MVP Yol Haritası](#mvp-yol-haritası)
15. [Derleme & Paketleme Stratejisi](#derleme--paketleme-stratejisi)
16. [Sözlük](#sözlük)

---

## Hedefler ve Garantiler

### Hedefler
- **Tek kaynak grafı → Çok hedef** (CPU SIMD + çok çekirdek, GPU CUDA/HIP/SPIR-V, opsiyonel WASM).
- **Lazy → planlı yürütme**: füzyon/tiling/memory planning sonrası kernel üretimi.
- **Compile-time planning**: mümkün olan her durumda statik plan; yoksa minimal runtime planlama.
- **Zero-cost abstraction**: yüksek seviye API sıcak yolda maliyetsiz (inline/monomorfik).
- **Zero-alloc (hot path)**: kernel içinde heap tahsisi yok; geçiciler arena/scratch’ta.
- **Determinism & numerics**: kontrol edilebilir sayısal politika ve deterministik reduce seçenekleri.

### Garantiler
- [x] Sıcak yol **sanal çağrı içermez** (vtable yok), **inline** ve **monomorfik**.  
- [x] **Heap tahsisi yok** (kernel içi).  
- [x] **Ara buffer** üretilmez (fusion); zorunlu geçiciler **önceden ayrılmış** scratch’tan.  
- [x] **Bounds/mask** maliyeti, mümkünse **compile-time**’da kaldırılır.  
- [x] **Alias/alignment** koşulları için derleme zamanı sözleşmeleri + gerekirse **guard + deopt**.

---

## Mimari Genel Bakış

```
VexLang API (eager facade)
        │
        ▼
Trace/Graph Builder  ──► VIR (Vex IR, SSA)
        │
        ├─► IR Passes: Canonicalize, CSE, DCE
        ├─► Fusion: ew-chain + reduce (+ stencil ileri aşama)
        ├─► Layout/Stride Normalize (SoA/AoS/AoSoA/Tiled)
        ├─► Tiling & Vectorization Hints
        └─► Memory Planner (liveness, in-place, reuse)

Planner (MetaSchedule + Rules + Arch Profiles)
        │
        ├─► Compile-Time Plan (statik)  → AOT/Prebuilt kernels
        └─► Runtime Plan (minimal)      → JIT/Seçim/Launch

Codegen & Backends
- CPU: LLVM + multiversioning (AVX2/AVX-512/NEON/SVE)
- GPU: NVPTX (CUDA), AMDGPU (HIP), SPIR-V (Vulkan/Level-Zero)
- WASM (opsiyonel): wasm64 + threads + simd

Runtime
- Thread pool, Streams/Events
- Unified Memory Manager (host/device/pinned/managed)
- Kernel Cache (hash: IR-slice + dtype/shape/layout + caps + policy)
- Telemetry & Tracing
```

---

## IR (Vex IR, VIR) Tasarımı

### Türler
- **TensorType**: `(dtype, shape, strides, layout_tag)`  
  - `layout_tag ∈ {SoA, AoS, AoSoA(k), Tiled(M,N)}`
- **Attrs**: `tile=(tX,tY,tZ)`, `vec_width`, `fast_math`, `deterministic`, `mem_scope`, `alignment`

### Dialect’ler
- `vex.ew`: element-wise (map/zip/unary/binary)
- `vex.red`: reduce/scan (sum, max, min, arg*, histogram)
- `vex.linalg`: matmul (GEMM), conv (ileri aşama), transpose
- `vex.mem`: alloc, view, reshape, layout_cast, copy
- `vex.schedule`: scheduler ipuçları (tile, unroll, vec)
- `vex.control`: barrier, stream, event

### Örnek (psödo-VIR)
```
// Host’tan getir
%a = vex.mem.fromHost ptr=%pa : tensor<f32[N], layout=SoA, align=64>
%b = vex.mem.fromHost ptr=%pb : tensor<f32[N], layout=SoA, align=64>

// İşlemler
%x = vex.ew.mul %b, const(2.0)
%y = vex.ew.add %a, %x                         // fuse adayı
%z = vex.red.sum %y {deterministic=true}

// Host’a yaz
vex.mem.toHost %y -> %py
```

**Pass’ler**: `%x` ve `%y` fuse → tek ew-kernel, `%z` ya ayrı reduce kernel ya da tree-reduce ile kısmi füzyon.

---

## Compile-Time Planning (MetaSchedule)

Compile-time plan, **statik bilgi** (shape/layout/policy) mevcutsa uygulanır. Amaç, *launch parametreleri ve kernel varyantını* derleme zamanında sabitlemek.

### Şartlar (statik plan için)
- Boyutların çoğu `constexpr` veya sınırlı varyant (template parametre / specialization constant).
- Layout/hizalama derleme zamanı biliniyor.
- Politika tipleri derleme zamanı: `fast_math`, `deterministic`, `vec_width`, `tile`.

### Çıktılar
- **Monomorfik kernel şablonu** (C++ template/const generics).
- **Sabit launch konfigürasyonu**: `blockDim/gridDim/smem` (GPU), `tile/VL/threads` (CPU).
- **Prebuilt/AOT çoklu varyant**: mimari profillerine göre (sm_80, sm_90; gfx1100; avx2/avx512…).

### Kural Tabanı (CPU)
- `VL = min(hwVL, alignment-driven)`.  
- `tile` seçimleri: L1/L2 sığdırma, write-allocate minimizasyonu.  
- `remainder` kaldırma: `N % VL == 0` ise masked/tail yok; aksi halde tek epilog.  
- `threads = min(P, ceil(N/threshold))`; **NUMA-aware** parçalayıcı.

### Kural Tabanı (GPU)
- `block = 128|256`, `items_per_thread = 1|2|4` (memory- vs compute-bound’a göre).  
- `grid = ceil(N / (block * items_per_thread))`.  
- `smem` limitleri & register baskısı (RPR) eşiği; **aşırı füzyon**u engelle.  
- Coalesced erişim garanti değilse **layout_cast** planla (tek seferlik).

### Pseudo-API
```cpp
template<class Backend, class Policy, class Layout, class T, int VL>
auto plan_compile_time(GraphSlice<T> g);

constexpr auto plan = plan_compile_time<backend::cuda<sm_90>,
                                        policy::block<256, /*items*/4>,
                                        layout::SoA, float, 16>(g);
plan.run(stream, scratch);
```

---

## Runtime Planning (Minimal Overhead)

Statik bilgi yetersizse **minimal** bir runtime planı devreye girer: karar verme O(1)/O(log K), çekirdek kod monomorfik.

- **Shape-polymorphic**: `n_iters = N / (VL*items)` runtime; kod yolu inline.  
- **Tail epilog** koşullu fakat inline.  
- **Arena tahsisi**: sadece planlama aşamasında; kernel içinde heap kullanılmaz.  
- **Önceden derlenmiş varyant seçimi**: tablo bakışı (ifunc/CPU feature probe, GPU’da arch seçimi).

---

## Bellek Modeli ve Zero-Alloc

### Bellek Sınıfları
- **Host**: normal, **Pinned** (H2D/D2H overlap için).  
- **Device**: global, shared (GPU), registers (impl).  
- **Managed/UVM**: opsiyonel (debug/kolaylık, üretimde kapatılabilir).

### Layout Politikaları
- `SoA`, `AoS`, `AoSoA(k)`, `Tiled(M,N)`; `view/reshape` sıfır kopya, `layout_cast` planlı tek kopya.  
- **Alignment sözleşmesi**: CPU 64B, GPU 128B varsayılan; API’da `@aligned(n)`.

### Arena/Scratch
- **Thread-local (CPU)** ve **stream-local (GPU)** arena.  
- Bump-pointer, **free yok**, `reset()` kernel batch sonunda.  
- Reduce/transpose gibi geçiciler **üst sınırı** compile-time’da; runtime sadece `n_used` ayarlar.

---

## CPU Backend

- **LLVM JIT/AOT + Multiversioning**: AVX2/AVX-512/NEON/SVE varyantları.  
- **Runtime dispatch**: CPU feature probe (ama sıcak döngüde sanal çağrı yok).  
- **Thread pool + work-stealing**, **NUMA-aware** parçalayıcı.  
- **Masked tail** (AVX-512/SVE) veya loop-peeling.  
- **Prefetching** opsiyonel; `restrict`/`noalias` sözleşmeleri ile yük-füzyon.

**Kernel Şablonu (örnek)**
```cpp
template<class T, int VL, class Layout, class Policy>
VEX_KERNEL void ew_fused(const T* __restrict a,
                         const T* __restrict b,
                         T* __restrict y,
                         size_t N);
```

---

## GPU Backend

- **Soyutlama**: CUDA (NVPTX), HIP (AMDGPU), SPIR-V (Vulkan/Level-Zero).  
- **Streams/Events**, **asenkron** H2D/D2H overlap; opsiyonel **CUDA Graphs**.  
- **Kernel şablonları**: ew-fused, reduce, block-GEMM, tile-conv.  
- **SMEM kullanımı**: tile bazlı, bank-conflict önlemleri; vektör yükleri.  
- **Register baskısı**na karşı füzyon limiti ve `items_per_thread` ayarı.

**Launch Örneği**
```cpp
constexpr LaunchCfg cfg = choose_cfg(arch::sm_90, N, Policy{});
launch<cfg>(ew_fused<T,VL,Layout,Policy>, args..., stream);
```

---

## Kernel Önbelleği (JIT/AOT Cache)

**Anahtar (Key):** `hash(IR-slice canonical)` + `(dtype, shape, layout)` + `device caps` + `policy flags`  
**Değer:** PTX/HSACO/SPIR-V blob + metaveri (regs, smem, block, items) + profil

- **LRU** + boyut sınırı; sıcak kernel’ler **pin**.  
- **Eager warming**: bilinen hotspot’lar önceden derlenir.  
- **Profil tabanı**: compile-time MetaSchedule tabloları, CI’de güncellenir.

---

## Numerik Politika & Determinism

- `fast_math ∈ {off, safe, aggressive}` (FMA, rsqrt, denorm FTZ).  
- `deterministic ∈ {true, false}`; deterministik reduce için staged tree + sabit bölme.  
- Varsayılan: `fast_math=safe`, `deterministic=false`.  
- Global override ve op-bazlı override desteklenir.

---

## API Yüzeyi (Kullanıcı & İç Arayüz)

### Kullanıcı (dil tarafı) – örnek
```ts
let y = a + b * 2;
let s = sum(y);

engine.run([s], {
  device: "auto",             // "cpu" | "gpu" | "auto"
  determinism: "auto",        // true | false | auto
  fast_math: "safe",          // off | safe | aggressive
});
```

### İç Arayüz (psödo-Rust/C++)
```cpp
struct Caps { /* arch özellikleri */ };
struct Plan { /* device, streams, tiles, layout casts, kernels */ };

trait Backend {
  compile(ir: &KernelIR, caps: &Caps) -> CompiledKernel;
  launch(ck: &CompiledKernel, args: &LaunchArgs, stream: Stream) -> Event;
}

struct Planner {
  choose_device(graph: &IR, stats: &Telemetry) -> Device;
  schedule(graph: &mut IR, caps: &Caps) -> Plan; // fusion, tiling, layout, streams
}

struct KernelCache {
  get_or_build(key: &Key, build: impl Fn() -> CompiledKernel) -> Handle;
}
```

---

## Hata Yönetimi, Telemetri ve Profiling

- **Hata sınıfları**: OOM, compile fail, illegal layout, timeout.  
- **Geri dönüş**: güvenli CPU yolu (opsiyonel); log + teşhis bilgisi.  
- **Telemetri**: süre, GB/s, GFLOP/s, occupancy, cache hit; JSON çıktısı.  
- **Tracer**: op → kernel eşlemeleri, füzyon kararları ve launch parametreleri.

---

## Test Planı & Doğrulama

1. **Objdump/IR denetimi**: sanal çağrı yok, heap yok, inline olmuş mu?  
2. **Mikro benchmark**: SAXPY, AXPY, reduce, GEMM, transpose. Boyut süpürmesi 4 KB → 4 GB.  
3. **Hot/Cold** koşular: JIT/AOT cache etkisi; pinned vs pageable.  
4. **NUMA & SMT** senaryoları; CPU multiversion doğrulaması.  
5. **GPU profili**: occupancy, mem coalescing, dram BW, smem bank conflicts.  
6. **Determinism**: tekrar testleri (bit-eşitlik), hız kıyası.  
7. **CI**: regresyon uyarısı, profil tablolarının güncellenmesi.

---

## MVP Yol Haritası

**Sürüm 0 (6–8 hafta):**
- IR çekirdeği: `ew` + `reduce` + `gemm` + `mem`.  
- Fusion v1: yalnız ew-zinciri + basit reduce.  
- Planner v1: compile-time kurallar + minimal runtime fallback.  
- CPU backend: AVX2/AVX-512, thread pool.  
- CUDA backend: ew-fused + reduce + block-gemm; 1–2 stream; H2D/D2H overlap.  
- Kernel cache v1: IR-hash, LRU.  
- Telemetry v1: süre, GB/s, GFLOP/s.

**Sürüm 1:**
- Layout policy & transform cache (SoA/AoS/AoSoA).  
- Deterministic reductions ve numerik bayraklar.  
- Advanced fusion (stencil, conv) ve tile autotune.  
- HIP/SPIR-V backendlere yayılım, CUDA Graphs.

---

## Derleme & Paketleme Stratejisi

- **AOT Paketleri**: CPU (avx2/avx512/sve/…) + GPU (sm_80/sm_90/gfx1100/spv) fatbin/hsaco/spv.  
- **IFUNC/Feature Probe**: en iyi varyant seçiminde hafif tablo bakışı.  
- **JIT Opsiyonel**: debug & dinamik ops; üretimde kapatılabilir.  
- **Build-time Profil**: CI autotune sonuçlarını tabla olarak göm.

---

## Sözlük

- **Zero-cost abstraction**: yüksek seviye API → derleme çıktısında fazladan maliyet bırakmaz.  
- **Zero-alloc (hot path)**: çalışma sırasında (kernel içinde) heap tahsisi yapılmaz.  
- **Fusion**: ara buffer üretmeden birden çok op’u tek kernele/döngüye birleştirme.  
- **MetaSchedule**: mimariye göre kural tabanlı compile-time plan.  
- **Arena/Scratch**: önceden ayrılmış, bump-pointer tahsisi yapan bellek havuzu.  
- **Deterministic reduction**: bit-eşit sonuç için sabit ağaç/işleme sırası.
