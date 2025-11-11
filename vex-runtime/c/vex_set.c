// vex_set.c
// Set<T> implementation as a thin wrapper over Map<T, ()>
// All operations delegate to vex_swisstable_v2.c hashmap functions
// 
// Performance: Uses Swiss Tables V3 - up to 2× faster than V2 on inserts/lookups
//   - Insert: ~19M ops/s (51 ns/op) on 100K items (ARM64)
//   - Lookup: ~13M ops/s (75 ns/op) on 100K items (ARM64)
//   - Remove: ~48M ops/s (21 ns/op) on 100K items (ARM64)

#include "vex.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Set is just Map with dummy values
// vex_set_t* is same as VexMap*

void *vex_set_new(void)
{
  VexMap *map = malloc(sizeof(VexMap));
  if (!map) return NULL;
  
  // Initialize the map structure (V2 modifies in-place)
  if (!vex_map_new_v3(map, 16)) {
    free(map);
    return NULL;
  }
  return map;
}

void *vex_set_with_capacity(int64_t capacity)
{
  VexMap *map = malloc(sizeof(VexMap));
  if (!map) return NULL;
  
  if (!vex_map_new_v3(map, capacity)) {
    free(map);
    return NULL;
  }
  return map;
}

bool vex_set_insert(void *set_ptr, void *value_ptr)
{
  if (!set_ptr || !value_ptr) return false;

  const char *key = *(const char **)value_ptr;
  fprintf(stderr, "[insert] set=%p value_ptr=%p key_ptr=%p\n", set_ptr, value_ptr, (void *)key);
  if (key)
    fprintf(stderr, "[insert] key preview: %.16s\n", key);
  if (!key) return false;

  // Insert with dummy i8 value (zero byte)
  static uint8_t dummy = 0;
  return vex_map_insert_v3((VexMap *)set_ptr, key, (void *)&dummy);
}

bool vex_set_contains(void *set_ptr, void *value_ptr)
{
  if (!set_ptr || !value_ptr) return false;

  const char *key = *(const char **)value_ptr;
  fprintf(stderr, "[contains] set=%p value_ptr=%p key_ptr=%p\n", set_ptr, value_ptr, (void *)key);
  if (!key) return false;

  // Check if get returns non-null
  void *result = vex_map_get_v3((const VexMap *)set_ptr, key);
  return (result != NULL);
}

bool vex_set_remove(void *set_ptr, void *value_ptr)
{
  if (!set_ptr || !value_ptr) return false;

  const char *key = *(const char **)value_ptr;
  fprintf(stderr, "[remove] set=%p value_ptr=%p key_ptr=%p\n", set_ptr, value_ptr, (void *)key);
  if (!key) return false;

  return vex_map_remove_v3((VexMap *)set_ptr, key);
}

int64_t vex_set_len(void *set_ptr)
{
  if (!set_ptr) return 0;
  return (int64_t)vex_map_len_v3((const VexMap *)set_ptr);
}

void vex_set_clear(void *set_ptr)
{
  if (!set_ptr) return;
  
  // ✅ IMPLEMENTED: Uses Swiss Tables V2 clear (keeps capacity)
  vex_map_clear_v3((VexMap *)set_ptr);
}

void vex_set_free(void *set_ptr)
{
  if (!set_ptr) return;
  
  vex_map_free_v3((VexMap *)set_ptr);
  free(set_ptr);
}
