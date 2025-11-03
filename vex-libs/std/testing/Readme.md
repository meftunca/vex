# Vex Testing/Benchmark Entegrasyon Kılavuzu

## 0) Amaç

`vex_testing.c`, Vex’in çekirdek “test & bench” altyapısıdır. Vex derleyicisi/RT’si bu C dosyasını **FFI ile çağırır**; kullanıcıya Go `testing/benchmark` benzeri API sunarsınız. Aşağıdaki maddeler **nasıl entegre edeceğinizi** ve **nasıl kullanılacağını** tarif eder.

---

## 1) Dosya yerleşimi ve build

- Konum önerisi:runtime/
  testing/
  vex_testing.c # bu dosya
  include/vex_testing.h # (opsiyonel) Vex-RT içinden kısayol başlık

- Derleme bayrakları:
  - `-O3 -std=c11 -Wall -Wextra`
  - x86’da cycle counter için: **varsayılan açık** (`VEX_TEST_ENABLE_RDTSC=1`)
  - Linux’ta CPU pinleme/RT ipucu: **varsayılan açık** (`VEX_TEST_ENABLE_AFFINITY=1`)
- Demo’yu kapalı tutun; yalnızca geliştirme sırasında `-DVEX_TESTING_DEMO` kullanın.

---

## 2) Vex ↔ C FFI yüzeyi (önerilen binding)

Aşağıdaki sembolleri Vex tarafında FFI ile dışarı verin (isimleri sabitleyin):

### Test API

- `int vex_run_tests(const vex_test_case *tests, size_t count);`
- `#define&#32;VEX_TEST(name) ...` ve `#define&#32;VEX_TEST_ENTRY(name) ...` **C makroları** — Vex tarafında “test discovery” yapacaksanız C dizisini sizin oluşturmanız gerekir (bkz. §5).
- **Alt test** çağrısı: `void vex_subtest(const char *name, vex_test_fn fn);`

**Not:** Vex dilinde testleri (ör. `test "name" { ... }`) tanımladığınızda, derleyici/RT, her test bloğu için **C’de bir sarmalayıcı fonksiyon** üretsin ve `vex_test_case` dizisine koysun.

### Bench API

- Çalıştırma:
  `vex_bench_res vex_bench_run(vex_bench_fn fn, void *ctx, vex_bench_cfg cfg);`
- Raporlama:
  `void vex_bench_report_text(const vex_bench_res*);`
  `const char* vex_bench_report_json(const vex_bench_res*, char *buf, size_t bufsz);`
- Zaman penceresi kontrolü (bench fonksiyonunun içinden):
  `vex_bench_reset_timer();`
  `vex_bench_start_timer();`
  `vex_bench_stop_timer();`
  `vex_bench_set_bytes(uint64_t bytes_per_op);`
- DCE bariyerleri (gerekirse bench fonksiyonlarında kullanın):
  `vex_black_box_u64`, `vex_black_box_f64`, `vex_black_box_ptr`

---

## 3) Go `testing` eşlemesi (nasıl karşılanır)

| Go özelliği        | Vex’te nasıl?                               | Not                                              |
| ------------------ | ------------------------------------------- | ------------------------------------------------ |
| `T.Run`            | `vex_subtest("name", fn)`                   | Alt test basit hiyerarşi                         |
| `t.Log/Logf`       | `VEX_TLOG(...)`                             | Non-fatal log                                    |
| `t.Error/Errorf`   | `VEX_TERROR(...)`                           | Non-fatal hata sayacı artar                      |
| `t.FailNow/ Fatal` | `VEX_TFAILNOW(...)` veya `VEX_ASSERT(cond)` | Fatal: trap                                      |
| `t.Skip/SkipNow`   | `VEX_SKIP("why")`                           | Test “SKIP” raporlanır                           |
| Timeout            | (Vex-RT eklemeli)                           | İsteğe bağlı watchdog (bkz. §8)                  |
| Coverage/Fuzz      | (ayrı modüller)                             | Şimdilik kapsam dışı, sonrası için kanca bırakın |

---

## 4) Go `benchmark` eşlemesi

| Go                      | Vex                                                     | Not                                            |
| ----------------------- | ------------------------------------------------------- | ---------------------------------------------- |
| `b.N` auto-calib        | `cfg.auto_calibrate=true`                               | Hedef süre: `VEX_TEST_AUTOTGT_NS` (default 1s) |
| `SetBytes(k)`           | `vex_bench_set_bytes(k)` **ya da** `cfg.bytes_per_op=k` | MB/s hesaplanır                                |
| `Reset/Start/StopTimer` | birebir var                                             | Hazırlık/alloc’u ölçüm dışına al               |
| `-benchmem`             | (opsiyonel ek)                                          | `mallinfo2`/arena Sayaçları ekleyebilirsiniz   |
| Paralel bench           | (gelecek)                                               | `vex_bench_run_parallel` planlayın             |

---

## 5) Test keşfi / çalıştırma akışı

**Öneri:** Vex derleyicisi, derleme sonunda testleri toplayıp **statik bir tablo** üretsin:

```javascript
extern void test_math(void);
extern void test_io(void);

static const vex_test_case g_tests[] = {
  VEX_TEST_ENTRY(test_math),
  VEX_TEST_ENTRY(test_io),
};

int main() {
  return vex_run_tests(g_tests, sizeof(g_tests)/sizeof(g_tests[0]));
}

```

Vex tarafında `vex test` komutu bu ikiliyi çalıştırır.

Alt testlerinizi test fonksiyonunun içinden şu şekilde uyandırın:

```javascript
VEX_TEST(test_math) {
  vex_subtest("add", test_math_add);
  vex_subtest("mul", test_math_mul);
}

```

---

## 6) Benchmark yazım kuralları (takım rehberi)

- **Sıcak bölüm** yalnızca `start/stop` arasında olmalı.
- **Hazırlık/alloc**: zaman penceresi dışında; gerekiyorsa `vex_bench_reset_timer()` ile pencereden düşür.
- **DCE koruması**: giriş/çıkışları `vex_black_box_*` ile geçir; aksi halde derleyici çalışmayı atabilir.
- **Auto-calibrasyon**: kısa işlevlerde `cfg.auto_calibrate=true` kullanın. Büyük işlerde `cfg.time_ns` ile süre-hedefli koşu seçilebilir.
- **Throughput**: mutlaka `vex_bench_set_bytes(k)` çağırın; raporda **MB/s** gözüksün (decimal MB: 1e6).
- **Tekrarlama**: gürültü için `cfg.repeats=5..10`. Raporlardan min/med/mean/stddev, p90/95/99’u izleyin.
- **CPU pinleme**: mikrobenchlerde `cfg.pin_cpu=0` gibi sabitleyin.
  (CI’de çekirdek sayısı/bursts farklı olabilir; bu seçenek opsiyonel.)

**Şablon (C tarafında, Vex’in FFI ile yakacağı fonksiyonlar):**

```javascript
typedef struct { uint8_t *a, *b; size_t n; } Ctx;

static void bench_memcpy_like(void *p) {
  Ctx *x = (Ctx*)p;
  vex_bench_start_timer();
  // sıcak bölüm
  memcpy(x->b, x->a, x->n);
  vex_bench_stop_timer();
  vex_bench_set_bytes(x->n);
}

```

---

## 7) Raporlama/çıktı

- İnsan-okur rapor: `vex_bench_report_text(&res)`
  ns/op, (x86’da) cyc/op, MB/s, min/med/mean/max, p90/95/99.
- Makine-okur (CI): `vex_bench_report_json(&res, buf, bufsz)`
  Pars edip **JUnit/Allure** veya kendi dashboard’unuza aktarın.

---

## 8) CI ve determinizm önerileri

- **CPU pinleme** + **realtime ipucu** açık kalsın (CI izin veriyorsa).
- Frekans dalgalanmasına duyarlı testleri **ayrı job**’a alın (ör. `performance` etiketi).
- İsteğe bağlı timeout: Vex-RT tarafında test/bench çağrısını bir watchdog ile koşup aşımları “FAIL/ TIMEOUT” olarak işaretleyin.
- Büyük benchmarklar için **ısıtma** (warmup) süresini 50–200ms arası tutun; otomatik kalibrasyonun ilk ölçümüne etkisini azaltır.

---

## 9) Hata sınıflandırması & Sözleşme

- **Fatal**: `VEX_ASSERT`, `VEX_TFAILNOW` → test derhal biter (trap).
- **Non-fatal**: `VEX_TERROR` → sayılır, test sonunda “FAIL (N)” olarak görünür.
- **Skip**: `VEX_SKIP` → “SKIP (reason)”.

Takım rehberi: _testlerde fatal’ı yalnızca “ilerleme mümkün değil” durumunda kullanın; normal doğrulamalarda non-fatal biriktirme tercih edin._

---

## 10) Yol haritası (gelecek ama **bu paket yapmayacak**, Vex yapmalı)

> Yalnızca yönlendirme (uygulama sizde):

- **Parallel Bench**: `vex_bench_run_parallel(fn, ctx, workers)` ve CPU listesi (`-cpu=1,2,4…`).
- **Alloc raporu**: `mallinfo2()` farkı ya da kendi **arena sayacı** ile `allocs/op`, `B/op`.
- **CLI bayrakları**: `-benchtime=`, `-count=`, `-cpu=`, `-json` (vex test runner’ın işi).
- **Profil**: `perf_event_open` ile CPU sayaçları; sonuçları JSON’a ekleyin.
- **Timeout**: test/bench başına deadline.
- **Coverage/Fuzz**: ayrı modül; bu pakete dokunmayın.

---

## 11) Sık yapılan hatalar (Dikkat!)

- **Timer’ı unutmak**: sıcak bölüm `start/stop` arasında olmalı; yoksa hazırlıklar ölçülür.
- **Bytes ayarlamamak**: MB/s = 0 görünür; throughput’lar kıyaslanamaz.
- **DCE’ye yakalanmak**: `black_box` kullanmadığınızda derleyici işinizi “optimize” edebilir.
- **Pinleme olmadan kıyas**: jitter artar, karşılaştırmalar güvenilmez.
- **Tek tekrar**: istatistik yoksunluğu → yanlış sonuç okuma.

---

## 12) Kısa kontrol listesi

- [ ] Testler: `VEX_TLOG/TERROR/ASSERT/SKIP` doğru kullanılıyor
- [ ] Alt testler: `vex_subtest` ile gruplandı
- [ ] Bench fonksiyonlarında **timer kontrolü** var
- [ ] **Bytes/op** ayarlanıyor → MB/s raporlanıyor
- [ ] `auto_calibrate` veya `time_ns` ile makul ölçüm penceresi
- [ ] (x86) cyc/op etkin; değilse ns/op’a bakıyoruz
- [ ] `repeats ≥ 5`, p90/p95/p99 gözlemleniyor
- [ ] CI’de pinleme & realtime ipucu açık

---

Sorun olursa ya da Vex runner/CLI tarafına bayrak taslağı istersen (ör. `vex test -run`, `vex bench -benchtime=1s -count=5 -json`), hızlıca bir öneri seti de yazabilirim.
