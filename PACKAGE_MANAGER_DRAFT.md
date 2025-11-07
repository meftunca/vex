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

**Temel Komutlar:**

- **`vex new <proje_adi>`**: Yeni bir Vex projesi oluşturur

  - `vex.json` ve temel dizin yapısını hazırlar
  - Örnek: `vex new my_server`

- **`vex build`**: Projeyi derler (gerekiyorsa bağımlılıkları indirir)

  - Varsayılan: `development` profile
  - `vex build --profile=production` veya `vex build --release`
  - HPC desteği: `vex build --accelerator=gpu-nvidia` (future)

- **`vex run`**: Projeyi derler ve çalıştırır

  - `vex run` veya `vex run --release`

- **`vex test`**: Proje testlerini (`_test.vx` dosyaları) çalıştırır

  - `testing` profile otomatik kullanılır
  - Örnek: `vex test --verbose`

- **`vex add <paket_yolu>`**: Yeni bir bağımlılık ekler

  - GitHub: `vex add github.com/vex-lang/json@v1.2.0`
  - GitLab: `vex add gitlab:company/lib@v1.0.0`
  - HTTP: `vex add https://cdn.com/pkg.tar.gz@v1.0.0`
  - `vex.json`'a otomatik ekler

- **`vex remove <paket_adi>`**: Bir bağımlılığı kaldırır

  - Örnek: `vex remove json`

- **`vex update`**: Tüm bağımlılıkları günceller

  - `vex.lock` dosyasını yeniden oluşturur

- **`vex clean`**: Build cache'ini temizler
  - `vex-builds/` dizinini siler

**Future Commands (Phase 2+):**

- **`vex ffi-bind <c_header.h>`**: FFI bağlamaları otomatik oluşturur

  - C header dosyasını parse edip Vex extern declarations üretir
  - Örnek: `vex ffi-bind openssl.h`

- **`vex publish`**: Paketi yayınlar (Git tag oluşturur)
  - Örnek: `vex publish v1.0.0`

## 4. Manifesto: `vex.json` (Detaylı Yapı)

`vex.json`, Go'nun `go.mod`'u ve NPM'in `package.json`'ının yerini alır. (Standart 3: Tip Güvenliği - JSON'un katı yapısı için).

```json
{
  "name": "my_vex_server",
  "version": "0.1.0",
  "description": "Vex ile yazılmış yüksek performanslı sunucu.",
  "authors": ["Adınız <email@adresiniz.com>"],
  "license": "MIT",

  "dependencies": {
    "github.com/vex-lang/json": "v1.2.0",
    "github.com/user/vex-router": "v2.0.0",
    "gitlab:company/internal-lib": "v1.5.0",
    "https://lib.dev/my-package.tar.gz": {
      "version": "v1.0.0",
      "headers": {
        "Authorization": "Bearer token123",
        "X-Custom-Header": "value"
      }
    }
  },

  "vex": {
    "borrowChecker": "strict"
  },

  "profiles": {
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
      "optimizationLevel": 3,
      "debugSymbols": false
    }
  }
}
```

**Dependency Protokolleri:**

1. **GitHub (Varsayılan)**: `github.com/user/repo`

   - Git clone: `https://github.com/user/repo.git`
   - Semantic versioning: `v1.2.3`, `v2.0.0`

2. **GitLab**: `gitlab:user/repo` veya `gitlab.com/user/repo`

   - Git clone: `https://gitlab.com/user/repo.git`
   - Private repos için git credentials kullanılır

3. **Diğer Git Servisleri**: `bitbucket:user/repo`

   - Explicit prefix ile tanımlanır

4. **HTTP/HTTPS Direct Download**: Tam URL + opsiyonel headers
   ```json
   "https://cdn.example.com/pkg-v1.0.0.tar.gz": {
       "version": "v1.0.0",
       "headers": {
           "Authorization": "Bearer secret_token"
       }
   }
   ```
   - Private CDN'ler için authentication headers
   - Version field opsiyonel (URL'de genelde version var)

**Profiles:**

- Eğer `profiles` tanımlanmamışsa, varsayılan profile kullanılır:
  ```json
  {
    "development": { "optimizationLevel": 0, "debugSymbols": true },
    "production": { "optimizationLevel": 3, "debugSymbols": false }
  }
  ```
- Profile seçimi: `vex build --profile=production` veya `vex build --release`

**FFI Dependencies (TODO - Phase 2+):**

```json
"ffiDependencies": {
    "openssl": "system",
    "zlib": "3.0.0"
}
```

- İlk aşamada FFI desteği YOK
- Dil tamamen bittikten sonra eklenecek
- System-installed libs → Phase 2
- Binary distribution → Phase 3

**Versioning Strategy:**

- **Semantic Versioning (SemVer)**: `v1.2.3` (major.minor.patch)
- Git tags ile eşleşir
- `v` prefix zorunlu (Go convention)

## 5. Kayıt Modeli: Merkezi Olmayan (Decentralized) (Go-style)

`vexpm`, NPM veya Cargo gibi _merkezi_ bir kayıt (central registry) sistemine _bağlı değildir_.

`vexpm`, `vex.json` dosyasındaki _protokol önekine_ (protocol prefix) göre paketleri çeker:

**Desteklenen Protokoller:**

1. **GitHub (Varsayılan)**:

   ```json
   "github.com/vex-lang/json": "v1.2.0"
   ```

   - `vexpm`, GitHub'a gider ve `v1.2.0` tag'ini checkout eder
   - Git clone: `https://github.com/vex-lang/json.git`
   - Go-style convention (prefix yok = GitHub)

2. **GitLab (Explicit)**:

   ```json
   "gitlab:company/internal-lib": "v1.5.0"
   ```

   - GitLab için `gitlab:` prefix zorunlu
   - Private repos için git credentials kullanılır
   - Git clone: `https://gitlab.com/company/internal-lib.git`

3. **Diğer Git Servisleri**:

   ```json
   "bitbucket:team/project": "v2.0.0"
   ```

   - Bitbucket, Gitea, vb. için explicit prefix

4. **HTTP/HTTPS Direct Download**:
   ```json
   "https://lib.dev/my-package.tar.gz": {
       "version": "v1.0.0",
       "headers": {
           "Authorization": "Bearer secret_token",
           "X-Custom-Header": "value"
       }
   }
   ```
   - Private CDN'ler veya artifact repositories için
   - `headers` field opsiyonel (authentication için)
   - NPM'in `url` dependencies benzeri ama daha güçlü

**Nexus Mirror (Opsiyonel - Future):**

`nexus.vex.dev`, Go'nun `GOPROXY`'si gibi davranan _isteğe bağlı_ bir "mirror" (ayna):

- İndirme hızlarını artırır (CDN cache)
- Immutability: Git repo silinse bile paketler korunur
- Merkezi olmayan yapıyı bozmaz (fallback olarak çalışır)
- **Durum:** Phase 2+ (şu an desteklenmeyecek)

**Future Environment Variable:**

```bash
export VEX_PROXY=https://nexus.vex.dev  # Phase 2+
vex build  # Önce proxy'den dener, sonra origin'e gider
```

## 6. Global Önbellek (Global Cache)

- **Node.js Hatası:** Vex, `node_modules` kabusunu **yaşamayacaktır**. (Standart 2: Bağımlılıkların Azaltılması).
- **Go/Cargo Modeli:** İndirilen tüm paketler (Vex ve FFI (BIM) kaynak kodları) `~/.vex/cache/` (veya platforma özel önbellek dizini) altında _global_ bir önbellekte saklanır.
- Farklı projeler, aynı paketin aynı sürümünü _paylaşır_, disk kullanımını drastik olarak azaltır.

## 7. FFI (BIM) Bağımlılık Yönetimi (TODO - Phase 2+)

**Durum:** İlk aşamada FFI desteği YOK - Dil tamamen bittikten sonra eklenecek

**Gelecek Tasarım (Phase 2+):**

`vex.json` dosyasında `ffiDependencies` field'ı:

```json
{
  "ffiDependencies": {
    "openssl": "system", // System-installed lib (pkg-config)
    "zlib": "3.0.0", // Specific version (Nexus binary)
    "sqlite3": "system"
  }
}
```

### Strateji 1: System-installed Libraries (Phase 2)

```bash
# macOS
brew install openssl zlib

# Linux
apt-get install libssl-dev zlib1g-dev

# vex build otomatik bulur (pkg-config)
vex build  # libssl.so, libz.so ile linkler
```

### Strateji 2: Pre-built Binaries (Phase 3+)

`vex add openssl` çalıştığında:

1. `nexus.vex.dev`'den platform-specific binary indirilir:
   - `openssl-v3.0.0-linux-x64.vexbin`
   - `openssl-v3.0.0-macos-arm64.vexbin`
2. `~/.vex/cache/ffi/` altına kaydedilir
3. `vex build` sırasında otomatik linkler

### Strateji 3: Source Build (Phase 4+)

Eğer binary bulunamazsa:

1. Kaynak kod indirilir (`openssl-v3.0.0-src.tar.gz`)
2. `gcc`/`clang`/`cmake` ile derlenir (Cargo'nun `build.rs` gibi)
3. Global cache'e kaydedilir

**Implementation:** Dil stable olduktan sonra (v1.0+)

---

## 8. Kilit Dosyası: `vex.lock` (Lock File)

`vex.lock`, **tekrarlanabilir build'leri** (reproducible builds) garanti eder. Makine tarafından yönetilir, kullanıcı düzenlemez.

**Format:** JSON

```json
{
  "version": 1,
  "lockTime": "2025-11-07T15:30:00Z",
  "dependencies": {
    "github.com/vex-lang/json": {
      "version": "v1.2.0",
      "resolved": "https://github.com/vex-lang/json/archive/v1.2.0.tar.gz",
      "integrity": "sha256:a3f5b2c1d4e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z7",
      "dependencies": {
        "github.com/vex-lang/utf8": "v2.1.0"
      }
    },
    "github.com/vex-lang/utf8": {
      "version": "v2.1.0",
      "resolved": "https://github.com/vex-lang/utf8/archive/v2.1.0.tar.gz",
      "integrity": "sha256:b4g6c3d5e7f8g9h0i1j2k3l4m5n6o7p8q9r0s1t2u3v4w5x6y7z8"
    }
  }
}
```

**Özellikler:**

- **Integrity Hash**: SHA-256 checksum (güvenlik + doğrulama)
- **Resolved URL**: Tam indirme adresi (immutability)
- **Transitive Dependencies**: Flat dependency tree (Go-style)
- **Lock Time**: Build zamanı (debugging için)

**Güncelleme:**

- `vex add`: Yeni bağımlılık eklendiğinde güncellenir
- `vex update`: Tüm bağımlılıklar yükseltilir
- Manuel değişiklik yapılmaz (Git'e commit edilir)

---

## 9. Bağımlılık Çözümleme (Dependency Resolution)

**Strateji:** Go-style Flat Dependency Tree

### Minimum Version Selection (MVS)

Birden fazla versiyon gereksinimi olduğunda **en yüksek** versiyon seçilir:

```
Proje gereksinimi:
  - github.com/vex-lang/json@v1.2.0
    └── github.com/vex-lang/utf8@v2.1.0

  - github.com/vex-lang/router@v3.0.0
    └── github.com/vex-lang/utf8@v2.0.0

Çözüm:
  ✅ utf8@v2.1.0 kullanılır (en yüksek SemVer)
```

### Conflict Resolution

SemVer uyumlu değilse **hata verir**:

```
Error: Version conflict for github.com/vex-lang/utf8
  - json@v1.2.0 requires utf8@v2.x
  - old-lib@v1.0.0 requires utf8@v1.x
  ❌ Cannot resolve (major version conflict)
```

**Çözüm:** Kullanıcı manuel müdahale etmelidir (bağımlılık güncelleme veya kaldırma).

---

## 10. Private Git Repositories - Authentication

**Strateji:** System Git Credentials kullan (Git config + SSH keys)

### GitHub Private Repos

```bash
# SSH key (önerilen)
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
ssh-add ~/.ssh/id_rsa

# HTTPS + token (alternatif)
git config --global credential.helper store
git clone https://username:token@github.com/company/private-repo.git
```

`vex add github.com/company/private-repo@v1.0.0` komutu:

1. Git'in system credential'larını kullanır
2. SSH key varsa SSH protokolü kullanır
3. HTTPS token varsa HTTPS kullanır
4. Auth başarısızsa hata verir: "Authentication failed for github.com/company/private-repo"

### GitLab Private Repos

```bash
# SSH key
ssh-add ~/.ssh/id_gitlab

# Access token
git config --global credential.helper store
```

`vex add gitlab:company/internal-lib@v1.0.0`:

- GitLab SSH/HTTPS credentials kullanır
- Token: `GITLAB_TOKEN` env variable (opsiyonel)

**Avantajlar:**

- ✅ Vex'e özel configuration gerekmez
- ✅ Mevcut Git workflow'u kullanılır
- ✅ SSH keys + credential helpers desteklenir
- ✅ CI/CD sistemleri ile uyumlu (system git config)

---

## 11. Package Structure (Paket Yapısı)

### Standart Yapı

```
my-package/
├── vex.json              # Package manifesto
├── vex.lock              # Lock file (git'e commit edilir)
├── src/
│   ├── lib.vx            # Varsayılan entrypoint (public API)
│   ├── internal.vx       # Private modül
│   └── utils.vx          # Internal utilities
├── tests/
│   ├── lib_test.vx       # Unit tests
│   └── integration_test.vx
└── examples/
    └── basic_usage.vx    # Örnek kullanım
```

### Entrypoint Kuralları

**Varsayılan:** `src/lib.vx`

Kullanıcı farklı entrypoint belirtebilir:

```json
{
  "name": "my-package",
  "version": "1.0.0",
  "main": "src/custom_entry.vx"
}
```

### Export Kuralları

`lib.vx` içinde public API tanımlanır:

```vex
// src/lib.vx
export fn parse(data: string): Result<Json, Error> {
    // Public function
}

fn internal_helper(): i32 {
    // Private function (export yok)
}
```

Bağımlılık olarak kullanım:

```vex
import { parse } from "github.com/vex-lang/json";

fn main(): i32 {
    let json = parse("{\"key\": \"value\"}");
    return 0;
}
```

### Binary vs Library

**Library Package** (varsayılan):

```json
{
  "name": "json-parser",
  "version": "1.0.0",
  "main": "src/lib.vx"
}
```

**Binary Package** (executable):

```json
{
  "name": "my-cli-tool",
  "version": "1.0.0",
  "main": "src/main.vx",
  "bin": {
    "my-tool": "src/main.vx"
  }
}
```

`vex build` → `vex-builds/my-tool` binary oluşturur

---

## 12. Nexus Mirror (Future - Phase 2+)

**Durum:** Şu an desteklenmeyecek (TODO)

**Gelecek Planlama (Phase 2+):**

```bash
# Environment variable
export VEX_PROXY=https://nexus.vex.dev

vex build  # Önce proxy'den, sonra origin'den çeker
```

**Nexus'un Görevleri:**

1. **CDN Cache**: Hızlı indirme (global distribution)
2. **Immutability**: Git repo silinse bile paketler korunur
3. **Offline Mirror**: Şirket içi mirror için
4. **Checksum Validation**: Güvenlik + integrity

**Implementation Priority:** Phase 2+ (Dil stable olduktan sonra)

---

## 13. `std` Kütüphane Yönetimi

- `std` paketleri `git` depolarından _indirilmez_
- `std` kütüphanesi **Vex derleyicisinin bir parçasıdır** (Go gibi)
- `vex` versiyonu = `std` versiyonu (örn: `vex v1.0.0` → `std v1.0.0`)
- `import { http } from "std"` doğrudan derleyiciye yönlendirilir

**Avantajlar:**

- ✅ Bağımlılık yok (std her zaman mevcut)
- ✅ Versiyon tutarlılığı garantili
- ✅ Hızlı build (önceden derlenmiş std lib)
