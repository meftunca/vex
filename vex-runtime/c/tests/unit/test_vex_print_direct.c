#include "vex.h"
#include <stdio.h>

int main()
{
  VexValue args[2];
  args[0].type = VEX_VALUE_I32;
  args[0].as_i32 = 42;
  args[1].type = VEX_VALUE_I32;
  args[1].as_i32 = 43;

  printf("Calling vex_println_args with 2 integers:\n");
  printf("Expected: 42 43\\n\n");
  vex_println_args(2, args);
  printf("Done.\n");

  // Test vex_print directly
  printf("\nTesting vex_print:\n");
  vex_print("Hello", 5);
  vex_print(" ", 1);
  vex_print("World", 5);
  vex_print("\n", 1);

  return 0;
}
