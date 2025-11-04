#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <time.h>
#include "internal.h"
#include "poller.h"

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
static unsigned __stdcall worker_main(void* arg);
static unsigned __stdcall poller_main(void* arg);
#else
#include <pthread.h>
#include <unistd.h>
static void* worker_main(void* arg);
static void* poller_main(void* arg);
#endif

static InternalTask* make_task(coro_resume_func fn, void* data) {
    InternalTask* t = (InternalTask*)xmalloc(sizeof(InternalTask));
    t->resume_fn = fn;
    t->coro_data = data;
    return t;
}

Runtime* runtime_create(int num_workers) {
    if (num_workers <= 0) {
#ifdef _WIN32
        SYSTEM_INFO si; GetSystemInfo(&si);
        num_workers = (int)si.dwNumberOfProcessors;
#else
        long n = sysconf(_SC_NPROCESSORS_ONLN);
        num_workers = (int)((n > 0) ? n : 1);
#endif
    }
    Runtime* rt = (Runtime*)xmalloc(sizeof(Runtime));
    atomic_store(&rt->running, false);
    rt->num_workers = num_workers;
    rt->workers = (Worker*)xmalloc(sizeof(Worker) * num_workers);
    rt->global_ready = lfq_create(1024);
    rt->poller = poller_create();
    rt->poller_thread = NULL;
    atomic_store(&rt->tracing, false);

    for (int i = 0; i < num_workers; ++i) {
        Worker* w = &rt->workers[i];
        w->id = i;
        w->rt = rt;
        w->local_ready = lfq_create(256);
        w->context = (struct WorkerContext*)xmalloc(sizeof(struct WorkerContext));
        w->context->owner = w;
        w->context->current_task = NULL;
        w->thread_handle = NULL;
    }
    return rt;
}

void runtime_destroy(Runtime* rt) {
    if (!rt) return;
    poller_destroy(rt->poller);
    for (int i = 0; i < rt->num_workers; ++i) {
        lfq_destroy(rt->workers[i].local_ready);
        xfree(rt->workers[i].context);
    }
    lfq_destroy(rt->global_ready);
    xfree(rt->workers);
    xfree(rt);
}

void runtime_spawn_global(Runtime* rt, coro_resume_func fn, void* data) {
    InternalTask* t = make_task(fn, data);
    while (!lfq_enqueue(rt->global_ready, t)) {
        // backoff
#ifdef _WIN32
        Sleep(0);
#else
        sched_yield();
#endif
    }
}

void runtime_set_tracing(Runtime* rt, bool enabled) { atomic_store(&rt->tracing, enabled); }

static void schedule_local(Worker* w, InternalTask* t) {
    while (!lfq_enqueue(w->local_ready, t)) {
#ifdef _WIN32
        Sleep(0);
#else
        sched_yield();
#endif
    }
}

void worker_spawn_local(WorkerContext* ctx, coro_resume_func fn, void* data) {
    InternalTask* t = make_task(fn, data);
    schedule_local(ctx->owner, t);
}

void worker_await_io(WorkerContext* ctx, int fd, EventType type) {
    // Register the current task to poller and mark yielded.
    InternalTask* t = ctx->current_task;
    (void)lfq_enqueue(ctx->owner->rt->global_ready, NULL); // hint for wakeups (noop slot)
    int rc = poller_add(ctx->owner->rt->poller, fd, type, t);
    (void)rc;
}

static InternalTask* steal(Runtime* rt, int self_id) {
    // Try global first
    InternalTask* t = NULL;
    if (lfq_dequeue(rt->global_ready, (void**)&t)) {
        if (t != NULL) return t;
    }
    // steal from others
    for (int i = 0; i < rt->num_workers; ++i) {
        if (i == self_id) continue;
        if (lfq_dequeue(rt->workers[i].local_ready, (void**)&t)) {
            if (t) return t;
        }
    }
    return NULL;
}

void runtime_shutdown(Runtime* rt) {
    atomic_store(&rt->running, false);
}

void runtime_run(Runtime* rt) {
    atomic_store(&rt->running, true);
#ifdef _WIN32
    rt->poller_thread = (void*)_beginthreadex(NULL, 0, poller_main, rt, 0, &rt->poller_tid);
#else
    pthread_t tid;
    pthread_create(&tid, NULL, poller_main, rt);
    rt->poller_thread = (void*)tid;
#endif

    // start workers
#ifdef _WIN32
    for (int i = 0; i < rt->num_workers; ++i) {
        unsigned tidw;
        rt->workers[i].thread_handle = (void*)_beginthreadex(NULL, 0, worker_main, &rt->workers[i], 0, &tidw);
        rt->workers[i].thread_id = tidw;
    }
#else
    for (int i = 0; i < rt->num_workers; ++i) {
        pthread_t th;
        pthread_create(&th, NULL, worker_main, &rt->workers[i]);
        rt->workers[i].thread_handle = (void*)th;
    }
#endif

    // Join workers
#ifdef _WIN32
    for (int i = 0; i < rt->num_workers; ++i) {
        WaitForSingleObject((HANDLE)rt->workers[i].thread_handle, INFINITE);
        CloseHandle((HANDLE)rt->workers[i].thread_handle);
    }
    WaitForSingleObject((HANDLE)rt->poller_thread, INFINITE);
    CloseHandle((HANDLE)rt->poller_thread);
#else
    for (int i = 0; i < rt->num_workers; ++i) {
        pthread_join((pthread_t)rt->workers[i].thread_handle, NULL);
    }
    pthread_join((pthread_t)rt->poller_thread, NULL);
#endif
}

#ifndef _WIN32
static void* worker_main(void* arg) {
#else
static unsigned __stdcall worker_main(void* arg) {
#endif
    Worker* w = (Worker*)arg;
    Runtime* rt = w->rt;
    while (atomic_load(&rt->running)) {
        InternalTask* t = NULL;
        if (!lfq_dequeue(w->local_ready, (void**)&t)) {
            t = steal(rt, w->id);
            if (!t) {
                // idle
#ifdef _WIN32
                Sleep(1);
#else
                struct timespec ts = {.tv_sec=0,.tv_nsec=1000000};
                nanosleep(&ts, NULL);
#endif
                continue;
            }
        }
        w->context->current_task = t;
        CoroStatus st = t->resume_fn(w->context, t->coro_data);
        w->context->current_task = NULL;
        if (st == CORO_STATUS_RUNNING) {
            schedule_local(w, t);
        } else if (st == CORO_STATUS_DONE) {
            xfree(t);
        } else { // YIELDED: do nothing, poller will requeue
            // no-op
        }
    }
#ifndef _WIN32
    return NULL;
#else
    return 0;
#endif
}

#ifndef _WIN32
static void* poller_main(void* arg) {
#else
static unsigned __stdcall poller_main(void* arg) {
#endif
    Runtime* rt = (Runtime*)arg;
    ReadyEvent evs[1024];
    while (atomic_load(&rt->running)) {
        int n = poller_wait(rt->poller, evs, 1024, 100); // 100ms tick
        for (int i = 0; i < n; ++i) {
            InternalTask* t = (InternalTask*)evs[i].user_data;
            // Requeue to global
            if (t) {
                while (!lfq_enqueue(rt->global_ready, t)) {
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
