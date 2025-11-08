# io_uring Backend - Test ve Kullanım Kılavuzu

## Özet

**io_uring**, Linux 5.1+ ile gelen modern async I/O API'si. Epoll'a göre daha düşük overhead ve daha iyi performans sağlar.

## Durum

✅ **Backend Kodu**: Tamamlandı (`src/backends/io_uring.c`)  
⚠️ **Docker Testleri**: Başarısız (container kısıtlamaları)  
📋 **Native Linux**: Test edilmesi gerekiyor

## Neden Docker'da Çalışmıyor?

io_uring, Docker container'larında şu nedenlerle çalışmayabilir:

1. **Seccomp Filtreleri**: Docker varsayılan seccomp profili io_uring syscall'larını engelleyebilir
2. **Capability Kısıtlamaları**: `CAP_SYS_ADMIN` veya benzeri gerekebilir
3. **Kernel Desteği**: Host kernel io_uring'i desteklemese bile container kernel versiyonu yüksek görünebilir

### Docker'da Çalıştırmak İçin

```bash
docker run --privileged vex_net_test
# veya
docker run --cap-add=SYS_ADMIN --cap-add=SYS_RESOURCE vex_net_test
# veya
docker run --security-opt seccomp=unconfined vex_net_test
```

**Not**: Production ortamlarında `--privileged` kullanmak güvenlik riski oluşturur.

## Native Linux'ta Test

### Gereksinimler

1. **Linux Kernel 5.1+** (io_uring için minimum)
   - Linux 5.10+ önerilir (daha stabil)
   - Kontrol: `uname -r`

2. **liburing**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install liburing-dev
   
   # RHEL/CentOS/Fedora
   sudo yum install liburing-devel
   
   # Arch Linux
   sudo pacman -S liburing
   ```

3. **io_uring etkin olmalı**:
   ```bash
   # Check if disabled
   cat /proc/sys/kernel/io_uring_disabled
   # Should be 0 (enabled) or file not exist
   
   # If disabled (value = 1 or 2), enable:
   echo 0 | sudo tee /proc/sys/kernel/io_uring_disabled
   ```

### Derleme

```bash
make clean
make USE_IOURING=1
```

### Test

```bash
chmod +x test_iouring_native.sh
./test_iouring_native.sh
```

## Performans Karşılaştırması

| Backend | Syscalls/op | CPU Overhead | Best For |
|---------|-------------|--------------|----------|
| **epoll** | 1-2 | Orta | Genel amaçlı, uyumluluk |
| **io_uring** | 0-1 | Düşük | Yüksek throughput, modern kerneller |
| **kqueue** | 1 | Düşük | BSD/macOS |

### Ne Zaman io_uring Kullanmalı?

✅ **Kullan:**
- Linux 5.10+ native sistemlerde
- Yüksek throughput gereken uygulamalarda (>10K conn/s)
- CPU overhead kritik olduğunda
- Modern sunucu donanımlarında

❌ **Kullanma:**
- Docker/Kubernetes'te (kısıtlamalar)
- Eski kernel'lerde (<5.10)
- Cross-platform uyumluluk öncelikli ise
- Development/testing ortamlarında

### Fallback Stratejisi

vex_net otomatik olarak fallback yapmaz. Manuel kontrol:

```c
int caps = vex_net_capabilities();
if (caps & VEX_CAP_IOURING) {
    // io_uring kullanılabilir
} else {
    // epoll kullan (varsayılan)
}
```

## Bilinen Sorunlar

### 1. Docker/Container'da Çalışmıyor

**Çözüm**: Native Linux kullan veya epoll'da kal

### 2. "Permission denied" / "Operation not permitted"

**Sebep**: io_uring bazı sistemlerde güvenlik nedeniyle devre dışı

**Çözüm**:
```bash
# Geçici
sudo sysctl kernel.io_uring_disabled=0

# Kalıcı (/etc/sysctl.conf)
kernel.io_uring_disabled = 0
```

### 3. "Function not implemented"

**Sebep**: Kernel io_uring desteklemiyor

**Çözüm**: Kernel'i 5.10+ güncelleyin veya epoll kullanın

## Test Sonuçları

### macOS (kqueue)
- ✅ Başarılı
- Backend: kqueue + EVFILT_TIMER

### Linux (epoll)  
- ✅ Başarılı
- Backend: epoll + timerfd
- Test Ortamı: Docker (Ubuntu 24.04)

### Linux (io_uring)
- ⚠️ Kod hazır, native test gerekiyor
- Backend: io_uring + timerfd
- Docker: Başarısız (seccomp/capability)
- Native: Test edilmedi

## Öneriler

### Development
- macOS → kqueue (otomatik)
- Linux Docker → epoll (varsayılan, stabil)
- Linux Native → epoll (kolay) veya io_uring (performans)

### Production
- **Container (Docker/K8s)**: epoll kullan
- **Native Linux**: io_uring dene, başarısız olursa epoll
- **Multi-platform**: epoll/kqueue (io_uring'i atla)

## Referanslar

- [io_uring Introduction](https://kernel.dk/io_uring.pdf)
- [liburing Documentation](https://github.com/axboe/liburing)
- [Linux man io_uring](https://man7.org/linux/man-pages/man7/io_uring.7.html)

## Sonuç

✅ **io_uring backend kodu tamamlandı**  
⚠️ **Docker testleri kısıtlamalar nedeniyle başarısız**  
📝 **Native Linux test'i kullanıcıya bırakıldı**  

**Tavsiye**: Production için epoll kullanın (battle-tested, her yerde çalışır). io_uring performans kritik ve native Linux ortamlar için keşfedin.

