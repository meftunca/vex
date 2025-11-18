# vex_fastenc — SIMD‑Hızlandırılmış Base16/Base32/Base64 & UUID (C)

**Hedef:** Yaygın binary‑text dönüşümleri ve UUID üretimi için **taşınabilir** ve **hızlı** bir C kütüphanesi.  
**SIMD:** Hex (Base16) için **AVX2 / AVX‑512BW / NEON** tam vektör yolu. Base64 için **SIMD‑yardımlı decode** (sınıflandırma SIMD, paketleme güvenli/kompakt). Tüm yollar **güvenli scalar** fallback ile birlikte.

## 📦 Modüller ve *kategori* eşlemesi

- `encoding/hex`   → `vex_hex_encode` / `vex_hex_decode` (RFC 4648 Base16)  
- `encoding/base64`→ `vex_base64_encode` / `vex_base64_decode` (RFC 4648, **std** ve **URL‑safe** + padding/line‑wrap)  
- `encoding/base32`→ `vex_base32_encode` / `vex_base32_decode` (RFC 4648 **Base32**, **Base32hex**, **Crockford**)  
- `id/uuid`        → `v1,v3,v4,v5,v6,v7,v8` üretim, `parse/format`  
- `crypto/hash`    → `md5` (v3), `sha1` (v5)  
- `crypto/random`  → `vex_os_random` (OS CSPRNG)

> Standart referanslar: Base16/32/64 için **RFC 4648**; UUID v1..v5 için **RFC 4122**; v6/v7/v8 için **RFC 9562** / IETF taslakları.  

## 🧩 API Özeti

```c
/* Hex */
size_t vex_hex_encode(const uint8_t* src, size_t n, char* dst, int uppercase);
ssize_t vex_hex_decode(const char* src, size_t n, uint8_t* dst);

/* Base64 */
typedef struct { vex_b64_alphabet alpha; int pad; int wrap; } vex_b64_cfg;
size_t vex_base64_encode(const uint8_t* src, size_t n, char* dst, vex_b64_cfg cfg);
ssize_t vex_base64_decode(const char* src, size_t n, uint8_t* dst, vex_b64_alphabet alpha);

/* Base32 */
typedef struct { vex_b32_alphabet alpha; int pad; } vex_b32_cfg;
size_t vex_base32_encode(const uint8_t* src, size_t n, char* dst, vex_b32_cfg cfg);
ssize_t vex_base32_decode(const char* src, size_t n, uint8_t* dst, vex_b32_alphabet alpha);

/* UUID */
int vex_uuid_v1(vex_uuid* u);  int vex_uuid_v3(vex_uuid* u, const vex_uuid* ns, const void* name, size_t len);
int vex_uuid_v4(vex_uuid* u);  int vex_uuid_v5(vex_uuid* u, const vex_uuid* ns, const void* name, size_t len);
int vex_uuid_v6(vex_uuid* u);  int vex_uuid_v7(vex_uuid* u);  int vex_uuid_v8(vex_uuid* u, const uint8_t custom[16]);
int vex_uuid_format(char out[37], const vex_uuid* u); int vex_uuid_parse(const char* s, vex_uuid* out);
```

## 🚀 SIMD Yolları

- **Hex**: AVX2 (32B/iter), AVX‑512BW (64B/iter), NEON (16B/iter). Nibble→ASCII dönüşümü ve çift‑bayt **interleave** tamamen vektörize.  
- **Base64 decode**: AVX2/NEON ile **sınıflandırma** (A‑Z, a‑z, 0‑9, +/‑, /_/). Paketleme (4×6‑bit → 3 byte) güvenli ve taşınabilir bir bit‑işlem hattı ile tamamlanır.  
- **Base32**: Scalar ama branch‑light; RFC 4648 ve varyant alfabeler **eksiksiz**. SIMD hook’ları (gather/shuffle maskeleri) ayrılmıştır.

> Tam vektörleştirilmiş Base64 encode/decode (24→32 / 32→24) AVX2/NEON kalıpları için iskelet ayrıldı; gerekirse ekleyebilirim.

## 🔐 UUID Varyantları

- **v1**: 100ns (1582 epoch) + clockseq + node (rastgele multicast)  
- **v3/v5**: Ad tabanlı (MD5/SHA‑1)  
- **v4**: Tam rastgele  
- **v6**: Zaman alanları **yeniden düzenlenmiş** (sıralanabilir)  
- **v7**: Unix epoch **ms** + rastgele alanlar (sıralanabilir)  
- **v8**: Serbest form (sürüm/çeşit bayrakları korunur)

## 🧪 Derleme ve Test

```bash
make
./tests/test_vectors
./bench/bench
```

## ⚠️ Notlar

- AVX/AVX‑512 için **OS XSAVE** etkin olmalı (Linux/Windows).  
- v1/v6 node kimliği MAC yerine **rand48** (multicast) ile üretilir; MAC gereksinimi varsa `uuid_all.c` içinde `random_node()` fonksiyonunu özelleştirin.  
- v7 için düzenleme **RFC 9562** ile uyumludur (48‑bit ms + rastgele alanlar).

## Lisans

MIT‑benzeri; proje içine gömülebilir.
