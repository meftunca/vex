/**
 * vex_option.c - Option<T> runtime helpers
 *
 * Option is compile-time struct: { u8 tag, T value }
 * Runtime provides unwrap and checking helpers.
 *
 * Part of Vex Builtin Types - Phase 0
 * Date: November 5, 2025
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include "vex.h"

/**
 * Unwrap Option value (panics if None)
 * @param opt_ptr Pointer to Option struct
 * @param type_size Size of inner type T
 * @param file Source file (for error message)
 * @param line Source line (for error message)
 * @return Pointer to value (skip 1-byte tag)
 */
void *vex_option_unwrap(void *opt_ptr, size_t type_size, const char *file, int line)
{
  uint8_t tag = *(uint8_t *)opt_ptr;
  if (tag == 0)
  { // None
    fprintf(stderr, "Unwrap failed at %s:%d - Option is None\n", file, line);
    abort();
  }
  // Return pointer to value (skip 1-byte tag)
  return (uint8_t *)opt_ptr + 1;
}

/**
 * Unwrap Option with custom message (panics if None)
 * @param opt_ptr Pointer to Option struct
 * @param type_size Size of inner type T
 * @param msg Custom error message
 * @param file Source file (for error message)
 * @param line Source line (for error message)
 * @return Pointer to value (skip 1-byte tag)
 */
void *vex_option_expect(void *opt_ptr, size_t type_size, const char *msg, const char *file, int line)
{
  uint8_t tag = *(uint8_t *)opt_ptr;
  if (tag == 0)
  { // None
    fprintf(stderr, "Expect failed at %s:%d - %s\n", file, line, msg);
    abort();
  }
  return (uint8_t *)opt_ptr + 1;
}

/**
 * Check if Option is Some
 * @param opt_ptr Pointer to Option struct
 * @return true if Some, false if None
 */
bool vex_option_is_some(void *opt_ptr)
{
  return *(uint8_t *)opt_ptr == 1;
}

/**
 * Check if Option is None
 * @param opt_ptr Pointer to Option struct
 * @return true if None, false if Some
 */
bool vex_option_is_none(void *opt_ptr)
{
  return *(uint8_t *)opt_ptr == 0;
}

/**
 * Unwrap Option or return default value
 * @param opt_ptr Pointer to Option struct
 * @param default_val Pointer to default value
 * @param type_size Size of inner type T
 * @param out Pointer to write result to
 */
void vex_option_unwrap_or(void *opt_ptr, const void *default_val, size_t type_size, void *out)
{
  uint8_t tag = *(uint8_t *)opt_ptr;
  if (tag == 1)
  { // Some
    void *value_ptr = (uint8_t *)opt_ptr + 1;
    memcpy(out, value_ptr, type_size);
  }
  else
  { // None
    memcpy(out, default_val, type_size);
  }
}
