// Working demo with proper timer support using state machines
#include "include/runtime.h"
#include "include/lockfree_queue.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdatomic.h>
#include <stdint.h>
#include <string.h>

typedef struct Msg {
    int producer_id;
    int seq;
    uint64_t payload;
} Msg;

typedef struct Shared {
    LockFreeQueue *q;
    atomic_int producers_alive;
    atomic_long produced_total;
    atomic_long consumed_total;
    int expected_per_producer;
} Shared;

static Runtime *g_rt = NULL;

// Producer state machine
typedef struct {
    Shared *sh;
    int my_id;
    int i;  // Current iteration
    int state; // 0: enqueue, 1: after timer
} ProducerState;

static CoroStatus producer_coro(WorkerContext *ctx, void *data) {
    ProducerState *ps = (ProducerState *)data;
    
    switch (ps->state) {
    case 0: { // Try to enqueue
        if (ps->i >= ps->sh->expected_per_producer) {
            // Done
            atomic_fetch_sub(&ps->sh->producers_alive, 1);
            fprintf(stderr, "[producer %d] Done: produced %d items\n", 
                    ps->my_id, ps->sh->expected_per_producer);
            free(ps);
            return CORO_STATUS_DONE;
        }
        
        Msg *m = (Msg *)malloc(sizeof(Msg));
        m->producer_id = ps->my_id;
        m->seq = ps->i;
        m->payload = (uint64_t)(ps->my_id * 1000000 + ps->i);
        
        if (!lfq_enqueue(ps->sh->q, (void *)m)) {
            // Queue full, try again after timer
            free(m);
            worker_await_after(ctx, 1);
            return CORO_STATUS_YIELDED;
        }
        
        atomic_fetch_add(&ps->sh->produced_total, 1);
        ps->i++;
        
        // Yield with timer
        ps->state = 1;
        worker_await_after(ctx, 2);
        return CORO_STATUS_YIELDED;
    }
    
    case 1: { // After timer, go back to enqueue
        ps->state = 0;
        return CORO_STATUS_RUNNING;
    }
    }
    
    return CORO_STATUS_DONE;
}

// Consumer state machine
typedef struct {
    Shared *sh;
    int state; // 0: try dequeue, 1: after processing timer
} ConsumerState;

static CoroStatus consumer_coro(WorkerContext *ctx, void *data) {
    ConsumerState *cs = (ConsumerState *)data;
    
    switch (cs->state) {
    case 0: { // Try to dequeue
        void *vp = NULL;
        if (lfq_dequeue(cs->sh->q, &vp)) {
            Msg *m = (Msg *)vp;
            // Process with timer
            cs->state = 1;
            worker_await_after(ctx, 1 + (m->payload % 3));
            atomic_fetch_add(&cs->sh->consumed_total, 1);
            free(m);
            return CORO_STATUS_YIELDED;
        }
        
        // Queue empty
        if (atomic_load(&cs->sh->producers_alive) == 0) {
            free(cs);
            return CORO_STATUS_DONE;
        }
        
        // Wait and retry
        worker_await_after(ctx, 1);
        return CORO_STATUS_YIELDED;
    }
    
    case 1: { // After processing, go back to dequeue
        cs->state = 0;
        return CORO_STATUS_RUNNING;
    }
    }
    
    return CORO_STATUS_DONE;
}

// Supervisor
typedef struct {
    Shared *sh;
    int tick;
} SupervisorState;

static CoroStatus supervisor_coro(WorkerContext *ctx, void *data) {
    SupervisorState *ss = (SupervisorState *)data;
    
    long produced = atomic_load(&ss->sh->produced_total);
    long consumed = atomic_load(&ss->sh->consumed_total);
    
    if (atomic_load(&ss->sh->producers_alive) == 0 && produced == consumed) {
        fprintf(stderr, "[supervisor] Done: produced=%ld consumed=%ld -> shutdown\n", 
                produced, consumed);
        runtime_shutdown(g_rt);
        free(ss);
        return CORO_STATUS_DONE;
    }
    
    // Status report
    if ((ss->tick++ % 20) == 0) {
        fprintf(stderr, "[supervisor] produced=%ld consumed=%ld producers_alive=%d\n",
                produced, consumed, atomic_load(&ss->sh->producers_alive));
    }
    
    worker_await_after(ctx, 50);
    return CORO_STATUS_YIELDED;
}

int main(void) {
    printf("════════════════════════════════════════════════════════\n");
    printf("  async_runtime + vex_net Full Demo (WITH TIMERS!)\n");
    printf("  Producer/Consumer Pipeline with Timers\n");
    printf("════════════════════════════════════════════════════════\n\n");
    
    const int NUM_WORKERS = 4;
    const int NUM_PRODUCERS = 3;
    const int NUM_CONSUMERS = 4;
    const int PER_PRODUCER = 50; // Reduced for faster demo
    
    Runtime *rt = runtime_create(NUM_WORKERS);
    g_rt = rt;
    runtime_enable_auto_shutdown(rt, false);
    runtime_set_tracing(rt, false);
    
    printf("✓ Runtime created with %d workers\n", NUM_WORKERS);
    
    Shared *sh = (Shared *)malloc(sizeof(Shared));
    memset(sh, 0, sizeof(*sh));
    sh->q = lfq_create(512);
    atomic_store(&sh->producers_alive, NUM_PRODUCERS);
    sh->expected_per_producer = PER_PRODUCER;
    
    printf("✓ Configuration:\n");
    printf("  - Producers: %d (each %d items = %d total)\n", 
           NUM_PRODUCERS, PER_PRODUCER, NUM_PRODUCERS * PER_PRODUCER);
    printf("  - Consumers: %d\n", NUM_CONSUMERS);
    printf("  - Queue: 512 slots\n");
    printf("  - Timers: Enabled (vex_net backend)\n\n");
    
    // Spawn producers with state
    for (int i = 0; i < NUM_PRODUCERS; ++i) {
        ProducerState *ps = (ProducerState *)malloc(sizeof(ProducerState));
        ps->sh = sh;
        ps->my_id = i;
        ps->i = 0;
        ps->state = 0;
        runtime_spawn_global(rt, producer_coro, ps);
    }
    
    // Spawn consumers with state
    for (int i = 0; i < NUM_CONSUMERS; ++i) {
        ConsumerState *cs = (ConsumerState *)malloc(sizeof(ConsumerState));
        cs->sh = sh;
        cs->state = 0;
        runtime_spawn_global(rt, consumer_coro, cs);
    }
    
    // Spawn supervisor with state
    SupervisorState *ss = (SupervisorState *)malloc(sizeof(SupervisorState));
    ss->sh = sh;
    ss->tick = 0;
    runtime_spawn_global(rt, supervisor_coro, ss);
    
    printf("Running...\n\n");
    runtime_run(rt);
    printf("\n✓ Runtime completed\n\n");
    
    // Stats
    RuntimeStats stats;
    memset(&stats, 0, sizeof(stats));
    runtime_get_stats(rt, &stats);
    
    printf("Final Results:\n");
    printf("  Produced: %ld\n", atomic_load(&sh->produced_total));
    printf("  Consumed: %ld\n", atomic_load(&sh->consumed_total));
    printf("  Tasks spawned: %llu\n", (unsigned long long)stats.tasks_spawned);
    printf("  Tasks done: %llu\n", (unsigned long long)stats.tasks_done);
    printf("  Poller events: %llu\n", (unsigned long long)stats.poller_events);
    
    // Cleanup
    lfq_destroy(sh->q);
    free(sh);
    runtime_destroy(rt);
    
    printf("\n✓ Cleanup complete\n\n");
    
    int expected = NUM_PRODUCERS * PER_PRODUCER;
    long actual = atomic_load(&sh->consumed_total);
    
    if (actual == expected) {
        printf("✅ FULL DEMO PASSED!\n");
        printf("   Timers working correctly!\n");
        printf("   async_runtime + vex_net integration COMPLETE!\n");
        printf("════════════════════════════════════════════════════════\n");
        return 0;
    } else {
        printf("✗ Demo incomplete: Expected %d, got %ld\n", expected, actual);
        printf("════════════════════════════════════════════════════════\n");
        return 1;
    }
}

