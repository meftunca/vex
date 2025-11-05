#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <time.h>
#include "internal.h"
#include "poller.h"

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

static InternalTask *make_task(coro_resume_func fn, void *data)
{
    InternalTask *t = (InternalTask *)xmalloc(sizeof(InternalTask));
    t->resume_fn = fn;
    t->coro_data = data;
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
    rt->global_ready = lfq_create(1024);
    rt->poller = poller_create();
    rt->poller_thread = NULL;
    atomic_store(&rt->tracing, false);

    for (int i = 0; i < num_workers; ++i)
    {
        Worker *w = &rt->workers[i];
        w->id = i;
        w->rt = rt;
        w->local_ready = lfq_create(256);
        w->context = (struct WorkerContext *)xmalloc(sizeof(struct WorkerContext));
        w->context->owner = w;
        w->context->current_task = NULL;
        w->thread_handle = NULL;
    }
    return rt;
}

void runtime_destroy(Runtime *rt)
{
    if (!rt)
        return;
    poller_destroy(rt->poller);
    for (int i = 0; i < rt->num_workers; ++i)
    {
        lfq_destroy(rt->workers[i].local_ready);
        xfree(rt->workers[i].context);
    }
    lfq_destroy(rt->global_ready);
    xfree(rt->workers);
    xfree(rt);
}

void runtime_spawn_global(Runtime *rt, coro_resume_func fn, void *data)
{
    InternalTask *t = make_task(fn, data);
    while (!lfq_enqueue(rt->global_ready, t))
    {
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
    InternalTask *t = ctx->current_task;
    (void)lfq_enqueue(ctx->owner->rt->global_ready, NULL);
    int rc = poller_add(ctx->owner->rt->poller, fd, type, t);
    (void)rc;
}

static InternalTask *steal(Runtime *rt, int self_id)
{
    InternalTask *t = NULL;
    if (lfq_dequeue(rt->global_ready, (void **)&t))
    {
        if (t != NULL)
            return t;
    }
    for (int i = 0; i < rt->num_workers; ++i)
    {
        if (i == self_id)
            continue;
        if (lfq_dequeue(rt->workers[i].local_ready, (void **)&t))
        {
            if (t)
                return t;
        }
    }
    return NULL;
}

// Tüm kuyrukların boş olup olmadığını kontrol et
static bool all_queues_empty(Runtime *rt)
{
    // Global queue kontrolü
    void *tmp = NULL;
    if (lfq_dequeue(rt->global_ready, &tmp))
    {
        if (tmp != NULL)
        {
            // Geri koy
            lfq_enqueue(rt->global_ready, tmp);
            return false;
        }
    }

    // Tüm worker local queue'larını kontrol et
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
    return true;
}

void runtime_shutdown(Runtime *rt)
{
    atomic_store(&rt->running, false);
}

void runtime_run(Runtime *rt)
{
    atomic_store(&rt->running, true);
#ifdef _WIN32
    rt->poller_thread = (void *)_beginthreadex(NULL, 0, poller_main, rt, 0, &rt->poller_tid);
#else
    pthread_t tid;
    pthread_create(&tid, NULL, poller_main, rt);
    rt->poller_thread = (void *)tid;
#endif

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

#ifdef _WIN32
    for (int i = 0; i < rt->num_workers; ++i)
    {
        WaitForSingleObject((HANDLE)rt->workers[i].thread_handle, INFINITE);
        CloseHandle((HANDLE)rt->workers[i].thread_handle);
    }
    WaitForSingleObject((HANDLE)rt->poller_thread, INFINITE);
    CloseHandle((HANDLE)rt->poller_thread);
#else
    for (int i = 0; i < rt->num_workers; ++i)
    {
        pthread_join((pthread_t)rt->workers[i].thread_handle, NULL);
    }
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
    const int MAX_IDLE_CYCLES = 10; // 10 boş döngü sonra kontrol et

    while (atomic_load(&rt->running))
    {
        InternalTask *t = NULL;
        if (!lfq_dequeue(w->local_ready, (void **)&t))
        {
            t = steal(rt, w->id);
            if (!t)
            {
                idle_cycles++;
                if (idle_cycles >= MAX_IDLE_CYCLES)
                {
                    // Tüm kuyruklar boş mu kontrol et
                    if (all_queues_empty(rt))
                    {
                        runtime_shutdown(rt);
                        break;
                    }
                    idle_cycles = 0;
                }
#ifdef _WIN32
                Sleep(1);
#else
                struct timespec ts = {.tv_sec = 0, .tv_nsec = 1000000};
                nanosleep(&ts, NULL);
#endif
                continue;
            }
        }

        idle_cycles = 0; // İş bulundu, sayacı sıfırla
        w->context->current_task = t;
        CoroStatus st = t->resume_fn(w->context, t->coro_data);
        w->context->current_task = NULL;

        if (st == CORO_STATUS_RUNNING)
        {
            schedule_local(w, t);
        }
        else if (st == CORO_STATUS_DONE)
        {
            xfree(t);
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
    while (atomic_load(&rt->running))
    {
        int n = poller_wait(rt->poller, evs, 1024, 100);
        for (int i = 0; i < n; ++i)
        {
            InternalTask *t = (InternalTask *)evs[i].user_data;
            if (t)
            {
                while (!lfq_enqueue(rt->global_ready, t))
                {
#ifndef _WIN32
                    sched_yield();
#else
                    Sleep(0);
#endif
                }
            }
        }
    }
#ifndef _WIN32
    return NULL;
#else
    return 0;
#endif
}

// === Pro extension: minimal monotonic clock ===
static uint64_t rt_now_ns(void) {
#ifdef _WIN32
    LARGE_INTEGER freq, counter;
    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&counter);
    long double s = (long double)counter.QuadPart / (long double)freq.QuadPart;
    return (uint64_t)(s * 1000000000.0L);
#else
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
#endif
}

// Lightweight globals for demo purposes (not thread-safe/lock-free final design)
typedef struct {
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

void runtime_enable_auto_shutdown(Runtime* rt, bool enabled) {
    (void)rt;
    atomic_store(&g_rt_counters.auto_shutdown, enabled);
}

void runtime_get_stats(Runtime* rt, RuntimeStats* out_stats) {
    (void)rt;
    if (!out_stats) return;
    out_stats->tasks_spawned = atomic_load(&g_rt_counters.tasks_spawned);
    out_stats->tasks_done    = atomic_load(&g_rt_counters.tasks_done);
    out_stats->poller_events = atomic_load(&g_rt_counters.poller_events);
    out_stats->io_submitted  = atomic_load(&g_rt_counters.io_submitted);
    out_stats->steals        = atomic_load(&g_rt_counters.steals);
    out_stats->parks         = atomic_load(&g_rt_counters.parks);
    out_stats->unparks       = atomic_load(&g_rt_counters.unparks);
}

// Cancellation token (very thin wrapper)
struct CancelToken { _Atomic(bool) flag; };

CancelToken* worker_cancel_token(WorkerContext* ctx) {
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

bool cancel_requested(const CancelToken* t) {
    if (!t) return false;
    return atomic_load(&((struct CancelToken*)t)->flag);
}

void cancel_request(CancelToken* t) {
    if (!t) return;
    atomic_store(&t->flag, true);
}

// Generic handle await: best-effort mapping
void worker_await_ioh(WorkerContext* ctx, IoHandle h, EventType type) {
#if defined(_WIN32)
    (void)h; // IOCP path: fd value is not relied upon when resuming; user_data matters.
    worker_await_io(ctx, -1, type);
#else
    int fd = (int)(uintptr_t)h;
    worker_await_io(ctx, fd, type);
#endif
}

// Timers: naive implementation piggybacks on poller wait timeout via a global queue.
// For simplicity we enqueue a NULL kick and rely on the scheduler loop to re-run the task on next tick.
void worker_await_deadline(WorkerContext* ctx, uint64_t deadline_ns) {
    (void)deadline_ns;
    // Minimal: immediate yield by enqueueing a no-op; a real impl should store (task, deadline)
    // and only re-enqueue when the deadline expires.
    (void)lfq_enqueue(ctx->owner->rt->global_ready, NULL);
}

void worker_await_after(WorkerContext* ctx, uint64_t millis) {
    uint64_t target = rt_now_ns() + millis * 1000000ull;
    worker_await_deadline(ctx, target);
}
