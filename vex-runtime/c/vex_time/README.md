# Vex Time — Ultra‑Full Pack (Go Layout + TZ + Strict Timers + io_uring Cancel)

**Tamamlayıcı** Vex zaman C‑ABI:

- Go layout `Format/Parse`: `January/Jan`, `Monday/Mon`, `02/2/_2`, `2006/06`, `15/03/3 + PM`, `04/4`, `05/5`, `002` (yılın günü), `.000/.000000/.000000000`, `MST`, `-0700`, `-07:00`, `Z07:00`.
- TZ:
  - POSIX: TZif v2/v3 okuyucu + `VT_TZDIR`/`vt_tz_set_dir()` ile konum aşımı
  - Windows: yerel saat dilimini **Windows→IANA** eşlemesiyle çözme (ör. _Turkey Standard Time → Europe/Istanbul_). VT_TZDIR sağlanırsa oradan TZif yüklenir.
  - `vt_tz_fixed`, `vt_tz_utc`, `vt_tz_load_from_memory()` ile gömülü kullanımlar.
- Scheduler:
  - POSIX/Windows: tek thread + min‑heap, **strict cancel** (heap’ten çıkarma)
  - Linux: opsiyonel **io_uring** zamanlayıcı yolu, `TIMEOUT_REMOVE` ile **deterministik iptal**.
- RFC3339 ve Duration parse/format mevcut.

## Derleme

```bash
make            # POSIX
./examples/full_demo

# Windows
cmake -B build -S .
cmake --build build --config Release
./build/full_demo.exe

# Linux'ta io_uring sürümü (liburing gerekir)
make uring
```

Aşağıdaki sürüm; Go `time` paketindeki kavramların tamamını hedefleyen bir C‑ABI yüzeyi, gelişmiş Go‑layout **Format/Parse**, **IANA TZif** yükleme, **Windows↔IANA** eşlemesiyle **yerel saat dilimi çözümü**, **strict cancel** timer/ticker ve **Linux’ta io_uring ile deterministik iptal** içerir. Go’nun `Time/Duration/Timer/Ticker` semantiğini ve “magic date” layout sistemini baz aldım.

---

## Bu sürümde neler **tamamlandı**?

### 1) Go Layout Format/Parse — **geniş kapsam (tamamlandı)**

- **Desteklenen tüm yaygın token’lar**:
  **Yıl** `2006`/`06`, **Ay** `January`/`Jan`/`01`/`1`, **Gün** `02`/`2`/**`_2`** (boşluk dolgu), **Haftanın günü** `Monday`/`Mon`,
  **Saat** `15` (24s) / `03` veya `3` + `PM`/`pm`, **Dakika** `04`/`4`, **Saniye** `05`/`5`,
  **Yılın günü** **`002`** (örn. 001–366), **Kesirli saniye** `.000`/`.000000`/`.000000000`,
  **Bölge** `MST` / `-0700` / `-07:00` / `Z07:00` (UTC için `Z`).`MST` layout token’ı **metin kısaltmasını** işler; ofset türetmek için sayısal bölge (`-07:00`, `Z07:00`) önerilir.
- **İsimli değerleri parse**: `January/Jan`, `Monday/Mon` girişleri artık **parse** ediliyor (daha önce sadece format vardı).
- **Boşluk dolgulu gün (`_2`)** ve **yılın günü (`002`)** eklendi.
- RFC3339 için ayrıca `vt_parse_rfc3339`/`vt_format_rfc3339_utc` korunuyor.

### 2) Zaman dilimi (TZ) — **POSIX + Windows tamamlama**

- **POSIX**: `vt_tz_load("Europe/Istanbul")` doğrudan **TZif v2/v3** dosyalarını okur. `vt_tz_set_dir()` veya `VT_TZDIR` ile özel zoneinfo dizini belirtilebilir (örn. container içinde).
- **Yerel TZ çözümü (POSIX)**: `/etc/localtime` → `zoneinfo/...` symlink yolunu okuyup doğru IANA adıyla yükler; bulunamazsa **o anki ofsete** sabitlenir.
- **Windows**: `vt_tz_local()` **Windows→IANA** eşlemesiyle (örn. _“Turkey Standard Time” → “Europe/Istanbul”_) IANA adını bulur; `VT_TZDIR`/`vt_tz_set_dir()` verildiyse TZif dosyasını oradan yükler; aksi hâlde **sistem bias’ına sabit** (geçici) düşer.
- **Gömülü kullanım**: `vt_tz_load_from_memory(name, bytes, len)` ile kendi TZif’nizi binary’e **embed** edip runtime’da yükleyebilirsiniz.
- **Sorgu**: `vt_tz_offset_at(tz, utc, &offset, &abbr)` her an için **doğru ofset/kısaltmayı** verir; `vt_utc_to_tz(tz, utc)` ofset uygulayıp lokal an döndürür.
- Go `time.Location` benzeri kullanım ve DST geçişleri için IANA TZif ile tam doğruluk sağlanır.

### 3) Timer/Ticker — **strict cancel** (POSIX/Windows) + **io_uring** (Linux)

- **POSIX/Windows**: Tek worker thread + min‑heap; `Stop/Reset` **heap’ten gerçek silme** yapar (yarışı azaltır).
- **Linux io_uring (opsiyonel)**: `vt_sched_create_uring()` ile `IORING_OP_TIMEOUT` tabanlı yol.
  - **Deterministik iptal**: `TIMEOUT_REMOVE` ile **tam iptal** (key = `user_data`); `vt_timer_stop`/`vt_ticker_stop` bunu kullanır.
- Kullanım Go’daki `Timer`/`Ticker` ergonomisine paralel: `create/start/reset/stop/destroy`.

### 4) Duration ve monosecond semantiği (Go ile uyumlu)

- `vt_parse_duration("1h30m", "250ms", "-1.25h", "500us/µs", "10s")` — `time.ParseDuration` muadili.
- `VexTime` **duvar (UTC) + monotonik** okuma taşır; `vt_sub`/`vt_since`/`vt_until` öncelikle **monotonik** kaynağı kullanır.

---

## Dosya yapısı

```javascript
vex_time_ultrafull/
  include/vex_time.h
  src/common/vex_time_common.c
  src/common/tz_and_windows.c
  src/posix/time_posix.c           # Linux/macOS/BSD
  src/win/time_win.c               # Windows
  src/linux/time_uring.c           # (opsiyonel) -DVEX_TIME_HAVE_URING
  examples/full_demo.c
  Makefile
  CMakeLists.txt
  README.md

```

---

## Derleme & Çalıştırma

**POSIX (Linux/macOS/BSD):**

```javascript
make
./examples/full_demo

```

**Windows (MSVC/MinGW, CMake):**

```javascript
cmake -B build -S .
cmake --build build --config Release
./build/full_demo.exe

```

**Linux’ta io_uring yolu (opsiyonel):**

```javascript
# liburing kurulu olmalı (ör. Ubuntu: liburing-dev)
make uring

```

> **Windows’ta TZ**: IANA TZif dosyalarını yanına koyup `vt_tz_set_dir("C:/tzdb")` veya `set VT_TZDIR=C:\tzdb` ile belirt. Yerel saat dilimi IANA adına çözüldüğünde buradan yüklenecek; aksi hâlde **sabit ofset** fallback’i kullanılır (bu fallback DST kurallarını modellemez).

---

## API kısayol kılavuzu

- **An/Süre**:
  `vt_now`, `vt_monotonic_now_ns`, `vt_add`, `vt_sub`, `vt_since`, `vt_until`,
  `vt_parse_duration`, `vt_format_duration`.

- **Format/Parse**:
  `vt_format_rfc3339_utc`, `vt_parse_rfc3339`, `vt_format_go`, `vt_parse_go`.

- **TZ**:
  `vt_tz_utc`, `vt_tz_fixed`, `vt_tz_local`, `vt_tz_set_dir`, `vt_tz_load`, `vt_tz_load_from_memory`, `vt_tz_offset_at`, `vt_utc_to_tz`.

- **Zamanlayıcılar**:
  `vt_sched_create[ _uring ]`, `vt_timer_*`, `vt_ticker_*`.

---

## Örnekler

Paketteki `examples/full_demo.c`:

- **Go layout** ile isimli ay/gün + yılın günü (`002`) formatı ve parse örneği,
- **TZ yükleme** (`Europe/Istanbul`),
- **Ticker** ile periyodik tetik + `stop` ile **strict cancel**.

Kendi uygulamanda callback içinde Vex **channel**’ına `try_send` yaparak Go’daki `Timer.C`/`Ticker.C` kanal deneyimini birebir yakalayabilirsin.

## Özet karşılaştırma

| Alan                                       | Vex+C‑ABI (bu paket)                                                                                                     | Go `time` (referans)                                                                                    | Sonuç                                                                                            |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| **`now()`&#32;çağrısı (monotonic + wall)** | POSIX’te `clock_gettime` (vDSO varsa syscallsız), Windows’ta `QPC` + `GetSystemTimePreciseAsFileTime`.                   | Linux’ta `time.Now()` da vDSO’yu kullanır; Windows’ta `QPC` tabanlı.\n\n                                | **Başa baş** (ns–µs bandı).                                                                      |
| **`Duration`&#32;parse/format**            | C’de el yazması ayrıştırıcı; `strtold` kullanımı var (ondalıklar için).                                                  | `time.ParseDuration` güçlü ve kapsamlı.\n\n                                                             | Kompakt “sıcak yol”larda **eşit veya biraz hızlı**, çok yoğun ondalık kullanımında **Go ≈ Vex**. |
| **RFC3339 parse/format**                   | Minimalist, tek amaçlı ayrıştırıcı/biçimleyici.                                                                          | `time.Parse / Format` genel amaçlı layout sistemi.\n\n                                                  | Sadece RFC3339 ise **Vex genelde daha hızlı** (daha az dallanma).                                |
| **Go‑layout Format/Parse**                 | “yaygın token” alt kümesini kapsamlı şekilde destekliyor.                                                                | Tam kapsamlı layout.\n\n                                                                                | Tam kapsama gerek yoksa **Vex hızlı**; tüm egzotik layout’lar gerekirse **Go daha esnek**.       |
| **Timer/Ticker (az sayıda)**               | Tek thread + min‑heap; lock içeren ama hafif; Linux’ta opsiyonel **io_uring timeout**.                                   | Runtime içinde şard edilmiş timer yapıları + netpoller ile entegre; `Timer/Ticker` kanal semantiği.\n\n | **Az/orta yükte başa baş**, `io_uring` ile **daha düşük wakeup maliyeti** mümkün.                |
| **Timer/Ticker (çok sayıda, 10⁵+)**        | Tek heap ve tek mutex olduğu için **ölçeklenme** baskısı artar (O(log n) + tek‑lock).                                    | Go runtime timer’ları çalışma zamanı ile birlikte ölçeklenir; yüksek sayıda timer için iyi bilinir.\n\n | **Go genelde üstün** (daha iyi şardlama/entegrasyon).                                            |
| **Gecikme/jitter**                         | POSIX’te `nanosleep` + koşul değişkeni; Windows’ta **hi‑res waitable timer**; Linux’ta **`IORING_OP_TIMEOUT`** opsiyonu. | Runtime scheduler ile sıkı entegrasyon; `time.AfterFunc` Goroutine tetikliyor.\n\n                      | Kernel/OS’e bağlı; **io_uring** yolunda Vex düşük jitter yakalayabilir.                          |

> Go `time` paketinin temel davranışları (monotonik saat, `Timer/Ticker` kanal temelli API, layout sistemi) için referans: **.md**.

---

## Detaylı gerekçe

### 1) `vt_now()` vs `time.Now()`

- **POSIX:** `clock_gettime(CLOCK_MONOTONIC/REALTIME)` çağrısı modern Linux’ta **vDSO** üzerinden kullanıcı‑modu okunur; Go da aynısını yapar. Bu yüzden **çağrı başına gecikme aynı sınıfta** (tipik olarak yüzlerce ns–düşük µs).
- **Windows:** her iki tarafta da **QPC** kullanımı benzer; biz ek olarak wall‑clock için `GetSystemTimePreciseAsFileTime` çağırıyoruz; Go da duvar saatini benzer şekilde elde ediyor. **Başa baş**.

### 2) Ayrıştırma/biçimleme (Duration, RFC3339, Go‑layout)

- **RFC3339:** Bizim kod net RFC3339 için optimize ve dar kapsamlı → Go’nun genel amaçlı `Parse`’ına göre **daha az dallanma** ve **olasılıkla daha düşük gecikme**.
- **Go‑layout:** Sık kullanılan token’ları kapsayan, dallanması düşük bir yol; **tam Go kapsamı** yerine pratik alt küme → **çoğu gerçek formatta hızlı**. Çok egzotik layout’lara gerek varsa Go daha esnek.

- **Duration:** Bizim ayrıştırıcı sade ama `strtold` (uzun ondalıklar) pahalı olabilir; burada “tam sayı tabanlı ayrıştırma”ya geçersen **Go ile net başa baş** (hatta küçük üstünlük) yakalarsın.

### 3) Timer/Ticker

- **Az/Orta sayıda timer:** Tek iş parçacıklı min‑heap (O(log n)) ve tek lock ile basit; **düşük contention**. Go’da da timer’lar runtime ile tek yerde yönetilir; bu seviyede **performanslar yakın**. Go’nun `Timer/Ticker`’ının kanal semantiğini Vex’te callback→channel köprüsüyle sağlıyorsun.

- **Çok sayıda timer (10⁵+):** Bizde global heap/lock ölçeklenmeyi sınırlayabilir. Go runtime tarafında timer verisi **işletim sistemi netpoller ve scheduler ile** daha sıkı entegredir; “yüksek miktarda timer” durumlarında **Go genelde daha iyi ölçeklenir**.

- **io_uring (Linux):** `IORING_OP_TIMEOUT` + `TIMEOUT_REMOVE` ile **deterministik iptal** ve düşük wakeup maliyeti elde edersin. Bu, çok sayıda kısa süreli timer pattern’inde **Vex’e avantaj** sağlayabilir (özellikle context‑switch ve sysenter sayısı azaldığında).

---

## Vex tarafında “Go’yu geçmek” için önerilen ayarlar

1. **Timer sharding:** Tek heap yerine **N shard** (CPU başına 1 heap) + work‑stealing → kilit rekabetini dramatik düşürür.
2. **Batch işlemleri:** Çoklu `reset/stop/start` için tek lock altında **batch** kabulü.
3. **Coalescing:** Aynı ms diliminde tetiklenecek timer’ları birleştir (tickless uyku zaten var).
4. **Duration ayrıştırıcıyı** tamamen **tamsayı tabanlı** yap (ondalıklı parçayı da tamsayı mantıkla ölçekle) → `strtold`’dan kaçın.
5. **`CLOCK_MONOTONIC_RAW`/Windows High‑res** (ihtiyaca göre): jitter kritik ise “raw” saat (NTP düzeltmelerinden bağımsız) tercih edilebilir.
6. **io_uring** için **tek ring → çoklu ring** (shard başına bir ring) ve **affinity**.
7. **Zaman dilimi** istekleri çok yoğunsa `vt_tz_offset_at` için **küçük LRU cache** (son 1–2 geçiş ve civarı).

---

## Nasıl ölçelim? (pragmatik kıyas planı)

**1)&#32;`now()`&#32;gecikmesi**

- C: 1e7 kez `vt_monotonic_now_ns()` + `vt_now()` döndür; medyan ve p99 süreyi ölç.
- Go: 1e7 kez `time.Now()` aynı süreçte.

**2) RFC3339 parse/format throughput**

- C: 1 M satır RFC3339 dizesi (aynı uzunluk) → `vt_parse_rfc3339`/`vt_format_rfc3339_utc` QPS.
- Go: `time.Parse(time.RFC3339, s)` ve `t.Format(time.RFC3339)`.

**3) Timer jitter & throughput**

- 10k timerı 100 ms ± jitter ile ateşle; **p50/p99 fire‑lateness** histogramı çıkar.
- Aynısını Go’da `time.NewTimer` / `time.AfterFunc` ile yap; `GOMAXPROCS` ve CPU izolasyonu sabit.

> Deneyleri **aynı çekirdeğe pinleyip** (Linux: `taskset`), DVFS’i mümkünse sabitleyip, benzer garbage‑collector/goroutine koşuları göz önüne alarak çalıştırmak önemli. Go tarafında GC dalgalanması t=ms düzeyinde jitter yaratabilir; Vex+C’de bu yok ama callback’ten sonra **Vex kanalına push** aşamasında yük olabilir.

---

## Sonuç (pratik karar)

- **Genel kullanımda** (şu anki paketle): **Go ile başa baş**.
- **Sadece RFC3339/klasik formatlar ve düşük/orta timer yükü**: **Vex avantajlı** senaryolar var (daha az genel amaçlı kod, `io_uring` ile düşük wakeup maliyeti).
- **Aşırı çok timer ve yoğun eşzamanlı reset/stop**: Go runtime’ın iç entegrasyonu nedeniyle **Go genelde üstün**.
- **Ama** önerdiğim sharding + batch + coalescing + integer‑only parse değişiklikleriyle, **Vex paketini belirli iş yüklerinde Go’nun üstüne taşımak mümkün**.
