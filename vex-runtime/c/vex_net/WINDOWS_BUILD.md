# Windows Build ve Test Kılavuzu

## Gereksinimler

1. **Visual Studio 2019/2022** veya **Build Tools for Visual Studio**
   - Download: https://visualstudio.microsoft.com/downloads/
   - "Desktop development with C++" workload seçin

2. **Git for Windows** (opsiyonel)
   - Download: https://git-scm.com/download/win

## Derleme (MSVC ile)

### 1. Developer Command Prompt'u açın
```
Başlat → Visual Studio 2022 → Developer Command Prompt for VS 2022
```

### 2. Projeye gidin
```powershell
cd vex-clibs\vex_net
```

### 3. Derleyin
```powershell
# Manuel derleme
cl /O2 /W3 /c /Iinclude src\socket_ops.c
cl /O2 /W3 /c /Iinclude src\dns_dialer.c
cl /O2 /W3 /c /Iinclude src\proxy_helpers.c
cl /O2 /W3 /c /Iinclude src\backends\iocp.c

# Library oluştur
lib /OUT:vexnet.lib socket_ops.obj dns_dialer.obj proxy_helpers.obj iocp.obj

# Echo server derle
cl /O2 /W3 /Iinclude examples\echo_server.c vexnet.lib ws2_32.lib
```

### 4. Test Edin
```powershell
# Terminal 1
.\echo_server.exe

# Terminal 2
Test-NetConnection -ComputerName localhost -Port 9000
# veya
.\test_windows.ps1
```

## Derleme (CMake ile)

```powershell
mkdir build
cd build
cmake .. -G "Visual Studio 17 2022" -A x64
cmake --build . --config Release
.\Release\echo_server.exe
```

## Sorun Giderme

### WinSock hataları
Eğer `ws2_32.lib` hatası alırsanız:
```powershell
# Link aşamasında ekleyin
cl ... /link ws2_32.lib
```

### Header bulunamıyor
```powershell
# include path'i kontrol edin
cl /I"C:\Program Files (x86)\Windows Kits\10\Include\..." ...
```

## Otomatik Test

PowerShell script ile:
```powershell
.\test_windows.ps1
```

Bu script:
- ✓ Binary kontrolü
- ✓ Server başlatma
- ✓ Connection testi
- ✓ Echo testi
- ✓ Cleanup

## Beklenen Çıktı

```
=======================================================
  Vex Net Windows Test Suite (IOCP)
=======================================================

▶ Test 1: Binary Check
✓ echo_server.exe found

▶ Test 2: Echo Server
✓ Server started (PID: 1234)

▶ Test 3: Connection Test
✓ Connection successful

▶ Test 4: Echo Test
✓ Echo working correctly

=======================================================
  ✅ ALL WINDOWS TESTS PASSED
=======================================================
```

## Notlar

- **IOCP backend**: Windows'un native event loop'u
- **Thread Pool Timer**: Yüksek performanslı zamanlayıcı
- **Winsock2**: Modern Windows socket API
- **C11 uyumluluk**: MSVC /std:c11 (VS 2019+)

## CI/CD Entegrasyonu

GitHub Actions ile otomatik test:
```yaml
# .github/workflows/windows-test.yml dosyası mevcut
```

GitHub'a push yaptığınızda otomatik Windows testleri çalışır.

