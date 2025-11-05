/**
 * vex_result.c - Result<T, E> runtime helpers
 *
 * Result is compile-time struct: { u8 tag, union { T ok, E err } }
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
 * Unwrap Result value (panics if Err)
 * @param result_ptr Pointer to Result struct
 * @param type_size Size of Ok type T
 * @param file Source file (for error message)
 * @param line Source line (for error message)
 * @return Pointer to Ok value (skip 1-byte tag)
 */
void *vex_result_unwrap(void *result_ptr, size_t type_size, const char *file, int line)
{
  uint8_t tag = *(uint8_t *)result_ptr;
  if (tag == 0)
  { // Err
    fprintf(stderr, "Unwrap failed at %s:%d - Result is Err\n", file, line);
    abort();
  }
  // Return pointer to Ok value (skip 1-byte tag)
  return (uint8_t *)result_ptr + 1;
}

/**
 * Unwrap Result with custom message (panics if Err)
 * @param result_ptr Pointer to Result struct
 * @param type_size Size of Ok type T
 * @param msg Custom error message
 * @param file Source file (for error message)
 * @param line Source line (for error message)
 * @return Pointer to Ok value (skip 1-byte tag)
 */
void *vex_result_expect(void *result_ptr, size_t type_size, const char *msg, const char *file, int line)
{
  uint8_t tag = *(uint8_t *)result_ptr;
  if (tag == 0)
  { // Err
    fprintf(stderr, "Expect failed at %s:%d - %s\n", file, line, msg);
    abort();
  }
  return (uint8_t *)result_ptr + 1;
}

/**
 * Check if Result is Ok
 * @param result_ptr Pointer to Result struct
 * @return true if Ok, false if Err
 */
bool vex_result_is_ok(void *result_ptr)
{
  return *(uint8_t *)result_ptr == 1;
}

/**
 * Check if Result is Err
 * @param result_ptr Pointer to Result struct
 * @return true if Err, false if Ok
 */
bool vex_result_is_err(void *result_ptr)
{
  return *(uint8_t *)result_ptr == 0;
}

/**
 * Unwrap Result or return default value
 * @param result_ptr Pointer to Result struct
 * @param default_val Pointer to default value
 * @param type_size Size of Ok type T
 * @param out Pointer to write result to
 */
void vex_result_unwrap_or(void *result_ptr, const void *default_val, size_t type_size, void *out)
{
  uint8_t tag = *(uint8_t *)result_ptr;
  if (tag == 1)
  { // Ok
    void *value_ptr = (uint8_t *)result_ptr + 1;
    memcpy(out, value_ptr, type_size);
  }
  else
  { // Err
    memcpy(out, default_val, type_size);
  }
}

/**
 * Get error value from Result (panics if Ok)
 * @param result_ptr Pointer to Result struct
 * @param err_type_size Size of Err type E
 * @param file Source file (for error message)
 * @param line Source line (for error message)
 * @return Pointer to Err value
 */
void *vex_result_unwrap_err(void *result_ptr, size_t err_type_size, const char *file, int line)
{
  uint8_t tag = *(uint8_t *)result_ptr;
  if (tag == 1)
  { // Ok
    fprintf(stderr, "Unwrap_err failed at %s:%d - Result is Ok\n", file, line);
    abort();
  }
  // Return pointer to Err value (skip 1-byte tag)
  return (uint8_t *)result_ptr + 1;
}
