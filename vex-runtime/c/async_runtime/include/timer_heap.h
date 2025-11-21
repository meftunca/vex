#pragma once
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// Timer entry for task scheduling
typedef struct TimerEntry
{
  uint64_t deadline_ns; // Absolute deadline (nanoseconds since epoch)
  void *task;           // InternalTask pointer
} TimerEntry;

// Min-heap timer queue for efficient deadline management
typedef struct TimerHeap
{
  TimerEntry *entries; // Dynamic array of timer entries
  size_t size;         // Current number of entries
  size_t capacity;     // Allocated capacity
} TimerHeap;

// Create a new timer heap
TimerHeap *timer_heap_create(size_t initial_capacity);

// Destroy timer heap (does not free tasks)
void timer_heap_destroy(TimerHeap *heap);

// Insert a timer entry (returns false if allocation fails)
bool timer_heap_insert(TimerHeap *heap, uint64_t deadline_ns, void *task);

// Get the minimum deadline without removing (returns UINT64_MAX if empty)
uint64_t timer_heap_peek_deadline(const TimerHeap *heap);

// Extract all expired timers up to `now_ns` and call callback for each
// Returns number of expired timers
size_t timer_heap_pop_expired(TimerHeap *heap, uint64_t now_ns,
                              void (*callback)(void *task, void *user_data),
                              void *user_data);

// Get current size
static inline size_t timer_heap_size(const TimerHeap *heap)
{
  return heap ? heap->size : 0;
}

// Check if empty
static inline bool timer_heap_empty(const TimerHeap *heap)
{
  return heap ? heap->size == 0 : true;
}
