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

// === Pro extensions (timers, cancellation, stats, IoHandle) ===

// Cross-platform opaque IO handle (fd/SOCKET/HANDLE).
#include <stdint.h>
typedef uintptr_t IoHandle;

// Cancellation token
typedef struct CancelToken CancelToken;

// Runtime counters for basic observability.
typedef struct {
    uint64_t tasks_spawned;
    uint64_t tasks_done;
    uint64_t poller_events;
    uint64_t io_submitted;
    uint64_t steals;
    uint64_t parks;
    uint64_t unparks;
} RuntimeStats;

// Timers: await for a deadline/relative delay (monotonic ns / ms).
void worker_await_deadline(WorkerContext* context, uint64_t deadline_ns);
void worker_await_after(WorkerContext* context, uint64_t millis);

// Alternative await with generic handle (maps to fd/SOCKET/HANDLE as appropriate).
void worker_await_ioh(WorkerContext* context, IoHandle h, EventType type);

// Cancellation tokens
CancelToken* worker_cancel_token(WorkerContext* context);
bool cancel_requested(const CancelToken* t);
void cancel_request(CancelToken* t);

// Auto-shutdown and stats
void runtime_enable_auto_shutdown(Runtime* rt, bool enabled);
void runtime_get_stats(Runtime* rt, RuntimeStats* out_stats);

