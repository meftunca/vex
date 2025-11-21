# Vex String / Str Türü - Bulgular ve Öneriler (Türkçe)

Bu doküman Vex dilindeki `str` ve `String` (kullanılan `VexStr`/`vex_string_t`) özelliklerini inceleyip Rust ve Go ile karşılaştırarak eksiklikleri, riskleri ve öncelikli önerileri özetler.

---

## ⚡ Özet

- Vex’te iki temel string temsili var: `str` (sıfır-kopya görünüm, `VexStr`) ve `String` (heap'lenmiş, `vex_string_t`).
- Runtime (C) seviyesinde hızlı, SIMD destekli temel string operasyonları mevcut: `strlen`, `strcmp`, `contains`, `replace`, UTF-8 doğrulama ve dönüştürme işlevleri (`vex_utf8_*`).
- Standart kütüphanedeki `vex-libs/std/string/src/lib.vx` Go-benzeri bazı fonksiyonları sunuyor; fakat API tutarsız (bazı fonksiyonlar `str`, bazıları `String` kullanıyor), eksik ve birkaç yerde `fmt` ile entegrasyon tamamlanmamış.

---

## 📌 Mevcut Durum (Öne çıkan dosyalar)

- `vex-libs/std/string/src/lib.vx` — Yüksek seviye string API (contains, prefix/suffix, case, replace, vb.)
- `vex-runtime/c/vex_string.c` — Hızlı string operasyonları (SIMD)
- `vex-runtime/c/vex_string_type.c` — `vex_string_t` (owned string) implementasyonu
- `vex-libs/std/fmt/src/lib.vx` — Biçimleme (format) API’si; çoğu stub durumda
- `vex-libs/std/path`, `vex-libs/std/fs`, `vex-libs/std/time` — `str` kullanan diğer modüller; olanaklar ve TODO’lar var

---

## 🔬 Rust ve Go ile Karşılaştırma (Kısa)

- Ownership & Mutability

  - Rust: `&str` (borrowed), `String` (owned). Borrow-checker net tanımlar sağlar.
  - Go: `string` immutable; `[]byte` mutable; GC yönetiyor.
  - Vex: `str` (görünüm) ve `String` (owned) var; ancak API'lerde tutarsızlık ve belirsizlik var (kimin `str` alacağı/`String` dönüşümü belirsiz).

- UTF-8 Güvenliği ve Politikası

  - Rust: `String` daima geçerli UTF-8; `from_utf8_unchecked` unsafe.
  - Go: `string` bayt dizisi; UTF-8 konvansiyonel; `utf8` paket destekleri var.
  - Vex: `is_valid_utf8`, `from_utf8`, `from_utf8_unchecked`; ama standart kütüphanede bazı dönüşümler otomatik doğrulama yapmıyor.

- Dilimleme ve İndeksleme

  - Rust: `&str` byte-dilimleme; kodpoint/index karmaşık ama `chars()` sağlanıyor.
  - Go: `s[i]` byte döndürür; rune için `for range`.
  - Vex: `vex_string_slice` ve `VexStr` yardımıyla dilim sağlanıyor; ancak `string` modülünde ergonomik `slice`/`substring` ve `chars()` eksik.

- Arama, Bölme, Birleştirme

  - Rust/Go: `contains`, `starts_with`, `ends_with`, `index`, `split`, `join`, `replace` vb. zengin araçlar.
  - Vex: `contains`, `has_prefix`, `has_suffix`, `replace` var; `split`, `join`, `index`/`last_index`, `split_whitespace` eksik.

- Mutation ve Concatenation

  - Rust: `push_str`, `push`, `insert`, `+` ve `format!`.
  - Go: `strings.Builder` kullanımını önerir.
  - Vex: Runtime `vex_string_push_str` var; high-level wrapper (builder, push, reserve) eksik.

- Formatting

  - Rust: `format!`, traits `Display`/`Debug`.
  - Go: `fmt.Sprintf`, `fmt.Println`.
  - Vex: `fmt` modülü mevcut, fakat fonksiyonların büyük bir kısmı hâlâ stub; dönüşümler `*u8` → `String` yapılmıyor.

- Unicode İleri Düzeyleri
  - Normalizasyon, graps/cluster, locale-aware case conversion: Rust/Go ek kütüphanelerle sağlanır.
  - Vex: Temel UTF-8 doğrulama ve UTF-16/32 dönüştürmeleri var; normalizasyon veya grapheme cluster API’leri yok.

---

## ⚠️ Tespit Edilen Eksiklikler (Özlü)

- API tutarsızlığı: `str` ve `String` kullanımı karışık (örn: `contains(s: str)`, `has_suffix(s: String)`) → kafa karışıklığı ve hatalara yol açıyor.
- `fmt` modülündeki işlevlerin çoğu hâlen placeholder döndürüyor; bunlar `*u8` rt dönüşlerini `String` objelerine çevirmiyor.
- Eksik fonksiyonlar: `split`, `join`, `index`, `last_index`, `substring`, `split_whitespace`, `replace_all`, `replace_first`, `trim_left`, `trim_right`.
- Iterasyon (chars/bytes) kolaylaştırıcıları eksik: `chars()`/`bytes()` iteratörleri yok.
- Unicode alanında: normalization, grapheme clusters, locale-aware casing yok.
- `String` builder/append API’si eksik: `push_str`, `reserve`, `capacity` wrapperları yetersiz.
- Test kapsaması eksik: Unicode edge case’leri, slice semantics, ownership/borrow semantics testleri zayıf.

---

## ✅ Öneriler (Öncelikli ve Uygulanabilir)

Aşağıda pratik, önceliklendirilmiş bir yol haritası bulunuyor.

### Yüksek Öncelikli (Hızlı kazanımlar)

1. **API Tutarlılığını Sağla**

   - Read-only fonksiyonlar `str` almalı; owned dönüşler `String` olmalı.
   - `vex-libs/std/string/src/lib.vx`'deki karışık imzaları düzelt.
   - Tüm standard modüllerde (`fmt`, `path`, `time`) tutarlı konvansiyon uygulanmalı.

2. **`String` ↔ `str` Yardımcıları**

   - `to_string(s: str): String` ve `as_str(s: String): str` koy.
   - `from_cstr` ve `from_utf8` dönüşlerini güvenli şekilde `String` üretir hale getir.

3. **`fmt` Tamamlama**

   - `vex-libs/std/fmt/src/lib.vx` içinde `vex_fmt_*`'in döndürdüğü `*u8` pointerları `String` objesine çevir.
   - `format`, `sprintf`, `println` gibi işlevleri tamamla.

4. **Temel String İşlevlerini Tamamla**

   - `split`, `join`, `index`, `last_index`, `substring` ve `split_whitespace` ekle.
   - `replace_all` ve `replace_first` ekle (runtime `vex_str_replace` varsa bunları sarmalayarak optimize et).

5. **`String` Builder ve Metotları**
   - `String.reserve`, `String.push_str`, `String.push_char`, `String.clear`, `String.clone`, `String.concat` wrapperlarını ekle.
   - Ayrıca `concat(s1: str, s2: str) -> String` gibi sıfırla-heap-atanmış fonksiyonlar ekle.

### Orta Öncelikli

6. `chars()`/`bytes()` iteratörleri, `len_bytes` vs `len_chars` karışıklığını gider.
7. `split` için zero-copy option (`Vec<str>`) ve owned option (`Vec<String>`).
8. Testler: ASCII, multi-byte, emoji, combining marks, boundary indices kapsamlı testler.

### Uzun Vadeli / İleri Düzey

9. Unicode normalizasyon (NFC/NFKC) ve grapheme cluster destekleri.
10. Locale-aware case folding ve collation.
11. Daha fazla optimizasyon: SIMD destekli `split` / `index` (runtime düzeyinde var, stdlib’e yayılmalı).

---

## 🔧 Uygulama Örnekleri ve Teknik Notlar

- `lib.vx` içinde read-only fonksiyonlar `s: str` alacak şekilde yeniden imzalandığında tüm çağrıları güncelle:

  - Örnek: `export fn has_suffix(s: str, suffix: str) -> bool` (eski `String` imzalarını değiştirin)

- `*u8` → `String` dönüşü çevrimi:

  - Runtime `vex_str_to_upper` gibi fonksiyonlar `*u8` döndürüyor. `lib.vx`'de bu `*u8`returned pointer’ı `string.from_cstr()` veya `vex_string_from_cstr` sarmalaması ile owned `String` haline getir.

- Split örneği (kullanıcı API’si):

```vex
export fn split(s: str, sep: str): Vec<str> {
    // Zero-copy: VexStr view’lar üretip Vec<str> döndür
}

export fn join(parts: Vec<str>, sep: str): String {
    // StringBuilder veya precompute len + allocate + push_str
}
```

- StringBuilder örneği:

```vex
export struct StringBuilder {
    s: String
}

export fn new_builder(capacity: u64): StringBuilder {
    // allocate via vex_string_with_capacity + wrap
}

export fn (b: &StringBuilder) push_str(s: str) {
    // call vex_string_push_str
}

export fn (b: StringBuilder) into_string(): String { return b.s; }
```

---

## 🧪 Test Önerileri

- Unicode sınır koşulları: emoji, combining marks, surrogate pairs
- Boundary indices: `substr` kesimleri multibyte arasından geçmeyecek
- Ownership testleri: `String` mutate sonrası orijinal `str` view’ın geçerliliği
- Performance testleri: `concat` vs `StringBuilder`

---

## 📅 Kısa Yol Haritası (Önümüzdeki 2-3 sprint için öneri)

1. _Sprint 1_ (1-2 hafta)

   - `lib.vx` fonksiyon imzalarını netleştir: read-only -> `str`, owned -> `String`.
   - `fmt` içindeki dönüşleri düzelt ve birkaç `fmt` fonksiyonunu `String` döndürür hale getir.
   - 10 temel unit test ekle: format, contains, prefix, suffix

2. _Sprint 2_ (2-3 hafta)

   - `String` builder ve push/append metotları.
   - `split`/`join`, `index`/`last_index`, `split_whitespace`.
   - Test genişletme: unicode/emoji/edge-case testleri.

3. _Sprint 3+_ (İleri düzey)
   - Grapheme cluster, normalization, locale-aware case dönüştürmeleri.
   - Performans iyileştirmeleri ve SIMD’yi daha fazla methoda yayma.

---

## Sonuç

Vex’in string runtime’ı (C tarafı) güçlü ve optimize edilmiştir; bu, yüksek performanslı string operasyonları sağlayacaktır. Ancak mevcut standart kütüphane (Vex dilinde) API tasarımı, eksik fonksiyonlar ve `fmt` ile uyumluluk konularında tamamlanmaya ihtiyaç duyuyor. Önerilen adımları uygulamak, Vex dilinin string UX’ini geliştirip Rust/Go ile karşılaştırılabilir bir zenginliğe kavuşturacaktır.

---

Eğer isterseniz bu dokümandaki ilk değişiklikleri (ör. `lib.vx` içindeki tutarsız imzaların düzeltilmesi veya `fmt` dönüşlerinin `String` haline getirilmesi) kodlayıp testlerle birlikte PR açabilirim. Hangi adımı önceliklendireyim?
