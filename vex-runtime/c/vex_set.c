// vex_set.c
// VEX Set - Simple set wrapper around SwissTable hash map
// All operations delegate to vex_swisstable.c hashmap functions

#include "vex.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Set is just Map with dummy values
// vex_set_t* is same as VexMap*

void *vex_set_new(void)
{
  VexMap *map = (VexMap *)vex_malloc(sizeof(VexMap));
  if (!map)
    return NULL;

  // Initialize the map structure
  if (!vex_map_new(map, 16))
  {
    vex_free(map);
    return NULL;
  }
  return map;
}

void *vex_set_with_capacity(int64_t capacity)
{
  VexMap *map = (VexMap *)vex_malloc(sizeof(VexMap));
  if (!map)
    return NULL;

  if (!vex_map_new(map, capacity))
  {
    vex_free(map);
    return NULL;
  }
  return map;
}

bool vex_set_insert(void *set_ptr, void *value_ptr)
{
  if (!set_ptr || !value_ptr)
    return false;

  const char *key = *(const char **)value_ptr;
  // fprintf(stderr, "[insert] set=%p value_ptr=%p key_ptr=%p\n", set_ptr, value_ptr, (void *)key);
  if (key)
    // fprintf(stderr, "[insert] key preview: %.16s\n", key);
  if (!key)
    return false;

  // Insert with dummy i8 value (zero byte)
  static uint8_t dummy = 0;
  size_t len = strlen(key);
  return vex_map_insert((VexMap *)set_ptr, key, len, (void *)&dummy);
}

bool vex_set_contains(void *set_ptr, void *value_ptr)
{
  if (!set_ptr || !value_ptr)
    return false;

  const char *key = *(const char **)value_ptr;
  // fprintf(stderr, "[contains] set=%p value_ptr=%p key_ptr=%p\n", set_ptr, value_ptr, (void *)key);
  if (!key)
    return false;

  size_t len = strlen(key);
  // Check if get returns non-null
  void *result = vex_map_get((const VexMap *)set_ptr, key, len);
  return (result != NULL);
}

bool vex_set_remove(void *set_ptr, void *value_ptr)
{
  if (!set_ptr || !value_ptr)
    return false;

  const char *key = *(const char **)value_ptr;
  // fprintf(stderr, "[remove] set=%p value_ptr=%p key_ptr=%p\n", set_ptr, value_ptr, (void *)key);
  if (!key)
    return false;

  size_t len = strlen(key);
  return vex_map_remove((VexMap *)set_ptr, key, len);
}

int64_t vex_set_len(void *set_ptr)
{
  if (!set_ptr)
    return 0;
  return (int64_t)vex_map_len((const VexMap *)set_ptr);
}

void vex_set_clear(void *set_ptr)
{
  if (!set_ptr)
    return;

  // Re-create map to clear (V1 doesn't have explicit clear)
  vex_map_free((VexMap *)set_ptr);
  vex_map_new((VexMap *)set_ptr, 16);
}

void vex_set_free(void *set_ptr)
{
  if (!set_ptr)
    return;

  vex_map_free((VexMap *)set_ptr);
  vex_free(set_ptr);
}
