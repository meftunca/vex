#include "timer_heap.h"
#include "internal.h"
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define TIMER_HEAP_MIN_CAPACITY 16

// Helper: Get current time in nanoseconds
static uint64_t get_time_ns(void)
{
  struct timespec ts;
  clock_gettime(CLOCK_MONOTONIC, &ts);
  return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

// Min-heap operations
static void heap_swap(TimerEntry *a, TimerEntry *b)
{
  TimerEntry tmp = *a;
  *a = *b;
  *b = tmp;
}

static void heap_bubble_up(TimerHeap *heap, size_t idx)
{
  while (idx > 0)
  {
    size_t parent = (idx - 1) / 2;
    if (heap->entries[idx].deadline_ns >= heap->entries[parent].deadline_ns)
      break;
    heap_swap(&heap->entries[idx], &heap->entries[parent]);
    idx = parent;
  }
}

static void heap_bubble_down(TimerHeap *heap, size_t idx)
{
  while (true)
  {
    size_t left = 2 * idx + 1;
    size_t right = 2 * idx + 2;
    size_t smallest = idx;

    if (left < heap->size &&
        heap->entries[left].deadline_ns < heap->entries[smallest].deadline_ns)
      smallest = left;

    if (right < heap->size &&
        heap->entries[right].deadline_ns < heap->entries[smallest].deadline_ns)
      smallest = right;

    if (smallest == idx)
      break;

    heap_swap(&heap->entries[idx], &heap->entries[smallest]);
    idx = smallest;
  }
}

TimerHeap *timer_heap_create(size_t initial_capacity)
{
  if (initial_capacity < TIMER_HEAP_MIN_CAPACITY)
    initial_capacity = TIMER_HEAP_MIN_CAPACITY;

  TimerHeap *heap = (TimerHeap *)xmalloc(sizeof(TimerHeap));
  heap->entries = (TimerEntry *)xmalloc(sizeof(TimerEntry) * initial_capacity);
  heap->size = 0;
  heap->capacity = initial_capacity;
  return heap;
}

void timer_heap_destroy(TimerHeap *heap)
{
  if (!heap)
    return;
  xfree(heap->entries);
  xfree(heap);
}

bool timer_heap_insert(TimerHeap *heap, uint64_t deadline_ns, void *task)
{
  if (!heap || !task)
    return false;

  // Grow if needed
  if (heap->size >= heap->capacity)
  {
    size_t new_capacity = heap->capacity * 2;
    TimerEntry *new_entries = (TimerEntry *)xmalloc(sizeof(TimerEntry) * new_capacity);
    memcpy(new_entries, heap->entries, sizeof(TimerEntry) * heap->size);
    xfree(heap->entries);
    heap->entries = new_entries;
    heap->capacity = new_capacity;
  }

  // Insert at end and bubble up
  heap->entries[heap->size].deadline_ns = deadline_ns;
  heap->entries[heap->size].task = task;
  heap_bubble_up(heap, heap->size);
  heap->size++;
  return true;
}

uint64_t timer_heap_peek_deadline(const TimerHeap *heap)
{
  if (!heap || heap->size == 0)
    return UINT64_MAX;
  return heap->entries[0].deadline_ns;
}

size_t timer_heap_pop_expired(TimerHeap *heap, uint64_t now_ns,
                              void (*callback)(void *task, void *user_data),
                              void *user_data)
{
  if (!heap || !callback)
    return 0;

  size_t count = 0;
  while (heap->size > 0 && heap->entries[0].deadline_ns <= now_ns)
  {
    void *task = heap->entries[0].task;

    // Remove root: move last to root, decrease size, bubble down
    heap->entries[0] = heap->entries[heap->size - 1];
    heap->size--;
    if (heap->size > 0)
      heap_bubble_down(heap, 0);

    // Process expired timer
    callback(task, user_data);
    count++;
  }
  return count;
}
