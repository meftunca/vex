// #include "runtime.h"
// #include <stdio.h>
// #include <stdlib.h>
// #include <time.h>

// typedef struct {
//     int id;
//     int remaining_ticks;
// } SleepPrint;

// static CoroStatus sleep_print(WorkerContext* ctx, void* data) {
//     SleepPrint* sp = (SleepPrint*)data;
//     printf("[coro %d] tick (%d left)\n", sp->id, sp->remaining_ticks);
//     if (--sp->remaining_ticks <= 0) {
//         free(sp);
//         return CORO_STATUS_DONE;
//     }
//     // simulate async yield without blocking: just yield control
//     return CORO_STATUS_RUNNING;
// }

// int main() {
//     Runtime* rt = runtime_create(4);
//     runtime_set_tracing(rt, false);

//     for (int i = 0; i < 8; ++i) {
//         SleepPrint* sp = (SleepPrint*)malloc(sizeof(SleepPrint));
//         sp->id = i;
//         sp->remaining_ticks = 5 + (i % 3);
//         runtime_spawn_global(rt, sleep_print, sp);
//     }

//     runtime_run(rt);
//     runtime_destroy(rt);
//     return 0;
// }

// tests/complex_pipeline_demo.c
// Daha kompleks bir örnek: MPMC kuyruk üzerinde çok üreticili / çok tüketicili işleme hattı.
// - 3 producer: her biri N mesaj üretir, aralarda await_after ile yield eder
// - 4 consumer: kuyruktan mesaj çeker, "işler", aralarda yield eder
// - supervisor: tüm işler bittiğinde runtime_shutdown çağırır
//
// Notlar:
// * Bu test LockFreeQueue API'sini de kullanır (MPMC halka kuyruk).
// * IO beklemeyen bir iş hattıdır; zamanlayıcı (worker_await_after) ile kooperatif ilerler.
// * Derleme: make && ./bin/tests_complex_pipeline_demo (Makefile hedefi yoksa elle derleyin)
//
#include <stdio.h>
#include <stdlib.h>
#include <stdatomic.h>
#include <stdint.h>
#include <string.h>
#include "../include/runtime.h"
#include "../include/lockfree_queue.h"

#ifndef ARRAY_SIZE
#define ARRAY_SIZE(a) ((int)(sizeof(a) / sizeof((a)[0])))
#endif

typedef struct Msg
{
    int producer_id;
    int seq;
    uint64_t payload;
} Msg;

// Paylaşılan durum
typedef struct Shared
{
    LockFreeQueue *q;
    atomic_int producers_alive;
    atomic_long produced_total;
    atomic_long consumed_total;
    int expected_per_producer;
} Shared;

static Runtime *g_rt = NULL; // supervisor içinde runtime_shutdown için kullanacağız

// Basit log makroları
#define LOGF(fmt, ...)                                              \
    do                                                              \
    {                                                               \
        fprintf(stderr, "[%s] " fmt "\n", __func__, ##__VA_ARGS__); \
    } while (0)

static CoroStatus producer_coro(WorkerContext *ctx, void *data)
{
    Shared *sh = (Shared *)data;
    // Producer kimliği için basit bir sayaç
    static _Atomic int next_id = 0;
    int my_id = atomic_fetch_add(&next_id, 1);

    int N = sh->expected_per_producer;
    for (int i = 0; i < N; ++i)
    {
        Msg *m = (Msg *)malloc(sizeof(Msg));
        m->producer_id = my_id;
        m->seq = i;
        m->payload = (uint64_t)(my_id * 1000000 + i);
        while (!lfq_enqueue(sh->q, (void *)m))
        {
            // Kuyruk doluysa kısa bir bekleme ile tekrar dene
            worker_await_after(ctx, 1); // 1ms
            return CORO_STATUS_YIELDED;  // Timer bekle!
        }
        atomic_fetch_add(&sh->produced_total, 1);
        // üretim hızı: 2ms
        worker_await_after(ctx, 2);
        return CORO_STATUS_YIELDED;  // Timer bekle!
    }

    // bitti
    atomic_fetch_sub(&sh->producers_alive, 1);
    return CORO_STATUS_DONE;
}

static CoroStatus consumer_coro(WorkerContext *ctx, void *data)
{
    Shared *sh = (Shared *)data;
    void *vp = NULL;
    if (lfq_dequeue(sh->q, &vp))
    {
        Msg *m = (Msg *)vp;
        // "işleme" simülasyonu
        worker_await_after(ctx, 1 + (m->payload % 3));
        atomic_fetch_add(&sh->consumed_total, 1);
        free(m);
        return CORO_STATUS_YIELDED;  // Timer bekle!
    }
    else
    {
        // Kuyruk boş: ya üretici üretiyor ya da tüm üreticiler bitti ve sırada iş yok
        if (atomic_load(&sh->producers_alive) == 0)
        {
            // kuyruk da boş => tamamen bitti
            return CORO_STATUS_DONE;
        }
        // kısa bekleme ile tekrar dene
        worker_await_after(ctx, 1);
        return CORO_STATUS_YIELDED;  // Timer bekle!
    }
}

static CoroStatus supervisor_coro(WorkerContext *ctx, void *data)
{
    (void)ctx;
    Shared *sh = (Shared *)data;
    long produced = atomic_load(&sh->produced_total);
    long consumed = atomic_load(&sh->consumed_total);

    if (atomic_load(&sh->producers_alive) == 0)
    {
        // Tüm üreticiler bitti; kuyruk drenajını bekle
        if (produced == consumed)
        {
            LOGF("Done: produced=%ld consumed=%ld -> shutdown", produced, consumed);
            runtime_shutdown(g_rt);
            return CORO_STATUS_DONE;
        }
    }

    // durum raporu periyodik
    static int tick = 0;
    if ((tick++ % 100) == 0)
    {
        fprintf(stderr, "[supervisor] produced=%ld consumed=%ld producers_alive=%d\n",
                produced, consumed, atomic_load(&sh->producers_alive));
    }

    worker_await_after(ctx, 5);
    return CORO_STATUS_YIELDED;  // Timer bekle!
}

int main(void)
{
    const int NUM_WORKERS = 4;
    const int NUM_PRODUCERS = 3;
    const int NUM_CONSUMERS = 4;
    const int PER_PRODUCER = 250; // toplam ~750 mesaj

    Runtime *rt = runtime_create(NUM_WORKERS);
    g_rt = rt;
    runtime_enable_auto_shutdown(rt, false); // kapanışı supervisor yapacak
    runtime_set_tracing(rt, false);

    Shared *sh = (Shared *)malloc(sizeof(Shared));
    memset(sh, 0, sizeof(*sh));
    sh->q = lfq_create(1024);
    atomic_store(&sh->producers_alive, NUM_PRODUCERS);
    sh->expected_per_producer = PER_PRODUCER;

    // Spawn producers
    for (int i = 0; i < NUM_PRODUCERS; ++i)
    {
        runtime_spawn_global(rt, producer_coro, sh);
    }
    // Spawn consumers
    for (int i = 0; i < NUM_CONSUMERS; ++i)
    {
        runtime_spawn_global(rt, consumer_coro, sh);
    }
    // Spawn supervisor
    runtime_spawn_global(rt, supervisor_coro, sh);

    fprintf(stderr, "complex_pipeline_demo: starting runtime...\n");
    runtime_run(rt);
    fprintf(stderr, "complex_pipeline_demo: runtime joined.\n");

    RuntimeStats stats;
    memset(&stats, 0, sizeof(stats));
    runtime_get_stats(rt, &stats);
    fprintf(stderr, "stats: tasks_done=%llu events=%llu io_submitted=%llu\n",
            (unsigned long long)stats.tasks_done,
            (unsigned long long)stats.poller_events,
            (unsigned long long)stats.io_submitted);

    lfq_destroy(sh->q);
    free(sh);
    runtime_destroy(rt);
    return 0;
}
