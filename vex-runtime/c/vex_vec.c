/**
 * vex_vec.c - Generic dynamic array (Vec<T>)
 *
 * Type-erased vector with 2x growth strategy.
 * Zero-copy operations via pointer returns.
 *
 * Part of Vex Builtin Types - Phase 0
 * Date: November 5, 2025
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "vex.h"

// vex_vec_t is defined in vex.h

/**
 * Create new empty vector (heap-allocated)
 * @param elem_size Size of each element in bytes
 * @return Pointer to initialized empty vector
 */
vex_vec_t *vex_vec_new(size_t elem_size)
{
  vex_vec_t *vec = malloc(sizeof(vex_vec_t));
  if (!vec)
  {
    fprintf(stderr, "Vec allocation failed\n");
    abort();
  }
  vec->data = NULL;
  vec->len = 0;
  vec->capacity = 0;
  vec->elem_size = elem_size;
  return vec;
}

/**
 * Internal: Grow capacity using 2x strategy
 * @param vec Vector to grow
 */
static void vex_vec_grow(vex_vec_t *vec)
{
  size_t new_cap = vec->capacity == 0 ? 8 : vec->capacity * 2;
  void *new_data = realloc(vec->data, new_cap * vec->elem_size);
  if (!new_data)
  {
    fprintf(stderr, "Vec allocation failed\n");
    abort();
  }
  vec->data = new_data;
  vec->capacity = new_cap;
}

/**
 * Push element to end of vector
 * @param vec Vector to push to
 * @param elem Pointer to element to copy
 */
void vex_vec_push(vex_vec_t *vec, const void *elem)
{
  if (vec->len >= vec->capacity)
  {
    vex_vec_grow(vec);
  }
  void *dest = (uint8_t *)vec->data + (vec->len * vec->elem_size);
  memcpy(dest, elem, vec->elem_size);
  vec->len++;
}

/**
 * Get element at index (zero-copy via pointer)
 * @param vec Vector to access
 * @param index Index to retrieve
 * @return Pointer to element (do not free!)
 */
void *vex_vec_get(vex_vec_t *vec, size_t index)
{
  if (index >= vec->len)
  {
    fprintf(stderr, "Vec index out of bounds: %zu >= %zu\n", index, vec->len);
    abort();
  }
  return (uint8_t *)vec->data + (index * vec->elem_size);
}

/**
 * Pop last element from vector
 * @param vec Vector to pop from
 * @param out Pointer to write popped element to
 * @return true if element popped, false if empty
 */
bool vex_vec_pop(vex_vec_t *vec, void *out)
{
  if (vec->len == 0)
  {
    return false;
  }
  vec->len--;
  void *src = (uint8_t *)vec->data + (vec->len * vec->elem_size);
  memcpy(out, src, vec->elem_size);
  return true;
}

/**
 * Reserve additional capacity
 * @param vec Vector to reserve in
 * @param additional Number of additional elements to reserve
 */
void vex_vec_reserve(vex_vec_t *vec, size_t additional)
{
  size_t required = vec->len + additional;
  if (required <= vec->capacity)
  {
    return;
  }
  size_t new_cap = vec->capacity == 0 ? 8 : vec->capacity;
  while (new_cap < required)
  {
    new_cap *= 2;
  }
  void *new_data = realloc(vec->data, new_cap * vec->elem_size);
  if (!new_data)
  {
    fprintf(stderr, "Vec reserve failed\n");
    abort();
  }
  vec->data = new_data;
  vec->capacity = new_cap;
}

/**
 * Get current length of vector
 * @param vec Vector to query
 * @return Number of elements
 */
size_t vex_vec_len(vex_vec_t *vec)
{
  return vec->len;
}

/**
 * Get current capacity of vector
 * @param vec Vector to query
 * @return Allocated capacity
 */
size_t vex_vec_capacity(vex_vec_t *vec)
{
  return vec->capacity;
}

/**
 * Check if vector is empty
 * @param vec Vector to check
 * @return true if empty
 */
bool vex_vec_is_empty(vex_vec_t *vec)
{
  return vec->len == 0;
}

/**
 * Clear vector (reset length but keep capacity)
 * @param vec Vector to clear
 */
void vex_vec_clear(vex_vec_t *vec)
{
  vec->len = 0;
}

/**
 * Free vector and its data
 * @param vec Vector to free
 */
void vex_vec_free(vex_vec_t *vec)
{
  if (vec->data)
  {
    free(vec->data);
    vec->data = NULL;
  }
  vec->len = 0;
  vec->capacity = 0;
}
