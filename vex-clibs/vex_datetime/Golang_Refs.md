# ⏰ Golang `time` Paketi Kapsamlı Referansı

Bu doküman, Go'nun standart kütüphanesindeki `time` paketinin temel yapılarını, metotlarını ve özelliklerini (features) kapsamaktadır. `time` paketi, zamanın ölçümü, gösterimi ve süre (duration) hesaplamaları için tam tip güvenliği (strong typing) sağlar.

## I. Temel Yapılar (Structs) ve Tipler

`time` paketi, zaman ve süre kavramlarını birbirinden ayıran ve net bir şekilde tipleyen üç ana yapıya dayanır.

| Tip Adı             | Temsil Ettiği Kavram           | İçerdiği Bilgi                                                                         |
| ------------------- | ------------------------------ | -------------------------------------------------------------------------------------- |
| **`time.Time`**     | **Belirli Bir An** (Timestamp) | Yıl, Ay, Gün, Saat, Nanosaniye, **Zaman Dilimi (Location)** ve Monotonik Saat okuması. |
| **`time.Duration`** | **Süre** (Interval)            | İki an arasındaki farkı nanoseaniye cinsinden temsil eden bir `int64` alias'ı.         |
| **`time.Location`** | **Zaman Dilimi** (Time Zone)   | UTC ofseti ve Yaz Saati Uygulaması (DST) kuralları.                                    |
| **`time.Timer`**    | Tek bir gecikmeli olay.        | Belirlenen süre sonunda sinyal gönderen bir kanal içerir (`C chan Time`).              |
| **`time.Ticker`**   | Tekrarlayan periyodik olay.    | Belirlenen aralıklarla sürekli sinyal gönderen bir kanal içerir (`C chan Time`).       |

## II. `time.Time` Metotları (Bir Anın İşlemleri)

`time.Time` tipinin üzerinde tanımlı, zamanı sorgulama, dönüştürme ve karşılaştırma amaçlı metotlardır.

### A. Oluşturma ve Dönüştürme (Constructors & Conversion)

| Metot/Fonksiyon       | İmzası (Signature)                                                          | Açıklama                                                                        |
| --------------------- | --------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| `time.Now()`          | `func Now() Time`                                                           | Mevcut yerel sistem zamanını döndürür.                                          |
| `time.Date(...)`      | `func Date(year, month, day, hour, min, sec, nsec int, loc *Location) Time` | Belirtilen parametrelerle yeni bir `Time` nesnesi oluşturur.                    |
| `time.Parse(...)`     | `func Parse(layout, value string) (Time, error)`                            | Zaman dizisini, verilen format (layout) kullanarak `Time` nesnesine dönüştürür. |
| `t.In(loc *Location)` | `func (t Time) In(loc *Location) Time`                                      | Zamanın anını koruyarak, zaman dilimini değiştirir.                             |
| `t.Local()`           | `func (t Time) Local() Time`                                                | Zamanı yerel sisteme dönüştürür.                                                |
| `t.UTC()`             | `func (t Time) UTC() Time`                                                  | Zamanı UTC'ye dönüştürür.                                                       |

### B. Bileşen Çekme (Getters)

| Metot          | İmzası                               | Açıklama                               |
| -------------- | ------------------------------------ | -------------------------------------- |
| `t.Year()`     | `func (t Time) Year() int`           | Yıl (4 basamaklı)                      |
| `t.Month()`    | `func (t Time) Month() Month`        | Ay (Tip: `time.Month`)                 |
| `t.Day()`      | `func (t Time) Day() int`            | Ayın günü (1-31)                       |
| `t.Hour()`     | `func (t Time) Hour() int`           | Saat (0-23)                            |
| `t.Weekday()`  | `func (t Time) Weekday() Weekday`    | Haftanın günü (Tip: `time.Weekday`)    |
| `t.Unix()`     | `func (t Time) Unix() int64`         | Unix zaman damgası (saniye cinsinden). |
| `t.Location()` | `func (t Time) Location() *Location` | İlişkili zaman dilimi (Location).      |

### C. Hesaplama ve Karşılaştırma

| Metot                     | İmzası                                                | Açıklama                                                 |
| ------------------------- | ----------------------------------------------------- | -------------------------------------------------------- |
| `t.Sub(u Time)`           | `func (t Time) Sub(u Time) Duration`                  | `t` ile `u` arasındaki farkı `Duration` olarak döndürür. |
| `t.Add(d Duration)`       | `func (t Time) Add(d Duration) Time`                  | Zamana verilen süreyi ekler.                             |
| `t.AddDate(y, m, d int)`  | `func (t Time) AddDate(years, months, days int) Time` | Zamana takvim bileşenleri ekler.                         |
| `t.Before(u Time)`        | `func (t Time) Before(u Time) bool`                   | `t`, `u`'dan önce mi?                                    |
| `t.After(u Time)`         | `func (t Time) After(u Time) bool`                    | `t`, `u`'dan sonra mı?                                   |
| `t.Equal(u Time)`         | `func (t Time) Equal(u Time) bool`                    | Zamanın anı ve konumu aynı mı?                           |
| `t.Format(layout string)` | `func (t Time) Format(layout string) string`          | Zamanı dize olarak biçimlendirir.                        |

## III. `time.Duration` Metotları (Süre İşlemleri)

`time.Duration` tipinin üzerindeki metotlar, nanoseaniye cinsinden tutulan süreyi farklı birimlere dönüştürür veya manipüle eder.

| Metot                          | İmzası                                           | Açıklama                                                    |
| ------------------------------ | ------------------------------------------------ | ----------------------------------------------------------- |
| `d.Hours()`                    | `func (d Duration) Hours() float64`              | Süreyi saat cinsinden ondalıklı olarak döndürür.            |
| `d.Minutes()`                  | `func (d Duration) Minutes() float64`            | Süreyi dakika cinsinden ondalıklı olarak döndürür.          |
| `d.Seconds()`                  | `func (d Duration) Seconds() float64`            | Süreyi saniye cinsinden ondalıklı olarak döndürür.          |
| `d.Milliseconds()`             | `func (d Duration) Milliseconds() int64`         | Süreyi tam sayı milisaniye olarak döndürür.                 |
| `d.Abs()`                      | `func (d Duration) Abs() Duration`               | Sürenin mutlak değerini döndürür.                           |
| `time.ParseDuration(s string)` | `func ParseDuration(s string) (Duration, error)` | Bir dizeyi ("1h30m", "500ms") `Duration` tipine dönüştürür. |

## IV. Eş Zamanlılık (Concurrency) Özellikleri

Go'nun güçlü `goroutine` modelini destekleyen ve zamanlamayı sağlayan temel yapılardır.

### `time.Timer` ve Fonksiyonları

- **`time.NewTimer(d Duration)`**: Verilen süre sonunda kanala (`t.C`) bir değer gönderecek yeni bir `Timer` oluşturur.
- **`time.AfterFunc(d Duration, f func()) *Timer`**: Verilen süre dolduktan sonra belirtilen fonksiyonu (`f`) yeni bir goroutine içinde çalıştırır.
- **`t.Stop()`**: Timer'ı durdurur ve kanala sinyal gönderilmesini engeller.
- **`t.Reset(d Duration)`**: Zaten durdurulmuş veya geçmiş bir Timer'ı yeni bir süre ile sıfırlar.

### `time.Ticker` ve Fonksiyonları

- **`time.NewTicker(d Duration)`**: Belirtilen aralıklarla (`d`) kanala (`t.C`) sürekli olarak zaman sinyalleri gönderecek yeni bir `Ticker` oluşturur.
- **`t.Stop()`**: Ticker'ı durdurur ve periyodik sinyal göndermeyi sonlandırır.
- **`t.Reset(d Duration)`**: Ticker'ın periyodunu sıfırlar ve yeni bir süre ile yeniden başlatır.

## V. Önemli Özellikler (Features)

### 1. Formatlama Referansı (Magic Date)

Go'da zaman formatlaması için benzersiz bir referans tarihi kullanılır: **`2006-01-02 15:04:05.000000000 -0700 MST`**

| Referans Rakam | Anlamı                  |
| -------------- | ----------------------- |
| `2006`         | Yıl                     |
| `01`           | Ay (Sayı)               |
| `02`           | Gün                     |
| `15`           | Saat (24 saat)          |
| `04`           | Dakika                  |
| `05`           | Saniye                  |
| `-0700`        | UTC Ofseti              |
| `MST`          | Zaman Dilimi Kısaltması |

### 2. Monotonik Saat (Monotonic Clock)

`time.Time` nesneleri, hem duvardaki saati (wall clock) hem de süreyi doğru hesaplamak için bir **monotonik okuma** içerir. Bu, sistem saatinin ileri veya geri alınması durumunda bile süre hesaplamalarının (`t.Sub(u)`) doğru kalmasını sağlar.

### 🇬🇧 İngilizce Öğrenme Köşesi: Grammar & Chunk

Bu teknik konuya uygun dilbilgisi ve kelime öbekleri:

| Kategori                             | Örnek Cümle                                                                | Açıklama                                                                                                                        |
| ------------------------------------ | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| **Grammar** (Reporting Verb: Stated) | The documentation **stated that** `time.Duration` is an alias for `int64`. | **"Stated that"** (Şunu belirtti ki), bir kaynaktan bilgi aktarırken kullanılır.                                                |
| **Chunk** (Technical Verb)           | Always **defer the cancellation** of a timer.                              | **"Defer the cancellation"** (İptali ertelemek), kaynak temizliği (resource cleanup) yaparken Go'da yaygın bir pratik/kalıptır. |
| **Chunk** (Fixed Phrase)             | This requires **explicit handling** of time zones.                         | **"Explicit handling"** (Açık/belirgin ele alma), bir durumun manuel olarak yönetilmesi gerektiğini vurgular.                   |
