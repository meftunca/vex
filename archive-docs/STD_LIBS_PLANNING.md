# Vex Standart Kütüphane (`std`) Mimarisi

Bu belge, `vex_specification.md` (v0.6) belgesinde tanımlanan Vex dili için standart kütüphanenin (`std`) nasıl inşa edileceğini ve soyutlama katmanlarının nasıl çalıştığını detaylandırır.

## 1. Felsefe: Güvenli Soyutlama, Güvensiz Çekirdek

Vex'in `std` kütüphanesi, Go'nun "pilleri dahil" (batteries-included) felsefesini, Rust'ın "sıfır maliyetli soyutlama" (zero-cost abstraction) hedefiyle birleştirir.

Temel kuralımız şudur: **`std`&#32;kütüphanesinin %99'u, %100 güvenli (safe) ve "native" Vex kodu olmalıdır.**

Tüm karmaşıklık, `unsafe` (Bölüm 2.6) ve "runtime intrinsic" (derleyiciye özel) kodun kullanıldığı, `std::io` ve `std::hpc` gibi çok küçük ve spesifik "çekirdek" (core) paketlere hapsedilir.

## 2. Soyutlama Katmanları

Vex'in `std` mimarisi 4 katmandan oluşur. Geliştiriciler (sen dahil) çoğunlukla Katman 3 ve 2'de çalışırsın.

- **Katman 3: Uygulama (Safe Native Vex)**
  - **Paketler:** `std::http`, `std::json`, `std::xml`, `std::log`
  - **Açıklama:** Bunlar son kullanıcıya yönelik, yüksek seviyeli paketlerdir. %100 güvenli Vex ile yazılırlar ve alt katmanları (`std::net`) kullanırlar.
- **Katman 2: Protokol (Safe Native Vex)**
  - **Paketler:** `std::net`, `std::sync`, `std::testing`
  - **Açıklama:** Bunlar, `std::http` gibi paketlere altyapı sağlayan, %100 güvenli Vex ile yazılmış protokol ve eşzamanlılık paketleridir. `std::io`'yu kullanırlar.
- **Katman 1: I/O Çekirdeği (Unsafe Vex Köprüsü)**
  - **Paketler:** `std::io`, `std::hpc`, `std::unsafe`, `std::ffi` (Bölüm 8)
  - **Açıklama:** Bu, Vex dilinin (kullanıcı modu) ile Vex Runtime'ının (kernel modu, `io_uring`) arasındaki "köprü"dür. Burası, `unsafe` (Bölüm 2.6) kodun ve `runtime:intrinsics`'in (Bölüm 5) yaşadığı tek yerdir.
- **Katman 0: Vex Runtime (Rust ile yazılır)**
  - **Açıklama:** Bu, Vex derleyicisiyle birlikte gelen, Rust ile yazılmış kod parçasıdır. `io_uring`'i, `async` görev (task) zamanlayıcısını (Bölüm 5) ve `launch` (Bölüm 6) işlemlerini yönetir.

## 3. Paket İnşa Süreci: `http.get` Akışı

"Native Vex" ile `std::http`'yi nasıl yazacağımızı, Katman 3'ten Katman 0'a inerek inceleyelim:

### Adım 1: `std::http` (Katman 3 - %100 Native Vex)

`std::http` paketi, `io_uring` veya soketlerden _haberdar değildir_. Sadece "Reader" ve "Writer" arayüzlerini (Bölüm 4) bilir.

```javascript
// Dosya: std/http/client.vx
// Bu dosya %100 güvenli, native Vex kodudur.

import { net, io } from "std";
import type { Reader, Writer } from "std::io"; // Tip importu (Bölüm 7.3.1)

// 'Connection' tipi, hem okuyucu hem yazıcı olan
// bir arayüzü (Bölüm 2.5.3) uygular
type Connection = Reader & Writer;

export struct Response {
    body: string,
    status_code: int,
}

// 'get' fonksiyonu SADECE 'std::net'i bilir.
export async fn get(url: string): (Response | error) {
    let host = try parse_host_from(url); // Native Vex

    // 1. 'std::net' katmanını kullan
    // 'conn' bir 'Connection' (Reader & Writer) tipidir.
    let conn = try await net.connect(host, 80);

    // 2. 'conn' (bir Writer) üzerine HTTP isteğini yaz
    let request_str = f"GET / HTTP/1.1\r\nHost: {host}\r\n\r\n";
    try await conn.write(request_str.to_bytes());

    // 3. 'conn' (bir Reader) üzerinden yanıtı oku
    let read_buf = make([byte], 4096); // Heap tahsisi (Bölüm 2.9)
    let n_read = try await conn.read(&mut read_buf);

    // 4. Yanıtı Vex'te parse et
    return Response::from_bytes(&read_buf[0..n_read]);
}

```

### Adım 2: `std::net` (Katman 2 - %100 Native Vex)

`std::net` paketi, HTTP'yi bilmez. O, sadece "dosyaları" (soketler) bilir ve bu dosyaları `std::io`'dan alır.

```javascript
// Dosya: std/net/tcp.vx
// Bu dosya da %100 güvenli, native Vex kodudur.

import { io } from "std";
import type { Reader, Writer } from "std::io";

// Bir 'TcpStream', 'std::io::File' struct'ını sarmalar (wrap).
export struct TcpStream {
    file: io.File,
}

// 'connect' fonksiyonu SADECE 'std::io'yu bilir.
export async fn connect(host: string, port: int): (TcpStream | error) {

    // 'io.open' bir soket (socket) açar (Linux'ta her şey dosyadır).
    // Bu, 'std::io' katmanına (Katman 1) yapılan bir çağrıdır.
    let f = try await io.open(f"/dev/tcp/{host}/{port}"); // Kavramsal yol

    return TcpStream{ file: f };
}

// 'TcpStream', 'Reader' arayüzünü (Bölüm 4) uygular.
fn (s: &TcpStream) read(buf: &mut [byte]): (int | error) {
    // Çağrıyı doğrudan 'std::io' katmanına devreder (delegates).
    return await s.file.read(buf);
}

// 'TcpStream', 'Writer' arayüzünü (Bölüm 4) uygular.
fn (s: &TcpStream) write(buf: &[byte]): (int | error) {
    // Çağrıyı doğrudan 'std::io' katmanına devreder.
    return await s.file.write(buf);
}

```

### Adım 3: `std::io` (Katman 1 - `unsafe` Vex Köprüsü)

**İşte "sihrin" gerçekleştiği yer burasıdır.** Bu paket, Vex Runtime'ı (Rust) ile konuşan _tek_ yerdir.

```javascript
// Dosya: std/io/file.vx
// BU DOSYA 'unsafe' KOD İÇERİR.

import { unsafe } from "std";
import { __runtime_io_read, __runtime_io_open } from "runtime:intrinsics";

export struct File {
    fd: i32, // 'file descriptor' (dosya tanıtıcısı)
}

// 'open', bir 'intrinsic' (özel fonksiyon) çağırır.
export async fn open(path: string): (File | error) {
    // 'runtime:intrinsics', derleyiciye (Rust) özel bir mesajdır.
    // Bu, Vex Runtime'ını (Rust) çağıran bir "kanca"dır (hook).
    let fd = await __runtime_io_open(path);

    if fd < 0 {
        return error.new("Dosya açılamadı.");
    }
    return File{ fd: fd };
}

// 'File.read' de bir 'intrinsic' çağırır.
fn (f: &File) read(buf: &mut [byte]): (int | error) {

    // 'unsafe' (Bölüm 2.6) kullanarak Vex'in güvenli
    // 'buf' slice'ından 'raw pointer' alırız.
    let buf_ptr: *mut byte = unsafe { &mut buf[0] as *mut byte };
    let buf_len: u32 = buf.len() as u32;

    // 1. Vex Task'ını (Bölüm 5) askıya al (suspend).
    // 2. Vex Runtime'ına (Katman 0 - Rust) 'io_uring' read isteği gönder.
    // 3. 'io_uring' tamamlandığında, Vex Runtime'ı bu task'ı uyandırır.
    let n_read = await __runtime_io_read(f.fd, buf_ptr, buf_len);

    if n_read < 0 {
        return error.new("Okuma hatası.");
    }
    return n_read as int;
}

```

### Adım 4: `std::hpc` ve `launch` (Diğer Köprü)

`std::io` gibi, `std::hpc` de bir "köprü" paketidir.

```javascript
// Dosya: std/hpc/launch.vx

// Bu fonksiyonlar, Vex Runtime'ının GPU/SIMD
// zamanlayıcısına bağlanan "intrinsic"lerdir.
import { __runtime_submit_hpc_kernel } from "runtime:intrinsics";

// Kullanıcının 'launch' (Bölüm 6.1) anahtar kelimesi,
// derleyici tarafından BU fonksiyona yapılan bir çağrıya dönüştürülür.
async fn launch_kernel(func_ptr: uintptr, ...args): (nil | error) {

    // Derleyici bayrağına (--accelerator) (Bölüm 6.2) göre,
    // bu 'intrinsic', işi CPU, SIMD veya GPU'ya gönderir.
    await __runtime_submit_hpc_kernel(func_ptr, ...args);
    return nil;
}

```

## Sonuç: İnşa Stratejisi

1. **Vex Runtime'ı (Rust):** Önce, `io_uring`'i ve `async` task zamanlayıcısını (Bölüm 5) yöneten Rust kütüphanesi yazılır. Bu kütüphane, `__runtime_io_read` gibi C-ABI (FFI) fonksiyonlarını "export" eder.
2. **`std::io`&#32;(Unsafe Vex):** Ardından, `runtime:intrinsics` kullanarak bu Rust fonksiyonlarına bağlanan Vex'in `std::io` paketi yazılır.
3. **`std::net`,&#32;`std::http`, vb. (Safe Vex):** Son olarak, _diğer tüm_ `std` paketleri, `std::io`'nun sağladığı güvenli ve "native" Vex API'sinin üzerine inşa edilir.

Bu sayede, `std::http`'yi yazan geliştiricinin (senin) `io_uring` veya `unsafe` pointer'lar hakkında düşünmesine gerek kalmaz.
