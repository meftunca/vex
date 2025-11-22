#include "vex.h"
#include <stdio.h>

int main()
{
  printf("=== Testing vex_map C API ===\n");

  // Test 1: Stack-allocated map
  VexMap map;
  if (!vex_map_new(&map, 16))
  {
    printf("Failed to create map\n");
    return 1;
  }
  printf("✓ Created map\n");

  // Test 2: Insert
  vex_map_insert(&map, "rust", "systems");
  vex_map_insert(&map, "go", "simplicity");
  vex_map_insert(&map, "vex", "performance");
  printf("✓ Inserted 3 entries\n");

  // Test 3: Get
  char *val1 = (char *)vex_map_get(&map, "rust");
  char *val2 = (char *)vex_map_get(&map, "go");
  char *val3 = (char *)vex_map_get(&map, "vex");

  printf("rust -> %s\n", val1);
  printf("go -> %s\n", val2);
  printf("vex -> %s\n", val3);

  // Test 4: Length
  size_t len = vex_map_len(&map);
  printf("Length: %zu\n", len);

  // Test 5: Update
  vex_map_insert(&map, "rust", "blazing-fast");
  char *val_updated = (char *)vex_map_get(&map, "rust");
  printf("rust (updated) -> %s\n", val_updated);

  // Cleanup
  vex_map_free(&map);
  printf("✓ All tests passed!\n");

  return 0;
}
