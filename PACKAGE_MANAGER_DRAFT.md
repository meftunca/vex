# Vex Paket Yöneticisi (vexpm) Mimarisi (v1.2)

## 1. Felsefe ve Hedefler (Proje Vizyonu - Standart 1)

Vex Paket Yöneticisi (`vexpm`), `vex` adlı tek bir komut satırı aracı (CLI tool) olarak sunulacaktır.

**Felsefe:** "Cargo'nun gücü ve tekrarlanabilirliği (reproducibility), Go Mod'un merkezi olmayan (decentralized) sadeliği."

**Ana Hedefler:**

1. **Hız:** Bağımlılıkları (dependencies) paralel olarak (ve `GOPROXY` benzeri bir 'mirror' üzerinden) indirme ve derleme.
2. **Güvenlik:** Kilit dosyaları (`vex.lock`) ile %100 "tekrarlanabilir build" (reproducible builds) sağlama. (Standart 4, 7)
3. **Ergonomi:** `vex add <paket_yolu>` gibi basit komutlarla kolay kullanım.
4. **FFI (BIM) Desteği:** Vex'in FFI (`vex_ffi_strategy.md`) stratejisini _birinci sınıf_ (first-class) bir özellik olarak yönetme.

## 2. Temel Bileşenler (Mimari)

`vexpm`, 5 ana bileşenden oluşur:

1. **CLI Aracı (`vex`):** Kullanıcının etkileşime girdiği komut (örn: `vex build`).
2. **Manifesto Dosyası (`vex.json`):** Projenin "beyni" (kullanıcı tarafından düzenlenir).
3. **Kilit Dosyası (`vex.lock`):** Tekrarlanabilir build'ler için (makine tarafından yönetilir).
4. **Global Önbellek (Cache):** Paketlerin `node_modules` yerine saklandığı yer.
5. **Kayıt Modeli (Go-style):** Merkezi olmayan (decentralized) `git` veya `http` tabanlı paket çözümlenmesi.

## 3. CLI Aracı: `vex` (Komutlar)

`vex` komutu, Go ve Cargo gibi, derleyici ve paket yöneticisini birleştirir:

- **`vex build`**: Projeyi derler (gerekiyorsa bağımlılıkları indirir).
  - `vex build --accelerator=gpu-nvidia` (HPC bayrağımızı (v0.6) destekler).
- **`vex run`**: Projeyi derler ve çalıştırır.
- **`vex test`**: Proje testlerini (`_test.vx` dosyaları) çalıştırır. (Standart 4.1)
- **`vex add <paket_yolu@versiyon>`**: Yeni bir Vex bağımlılığı ekler (örn: `vex add git:github.com/vex-lang/json@v1.2.0` veya `vex add https://lib.dev/paket.tar.gz@v1.0.0`).
- **`vex rm <paket_adi>`**: Bir bağımlılığı kaldırır.
- **`vex ffi-bind <c_baslik_dosyasi.h>`**: (Öneri) `vex_libc_targets.md` dosyasındaki FFI (BIM) bağlamalarını otomatikleştirmek için C başlık dosyasını okur ve "native" Vex FFI (BIM) modülünü _otomatik olarak_ oluşturur.
- **`vex publish`**: (Not: Merkezi olmayan modelde bu komut, paketi bir Git 'tag' (etiket) olarak yayınlamak anlamına gelir.)

## 4. Manifesto: `vex.json` (Detaylı Yapı)

`vex.json`, Go'nun `go.mod`'u ve NPM'in `package.json`'ının yerini alır. (Standart 3: Tip Güvenliği - JSON'un katı yapısı için).

```javascript
{
    "name": "my_vex_server",
    "version": "0.1.0",
    "description": "Vex ile yazılmış yüksek performanslı sunucu.",
    "authors": [ "Adınız <email@adresiniz.com>" ],
    "license": "MIT",

    // Vex paket bağımlılıkları (Protokol bazlı)
    "dependencies": {
        "git:vex-lang/json": "v1.2.0",
        "git:user/vex-router": "v2.0.0",
        "https://some-lib.com/my-package.tar.gz": "v1.0.0"
    },

    // 'vex_ffi_strategy.md' (BIM Stratejisi) için KRİTİK BÖLÜM
    // C/C++/Rust kütüphane bağımlılıkları
    "ffiDependencies": {
        "openssl": "3.0.0",
        "zlib": "1.2.11"
    },

    // NPM 'scripts' gibi basit görev otomasyonu
    "scripts": {
        "test": "vex test --verbose", // profiles.testing kullanır
        "dev": "vex build --watch --accelerator=cpu-simd", // profiles.development kullanır
        "build": "vex build --release --accelerator=cpu-simd", // profiles.production kullanır
        "lint": "vex-linter ./...",
        "build:prod": "vex build --release --accelerator=cpu-simd"
    },

    // Derleyiciye özel Vex yapılandırması
    "vex": {
        "borrowChecker": "strict" // (v0.9 Planımız)
    },
    "profiles":{
        "development": {
                "optimizationLevel": 0,
                "debugSymbols": true
        },
        "testing": {
                "optimizationLevel": 1,
                "debugSymbols": true,
                "memProfiling": true,
                "cpuProfiling": true
        },
        "production": {
                "optimizationLevel": 3
        }
    }
}

```

## 5. Kayıt Modeli: Merkezi Olmayan (Decentralized) (Go-style)

`vexpm`, NPM veya Cargo gibi _merkezi_ bir kayıt (central registry) sistemine _bağlı değildir_.

`vexpm`, `vex.json` dosyasındaki _protokol önekine_ (protocol prefix) göre paketleri çeker:

- `"git:github.com/vex-lang/json": "v1.2.0"` bağımlılığı, `vexpm`'e o `git` deposuna gitmesini ve `v1.2.0` etiketini (tag) 'check out' etmesini söyler (Go `go get` gibi).
- `"https://some-lib.com/my-package.tar.gz": "v1.0.0"` bağımlılığı, `vexpm`'e o `tar.gz` dosyasını doğrudan (direct download) indirmesini söyler (NPM'in `url` bağımlılıkları gibi).

`nexus.vex.dev` (Önceki Bölüm 5) artık bir _kayıt (registry)_ değil, Go'nun `GOPROXY`'si gibi davranan _isteğe bağlı (optional)_ bir "mirror" (ayna) veya "proxy" (vekil sunucu) olarak hizmet verebilir. Bu, indirme hızlarını artırır ve `git` deposu silinse bile paketlerin kaybolmamasını (immutability) sağlar.

## 6. Global Önbellek (Global Cache)

- **Node.js Hatası:** Vex, `node_modules` kabusunu **yaşamayacaktır**. (Standart 2: Bağımlılıkların Azaltılması).
- **Go/Cargo Modeli:** İndirilen tüm paketler (Vex ve FFI (BIM) kaynak kodları) `~/.vex/cache/` (veya platforma özel önbellek dizini) altında _global_ bir önbellekte saklanır.
- Farklı projeler, aynı paketin aynı sürümünü _paylaşır_, disk kullanımını drastik olarak azaltır.

## 7. FFI (BIM) Bağımlılık Yönetimi (Vex'in "Katil Özelliği" - Standart 4: Risk Yönetimi)

`vex.json` dosyasındaki `ffiDependencies` (`vex_ffi_strategy.md`) Vex'in en büyük zorluğu ve _en büyük fırsatıdır_.

`vex add openssl` (FFI) komutu çalıştığında `vexpm` ne yapar?

**Strateji 1 (Öncelikli): Ön-Derlenmiş (Pre-Built) İkili (Binary) İndirme**

1. `vexpm`, `nexus.vex.dev`'e (veya yapılandırılmış FFI (BIM) 'mirror'ına) gider.
2. `openssl-v3.0.0-linux-x64.vexbin` (veya `macos-arm64` vb.) gibi _önceden derlenmiş_ (pre-compiled) C kütüphanesini indirir.
3. Bunu global önbelleğe (`~/.vex/cache/`) koyar.
4. `vex build` sırasında, derleyiciye "bu ikiliye (binary) karşı linkle (bağla)" der.
5. **Avantajı:** Kullanıcının makinesinde `gcc`, `clang` veya `make` olmasına _gerek yoktur_ (NPM'in `node-sass`'ı gibi).

**Strateji 2 (Yedek): Kaynaktan Derleme (Compile from Source)**

1. Eğer `vexbin` bulunamazsa, `vexpm` `openssl-v3.0.0-src.tar.gz` kaynak kodunu indirir.
2. Kullanıcının makinesindeki `gcc`/`clang`/`make`'i kullanarak onu derlemeye çalışır (Cargo'nun `build.rs` script'i gibi).
3. **Avantajı:** Daha esnektir, ancak kullanıcının sisteminde C derleme araç zinciri (toolchain) gerektirir. (Standart 4: Risk Yönetimi - Bu bir risktir).

## 8. `std` Kütüphane Yönetimi (`vex_std_packages.md`)

- `std` paketleri (Bölüm 1-11), `git` depolarından _indirilmez_.
- `std` kütüphanesi, Go gibi, **Vex derleyicisinin (compiler) bir parçasıdır**.
- `vexpm`, Vex derleyicisinin sürümünü (örn: `v1.0.0`) bilir ve o sürümün `std::http` (Bölüm 3.3) veya `std::crypto` (Bölüm 10) içerip içermediğini anlar.
- `import { http } from "std"` (Vex Kodu) çağrısı, `vexpm` tarafından doğrudan derleyiciye (compiler) yönlendirilir.
