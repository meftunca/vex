# Kontrat: Vex Dili Metod Mutasyon Sözdizimi

**Tarih:** 10 Kasım 2025
**Versiyon:** 1.0
**Durum:** Onaylandı

Bu doküman, Vex programlama dilindeki `mutable` metodların nasıl tanımlanacağı, kullanılacağı ve çağrılacağına dair kesin kuralları belirler ve geliştirici ile AI agent arasında bir kontrat niteliği taşır.

## 1. Temel Felsefe: Bağlama Duyarlı Sözdizimi (Hibrit Model)

Vex, kodun yazıldığı bağlama göre en uygun ve en az gürültülü sözdizimini sunmayı hedefler. Bu pragmatik yaklaşımla, `inline` (struct/trait içi) ve `external` (Golang-style) metod tanımları için farklı ancak kendi içinde tutarlı kurallar benimsenmiştir.

## 2. "Vex Yolu": İki Kural Sistemi

### Kural 1: Inline Metodlar (Struct ve Trait İçinde)

**Amaç:** Kod tekrarını önlemek ve `struct`/`trait` tanımlarını temiz tutmak.

- **Tanımlama:** Metodun `mutable` olduğu, imzanın sonuna eklenen `!` işareti ile belirtilir. Receiver (`self`) bu stilde implisittir ve yazılmaz.
  - **Sözdizimi:** `fn method_name(params)!: ReturnType`
- **Gövde Erişimi:** Metod gövdesinde, `mutable` alanlara erişim ve atama `self!` anahtar kelimesi ile yapılır.
  - **Sözdizimi:** `self.field = new_value`
- **Çağrı:** `Mutable` metodlar, çağrı anında `!` son eki ile kullanılır.
  - **Sözdizimi:** `object.method_name()!`

**Örnek:**

```vex
struct Counter {
    value: i32,

    // 'self' yazılmaz, '!' ile mutable olduğu belirtilir.
    fn increment()! {
        self.value = self.value + 1;
    }
}

let! c = Counter { value: 0 };
c.increment(); // Çağrıda '!' zorunlu değildir, compile time kontrolü yapılır.
```

### Kural 2: External Metodlar (Golang-Style)

**Amaç:** Metodun hangi veri tipi üzerinde çalıştığını ve `mutable` olup olmadığını receiver tanımında açıkça belirtmek.

- **Tanımlama:** Metodun `mutable` olduğu, receiver tanımındaki `&Type!` ifadesi ile belirtilir. Metod imzasının sonunda `!` **kullanılmaz**.
  - **Sözdizimi:** `fn (self: &MyType!) method_name(params): ReturnType`
- **Gövde Erişimi:** Metod gövdesinde, alanlara erişim ve atama doğrudan `self` üzerinden yapılır. `self!` **kullanılmaz**.
  - **Sözdizimi:** `self.field = new_value`
- **Çağrı:** Çağrı sırasında `!` işareti **kullanılmaz**.
  - **Sözdizimi:** `object.method_name()`

**Örnek:**

```vex
// Container<T> için external bir metod
fn (self: &Container<T>!) set(val: T): i32 {
    self.value = val; // Doğrudan atama
    return 0;
}

// Çağrı
container.set(20); // '!' olmadan.
```

## 3. Uygulama ve Sorumluluklar

1.  **Derleyici Hatasının Giderilmesi:** Mevcut derleyici, **Kural 2**'de belirtilen `self.field = new_value` atamasını doğru şekilde işleyememektedir. Bu hata, bu kontrata uygun olarak öncelikli olarak düzeltilecektir.
2.  **Sözdizimi Tutarlılığı:** Tüm Vex kodu, bu iki kurala uygun hale getirilecektir. AI agent, yeni kod üretirken veya mevcut kodu düzenlerken bu kontrata harfiyen uyacaktır.
3.  **Belgelendirme:** Dilin resmi belgeleri (`REFERENCE.md` vb.), bu hibrit modeli yansıtacak şekilde güncellenecektir.
