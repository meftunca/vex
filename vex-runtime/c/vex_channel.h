#ifndef VEX_CHANNEL_H
#define VEX_CHANNEL_H

#include <stdatomic.h>
#include <stdalign.h>
#include <stdbool.h>
#include <stddef.h>

// Represents the status of a channel operation
typedef enum
{
  VEX_CHANNEL_OK,      // Operation successful
  VEX_CHANNEL_FULL,    // Channel is full (for non-blocking send)
  VEX_CHANNEL_EMPTY,   // Channel is empty (for non-blocking receive)
  VEX_CHANNEL_CLOSED,  // Channel is closed
  VEX_CHANNEL_INVALID, // Invalid operation or arguments
} vex_channel_status_t;

// Represents a single slot in the channel buffer.
typedef struct
{
  _Atomic(void *) data;
  atomic_size_t turn;
} vex_channel_slot_t;

// MPSC (Multi-Producer, Single-Consumer) Channel.
typedef struct
{
  size_t capacity;
  size_t mask;
  vex_channel_slot_t *buffer;

  // Consumer side - cache line padding to prevent false sharing
  alignas(64) atomic_size_t head;

  // Producer side - cache line padding
  alignas(64) atomic_size_t tail;

  // Used to signal when the channel is closed.
  alignas(64) atomic_bool closed;

} vex_channel_t;

/**
 * @brief Creates a new channel with a given capacity.
 *
 * @param capacity The capacity of the channel. Must be a power of 2.
 * @return A pointer to the new channel, or NULL if allocation fails or capacity is invalid.
 */
vex_channel_t *vex_channel_create(size_t capacity);

/**
 * @brief Destroys a channel and frees its resources.
 *
 * @param chan The channel to destroy.
 */
void vex_channel_destroy(vex_channel_t *chan);

/**
 * @brief Sends a value into the channel.
 * This is the producer function. It is thread-safe to call from multiple producers.
 * It will spin-wait if the channel is full.
 *
 * @param chan The channel.
 * @param data The data pointer to send.
 * @return VEX_CHANNEL_OK if successful, VEX_CHANNEL_CLOSED if the channel was closed.
 */
vex_channel_status_t vex_channel_send(vex_channel_t *chan, void *data);

/**
 * @brief Receives a value from the channel.
 * This is the consumer function. It is NOT thread-safe; only one consumer at a time.
 * It will spin-wait if the channel is empty.
 *
 * @param chan The channel.
 * @param data_out A pointer to a void* where the received data will be stored.
 * @return VEX_CHANNEL_OK if successful, VEX_CHANNEL_CLOSED if the channel is empty and closed.
 */
vex_channel_status_t vex_channel_recv(vex_channel_t *chan, void **data_out);

/**
 * @brief Attempts to receive a value from the channel without blocking.
 * This is the consumer function. It is NOT thread-safe.
 *
 * @param chan The channel.
 * @param data_out A pointer to a void* where the received data will be stored.
 * @return VEX_CHANNEL_OK if a value was received.
 *         VEX_CHANNEL_EMPTY if the channel is currently empty.
 *         VEX_CHANNEL_CLOSED if the channel is empty and closed.
 */
vex_channel_status_t vex_channel_try_recv(vex_channel_t *chan, void **data_out);

/**
 * @brief Closes the channel, preventing further sends.
 *
 * @param chan The channel to close.
 */
void vex_channel_close(vex_channel_t *chan);

#endif // VEX_CHANNEL_H
