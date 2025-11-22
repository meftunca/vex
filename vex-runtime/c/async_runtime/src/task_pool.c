// Task Object Pool - Zero-allocation task recycling
#include "task_pool.h"
#include "internal.h"
#include <string.h>
#include <stdatomic.h>

// Per-worker task pool (thread-local, lock-free)
#define POOL_SIZE 4096

typedef struct TaskPool
{
    InternalTask tasks[POOL_SIZE]; // Pre-allocated tasks
    InternalTask *free_list;       // Lock-free free list
    _Atomic(int) allocated;        // Track usage for debugging
    char padding[64 - sizeof(void*) - sizeof(int)]; // Cache line padding
} TaskPool;

// Thread-local pool (zero contention!)
#ifdef _WIN32
__declspec(thread) static TaskPool *g_pool = NULL;
#else
static __thread TaskPool *g_pool = NULL;
#endif

// Initialize pool with pre-allocated tasks
TaskPool *task_pool_create(void)
{
    TaskPool *pool = (TaskPool *)xmalloc(sizeof(TaskPool));
    if (!pool)
        return NULL;

    memset(pool, 0, sizeof(TaskPool));

    // Initialize free list - link all tasks
    for (int i = 0; i < POOL_SIZE - 1; i++)
    {
        // Use coro_data as 'next' pointer when task is free
        pool->tasks[i].coro_data = &pool->tasks[i + 1];
    }
    pool->tasks[POOL_SIZE - 1].coro_data = NULL;

    pool->free_list = &pool->tasks[0];
    atomic_store(&pool->allocated, 0);

    return pool;
}

// Get thread-local pool (lazy init)
static TaskPool *get_pool(void)
{
    if (!g_pool)
    {
        g_pool = task_pool_create();
    }
    return g_pool;
}

// Allocate task from pool (FAST PATH - no malloc!)
InternalTask *task_pool_alloc(void)
{
    TaskPool *pool = get_pool();
    if (!pool)
        return NULL;

    // Pop from free list (lock-free)
    InternalTask *task = pool->free_list;
    if (task)
    {
        pool->free_list = (InternalTask *)task->coro_data;
        atomic_fetch_add(&pool->allocated, 1);
        
        // Clear task for reuse
        task->coro_data = NULL;
        atomic_store(&task->state, 0);
        task->last_fd = -1;
        
        return task;
    }

    // Pool exhausted - fallback to malloc (should be rare!)
    return (InternalTask *)xmalloc(sizeof(InternalTask));
}

// Free task back to pool (instant recycling)
void task_pool_free(InternalTask *task)
{
    TaskPool *pool = get_pool();
    if (!pool)
    {
        xfree(task);
        return;
    }

    // Check if task is from our pool
    if (task >= &pool->tasks[0] && task < &pool->tasks[POOL_SIZE])
    {
        // Push to free list
        task->coro_data = pool->free_list;
        pool->free_list = task;
        atomic_fetch_sub(&pool->allocated, 1);
    }
    else
    {
        // Task from malloc fallback
        xfree(task);
    }
}

// Get pool statistics
void task_pool_stats(TaskPoolStats *stats)
{
    TaskPool *pool = get_pool();
    if (!pool || !stats)
        return;

    stats->capacity = POOL_SIZE;
    stats->allocated = atomic_load(&pool->allocated);
    stats->free = POOL_SIZE - stats->allocated;
}

void task_pool_destroy(TaskPool *pool)
{
    if (pool)
        xfree(pool);
}
