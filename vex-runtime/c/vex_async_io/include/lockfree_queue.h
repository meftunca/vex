#pragma once
#include <stdatomic.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Bounded MPMC queue based on Dmitry Vyukov's algorithm.
typedef struct LFQSlot {
    _Atomic(size_t) seq;
    void* data;
} LFQSlot;

typedef struct LockFreeQueue {
    size_t mask;
    LFQSlot* buffer;
    _Atomic(size_t) head;
    _Atomic(size_t) tail;
} LockFreeQueue;

LockFreeQueue* lfq_create(size_t capacity_pow2);
void lfq_destroy(LockFreeQueue* q);
bool lfq_enqueue(LockFreeQueue* q, void* ptr);
bool lfq_dequeue(LockFreeQueue* q, void** out_ptr);

#ifdef __cplusplus
}
#endif
