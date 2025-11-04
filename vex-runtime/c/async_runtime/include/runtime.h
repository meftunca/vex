#pragma once
#include "poller.h"
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

struct Runtime;
typedef struct Runtime Runtime;

struct WorkerContext;
typedef struct WorkerContext WorkerContext;

typedef enum {
    CORO_STATUS_RUNNING,
    CORO_STATUS_YIELDED,
    CORO_STATUS_DONE
} CoroStatus;

typedef CoroStatus (*coro_resume_func)(WorkerContext* context, void* coro_data);

// Admin
Runtime* runtime_create(int num_workers);
void runtime_destroy(Runtime* runtime);
void runtime_spawn_global(Runtime* runtime, coro_resume_func resume_fn, void* coro_data);
void runtime_run(Runtime* runtime);
void runtime_shutdown(Runtime* runtime);

// From coroutine
void worker_await_io(WorkerContext* context, int fd, EventType type);
void worker_spawn_local(WorkerContext* context, coro_resume_func resume_fn, void* coro_data);

// Minimal tracing toggle
void runtime_set_tracing(Runtime* rt, bool enabled);

#ifdef __cplusplus
}
#endif
