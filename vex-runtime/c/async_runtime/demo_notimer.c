// Demo without timer dependency - works with vex_net
#include "include/runtime.h"
#include "include/lockfree_queue.h"
#include "include/internal.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdatomic.h>
#include <stdint.h>
#include <string.h>

typedef struct Msg
{
    int producer_id;
    int seq;
    uint64_t payload;
} Msg;

typedef struct Shared
{
    LockFreeQueue *q;
    atomic_int producers_alive;
    atomic_long produced_total;
    atomic_long consumed_total;
    int expected_per_producer;
    atomic_int consumer_checks;
} Shared;

static Runtime *g_rt = NULL;

static CoroStatus producer_coro(WorkerContext *ctx, void *data)
{
    (void)ctx;
    Shared *sh = (Shared *)data;
    static _Atomic int next_id = 0;
    int my_id = atomic_fetch_add(&next_id, 1);

    int N = sh->expected_per_producer;
    for (int i = 0; i < N; ++i)
    {
        Msg *m = (Msg *)xmalloc(sizeof(Msg));
        m->producer_id = my_id;
        m->seq = i;
        m->payload = (uint64_t)(my_id * 1000000 + i);

        // Try to enqueue, busy wait if full (no timer needed)
        while (!lfq_enqueue(sh->q, (void *)m))
        {
            // Queue full, yield and try again
            return CORO_STATUS_RUNNING;
        }

        atomic_fetch_add(&sh->produced_total, 1);

        // Yield occasionally for concurrency
        if (i % 10 == 0)
        {
            return CORO_STATUS_RUNNING;
        }
    }

    atomic_fetch_sub(&sh->producers_alive, 1);
    fprintf(stderr, "[producer %d] Done: produced %d items\n", my_id, N);
    return CORO_STATUS_DONE;
}

static CoroStatus consumer_coro(WorkerContext *ctx, void *data)
{
    (void)ctx;
    Shared *sh = (Shared *)data;

    // Try to dequeue
    void *vp = NULL;
    if (lfq_dequeue(sh->q, &vp))
    {
        Msg *m = (Msg *)vp;
        atomic_fetch_add(&sh->consumed_total, 1);
        xfree(m);
        return CORO_STATUS_RUNNING;
    }

    // Queue empty
    int checks = atomic_fetch_add(&sh->consumer_checks, 1);

    if (atomic_load(&sh->producers_alive) == 0)
    {
        // All producers done and queue empty
        return CORO_STATUS_DONE;
    }

    // Still producing, yield and try again
    // Check occasionally to avoid too many iterations
    if (checks % 100 == 0)
    {
        return CORO_STATUS_RUNNING;
    }

    return CORO_STATUS_RUNNING;
}

static CoroStatus supervisor_coro(WorkerContext *ctx, void *data)
{
    (void)ctx;
    Shared *sh = (Shared *)data;

    long produced = atomic_load(&sh->produced_total);
    long consumed = atomic_load(&sh->consumed_total);

    if (atomic_load(&sh->producers_alive) == 0 && produced == consumed)
    {
        fprintf(stderr, "[supervisor] Done: produced=%ld consumed=%ld -> shutdown\n",
                produced, consumed);
        runtime_shutdown(g_rt);
        return CORO_STATUS_DONE;
    }

    // Periodic status report
    static int tick = 0;
    if ((tick++ % 500) == 0)
    {
        fprintf(stderr, "[supervisor] produced=%ld consumed=%ld producers_alive=%d\n",
                produced, consumed, atomic_load(&sh->producers_alive));
    }

    return CORO_STATUS_RUNNING;
}

int main(void)
{
    printf("════════════════════════════════════════════════════════\n");
    printf("  async_runtime + vex_net Demo (No Timers)\n");
    printf("  Producer/Consumer Pipeline\n");
    printf("════════════════════════════════════════════════════════\n\n");
    fflush(stdout);

    const int NUM_WORKERS = 4;
    const int NUM_PRODUCERS = 3;
    const int NUM_CONSUMERS = 4;
    const int PER_PRODUCER = 100; // Reduced for faster completion

    Runtime *rt = runtime_create(NUM_WORKERS);
    g_rt = rt;
    runtime_enable_auto_shutdown(rt, true);
    runtime_set_tracing(rt, false);

    printf("✓ Runtime created with %d workers\n", NUM_WORKERS);

    Shared *sh = (Shared *)xmalloc(sizeof(Shared));
    memset(sh, 0, sizeof(*sh));
    sh->q = lfq_create(512);
    atomic_store(&sh->producers_alive, NUM_PRODUCERS);
    sh->expected_per_producer = PER_PRODUCER;
    atomic_store(&sh->consumer_checks, 0);

    printf("✓ Shared state initialized\n");
    printf("  Queue size: 512\n");
    printf("  Producers: %d (each producing %d items)\n", NUM_PRODUCERS, PER_PRODUCER);
    printf("  Consumers: %d\n", NUM_CONSUMERS);
    printf("  Expected total: %d items\n\n", NUM_PRODUCERS * PER_PRODUCER);

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

    printf("Running pipeline...\n\n");
    runtime_run(rt);
    printf("\n✓ Runtime completed\n\n");

    // Get stats
    RuntimeStats stats;
    memset(&stats, 0, sizeof(stats));
    runtime_get_stats(rt, &stats);

    printf("Results:\n");
    printf("  Produced: %ld\n", atomic_load(&sh->produced_total));
    printf("  Consumed: %ld\n", atomic_load(&sh->consumed_total));
    printf("  Tasks spawned: %llu\n", (unsigned long long)stats.tasks_spawned);
    printf("  Tasks done: %llu\n", (unsigned long long)stats.tasks_done);
    printf("  Poller events: %llu\n", (unsigned long long)stats.poller_events);

    // Cleanup
    lfq_destroy(sh->q);
    xfree(sh);
    runtime_destroy(rt);

    printf("\n✓ Runtime destroyed\n\n");

    int expected = NUM_PRODUCERS * PER_PRODUCER;
    int actual = atomic_load(&sh->consumed_total);

    if (actual == expected)
    {
        printf("✅ DEMO PASSED!\n");
        printf("════════════════════════════════════════════════════════\n");
        return 0;
    }
    else
    {
        printf("✗ DEMO FAILED: Expected %d, got %d\n", expected, actual);
        printf("════════════════════════════════════════════════════════\n");
        return 1;
    }
}
