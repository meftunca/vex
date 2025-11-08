# Vex Net — Ultra Pack (Cross-Platform)

Bu paket, Vex’in net std kütüphanesine gömülmek üzere ince bir C-ABI yüzeyi sağlar.

## Platformlar

- **Linux**: epoll + timerfd; UDP GSO/MSG_ZEROCOPY (best effort); opsiyonel io_uring probe
- **BSD/macOS**: kqueue + EVFILT_TIMER
- **Windows**: IOCP + Threadpool Timer (IOCQueue’ya completion post’lanır)

## Özellikler

- Event loop (`register/modify/unregister/tick`), per-shot timer
- TCP/UDP soket yardımcıları, keepalive/nodelay/tos/buffer
- **Dialer**: nonblocking resolve+dial + **Happy Eyeballs v2 helper** (arka plan thread ile)
- **Proxy**: HTTP CONNECT, SOCKS5 (kısa bloklamalı; istersen tamamen nonblocking FSM, Vex katmanında)
- **Linux extralar**: UDP GSO, MSG_ZEROCOPY enable fonksiyonları
- **TLS hook**: fd’yi Vex tarafındaki TLS katmanına geçirebileceğin sade arayüz

## Derleme

```bash
make
./examples/echo_server        # 0.0.0.0:9000
./examples/client_connect example.com 80
```

İçerik (özet)

include/vex_net.h — tek C-ABI yüzeyi: loop (register/modify/unregister/tick), timers, sockets, DNS/Dialer (HEv2 helper), I/O, proxy helpers, Linux extras, TLS hook yüzeyi.

src/backends/epoll.c — Linux epoll + timerfd; caps: EPOLLEXCLUSIVE, UDP_GSO, MSG_ZEROCOPY.

src/backends/kqueue.c — BSD/macOS kqueue + EVFILT_TIMER.

src/backends/iocp.c — Windows IOCP; CreateThreadpoolTimer ile süre dolunca PostQueuedCompletionStatus.

src/socket_ops.c — platform-bağımsız soket işlemleri (TCP/UDP, keepalive, nodelay, tos, buffer).

src/dns_dialer.c — nonblocking resolve+dial ve Happy Eyeballs v2 helper (arka plan thread ile tetiklenip, loop’a completion post’lanır).

src/proxy_helpers.c — HTTP CONNECT ve SOCKS5 (kısa timeout’lu, adım adım; istersen tamamen nonblocking FSM’i Vex katmanında kur).

src/linux/uring_probe.c — VEX_NET_HAVE_URING tanımlıysa liburing ile probelama (tam I/O yolu için genişletilebilir).

examples/echo_server.c — 0.0.0.0:9000 üzerinde echo.

examples/client_connect.c — HEv2 ile TCP bağlanıp basit HTTP GET; proxy kullanımına örnek not.

Makefile & CMakeLists.txt — POSIX ve Windows yapıları.
Windows için CMake projesi dahildir.

## Test Etme

### macOS (kqueue backend)

```bash
make
./test_basic.sh
```

### Linux (epoll backend)

Docker ile:

```bash
./run_docker_tests.sh
```

Doğrudan Linux'ta:

```bash
make
./test_linux.sh
```

### Windows (IOCP backend)

**Not**: Windows testleri Windows host gerektirir. macOS/Linux Docker'da çalışmaz.

#### Seçenek 1: GitHub Actions (Önerilen)

GitHub'a push yapın, CI/CD otomatik Windows testlerini çalıştırır.

#### Seçenek 2: Windows Host + Docker

```powershell
docker build -f Dockerfile.windows -t vex_net_windows .
docker run --rm vex_net_windows
```

#### Seçenek 3: Doğrudan Windows'ta

```powershell
# MSVC ile derleme
nmake /f Makefile.win

# Testleri çalıştır
.\test_windows.ps1
```

### Manuel Test

```bash
./examples/echo_server        # 0.0.0.0:9000
# Başka terminalde:
echo "Hello" | nc localhost 9000
```

## Notlar

- io_uring tam I/O yolu bu pakette örneklenmedi; `VEX_NET_HAVE_URING` ile probe dosyası var.
- Proxy yardımcıları opsiyoneldir; yüksek performans gereksiniminde nonblocking FSM'i Vex tarafında kurmanızı tavsiye ederiz.
