// vex_set.c
// Set<T> implementation as a thin wrapper over Map<T, ()>
// All operations delegate to vex_swisstable.c hashmap functions

#include "vex.h"
#include <stdlib.h>
#include <string.h>

// Set is just Map with dummy values
// vex_set_t* is same as vex_map_t*

void *vex_set_new(void)
{
  return vex_map_create(16); // Default capacity
}

void *vex_set_with_capacity(int64_t capacity)
{
  return vex_map_create(capacity);
}

bool vex_set_insert(void *set_ptr, void *value_ptr)
{
  // Insert with dummy i8 value (zero byte)
  static uint8_t dummy = 0;
  return vex_map_insert(set_ptr, value_ptr, (void *)&dummy);
}

bool vex_set_contains(void *set_ptr, void *value_ptr)
{
  // Check if get returns non-null
  void *result = vex_map_get(set_ptr, value_ptr);
  return (result != NULL);
}

bool vex_set_remove(void *set_ptr, void *value_ptr)
{
  // Map doesn't have remove yet - just return false for now
  // TODO: Implement vex_map_remove in vex_swisstable.c
  (void)set_ptr;
  (void)value_ptr;
  return false;
}

int64_t vex_set_len(void *set_ptr)
{
  return vex_map_len(set_ptr);
}

void vex_set_clear(void *set_ptr)
{
  // Map doesn't have clear yet - just do nothing for now
  // TODO: Implement vex_map_clear in vex_swisstable.c
  (void)set_ptr;
}

void vex_set_free(void *set_ptr)
{
  vex_map_free(set_ptr);
}
