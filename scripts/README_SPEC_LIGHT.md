# Specification Optimization Scripts

Bu klasörde Vex spesifikasyon dosyalarını optimize etmek için iki alternatif script bulunur.

## 📊 Performans Karşılaştırması

| Script             | Dil          | Hız           | Boyut | Okunabilirlik |
| ------------------ | ------------ | ------------- | ----- | ------------- |
| `mk_spec_light.py` | Python 3     | **0.063s** ⚡ | 248K  | ⭐⭐⭐⭐⭐    |
| `mk_spec_light.sh` | Bash/AWK/sed | 0.361s        | 248K  | ⭐⭐⭐        |

**Python versiyonu 5-6x daha hızlı!**

---

## 🐍 Python Version (Önerilen)

**Avantajları:**

- ✅ 5-6x daha hızlı
- ✅ Daha okunaklı kod
- ✅ Unicode desteği native
- ✅ Kolay bakım ve genişletme
- ✅ Test edilebilir

**Kullanım:**

```bash
# Varsayılan (MAX_CODE_LINES=10)
./scripts/mk_spec_light.py

# Daha agresif optimizasyon
MAX_CODE_LINES=8 ./scripts/mk_spec_light.py

# Tüm kod bloklarını özetle
PRESERVE_SMALL_CODE=0 ./scripts/mk_spec_light.py

# Özel kaynak/hedef dizinler
./scripts/mk_spec_light.py Specifications spec_light
```

---

## 🐚 Bash Version (Fallback)

**Avantajları:**

- ✅ Dependency yok (sadece standard Unix tools)
- ✅ macOS/Linux uyumlu

**Dezavantajları:**

- ❌ 5-6x daha yavaş
- ❌ Karmaşık AWK/sed syntax
- ❌ Bakımı zor

**Kullanım:**

```bash
# Varsayılan
./scripts/mk_spec_light.sh

# Parametrelerle
MAX_CODE_LINES=8 ./scripts/mk_spec_light.sh
```

---

## 📝 Optimizasyon Detayları

Her iki script de aynı optimizasyonları yapar:

### 1. Kod Bloklarını Özetleme

- Küçük kod blokları (≤ MAX_CODE_LINES): Korunur
- Büyük kod blokları (> MAX_CODE_LINES): `[N lines code: lang]` şeklinde özetlenir

### 2. Görsel/Link Basitleştirme

- `![alt](url)` → `[Image: alt]`
- `[text](url)` → `text`
- Badge/shield görselleri atılır

### 3. Formatlamayı Kaldırma (Agresif)

- `**bold**` → `bold`
- `*italic*` → `italic`
- `***bold+italic***` → `bold+italic`
- `__bold__` → `bold`
- `_italic_` → `italic`

### 4. Başlık Normalizasyonu

- Maksimum 3 seviye (`###`)
- Trailing `#` karakterleri temizlenir

### 5. Tablo Sadeleştirme

- Tablo satırları liste formatına dönüştürülür
- `| A | B | C |` → `• A — B — C`

### 6. Boşluk Optimizasyonu

- Fazla boşluklar temizlenir
- Art arda boş satırlar azaltılır

### 7. Metadata Temizleme

- YAML front-matter atılır
- HTML yorumları atılır
- Table of Contents bölümleri atılır
- Dosya sonu "Maintained by" / "Last Updated" footer'ları temizlenir

---

## 📈 Sonuçlar

**Orijinal:**

- Boyut: 356K
- Satır: 14,957

**Optimized (MAX_CODE_LINES=8, TOC/Footer removed):**

- Boyut: 240K (**33% azalma**)
- Satır: 9,786 (**35% azalma**)
- Context: **Korunuyor** ✅
- Anlamsal bütünlük: **Bozulmuyor** ✅
- GitHub Spaces için ideal ⚡

**Ultra Optimized (MAX_CODE_LINES=6):**

- Boyut: 232K (**35% azalma**)
- Satır: ~9,300 (**38% azalma**)

---

## 🎯 Öneriler

1. **Geliştirme için**: Python versiyonunu kullanın (hızlı, maintainable)
2. **CI/CD için**: Python versiyonu (dependency: `python3`)
3. **Minimal sistem için**: Bash versiyonu (sadece Unix tools)
4. **GitHub Spaces için**: `MAX_CODE_LINES=8` optimal (context korunur, boyut %33 azalır)

---

**Version**: 1.1.0  
**Last Updated**: November 11, 2025
