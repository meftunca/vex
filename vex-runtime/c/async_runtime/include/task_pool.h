#pragma once
#include <stddef.h>
#include <stdint.h>

typedef struct InternalTask InternalTask;
typedef struct TaskPool TaskPool;

typedef struct
{
    size_t capacity;
    size_t allocated;
    size_t free;
} TaskPoolStats;

// Create task pool (per-worker)
TaskPool *task_pool_create(void);

// Allocate task from pool (zero-allocation fast path)
InternalTask *task_pool_alloc(void);

// Free task back to pool (instant recycling)
void task_pool_free(InternalTask *task);

// Get pool statistics
void task_pool_stats(TaskPoolStats *stats);

// Destroy pool
void task_pool_destroy(TaskPool *pool);
