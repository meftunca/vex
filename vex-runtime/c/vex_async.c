// vex_async.c
// Async runtime integration functions for Vex language

#include "async_runtime/include/runtime.h"
#include <stdint.h>

// async_sleep(millis: i64) - Sleep for specified milliseconds
// Must be called from within an async context (WorkerContext)
void vex_async_sleep(WorkerContext *ctx, uint64_t millis)
{
    if (!ctx) {
        return; // No context, can't await
    }
    
    worker_await_after(ctx, millis);
}
