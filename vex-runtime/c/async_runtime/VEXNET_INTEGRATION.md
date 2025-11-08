# vex_net Integration Proposal

## Motivasyon

`vex_net` şu anda duplicate event loop implementasyonu:
- async_runtime: `poller_*.c` (85 LOC her biri)
- vex_net: `backends/*.c` (100-200 LOC her biri)

**Avantajlar:**
1. ✅ Kod tekrarını önler
2. ✅ vex_net'in ek özelliklerini kullanır (timer, socket helpers)
3. ✅ Tek bir test suite
4. ✅ Daha kolay bakım

## Entegrasyon Planı

### Adım 1: Poller API'sini vex_net ile Değiştir

**Önce** (async_runtime/src/poller_kqueue.c):
```c
typedef struct Poller {
    int kq;
} Poller;

Poller* poller_create() {
    Poller* p = malloc(sizeof(Poller));
    p->kq = kqueue();
    return p;
}
```

**Sonra** (async_runtime/src/poller_vexnet.c):
```c
#include "poller.h"
#include "vex_net.h"

typedef struct Poller {
    VexNetLoop loop;
} Poller;

Poller* poller_create() {
    Poller* p = malloc(sizeof(Poller));
    if (!p) return NULL;
    
    if (vex_net_loop_create(&p->loop) != 0) {
        free(p);
        return NULL;
    }
    
    return p;
}

void poller_destroy(Poller* p) {
    if (!p) return;
    vex_net_loop_close(&p->loop);
    free(p);
}

int poller_add(Poller* p, int fd, EventType type, void* user_data) {
    if (!p) return -1;
    
    uint32_t events = 0;
    if (type & EVENT_TYPE_READABLE) events |= VEX_EVT_READ;
    if (type & EVENT_TYPE_WRITABLE) events |= VEX_EVT_WRITE;
    
    return vex_net_register(&p->loop, fd, events, (uintptr_t)user_data);
}

int poller_remove(Poller* p, int fd) {
    if (!p) return -1;
    return vex_net_unregister(&p->loop, fd);
}

int poller_wait(Poller* p, ReadyEvent* events, int max_events, int timeout_ms) {
    if (!p || !events) return 0;
    
    VexEvent vex_events[1024];
    if (max_events > 1024) max_events = 1024;
    
    int n = vex_net_tick(&p->loop, vex_events, max_events, timeout_ms);
    if (n <= 0) return 0;
    
    // Convert VexEvent -> ReadyEvent
    for (int i = 0; i < n; i++) {
        events[i].fd = vex_events[i].fd;
        events[i].type = EVENT_TYPE_NONE;
        
        if (vex_events[i].events & VEX_EVT_READ)  
            events[i].type |= EVENT_TYPE_READABLE;
        if (vex_events[i].events & VEX_EVT_WRITE) 
            events[i].type |= EVENT_TYPE_WRITABLE;
        
        events[i].user_data = (void*)vex_events[i].userdata;
    }
    
    return n;
}
```

### Adım 2: Makefile Güncelleme

**async_runtime/Makefile**:
```makefile
# Add vex_net dependency
VEXNET_DIR = ../../../vex-clibs/vex_net
VEXNET_LIB = $(VEXNET_DIR)/libvexnet.a

CFLAGS += -I$(VEXNET_DIR)/include
LDFLAGS += $(VEXNET_LIB)

# Replace platform-specific pollers with vexnet adapter
SRC = src/runtime.c src/worker_context.c src/common.c \
      src/lockfree_queue.c src/poller_vexnet.c

# Build vexnet first
$(VEXNET_LIB):
\t$(MAKE) -C $(VEXNET_DIR)

libvex_async.a: $(OBJS) $(VEXNET_LIB)
\tar rcs $@ $(OBJS)
```

### Adım 3: Ek Özellikler (Bonus)

vex_net'in sunduğu ekstra özellikler:

#### 3.1 Timer Desteği
```c
// runtime.h'a ekle
void worker_await_timer_vexnet(WorkerContext* context, uint64_t ms);

// implementation
void worker_await_timer_vexnet(WorkerContext* context, uint64_t ms) {
    Poller* p = get_poller(context);
    vex_net_timer_after(&p->loop, ms, (uintptr_t)context->current_task);
    // Task will be resumed when timer fires
}
```

#### 3.2 Socket Helpers
```c
// Expose vex_net socket helpers in runtime
int runtime_socket_tcp(bool ipv6);
int runtime_socket_bind(int fd, const char* ip, uint16_t port);
int runtime_socket_listen(int fd, int backlog);
int runtime_socket_accept(int fd, char* ip_buf, size_t len, uint16_t* port);
int runtime_socket_connect(int fd, const char* ip, uint16_t port);
```

## Değişiklik Özeti

### Silinecek Dosyalar
- ❌ `src/poller_kqueue.c` (85 LOC)
- ❌ `src/poller_epoll.c` (90 LOC)
- ❌ `src/poller_io_uring.c` (120 LOC)
- ❌ `src/poller_iocp.c` (95 LOC)

**Toplam**: ~390 LOC silinecek

### Eklenecek Dosyalar
- ✅ `src/poller_vexnet.c` (60 LOC adapter)
- ✅ Dependency: `vex_net` (zaten mevcut)

**Net**: ~330 LOC azalma + vex_net özellikleri

## Test Planı

1. ✅ Mevcut async_runtime testlerini çalıştır
   ```bash
   cd async_runtime
   make test
   ```

2. ✅ vex_net testlerini çalıştır
   ```bash
   cd vex_net
   ./test_basic.sh        # macOS
   ./run_docker_tests.sh  # Linux
   ```

3. ✅ Integration test
   ```bash
   # async_runtime'ın socket testini vex_net ile çalıştır
   ./tests/test_real_io_socket
   ```

## Zaman Çizelgesi

- **Hazırlık**: 30 dakika (adapter yazma)
- **Test**: 1 saat (tüm platformları kontrol)
- **Cleanup**: 15 dakika (eski dosyaları silme)

**Toplam**: ~2 saat

## Risk Değerlendirmesi

| Risk | Olasılık | Etki | Çözüm |
|------|----------|------|-------|
| API uyumsuzluğu | Düşük | Orta | Adapter layer zaten benzer |
| Performans kaybı | Çok Düşük | Düşük | Aynı syscall'lar |
| Test başarısızlığı | Düşük | Orta | Her platform test edildi |

## Sonuç

✅ **ÖNERİLİR**: vex_net ile entegrasyon
- Kod tekrarını azaltır
- Daha fazla özellik (timer, socket helpers)
- Tek bir bakım noktası
- Test kapsamı iki katına çıkar

---

## Hemen Başlamak İçin

```bash
cd /Users/mapletechnologies/Desktop/big_projects/vex_lang/vex-runtime/c/async_runtime

# 1. Adapter oluştur
cat > src/poller_vexnet.c << 'EOF'
#include "poller.h"
#include "vex_net.h"
#include <stdlib.h>

typedef struct Poller {
    VexNetLoop loop;
} Poller;

Poller* poller_create() {
    Poller* p = malloc(sizeof(Poller));
    if (!p) return NULL;
    if (vex_net_loop_create(&p->loop) != 0) {
        free(p);
        return NULL;
    }
    return p;
}

void poller_destroy(Poller* p) {
    if (!p) return;
    vex_net_loop_close(&p->loop);
    free(p);
}

int poller_add(Poller* p, int fd, EventType type, void* user_data) {
    if (!p) return -1;
    uint32_t events = 0;
    if (type & EVENT_TYPE_READABLE) events |= VEX_EVT_READ;
    if (type & EVENT_TYPE_WRITABLE) events |= VEX_EVT_WRITE;
    return vex_net_register(&p->loop, fd, events, (uintptr_t)user_data);
}

int poller_remove(Poller* p, int fd) {
    if (!p) return -1;
    return vex_net_unregister(&p->loop, fd);
}

int poller_wait(Poller* p, ReadyEvent* events, int max_events, int timeout_ms) {
    if (!p || !events) return 0;
    
    VexEvent vex_events[1024];
    if (max_events > 1024) max_events = 1024;
    
    int n = vex_net_tick(&p->loop, vex_events, max_events, timeout_ms);
    if (n <= 0) return 0;
    
    for (int i = 0; i < n; i++) {
        events[i].fd = vex_events[i].fd;
        events[i].type = EVENT_TYPE_NONE;
        if (vex_events[i].events & VEX_EVT_READ)  events[i].type |= EVENT_TYPE_READABLE;
        if (vex_events[i].events & VEX_EVT_WRITE) events[i].type |= EVENT_TYPE_WRITABLE;
        events[i].user_data = (void*)vex_events[i].userdata;
    }
    
    return n;
}
EOF

# 2. Makefile'ı güncelle (USE_VEXNET=1 flag ekle)

# 3. Test et
make clean
make USE_VEXNET=1
./run_tests.sh
```

