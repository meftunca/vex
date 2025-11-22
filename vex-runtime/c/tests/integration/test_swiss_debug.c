#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "vex.h"

int main()
{
  VexMap m;
  if (!vex_map_new(&m, 8))
  {
    fprintf(stderr, "map_new failed\n");
    return 1;
  }

  // Simple sequential test
  int vals[1000];
  char keys[1000][32];

  // Insert 1000 items
  for (int i = 0; i < 1000; i++)
  {
    snprintf(keys[i], 32, "key_%d", i);
    vals[i] = i * 2;
    if (!vex_map_insert(&m, keys[i], &vals[i]))
    {
      fprintf(stderr, "Insert failed at %d\n", i);
      return 1;
    }
  }

  printf("Inserted 1000 items\n");
  printf("Map length: %zu\n", vex_map_len(&m));

  // Verify all keys
  int missing = 0;
  int wrong = 0;
  for (int i = 0; i < 1000; i++)
  {
    int *p = (int *)vex_map_get(&m, keys[i]);
    if (!p)
    {
      printf("Missing key: %s (expected %d)\n", keys[i], vals[i]);
      missing++;
    }
    else if (*p != vals[i])
    {
      printf("Wrong value for %s: got %d, expected %d\n", keys[i], *p, vals[i]);
      wrong++;
    }
  }

  printf("Missing: %d, Wrong: %d\n", missing, wrong);

  vex_map_free(&m);
  return (missing || wrong) ? 1 : 0;
}
