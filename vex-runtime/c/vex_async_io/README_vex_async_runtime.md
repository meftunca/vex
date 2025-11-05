# Vex Async Runtime (Pro Extensions Edition)

Vex Async Runtime, **Vex dili** için platformlar arası, yüksek performanslı bir **M:N async I/O runtime** çekirdeğidir.  
Bu sürüm (`with_pro_extras`) klasik scheduler altyapısına **timer**, **iptal (cancellation)**, **istatistik**, ve **auto-shutdown** desteklerini ekler.

---

## 📁 Proje Yapısı

```
vex_async_runtime/
├─ Makefile
├─ include/
│  ├─ runtime.h            # C ABI (Runtime + WorkerContext + coroutine tipleri)
│  ├─ poller.h             # C ABI (I/O reactor arayüzü)
│  ├─ lockfree_queue.h     # Lock-free MPMC ring buffer (Vyukov)
│  ├─ internal.h           # İç tipler ve yardımcılar
├─ src/
│  ├─ runtime.c            # M:N scheduler, worker/poller thread yönetimi (+ Pro eklemeler)
│  ├─ lockfree_queue.c     # MPMC kuyruk implementasyonu
│  ├─ poller_epoll.c       # Linux epoll reactor
│  ├─ poller_io_uring.c    # Linux io_uring reactor
│  ├─ poller_kqueue.c      # macOS/FreeBSD kqueue reactor
│  ├─ poller_iocp.c        # Windows IOCP reactor
│  └─ common.c             # xmalloc/xfree vs.
└─ tests/
   └─ example_async_demo.c # async sleep/print demo
```

---

## ⚙️ Derleme (Build)

### Otomatik Poller Seçimi
Makefile, bulunduğun platforma göre poller backend’ini otomatik belirler:

| Platform | Backend       | Açıklama |
|-----------|----------------|-----------|
| Linux (>=5.11) | `io_uring` | En yüksek performans |
| Linux (<5.11) | `epoll` | Uyumluluk modu |
| macOS / FreeBSD | `kqueue` | Native poller |
| Windows | `IOCP` | Kernel-level I/O Completion Ports |

Elle seçim yapmak için:
```bash
make POLLER=epoll     # Linux
make POLLER=io_uring  # Linux 5.11+
make POLLER=kqueue    # macOS/FreeBSD
make POLLER=iocp      # Windows
```

---

## 🧩 C API Özeti

### Runtime API
```c
Runtime* runtime_create(int num_workers);
void runtime_destroy(Runtime* runtime);
void runtime_spawn_global(Runtime* runtime, coro_resume_func fn, void* data);
void runtime_run(Runtime* runtime);
void runtime_shutdown(Runtime* runtime);
void runtime_set_tracing(Runtime* runtime, bool enabled);
```

### Worker API
```c
void worker_await_io(WorkerContext* ctx, int fd, EventType type);
void worker_spawn_local(WorkerContext* ctx, coro_resume_func fn, void* data);
```

---

## 🚀 Pro Özellikleri (Yeni API’ler)

### Timer API
```c
void worker_await_deadline(WorkerContext* ctx, uint64_t deadline_ns);
void worker_await_after(WorkerContext* ctx, uint64_t millis);
```
Coroutine belirli bir süre sonra yeniden planlanır.  
İçeride basit bir `poller_wait()` tabanlı timer kuyruğu simüle edilir.

### Cancellation API
```c
CancelToken* worker_cancel_token(WorkerContext* ctx);
bool cancel_requested(const CancelToken* t);
void cancel_request(CancelToken* t);
```
İptal sinyalleri coroutine’lere kontrollü durdurma sağlar.

### IO Handle Abstraction
```c
typedef uintptr_t IoHandle;
void worker_await_ioh(WorkerContext* ctx, IoHandle h, EventType type);
```
Tüm platformlarda `fd/SOCKET/HANDLE` tiplerini soyutlar.

### Auto Shutdown ve İstatistik
```c
void runtime_enable_auto_shutdown(Runtime* rt, bool enabled);
void runtime_get_stats(Runtime* rt, RuntimeStats* out);
```
`RuntimeStats` içinde temel sayaçlar tutulur (spawned, done, poller_events, vb).

---

## 🧪 Örnek Kullanım

```c
static CoroStatus my_coro(WorkerContext* ctx, void* data) {
    MyState* st = data;

    if (st->fd > 0) {
        worker_await_io(ctx, st->fd, EVENT_TYPE_READABLE);
        return CORO_STATUS_YIELDED;
    }

    if (st->countdown-- == 0) {
        free(st);
        return CORO_STATUS_DONE;
    }

    worker_await_after(ctx, 100); // 100ms sonra tekrar çalış
    return CORO_STATUS_RUNNING;
}

int main() {
    Runtime* rt = runtime_create(4);
    runtime_enable_auto_shutdown(rt, true);

    for (int i = 0; i < 8; ++i) {
        MyState* st = malloc(sizeof(*st));
        st->fd = -1;
        st->countdown = 5;
        runtime_spawn_global(rt, my_coro, st);
    }

    runtime_run(rt);
    runtime_destroy(rt);
    return 0;
}
```

---

## 📊 RuntimeStats Örneği
```c
RuntimeStats stats;
runtime_get_stats(rt, &stats);
printf("Tasks done: %llu, Events: %llu\n",
       (unsigned long long)stats.tasks_done,
       (unsigned long long)stats.poller_events);
```

---

## 🛠️ Geliştirici Notları

- **LockFreeQueue**: Dmitry Vyukov MPMC algoritması.
- **Thread modeli**: 1 poller + N worker thread.
- **Scheduler**: Lokal/global ready kuyrukları, M:N coroutine dağıtımı.
- **Hata sözleşmesi**: `int` dönenlerde `0=OK`, `-1=ERROR (+errno)`.

---

## 📚 Lisans ve Katkı

MIT Lisansı altındadır.  
Katkı ve öneriler için PR veya Issue açabilirsiniz.

---

© 2025 Muhammed Burak Şentürk — *Vex Language Runtime Core*
