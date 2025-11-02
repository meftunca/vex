### Kategori 1: Frontend (Lexer & Parser)

Bu kütüphaneler, `.vx` dosyalarındaki Vex kodunuzu okuyup bir "Soyut Sözdizimi Ağacı"na (AST) dönüştürmenizi sağlar.

1. **Lexer (Token Ayırıcı):&#32;`logos`**
   - **Neden:** İnanılmaz hızlı, regex tabanlı bir lexer (token ayırıcı) üretecidir. Kodunuzu `fn`, `let`, `i32`, `+`, `ident(my_var)` gibi "token"lara ayırmak için en verimli yoldur.
2. **Parser (Sözdizimi Analizcisi):&#32;`lalrpop`**
   - **Neden:** Bu bir "parser generator" (ayrıştırıcı üreteci). Dilinizin gramerini (BNF benzeri) bir `.lalrpop` dosyasında tanımlarsınız, o da sizin için Rust kodunda hatasız ve hızlı bir parser üretir. Elle parser yazmaktan _çok daha hızlı_ ve daha az hataya açıktır.

### Kategori 2: Backend (LLVM IR Üretimi - CPU/SIMD)

Bu, `fn`, `struct` ve `@vectorize` gibi komutlarınızı alıp LLVM'in anlayacağı "Ara Temsil" (IR) koduna çevirecek kısımdır.

3. **LLVM Bağlantıları:&#32;`inkwell`**
   - **Neden:** Bu, Rust için modern, tip-güvenli (type-safe) ve ergonomik LLVM sarmalayıcısıdır. LLVM'in C API'ı (`llvm-sys`) karmaşık ve "unsafe"tir. `inkwell`, bu karmaşıklığı sizin için soyutlar ve Rust'ın güvenlik garantileriyle LLVM IR üretmenizi sağlar. Vex'in CPU ve SIMD hedefleri için _anahtar_ kütüphane budur.

### Kategori 3: GPU Backend (SPIR-V Üretimi)

Canvas'ta belirttiğimiz gibi, `gpu fn` fonksiyonları LLVM IR'a değil, SPIR-V'ye derlenecek.

4. **SPIR-V Üreteci:&#32;`rspirv`**
   - **Neden:** `rspirv`, programatik olarak (Rust kodundan) adım adım SPIR-V modülleri oluşturmanızı sağlayan bir kütüphanedir. Derleyiciniz, `gpu fn`'in AST'sini gezerken, `rspirv`'nin "builder" API'ını kullanarak SPIR-V bytecode'unu üretecektir.
   - _(Alternatif:&#32;`spirv-builder`&#32;diye bir kütüphane daha vardır, ancak o daha çok Rust kodunu GPU'da çalıştırmak içindir. Sizin ihtiyacınız, kendi dilinizden SPIR-V üretmek olduğu için&#32;`rspirv`&#32;daha uygundur.)_

### Kategori 4: Vex Runtime ve CLI (Gereklilikler)

Vex dilinin kendisi, `io_uring` ve `launch` gibi özellikler için bir "çalışma zamanına" (runtime) ihtiyaç duyar. Bu runtime'ı da muhtemelen Rust ile yazacaksınız.

5. **Async Runtime (io_uring için):&#32;`tokio`&#32;ve&#32;`tokio-uring`**
   - **Neden:** Vex'in `async/await` ve `go` özellikleri, `io_uring` tabanlı bir "task scheduler" (görev zamanlayıcı) gerektirir. `tokio`, Rust'taki en olgun async runtime'dır ve `tokio-uring` kütüphanesi ile doğrudan `io_uring` halkalarını yönetmenizi sağlar. Vex'in "Akıllı Context Switching" hedefinin temelini bu oluşturacaktır.
6. **Derleyici CLI (Arayüz):&#32;`clap`**
   - **Neden:** `vex compile input.vx -o output` gibi komut satırı arayüzlerini oluşturmak için Rust'taki standart ve en güçlü kütüphanedir.
