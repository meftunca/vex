#include "vex_channel.h"
#include "vex.h"
#include <stdint.h>
#include <string.h>

// Helper to check if a number is a power of two
static bool is_power_of_two(size_t n)
{
  return (n > 0) && ((n & (n - 1)) == 0);
}

vex_channel_t *vex_channel_create(size_t capacity)
{
  if (!is_power_of_two(capacity))
  {
    return NULL; // Capacity must be a power of two for efficient masking
  }

  // Single allocation: channel header + slots inline (optimized!)
  size_t total_size = sizeof(vex_channel_t) + (sizeof(vex_channel_slot_t) * capacity);
  vex_channel_t *chan = (vex_channel_t *)vex_malloc(total_size);
  if (!chan)
  {
    return NULL;
  }

  // Slots are right after channel header
  chan->buffer = (vex_channel_slot_t *)((uint8_t *)chan + sizeof(vex_channel_t));
  chan->capacity = capacity;
  chan->mask = capacity - 1;

  for (size_t i = 0; i < capacity; ++i)
  {
    atomic_init(&chan->buffer[i].data, NULL);
    atomic_init(&chan->buffer[i].turn, i);
  }

  atomic_init(&chan->head, 0);
  atomic_init(&chan->tail, 0);
  atomic_init(&chan->closed, false);

  return chan;
}

void vex_channel_destroy(vex_channel_t *chan)
{
  if (chan)
  {
    // Single allocation: just free the channel (buffer is inline)
    vex_free(chan);
  }
}

vex_channel_status_t vex_channel_send(vex_channel_t *chan, void *data)
{
  if (!chan)
    return VEX_CHANNEL_INVALID;

  size_t tail = atomic_load_explicit(&chan->tail, memory_order_relaxed);

  while (true)
  {
    if (atomic_load_explicit(&chan->closed, memory_order_acquire))
    {
      return VEX_CHANNEL_CLOSED;
    }

    vex_channel_slot_t *slot = &chan->buffer[tail & chan->mask];
    size_t turn = atomic_load_explicit(&slot->turn, memory_order_acquire);

    if (turn == tail)
    {
      // This is our turn to write.
      atomic_store_explicit(&slot->data, data, memory_order_relaxed);
      atomic_store_explicit(&slot->turn, tail + 1, memory_order_release);
      atomic_fetch_add_explicit(&chan->tail, 1, memory_order_relaxed); // Use fetch_add for producers
      return VEX_CHANNEL_OK;
    }

    // Another producer is writing to this slot, or the consumer is slow.
    // Let's see if another producer has already advanced the tail.
    size_t new_tail = atomic_load_explicit(&chan->tail, memory_order_relaxed);
    if (new_tail == tail)
    {
      // Tail hasn't moved, channel is likely full. Spin.
      // In a real-world scenario, we might yield here (e.g., sched_yield).
      continue;
    }
    tail = new_tail; // Try again with the new tail.
  }
}

vex_channel_status_t vex_channel_recv(vex_channel_t *chan, void **data_out)
{
  if (!chan || !data_out)
    return VEX_CHANNEL_INVALID;

  size_t head = atomic_load_explicit(&chan->head, memory_order_relaxed);

  while (true)
  {
    vex_channel_slot_t *slot = &chan->buffer[head & chan->mask];
    size_t turn = atomic_load_explicit(&slot->turn, memory_order_acquire);

    if (turn == head + 1)
    {
      // Data is ready for us to read.
      *data_out = atomic_load_explicit(&slot->data, memory_order_relaxed);
      atomic_store_explicit(&slot->turn, head + chan->capacity, memory_order_release);
      atomic_fetch_add_explicit(&chan->head, 1, memory_order_relaxed); // Use fetch_add for the single consumer
      return VEX_CHANNEL_OK;
    }

    // Channel is empty. Check if it's also closed.
    if (atomic_load_explicit(&chan->closed, memory_order_acquire))
    {
      // To be sure, check the tail one last time.
      size_t tail = atomic_load_explicit(&chan->tail, memory_order_acquire);
      if (head == tail)
      {
        return VEX_CHANNEL_CLOSED; // Empty and closed.
      }
    }

    // Spin-wait. In a real application, you might use a condition variable or yield.
    continue;
  }
}

void vex_channel_close(vex_channel_t *chan)
{
  if (chan)
  {
    atomic_store_explicit(&chan->closed, true, memory_order_release);
  }
}

bool vex_channel_is_closed(vex_channel_t *chan)
{
  if (!chan)
    return true;
  return atomic_load_explicit(&chan->closed, memory_order_acquire);
}

vex_channel_status_t vex_channel_try_recv(vex_channel_t *chan, void **data_out)
{
  if (!chan || !data_out)
    return VEX_CHANNEL_INVALID;

  size_t head = atomic_load_explicit(&chan->head, memory_order_relaxed);
  vex_channel_slot_t *slot = &chan->buffer[head & chan->mask];
  size_t turn = atomic_load_explicit(&slot->turn, memory_order_acquire);

  if (turn == head + 1)
  {
    // Data is ready.
    *data_out = atomic_load_explicit(&slot->data, memory_order_relaxed);
    atomic_store_explicit(&slot->turn, head + chan->capacity, memory_order_release);
    atomic_store_explicit(&chan->head, head + 1, memory_order_relaxed);
    return VEX_CHANNEL_OK;
  }

  // Channel is empty. Check if closed.
  if (atomic_load_explicit(&chan->closed, memory_order_acquire))
  {
    size_t tail = atomic_load_explicit(&chan->tail, memory_order_acquire);
    if (head == tail)
    {
      return VEX_CHANNEL_CLOSED;
    }
  }

  return VEX_CHANNEL_EMPTY;
}
