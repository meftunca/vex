// Synthetic Benchmark - Pure Scheduling Overhead Test
// No network I/O, just task spawn/execute/complete cycle
#include "include/runtime.h"
#include "include/internal.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdatomic.h>
#include <time.h>

// Shared counter for completed tasks
static _Atomic(uint64_t) g_completed = 0;
static _Atomic(uint64_t) g_spawned = 0;
static Runtime *g_rt = NULL;

// Minimal task - just increments counter and completes
static CoroStatus minimal_task(WorkerContext *ctx, void *data)
{
    (void)ctx;
    (void)data;
    atomic_fetch_add(&g_completed, 1);
    return CORO_STATUS_DONE; // Complete immediately
}

// Producer task - spawns N tasks as fast as possible
static CoroStatus producer_task(WorkerContext *ctx, void *data)
{
    (void)ctx;
    uint64_t target = (uint64_t)data;
    
    while (atomic_load(&g_spawned) < target)
    {
        runtime_spawn_global(g_rt, minimal_task, NULL);
        atomic_fetch_add(&g_spawned, 1);
        
        // Yield every 100 spawns to avoid monopolizing worker
        if (atomic_load(&g_spawned) % 100 == 0)
        {
            return CORO_STATUS_RUNNING;
        }
    }
    
    return CORO_STATUS_DONE;
}

// Supervisor - checks completion and shuts down
static CoroStatus supervisor_task(WorkerContext *ctx, void *data)
{
    (void)ctx;
    uint64_t target = (uint64_t)data;
    
    uint64_t completed = atomic_load(&g_completed);
    
    if (completed >= target)
    {
        fprintf(stderr, "\n✅ Benchmark complete: %llu/%llu tasks\n", 
                (unsigned long long)completed, (unsigned long long)target);
        runtime_shutdown(g_rt);
        return CORO_STATUS_DONE;
    }
    
    // Report progress every ~1000 iterations
    static int tick = 0;
    if ((tick++ % 1000) == 0)
    {
        fprintf(stderr, "\r[Progress] %llu/%llu tasks (%.1f%%)", 
                (unsigned long long)completed, (unsigned long long)target,
                (double)completed / target * 100.0);
        fflush(stderr);
    }
    
    return CORO_STATUS_RUNNING;
}

int main(int argc, char **argv)
{
    uint64_t num_tasks = 1000000; // 1M tasks by default
    int num_workers = 4;
    
    if (argc >= 2)
    {
        num_tasks = strtoull(argv[1], NULL, 10);
    }
    if (argc >= 3)
    {
        num_workers = atoi(argv[2]);
    }
    
    printf("═══════════════════════════════════════════════════════\n");
    printf("  Async Runtime Synthetic Benchmark (Pure Scheduling)\n");
    printf("═══════════════════════════════════════════════════════\n");
    printf("  Tasks: %llu\n", (unsigned long long)num_tasks);
    printf("  Workers: %d\n", num_workers);
    printf("  Goal: Measure pure scheduling overhead (no I/O)\n");
    printf("═══════════════════════════════════════════════════════\n\n");
    fflush(stdout);
    
    Runtime *rt = runtime_create(num_workers);
    g_rt = rt;
    
    runtime_enable_auto_shutdown(rt, false);
    runtime_set_tracing(rt, false);
    
    // Start timing
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    
    // Spawn producer and supervisor
    runtime_spawn_global(rt, producer_task, (void *)num_tasks);
    runtime_spawn_global(rt, supervisor_task, (void *)num_tasks);
    
    // Run runtime
    runtime_run(rt);
    
    // End timing
    clock_gettime(CLOCK_MONOTONIC, &end);
    
    double elapsed = (end.tv_sec - start.tv_sec) + 
                     (end.tv_nsec - start.tv_nsec) / 1e9;
    
    uint64_t completed = atomic_load(&g_completed);
    double throughput = completed / elapsed;
    double latency_us = (elapsed / completed) * 1e6;
    
    printf("\n");
    printf("═══════════════════════════════════════════════════════\n");
    printf("  RESULTS\n");
    printf("═══════════════════════════════════════════════════════\n");
    printf("  Completed: %llu tasks\n", (unsigned long long)completed);
    printf("  Time: %.3f seconds\n", elapsed);
    printf("  Throughput: %.0f tasks/sec\n", throughput);
    printf("  Latency: %.2f μs/task\n", latency_us);
    printf("═══════════════════════════════════════════════════════\n");
    
    // Get runtime stats
    RuntimeStats stats;
    runtime_get_stats(rt, &stats);
    printf("\n  Runtime Stats:\n");
    printf("    Tasks spawned: %llu\n", (unsigned long long)stats.tasks_spawned);
    printf("    Tasks done: %llu\n", (unsigned long long)stats.tasks_done);
    printf("    Steals: %llu\n", (unsigned long long)stats.steals);
    printf("═══════════════════════════════════════════════════════\n\n");
    
    // Target analysis
    if (throughput >= 1000000)
    {
        printf("✅ EXCELLENT: Achieved >1M tasks/sec!\n");
    }
    else if (throughput >= 500000)
    {
        printf("✅ GOOD: Achieved >500K tasks/sec\n");
        printf("   With optimization, 1M tasks/sec is reachable.\n");
    }
    else if (throughput >= 100000)
    {
        printf("⚠️  MODERATE: Achieved >100K tasks/sec\n");
        printf("   Significant optimization needed for 1M target.\n");
    }
    else
    {
        printf("❌ LOW: <100K tasks/sec\n");
        printf("   Fundamental architecture changes needed.\n");
    }
    
    runtime_destroy(rt);
    
    return 0;
}
