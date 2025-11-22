/**
 * Test for builtin types (Vec, Option, Result, Box)
 * Phase 0 - C Runtime Test
 */

#include <stdio.h>
#include <assert.h>
#include "vex.h"

void test_vec()
{
  printf("Testing Vec<i32>...\n");

  // Create vector (now returns pointer)
  vex_vec_t *vec = vex_vec_new(sizeof(int32_t));
  assert(vex_vec_len(vec) == 0);
  assert(vex_vec_is_empty(vec));

  // Push elements
  int32_t val1 = 10;
  int32_t val2 = 20;
  int32_t val3 = 30;
  vex_vec_push(vec, &val1);
  vex_vec_push(vec, &val2);
  vex_vec_push(vec, &val3);

  assert(vex_vec_len(vec) == 3);
  assert(!vex_vec_is_empty(vec));

  // Get elements
  int32_t *ptr1 = (int32_t *)vex_vec_get(vec, 0);
  int32_t *ptr2 = (int32_t *)vex_vec_get(vec, 1);
  int32_t *ptr3 = (int32_t *)vex_vec_get(vec, 2);
  assert(*ptr1 == 10);
  assert(*ptr2 == 20);
  assert(*ptr3 == 30);

  // Pop element
  int32_t popped;
  bool success = vex_vec_pop(vec, &popped);
  assert(success);
  assert(popped == 30);
  assert(vex_vec_len(vec) == 2);

  // Free
  vex_vec_free(vec);

  printf("  ✓ Vec tests passed!\n");
}

void test_option()
{
  printf("Testing Option<i32>...\n");

  // Option is compile-time struct: { u8 tag, i32 value }
  // Note: Packed to avoid padding issues in C runtime
  struct __attribute__((packed))
  {
    uint8_t tag; // 0 = None, 1 = Some
    int32_t value;
  } opt_some, opt_none;

  // Some(42)
  opt_some.tag = 1;
  opt_some.value = 42;

  assert(vex_option_is_some(&opt_some));
  assert(!vex_option_is_none(&opt_some));

  // None
  opt_none.tag = 0;
  opt_none.value = 0; // Doesn't matter

  assert(!vex_option_is_some(&opt_none));
  assert(vex_option_is_none(&opt_none));

  // Unwrap Some
  int32_t *unwrapped = (int32_t *)vex_option_unwrap(&opt_some, sizeof(int32_t), __FILE__, __LINE__);
  assert(*unwrapped == 42);

  // Unwrap_or
  int32_t default_val = 99;
  int32_t result;
  vex_option_unwrap_or(&opt_none, &default_val, sizeof(int32_t), &result);
  assert(result == 99);

  printf("  ✓ Option tests passed!\n");
}

void test_result()
{
  printf("Testing Result<i32, i32>...\n");

  // Result is compile-time struct: { u8 tag, union { T ok, E err } }
  struct
  {
    uint8_t tag;   // 0 = Err, 1 = Ok
    int32_t value; // ok or err (same size here)
  } res_ok, res_err;

  // Ok(42)
  res_ok.tag = 1;
  res_ok.value = 42;

  assert(vex_result_is_ok(&res_ok));
  assert(!vex_result_is_err(&res_ok));

  // Err(-1)
  res_err.tag = 0;
  res_err.value = -1;

  assert(!vex_result_is_ok(&res_err));
  assert(vex_result_is_err(&res_err));

  // Unwrap Ok
  int32_t *unwrapped = (int32_t *)vex_result_unwrap(&res_ok, sizeof(int32_t), __FILE__, __LINE__);
  assert(*unwrapped == 42);

  // Unwrap_err
  int32_t *err_val = (int32_t *)vex_result_unwrap_err(&res_err, sizeof(int32_t), __FILE__, __LINE__);
  assert(*err_val == -1);

  printf("  ✓ Result tests passed!\n");
}

void test_box()
{
  printf("Testing Box<i32>...\n");

  // Box.new(42) - now returns pointer
  int32_t val = 42;
  vex_box_t *box = vex_box_new(&val, sizeof(int32_t));

  // Borrow
  int32_t *ptr = (int32_t *)vex_box_get(box);
  assert(*ptr == 42);

  // Modify through mutable borrow
  int32_t *ptr_mut = (int32_t *)vex_box_get_mut(box);
  *ptr_mut = 100;
  assert(*ptr_mut == 100);

  // Clone
  vex_box_t *box2 = vex_box_clone(box);
  int32_t *ptr2 = (int32_t *)vex_box_get(box2);
  assert(*ptr2 == 100);

  // Free both
  vex_box_free(box);
  vex_box_free(box2);

  printf("  ✓ Box tests passed!\n");
}

int main()
{
  printf("=== Builtin Types C Runtime Tests ===\n\n");

  test_vec();
  test_option();
  test_result();
  test_box();

  printf("\n=== All tests passed! ===\n");
  return 0;
}
