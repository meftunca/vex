# Manual Testing Guide: async_runtime with vex_net

Bu dokümman async_runtime'ı vex_net ile manuel test etmek için adım adım kılavuz.

## Adım 1: Konfigürasyonu Kontrol Et

```bash
cd /Users/mapletechnologies/Desktop/big_projects/vex_lang/vex-runtime/c/async_runtime
make print-config USE_VEXNET=1
```

**Beklenen çıktı**:
```
OS=Darwin (veya Linux)
KERNEL=...
POLLER=vexnet
USE_VEXNET=1
VEXNET_LIB=../vex_net/libvexnet.a
```

## Adım 2: vex_net'i Derle

```bash
cd ../vex_net
make clean
make
```

**Kontrol**:
```bash
ls -lh libvexnet.a
# Dosya var mı ve boyutu > 0 mı?
```

## Adım 3: async_runtime'ı vex_net ile Derle

```bash
cd ../async_runtime
make clean
make USE_VEXNET=1
```

**Beklenen**:
- ✅ `src/poller_vexnet.o` oluşmalı
- ✅ `async_runtime_demo` oluşmalı
- ❌ "error:" mesajı olmamalı

## Adım 4: Demo'yu Çalıştır

```bash
./async_runtime_demo
```

**Beklenen**:
- Program sonlanmalı (timeout olmadan)
- "done" veya benzeri başarı mesajı

## Adım 5: Integration Test'i Derle ve Çalıştır

```bash
make test_vexnet_integration USE_VEXNET=1
./test_vexnet_integration
```

**Beklenen çıktı**:
```
═══════════════════════════════════════════════════════
  vex_net + async_runtime Integration Test
═══════════════════════════════════════════════════════

✓ Server listening on port 18888
✓ Runtime created (using vex_net backend)
✓ Spawned 10 client tasks

▶ Running async tasks...

▶ Results:
  Messages echoed: 10/10
  Tasks spawned: ...
  Tasks completed: ...
  Poller events: ...

✅ Integration test PASSED!
═══════════════════════════════════════════════════════
```

## Adım 6: Mevcut Test Suite'i Çalıştır

```bash
# Önce native poller ile test et (karşılaştırma için)
make clean
make
./run_tests.sh

# Şimdi vex_net ile
make clean
make USE_VEXNET=1
./run_tests.sh
```

**Karşılaştır**: Her iki durumda da aynı sayıda test geçmeli.

## Adım 7: Basit Performans Testi

```bash
# Küçük bir test programı
cat > /tmp/perf_simple.c << 'EOF'
#include "include/runtime.h"
#include <stdio.h>
#include <time.h>

static int completed = 0;

static CoroStatus simple_task(WorkerContext* ctx, void* data) {
    (void)ctx; (void)data;
    completed++;
    return CORO_STATUS_DONE;
}

int main() {
    struct timespec start, end;
    Runtime* rt = runtime_create(2);
    runtime_enable_auto_shutdown(rt, 1);
    
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    for (int i = 0; i < 1000; i++) {
        runtime_spawn_global(rt, simple_task, NULL);
    }
    
    runtime_run(rt);
    
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    double elapsed = (end.tv_sec - start.tv_sec) + 
                    (end.tv_nsec - start.tv_nsec) / 1e9;
    
    printf("Completed: %d/1000 tasks\n", completed);
    printf("Time: %.3f seconds\n", elapsed);
    printf("Throughput: %.0f tasks/sec\n", completed / elapsed);
    
    runtime_destroy(rt);
    
    return (completed == 1000) ? 0 : 1;
}
EOF

# Derle
gcc -std=c11 -O2 -Wall -pthread \
    -I. -I../vex_net/include \
    -o /tmp/perf_simple /tmp/perf_simple.c \
    src/runtime.o src/worker_context.o src/lockfree_queue.o \
    src/common.o src/poller_vexnet.o \
    ../vex_net/libvexnet.a

# Çalıştır
/tmp/perf_simple
```

**Beklenen**:
```
Completed: 1000/1000 tasks
Time: 0.XXX seconds
Throughput: XXXX tasks/sec
```

## Adım 8: Memory Leak Check (Opsiyonel)

```bash
# valgrind varsa
valgrind --leak-check=full --show-leak-kinds=all \
    ./async_runtime_demo

# veya
valgrind --leak-check=summary \
    ./test_vexnet_integration
```

## Başarı Kriterleri

✅ **BAŞARILI** eğer:
1. vex_net ve async_runtime derleme hatası yok
2. async_runtime_demo çalışıyor
3. test_vexnet_integration PASSED
4. Mevcut test suite'i aynı sonuçları veriyor
5. Performans makul (>1000 tasks/sec)
6. Bellek sızıntısı yok

## Hata Ayıklama

### Derleme Hatası: "vex_net.h not found"

```bash
# Kontrol et
ls -l ../vex_net/include/vex_net.h

# Makefile'da VEXNET_INC doğru olmalı
make print-config USE_VEXNET=1
```

### Linking Hatası: "undefined reference"

```bash
# vex_net library'yi kontrol et
ls -lh ../vex_net/libvexnet.a
nm ../vex_net/libvexnet.a | grep vex_net_loop_create

# Yeniden derle
cd ../vex_net && make clean && make
cd ../async_runtime && make clean && make USE_VEXNET=1
```

### Runtime Hatası: Segfault

```bash
# gdb ile debug
gdb ./async_runtime_demo
(gdb) run
(gdb) bt

# veya
lldb ./async_runtime_demo
(lldb) run
(lldb) bt
```

### Performans Düşük

```bash
# Optimization flag'leri kontrol et
make print-config USE_VEXNET=1 | grep CFLAGS

# -O2 veya -O3 olmalı
```

## Platform Notları

### macOS (kqueue backend)
- ✅ Test edildi, çalışıyor
- Backend: vex_net → kqueue → async_runtime

### Linux (epoll backend)
- ✅ Test edilmeli
- Backend: vex_net → epoll → async_runtime

### Linux (io_uring backend)
- ⚠️ vex_net USE_IOURING=1 ile derlenebilir
- Native Linux gerektirir (Docker'da sorunlu)

## Sonuç

Bu adımların hepsi başarılıysa:

**✅ async_runtime vex_net ile mükemmel çalışıyor!**

- Kod tekrarı önlendi (~390 LOC)
- Unified event loop
- Cross-platform
- Performans korundu
- API backward compatible

---

**Test Raporu Şablonu**:

```
Platform: macOS/Linux
Date: $(date)
vex_net backend: $(make -C ../vex_net print-config | grep backend)

Test Results:
[ ] Compilation: PASS/FAIL
[ ] Demo: PASS/FAIL
[ ] Integration: PASS/FAIL  
[ ] Test Suite: X/Y tests passed
[ ] Performance: XXX tasks/sec
[ ] Memory: No leaks detected

Status: PASS/FAIL
Notes: ...
```

