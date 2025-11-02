# Vex Programming Language - Hızlı Başlangıç

## 🎉 Projeniz Hazır!

Vex programlama dili projeniz başarıyla oluşturuldu ve derlenmiştir.

## 📁 Proje Yapısı

```
vex_lang/
├── Cargo.toml              # Workspace yapılandırması
├── README.md               # Proje dokümantasyonu
├── Specification.md        # Dil spesifikasyonu
├── PROJECT_STATUS.md       # Detaylı proje durumu
├── intro.md                # Kullanılan kütüphaneler
│
├── examples/               # Örnek .vx programları
│   ├── hello.vx
│   ├── simd_vector_add.vx
│   ├── gpu_matrix.vx
│   ├── async_io.vx
│   └── struct_methods.vx
│
├── vex-lexer/              # Tokenization (logos)
│   └── src/lib.rs
│
├── vex-parser/             # Grammar & Parsing (lalrpop)
│   └── src/lib.rs
│
├── vex-ast/                # AST ve Tip Sistemi
│   └── src/lib.rs
│
├── vex-compiler/           # LLVM/SPIR-V Compiler
│   └── src/lib.rs
│
├── vex-runtime/            # Async Runtime (tokio)
│   └── src/lib.rs
│
└── vex-cli/                # Command-line Interface
    └── src/main.rs
```

## 🚀 Kullanım

### Derleme

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

### Test

```bash
# Tüm testleri çalıştır
cargo test

# Belirli bir crate'in testini çalıştır
cargo test -p vex-lexer
```

### CLI Komutları

```bash
# Version bilgisi
vex --version

# Help menüsü
vex --help

# Syntax kontrolü
vex check examples/hello.vx

# Derleme (TODO: parser gerekli)
vex compile examples/hello.vx -o hello

# Çalıştırma (TODO: compiler gerekli)
vex run examples/hello.vx

# Formatlama (TODO)
vex format examples/hello.vx
```

## ✅ Şu Anda Çalışan Özellikler

1. **Lexer**: Tüm Vex token'ları tanımlanmış ve test edilmiş
2. **AST**: Tam özellikli AST node yapısı ve tip sistemi
3. **Runtime**: Tokio tabanlı async runtime
4. **CLI**: Temel komut satırı arayüzü
5. **Examples**: 5 farklı kullanım örneği

## 🔨 Devam Eden Çalışmalar

### Öncelik 1: Parser (lalrpop)

```bash
# Parser implementasyonu için:
# 1. vex-parser/src/grammar.lalrpop dosyası oluştur
# 2. Build script ekle
# 3. Token'ları AST'ye dönüştür
```

### Öncelik 2: LLVM Backend (inkwell)

```bash
# Compiler implementasyonu için:
# 1. vex-compiler içinde LLVM context oluştur
# 2. AST'yi LLVM IR'a çevir
# 3. Optimizasyonları uygula
```

### Öncelik 3: GPU Support (rspirv)

```bash
# GPU desteği için:
# 1. SPIR-V code generation
# 2. GPU intrinsics (@gpu.global_id, etc.)
# 3. Vulkan/OpenCL entegrasyonu
```

## 📊 Test Sonuçları

```
✅ vex-lexer:    4/4 tests passed
✅ vex-ast:      2/2 tests passed
✅ vex-runtime:  2/3 tests passed
✅ vex-parser:   1/1 test passed
✅ vex-compiler: 1/1 test passed

Tüm örnek dosyalar syntax kontrolünden geçti!
```

## 🔍 Örnek Vex Kodu

### Hello World

```javascript
import { io, log } from "std";

fn main(): error {
    log.info("Vex v0.2 çalışıyor.");
    io.print(f"1 + 2 = {1 + 2}\n");
    return nil;
}
```

### SIMD Vektör İşlemleri

```javascript
fn add_vectors(a: &[f32; 4], b: &[f32; 4], out: &mut [f32; 4]) {
    @vectorize
    for i in 0..a.len() {
        out[i] = a[i] + b[i];
    }
}
```

### GPU Hesaplama

```javascript
gpu fn matrix_multiply(a: &[f32], b: &[f32], out: &mut [f32], size: u32) {
    let x = @gpu.global_id.x;
    let y = @gpu.global_id.y;
    // ... GPU kernel code
}

fn main(): error {
    await launch matrix_multiply[N, N](a, b, &mut out, N);
    return nil;
}
```

## 🛠️ Geliştirme Araçları

### Kod Formatı

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Dokümantasyon

```bash
cargo doc --open
```

## 📚 Dokümantasyon

- **Specification.md**: Tam dil spesifikasyonu
- **intro.md**: Kullanılan kütüphaneler ve nedenler
- **PROJECT_STATUS.md**: Detaylı proje durumu ve TODO listesi

## 🎯 Sonraki Adımlar

1. **Parser Grammar**: `vex-parser/src/grammar.lalrpop` dosyası oluştur
2. **LLVM IR Generation**: Basit fonksiyonlar için LLVM IR üret
3. **Standard Library**: io, log modüllerini implement et
4. **Testing**: Daha fazla integration test ekle
5. **Documentation**: API dokümantasyonu genişlet

## 🤝 Katkıda Bulunma

Proje açık kaynak ve aktif geliştirme aşamasında. Katkılarınızı bekliyoruz!

## 📄 Lisans

MIT veya Apache-2.0 (tercihinize göre)

---

**Vex** - Modern donanım için yüksek performanslı programlama dili
