#pragma once
#include <stdatomic.h>
#include <stdbool.h>
#include <stdint.h>
#include "lockfree_queue.h"
#include "runtime.h"

typedef struct InternalTask
{
    coro_resume_func resume_fn;
    void *coro_data;
} InternalTask;

typedef struct Worker
{
    // thread handle stored platform-specifically
#ifdef _WIN32
    void *thread_handle;
    unsigned thread_id;
#else
    void *thread_handle; // pthread_t stored as void*
#endif
    LockFreeQueue *local_ready;
    struct WorkerContext *context;
    struct Runtime *rt;
    int id;
} Worker;

struct WorkerContext
{
    Worker *owner;
    InternalTask *current_task;
};

struct Runtime
{
    _Atomic(bool) running;
    int num_workers;
    Worker *workers;
    LockFreeQueue *global_ready;
    Poller *poller;
#ifdef _WIN32
    void *poller_thread;
    unsigned poller_tid;
#else
    void *poller_thread;
#endif
    _Atomic(bool) tracing;
};

// util
void *xmalloc(size_t n);
void xfree(void *p);
