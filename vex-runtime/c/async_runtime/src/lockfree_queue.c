#include "lockfree_queue.h"
#include <stdlib.h>
#include <string.h>

static size_t round_up_pow2(size_t v) {
    if (v < 2) return 2;
    v--;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
#if ULONG_MAX > 0xffffffffUL
    v |= v >> 32;
#endif
    v++;
    return v;
}

LockFreeQueue* lfq_create(size_t capacity_pow2) {
    LockFreeQueue* q = (LockFreeQueue*)malloc(sizeof(LockFreeQueue));
    if (!q) return NULL;
    size_t cap = round_up_pow2(capacity_pow2);
    q->mask = cap - 1;
    q->buffer = (LFQSlot*)aligned_alloc(64, sizeof(LFQSlot) * cap);
    if (!q->buffer) { free(q); return NULL; }
    for (size_t i = 0; i < cap; ++i) {
        atomic_store(&q->buffer[i].seq, i);
        q->buffer[i].data = NULL;
    }
    atomic_store(&q->head, 0);
    atomic_store(&q->tail, 0);
    return q;
}

void lfq_destroy(LockFreeQueue* q) {
    if (!q) return;
    free(q->buffer);
    free(q);
}

bool lfq_enqueue(LockFreeQueue* q, void* ptr) {
    LFQSlot* slot;
    size_t pos = atomic_load_explicit(&q->tail, memory_order_relaxed);
    for (;;) {
        slot = &q->buffer[pos & q->mask];
        size_t seq = atomic_load_explicit(&slot->seq, memory_order_acquire);
        intptr_t dif = (intptr_t)seq - (intptr_t)pos;
        if (dif == 0) {
            if (atomic_compare_exchange_weak_explicit(&q->tail, &pos, pos + 1,
                                                      memory_order_relaxed, memory_order_relaxed)) {
                break;
            }
        } else if (dif < 0) {
            return false; // full
        } else {
            pos = atomic_load_explicit(&q->tail, memory_order_relaxed);
        }
    }
    slot->data = ptr;
    atomic_store_explicit(&slot->seq, pos + 1, memory_order_release);
    return true;
}

bool lfq_dequeue(LockFreeQueue* q, void** out_ptr) {
    LFQSlot* slot;
    size_t pos = atomic_load_explicit(&q->head, memory_order_relaxed);
    for (;;) {
        slot = &q->buffer[pos & q->mask];
        size_t seq = atomic_load_explicit(&slot->seq, memory_order_acquire);
        intptr_t dif = (intptr_t)seq - (intptr_t)(pos + 1);
        if (dif == 0) {
            if (atomic_compare_exchange_weak_explicit(&q->head, &pos, pos + 1,
                                                      memory_order_relaxed, memory_order_relaxed)) {
                break;
            }
        } else if (dif < 0) {
            return false; // empty
        } else {
            pos = atomic_load_explicit(&q->head, memory_order_relaxed);
        }
    }
    *out_ptr = slot->data;
    atomic_store_explicit(&slot->seq, pos + q->mask + 1, memory_order_release);
    return true;
}
