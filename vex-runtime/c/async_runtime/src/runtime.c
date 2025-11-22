#include <string.h>
#include <errno.h>
#include <time.h>
#include <stdio.h>
#include <fcntl.h>
#include "internal.h"
#include "poller.h"
#include "task_pool.h"

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
static unsigned __stdcall worker_main(void *arg);
static unsigned __stdcall poller_main(void *arg);
#else
#include <pthread.h>
#include <unistd.h>
static void *worker_main(void *arg);
static void *poller_main(void *arg);
#endif

// === Pro extension: Runtime counters ===
typedef struct
{
    _Atomic(uint64_t) tasks_spawned;
    _Atomic(uint64_t) tasks_done;
    _Atomic(uint64_t) poller_events;
    _Atomic(uint64_t) io_submitted;
    _Atomic(uint64_t) steals;
    _Atomic(uint64_t) parks;
    _Atomic(uint64_t) unparks;
    _Atomic(bool) auto_shutdown;
} RtCounters;

static RtCounters g_rt_counters = {0};

// Monotonic clock helper
uint64_t rt_now_ns(void)
{
#ifdef _WIN32
    LARGE_INTEGER freq, counter;
    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&counter);
    long double s = (long double)counter.QuadPart / (long double)freq.QuadPart;
    return (uint64_t)(s * 1000000000.0L);
#else
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
#endif
}

static InternalTask *make_task(coro_resume_func fn, void *data)
{
    InternalTask *t = task_pool_alloc(); // Zero-allocation from pool!
    if (!t)
        return NULL;
    t->resume_fn = fn;
    t->coro_data = data;
    atomic_store(&t->state, 0); // ready
    t->last_fd = -1;
    return t;
}

Runtime *runtime_create(int num_workers)
{
    if (num_workers <= 0)
    {
#ifdef _WIN32
        SYSTEM_INFO si;
        GetSystemInfo(&si);
        num_workers = (int)si.dwNumberOfProcessors;
#else
        long n = sysconf(_SC_NPROCESSORS_ONLN);
        num_workers = (int)((n > 0) ? n : 1);
#endif
    }
    Runtime *rt = (Runtime *)xmalloc(sizeof(Runtime));
    atomic_store(&rt->running, false);
    rt->num_workers = num_workers;
    rt->workers = (Worker *)xmalloc(sizeof(Worker) * num_workers);
    rt->global_ready = lfq_create(65536);   // Primary queue: 64K capacity
    rt->overflow_queue = lfq_create(65536); // Overflow queue: 64K capacity (unbounded-like)
    rt->timer_heap = timer_heap_create(64); // Initial capacity for timers
    rt->poller = poller_create();
    rt->poller_thread = NULL;
    atomic_store(&rt->tracing, false);
    atomic_store(&rt->pending_io_count, 0);

    for (int i = 0; i < num_workers; ++i)
    {
        Worker *w = &rt->workers[i];
        w->id = i;
        w->rt = rt;
        w->local_ready = lfq_create(256);
        w->context = (struct WorkerContext *)xmalloc(sizeof(struct WorkerContext));
        w->context->owner = w;
        w->context->current_task = NULL;
        w->context->timer_pending = false;
        w->thread_handle = NULL;
    }
    return rt;
}

void runtime_destroy(Runtime *rt)
{
    if (!rt)
        return;
    poller_destroy(rt->poller);
    timer_heap_destroy(rt->timer_heap);
    for (int i = 0; i < rt->num_workers; ++i)
    {
        lfq_destroy(rt->workers[i].local_ready);
        xfree(rt->workers[i].context);
    }
    lfq_destroy(rt->global_ready);
    lfq_destroy(rt->overflow_queue);
    xfree(rt->workers);
    xfree(rt);
}

void runtime_spawn_global(Runtime *rt, coro_resume_func fn, void *data)
{
    InternalTask *t = make_task(fn, data);

    // Try global queue first (fast path)
    if (lfq_enqueue(rt->global_ready, t))
    {
        return;
    }

    // Global full, try overflow queue (unbounded fallback)
    if (lfq_enqueue(rt->overflow_queue, t))
    {
        return;
    }

    // Both queues full, spin with yield (extremely rare)
    while (true)
    {
        if (lfq_enqueue(rt->global_ready, t))
            return;
        if (lfq_enqueue(rt->overflow_queue, t))
            return;
#ifdef _WIN32
        Sleep(0);
#else
        sched_yield();
#endif
    }
}

void runtime_set_tracing(Runtime *rt, bool enabled) { atomic_store(&rt->tracing, enabled); }

static void schedule_local(Worker *w, InternalTask *t)
{
    while (!lfq_enqueue(w->local_ready, t))
    {
#ifdef _WIN32
        Sleep(0);
#else
        sched_yield();
#endif
    }
}

void worker_spawn_local(WorkerContext *ctx, coro_resume_func fn, void *data)
{
    InternalTask *t = make_task(fn, data);
    schedule_local(ctx->owner, t);
}

void worker_await_io(WorkerContext *ctx, int fd, EventType type)
{
    if (!ctx || !ctx->current_task)
        return;

    InternalTask *t = ctx->current_task;
    Runtime *rt = ctx->owner->rt;

    // Register with poller - task will be re-queued when I/O ready
    // Register with poller - task will be re-queued when I/O ready
    int rc = poller_add(rt->poller, fd, type, t);

    if (rc < 0)
    {
        // Poller registration failed, re-queue immediately to avoid deadlock
        atomic_store(&t->state, 1); // in_queue
        while (!lfq_enqueue(rt->global_ready, t))
        {
#ifdef _WIN32
            Sleep(0);
#else
            sched_yield();
#endif
        }
        return;
    }

    // Track pending I/O
    atomic_fetch_add(&rt->pending_io_count, 1);

    // Mark task as waiting for I/O, clear current_task
    atomic_store(&t->state, 3); // io_waiting
    t->last_fd = fd;
    ctx->current_task = NULL;
}

static InternalTask *steal(Runtime *rt, int self_id)
{
    InternalTask *t = NULL;

    // Priority 1: Try global queue (highest priority)
    if (lfq_dequeue(rt->global_ready, (void **)&t))
    {
        if (t != NULL)
        {
            atomic_fetch_add(&g_rt_counters.steals, 1);
            return t;
        }
    }

    // Priority 2: Try overflow queue (unbounded fallback)
    if (lfq_dequeue(rt->overflow_queue, (void **)&t))
    {
        if (t != NULL)
        {
            atomic_fetch_add(&g_rt_counters.steals, 1);
            return t;
        }
    }

    // Priority 3: Random victim selection from other workers
    int num_workers = rt->num_workers;
    if (num_workers <= 1)
        return NULL;

    // Start at random offset
    static _Atomic(uint32_t) rng_state = 12345;
    uint32_t state = atomic_fetch_add(&rng_state, 1);
    // Simple xorshift for randomness
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    int start = (int)(state % (uint32_t)num_workers);

    // Try each worker starting from random position
    for (int offset = 0; offset < num_workers; ++offset)
    {
        int victim_id = (start + offset) % num_workers;
        if (victim_id == self_id)
            continue;

        if (lfq_dequeue(rt->workers[victim_id].local_ready, (void **)&t))
        {
            if (t)
            {
                atomic_fetch_add(&g_rt_counters.steals, 1);
                return t;
            }
        }
    }
    return NULL;
}

// Tüm kuyrukların boş olup olmadığını kontrol et
static bool all_queues_empty(Runtime *rt)
{
    void *tmp = NULL;

    // Check global queue
    if (lfq_dequeue(rt->global_ready, &tmp))
    {
        if (tmp != NULL)
        {
            lfq_enqueue(rt->global_ready, tmp);
            return false;
        }
    }

    // Check overflow queue
    if (lfq_dequeue(rt->overflow_queue, &tmp))
    {
        if (tmp != NULL)
        {
            lfq_enqueue(rt->overflow_queue, tmp);
            return false;
        }
    }

    // Check all worker local queues
    for (int i = 0; i < rt->num_workers; ++i)
    {
        if (lfq_dequeue(rt->workers[i].local_ready, &tmp))
        {
            if (tmp != NULL)
            {
                lfq_enqueue(rt->workers[i].local_ready, tmp);
                return false;
            }
        }
    }

    // Check timer heap
    if (!timer_heap_empty(rt->timer_heap))
        return false;

    // Check pending I/O
    if (atomic_load(&rt->pending_io_count) > 0)
        return false;

    return true;
}

void runtime_shutdown(Runtime *rt)
{
    atomic_store(&rt->running, false);
}

// Timer processing callback - enqueue expired task
static void enqueue_expired_task(void *task, void *user_data)
{
    Runtime *rt = (Runtime *)user_data;
    InternalTask *t = (InternalTask *)task;

    // Try global queue first, then overflow
    if (lfq_enqueue(rt->global_ready, t))
        return;
    if (lfq_enqueue(rt->overflow_queue, t))
        return;

    // Rare fallback: spin until space available
    while (true)
    {
        if (lfq_enqueue(rt->global_ready, t))
            return;
        if (lfq_enqueue(rt->overflow_queue, t))
            return;
#ifdef _WIN32
        Sleep(0);
#else
        sched_yield();
#endif
    }
}

// Process expired timers - called by worker 0
static void process_expired_timers(Runtime *rt)
{
    uint64_t now_ns = rt_now_ns();
    timer_heap_pop_expired(rt->timer_heap, now_ns, enqueue_expired_task, rt);
}

void runtime_run(Runtime *rt)
{
    atomic_store(&rt->running, true);

    // Start poller thread first
#ifdef _WIN32
    rt->poller_thread = (void *)_beginthreadex(NULL, 0, poller_main, rt, 0, &rt->poller_tid);
#else
    pthread_t poller_tid;
    pthread_create(&poller_tid, NULL, poller_main, rt);
    rt->poller_thread = (void *)poller_tid;
#endif

    // Start worker threads
#ifdef _WIN32
    for (int i = 0; i < rt->num_workers; ++i)
    {
        unsigned tidw;
        rt->workers[i].thread_handle = (void *)_beginthreadex(NULL, 0, worker_main, &rt->workers[i], 0, &tidw);
        rt->workers[i].thread_id = tidw;
    }
#else
    for (int i = 0; i < rt->num_workers; ++i)
    {
        pthread_t th;
        pthread_create(&th, NULL, worker_main, &rt->workers[i]);
        rt->workers[i].thread_handle = (void *)th;
    }
#endif

    // Wait for all workers to complete
#ifdef _WIN32
    for (int i = 0; i < rt->num_workers; ++i)
    {
        WaitForSingleObject((HANDLE)rt->workers[i].thread_handle, INFINITE);
        CloseHandle((HANDLE)rt->workers[i].thread_handle);
    }
#else
    for (int i = 0; i < rt->num_workers; ++i)
    {
        pthread_join((pthread_t)rt->workers[i].thread_handle, NULL);
    }
#endif

    // Signal poller to stop and wait for it
    atomic_store(&rt->running, false);
#ifdef _WIN32
    WaitForSingleObject((HANDLE)rt->poller_thread, INFINITE);
    CloseHandle((HANDLE)rt->poller_thread);
#else
    pthread_join((pthread_t)rt->poller_thread, NULL);
#endif
}

#ifndef _WIN32
static void *worker_main(void *arg)
{
#else
static unsigned __stdcall worker_main(void *arg)
{
#endif
    Worker *w = (Worker *)arg;
    Runtime *rt = w->rt;
    int idle_cycles = 0;
    const int MAX_IDLE_CYCLES = 100;

    while (atomic_load(&rt->running))
    {
        // Check for expired timers (only one worker processes timers to avoid races)
        if (w->id == 0)
        { // Worker 0 responsible for timer processing
            process_expired_timers(rt);
        }

        InternalTask *t = NULL;

        // Priority 1: Check global queue first (important for I/O wakeups)
        if (lfq_dequeue(rt->global_ready, (void **)&t) && t != NULL)
        {
            atomic_store(&t->state, 2); // executing
            // Got task from global
        }
        // Priority 2: Check overflow queue
        else if (lfq_dequeue(rt->overflow_queue, (void **)&t) && t != NULL)
        {
            atomic_store(&t->state, 2); // executing
            // Got task from overflow
        }
        // Priority 3: Check local queue
        else if (lfq_dequeue(w->local_ready, (void **)&t) && t != NULL)
        {
            atomic_store(&t->state, 2); // executing
            // Got task from local
        }
        // Priority 4: Steal from other workers
        else
        {
            t = steal(rt, w->id);
            if (!t)
            {
                idle_cycles++;
                if (w->id == 0 && idle_cycles % 20 == 0)
                {
                    // Removed debug spam
                }

                // Auto-shutdown check
                if (atomic_load(&g_rt_counters.auto_shutdown) && idle_cycles >= MAX_IDLE_CYCLES)
                {
                    // All queues empty check
                    if (all_queues_empty(rt))
                    {
                        if (w->id == 0)
                            fprintf(stderr, "[Worker %d] Auto-shutdown triggered\n", w->id);
                        runtime_shutdown(rt);
                        break;
                    }
                    idle_cycles = 0;
                }
#ifdef _WIN32
                Sleep(0); // Yield CPU
#else
                sched_yield(); // Yield CPU instead of sleep
#endif
                continue;
            }
        }

        idle_cycles = 0; // İş bulundu, sayacı sıfırla
        w->context->current_task = t;
        w->context->timer_pending = false; // Reset timer flag
        CoroStatus st = t->resume_fn(w->context, t->coro_data);

        // Check if task is still current (not cleared by await_io/timer)
        bool task_suspended = (w->context->current_task == NULL);
        w->context->current_task = NULL;

        // If timer was set during execution, don't reschedule
        if (w->context->timer_pending)
        {
            // Task is now in timer heap, will be rescheduled when timer expires
            continue;
        }

        // If task was suspended (I/O or other async operation), don't reschedule
        // It will be re-queued by poller or other mechanism
        if (task_suspended)
        {
            continue;
        }

        if (st == CORO_STATUS_RUNNING)
        {
            atomic_store(&t->state, 0); // ready
            schedule_local(w, t);
        }
        else if (st == CORO_STATUS_YIELDED)
        {
            // Task yielded but not suspended - re-queue immediately
            atomic_store(&t->state, 0); // ready
            schedule_local(w, t);
        }
        else if (st == CORO_STATUS_DONE)
        {
            task_pool_free(t); // Return to pool for reuse
        }
    }
#ifndef _WIN32
    return NULL;
#else
    return 0;
#endif
}

#ifndef _WIN32
static void *poller_main(void *arg)
{
#else
static unsigned __stdcall poller_main(void *arg)
{
#endif
    Runtime *rt = (Runtime *)arg;
    ReadyEvent evs[1024];
    InternalTask *batch[256]; // Batch buffer for ready tasks
    int batch_count = 0;
    
    while (atomic_load(&rt->running))
    {
        // Calculate timeout based on next timer deadline
        int timeout_ms = 100; // Default timeout
        uint64_t next_deadline = timer_heap_peek_deadline(rt->timer_heap);
        if (next_deadline != UINT64_MAX)
        {
            uint64_t now_ns = rt_now_ns();
            if (next_deadline <= now_ns)
            {
                timeout_ms = 0; // Timer already expired, poll immediately
            }
            else
            {
                uint64_t diff_ns = next_deadline - now_ns;
                timeout_ms = (int)(diff_ns / 1000000); // Convert ns to ms
                if (timeout_ms > 100)
                    timeout_ms = 100; // Cap at 100ms
            }
        }

        int n = poller_wait(rt->poller, evs, 1024, timeout_ms);

        // Batch process events
        for (int i = 0; i < n; ++i)
        {
            InternalTask *t = (InternalTask *)evs[i].user_data;
            if (t)
            {
                // Atomic state transition: io_waiting -> in_queue
                int expected = 3; // io_waiting
                if (!atomic_compare_exchange_strong(&t->state, &expected, 1))
                {
                    continue; // Spurious or stale event
                }
                
                // Decrement pending I/O count
                atomic_fetch_sub(&rt->pending_io_count, 1);

                // Add to batch instead of immediate enqueue
                batch[batch_count++] = t;

                // Flush batch when full (reduces queue contention)
                if (batch_count >= 256)
                {
                    for (int j = 0; j < batch_count; j++)
                    {
                        while (!lfq_enqueue(rt->global_ready, batch[j]))
                        {
#ifndef _WIN32
                            sched_yield();
#else
                            Sleep(0);
#endif
                        }
                    }
                    batch_count = 0;
                }
            }
        }

        // Flush remaining batch
        if (batch_count > 0)
        {
            for (int j = 0; j < batch_count; j++)
            {
                while (!lfq_enqueue(rt->global_ready, batch[j]))
                {
#ifndef _WIN32
                    sched_yield();
#else
                    Sleep(0);
#endif
                }
            }
            batch_count = 0;
        }
    }
#ifndef _WIN32
    return NULL;
#else
    return 0;
#endif
}

void runtime_enable_auto_shutdown(Runtime *rt, bool enabled)
{
    (void)rt;
    atomic_store(&g_rt_counters.auto_shutdown, enabled);
}

void runtime_get_stats(Runtime *rt, RuntimeStats *out_stats)
{
    (void)rt;
    if (!out_stats)
        return;
    out_stats->tasks_spawned = atomic_load(&g_rt_counters.tasks_spawned);
    out_stats->tasks_done = atomic_load(&g_rt_counters.tasks_done);
    out_stats->poller_events = atomic_load(&g_rt_counters.poller_events);
    out_stats->io_submitted = atomic_load(&g_rt_counters.io_submitted);
    out_stats->steals = atomic_load(&g_rt_counters.steals);
    out_stats->parks = atomic_load(&g_rt_counters.parks);
    out_stats->unparks = atomic_load(&g_rt_counters.unparks);
}

// Cancellation token (very thin wrapper)
struct CancelToken
{
    _Atomic(bool) flag;
};

CancelToken *worker_cancel_token(WorkerContext *ctx)
{
    (void)ctx;
    // For demo, return an address unique per-context would be ideal.
    // Here we return a thread-local token to keep ABI simple.
#ifdef _WIN32
    __declspec(thread) static struct CancelToken t;
#else
    static __thread struct CancelToken t;
#endif
    return &t;
}

bool cancel_requested(const CancelToken *t)
{
    if (!t)
        return false;
    return atomic_load(&((struct CancelToken *)t)->flag);
}

void cancel_request(CancelToken *t)
{
    if (!t)
        return;
    atomic_store(&t->flag, true);
}

// Generic handle await: best-effort mapping
void worker_await_ioh(WorkerContext *ctx, IoHandle h, EventType type)
{
#if defined(_WIN32)
    (void)h; // IOCP path: fd value is not relied upon when resuming; user_data matters.
    worker_await_io(ctx, -1, type);
#else
    int fd = (int)(uintptr_t)h;
    worker_await_io(ctx, fd, type);
#endif
}

// Timers: pause current task and schedule wake-up
void worker_await_deadline(WorkerContext *ctx, uint64_t deadline_ns)
{
    if (!ctx || !ctx->current_task || !ctx->owner || !ctx->owner->rt)
        return;

    Runtime *rt = ctx->owner->rt;
    InternalTask *task = ctx->current_task;

    // Insert task into timer heap (will be woken up when deadline expires)
    timer_heap_insert(rt->timer_heap, deadline_ns, task);

    // Set flag so worker won't reschedule this task
    ctx->timer_pending = true;

    // Clear current_task (it's now waiting on timer)
    ctx->current_task = NULL;
}

void worker_await_after(WorkerContext *ctx, uint64_t millis)
{
    uint64_t now_ns = rt_now_ns();
    uint64_t target = now_ns + millis * 1000000ULL;
    worker_await_deadline(ctx, target);
}
