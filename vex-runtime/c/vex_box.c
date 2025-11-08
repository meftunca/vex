/**
 * vex_box.c - Box<T> heap allocation
 *
 * Heap-allocated value with ownership semantics.
 * Enables recursive types (linked lists, trees).
 *
 * Optimized: Single allocation (Box + value inline)
 * - 2x fewer malloc calls
 * - Better cache locality
 * - Less fragmentation
 *
 * Part of Vex Builtin Types - Phase 0
 * Date: November 5, 2025
 */

#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include "vex.h"

// vex_box_t is defined in vex.h

/**
 * Create new Box with copied value (heap-allocated)
 * Optimized: Single allocation for Box + value (better cache locality)
 * @param value Pointer to value to copy
 * @param size Size of value in bytes
 * @return Pointer to Box containing heap-allocated copy
 */
vex_box_t *vex_box_new(const void *value, size_t size)
{
  // Single allocation: Box header + value data inline
  size_t total_size = sizeof(vex_box_t) + size;
  vex_box_t *box = vex_malloc(total_size);
  if (!box)
  {
    fprintf(stderr, "Box allocation failed (size: %zu)\n", total_size);
    abort();
  }

  // Value data is right after box header
  void *ptr = (void *)((uint8_t *)box + sizeof(vex_box_t));
  memcpy(ptr, value, size);

  box->ptr = ptr;
  box->size = size;
  return box;
}

/**
 * Borrow pointer to Box value (immutable)
 * @param box Pointer to Box
 * @return Pointer to inner value (do not free!)
 */
void *vex_box_get(vex_box_t *box)
{
  return box->ptr;
}

/**
 * Borrow mutable pointer to Box value
 * @param box Pointer to Box
 * @return Mutable pointer to inner value (do not free!)
 */
void *vex_box_get_mut(vex_box_t *box)
{
  return box->ptr;
}

/**
 * Move out inner value (caller takes ownership)
 * Box is consumed, caller must free returned pointer.
 * @param box Box to consume (by value)
 * @return Inner pointer (caller owns)
 */
void *vex_box_into_inner(vex_box_t box)
{
  return box.ptr; // Don't free, caller owns
}

/**
 * Free Box and its inner value
 * Note: Single allocation means one free call
 * @param box Pointer to Box to free
 */
void vex_box_free(vex_box_t *box)
{
  if (box)
  {
    // Single allocation: just free the box (value is inline)
    vex_free(box);
  }
}

/**
 * Clone Box (deep copy)
 * Optimized: Single allocation
 * @param box Pointer to Box to clone
 * @return Pointer to new Box with copied value
 */
vex_box_t *vex_box_clone(vex_box_t *box)
{
  // Single allocation: Box header + value data
  size_t total_size = sizeof(vex_box_t) + box->size;
  vex_box_t *new_box = vex_malloc(total_size);
  if (!new_box)
  {
    fprintf(stderr, "Box clone allocation failed (size: %zu)\n", total_size);
    abort();
  }

  // Value data is right after box header
  void *new_ptr = (void *)((uint8_t *)new_box + sizeof(vex_box_t));
  memcpy(new_ptr, box->ptr, box->size);

  new_box->ptr = new_ptr;
  new_box->size = box->size;
  return new_box;
}
