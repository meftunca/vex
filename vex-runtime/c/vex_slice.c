// vex_slice.c - Slice<T> implementation

#include "vex.h"
#include <string.h>

/**
 * Create a slice from Vec
 * @param vec Vec to slice
 * @return Slice view into Vec's data
 */
VexSlice vex_slice_from_vec(vex_vec_t *vec)
{
  VexSlice slice;
  slice.data = vec->data;
  slice.len = vec->len;
  slice.elem_size = vec->elem_size;
  return slice;
}

/**
 * Create a slice from array
 * @param data Pointer to array data
 * @param len Number of elements
 * @param elem_size Size of each element
 * @return Slice view
 */
VexSlice vex_slice_new(void *data, size_t len, size_t elem_size)
{
  VexSlice slice;
  slice.data = data;
  slice.len = len;
  slice.elem_size = elem_size;
  return slice;
}

/**
 * Get element from slice (bounds checked)
 * @param slice Slice to index
 * @param index Element index
 * @return Pointer to element, or NULL if out of bounds
 */
void *vex_slice_get(const VexSlice *slice, size_t index)
{
  if (index >= slice->len)
  {
    return NULL; // Out of bounds
  }

  char *base = (char *)slice->data;
  return (void *)(base + (index * slice->elem_size));
}

/**
 * Get slice length
 * @param slice Slice to measure
 * @return Number of elements
 */
size_t vex_slice_len(const VexSlice *slice)
{
  return slice->len;
}

/**
 * Check if slice is empty
 * @param slice Slice to check
 * @return true if empty
 */
bool vex_slice_is_empty(const VexSlice *slice)
{
  return slice->len == 0;
}

/**
 * Create a sub-slice [start..end)
 * @param slice Source slice
 * @param start Start index (inclusive)
 * @param end End index (exclusive)
 * @return New slice view
 */
VexSlice vex_slice_subslice(const VexSlice *slice, size_t start, size_t end)
{
  VexSlice sub;

  // Bounds check
  if (start > slice->len)
  {
    start = slice->len;
  }
  if (end > slice->len)
  {
    end = slice->len;
  }
  if (start > end)
  {
    start = end;
  }

  char *base = (char *)slice->data;
  sub.data = (void *)(base + (start * slice->elem_size));
  sub.len = end - start;
  sub.elem_size = slice->elem_size;

  return sub;
}
