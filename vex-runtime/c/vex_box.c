/**
 * vex_box.c - Box<T> heap allocation
 *
 * Heap-allocated value with ownership semantics.
 * Enables recursive types (linked lists, trees).
 *
 * Part of Vex Builtin Types - Phase 0
 * Date: November 5, 2025
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "vex.h"

// vex_box_t is defined in vex.h

/**
 * Create new Box with copied value (heap-allocated)
 * @param value Pointer to value to copy
 * @param size Size of value in bytes
 * @return Pointer to Box containing heap-allocated copy
 */
vex_box_t *vex_box_new(const void *value, size_t size)
{
  vex_box_t *box = malloc(sizeof(vex_box_t));
  if (!box)
  {
    fprintf(stderr, "Box struct allocation failed\n");
    abort();
  }

  void *ptr = malloc(size);
  if (!ptr)
  {
    fprintf(stderr, "Box value allocation failed (size: %zu)\n", size);
    free(box);
    abort();
  }
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
 * @param box Pointer to Box to free
 */
void vex_box_free(vex_box_t *box)
{
  if (box && box->ptr)
  {
    free(box->ptr);
    free(box);
  }
}

/**
 * Clone Box (deep copy)
 * @param box Pointer to Box to clone
 * @return Pointer to new Box with copied value
 */
vex_box_t *vex_box_clone(vex_box_t *box)
{
  vex_box_t *new_box = malloc(sizeof(vex_box_t));
  if (!new_box)
  {
    fprintf(stderr, "Box clone struct allocation failed\n");
    abort();
  }

  void *new_ptr = malloc(box->size);
  if (!new_ptr)
  {
    fprintf(stderr, "Box clone value allocation failed (size: %zu)\n", box->size);
    free(new_box);
    abort();
  }
  memcpy(new_ptr, box->ptr, box->size);

  new_box->ptr = new_ptr;
  new_box->size = box->size;
  return new_box;
}
