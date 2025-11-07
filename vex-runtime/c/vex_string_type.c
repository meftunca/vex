/**
 * Vex String Type - UTF-8 String Operations
 * Heap-allocated, growable string with UTF-8 validation
 */

#include "vex.h"
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

// ============================================================================
// STRING TYPE OPERATIONS
// ============================================================================

/**
 * Create a new empty string
 */
vex_string_t *vex_string_new(void)
{
  vex_string_t *str = (vex_string_t *)vex_malloc(sizeof(vex_string_t));
  if (!str)
    return NULL;

  str->capacity = 16; // Initial capacity
  str->data = (char *)vex_malloc(str->capacity);
  if (!str->data)
  {
    vex_free(str);
    return NULL;
  }

  str->data[0] = '\0';
  str->len = 0;
  return str;
}

/**
 * Create a string from a C string
 */
vex_string_t *vex_string_from_cstr(const char *cstr)
{
  if (!cstr)
    return vex_string_new();

  size_t len = vex_strlen(cstr);
  vex_string_t *str = (vex_string_t *)vex_malloc(sizeof(vex_string_t));
  if (!str)
    return NULL;

  str->len = len;
  str->capacity = len + 1; // +1 for null terminator
  str->data = (char *)vex_malloc(str->capacity);
  if (!str->data)
  {
    vex_free(str);
    return NULL;
  }

  vex_memcpy(str->data, cstr, len + 1); // Copy including null terminator
  return str;
}

/**
 * Create a string with specified capacity
 */
vex_string_t *vex_string_with_capacity(size_t capacity)
{
  vex_string_t *str = (vex_string_t *)vex_malloc(sizeof(vex_string_t));
  if (!str)
    return NULL;

  str->capacity = capacity > 0 ? capacity : 16;
  str->data = (char *)vex_malloc(str->capacity);
  if (!str->data)
  {
    vex_free(str);
    return NULL;
  }

  str->data[0] = '\0';
  str->len = 0;
  return str;
}

/**
 * Ensure string has enough capacity for additional bytes
 */
static void vex_string_reserve(vex_string_t *str, size_t additional)
{
  size_t required = str->len + additional + 1; // +1 for null terminator
  if (required <= str->capacity)
    return;

  size_t new_capacity = str->capacity * 2;
  while (new_capacity < required)
  {
    new_capacity *= 2;
  }

  char *new_data = (char *)vex_realloc(str->data, new_capacity);
  if (!new_data)
  {
    fprintf(stderr, "Fatal: Failed to reallocate string memory\n");
    exit(1);
  }

  str->data = new_data;
  str->capacity = new_capacity;
}

/**
 * Append a C string to the string
 */
void vex_string_push_str(vex_string_t *str, const char *cstr)
{
  if (!str || !cstr)
    return;

  size_t cstr_len = vex_strlen(cstr);
  if (cstr_len == 0)
    return;

  vex_string_reserve(str, cstr_len);
  vex_memcpy(str->data + str->len, cstr, cstr_len);
  str->len += cstr_len;
  str->data[str->len] = '\0';
}

/**
 * Append a Unicode codepoint as UTF-8
 */
void vex_string_push_char(vex_string_t *str, uint32_t codepoint)
{
  if (!str)
    return;

  char utf8_bytes[4];
  size_t bytes_written = 0;

  if (codepoint <= 0x7F)
  {
    // 1-byte UTF-8 (ASCII)
    utf8_bytes[0] = (char)codepoint;
    bytes_written = 1;
  }
  else if (codepoint <= 0x7FF)
  {
    // 2-byte UTF-8
    utf8_bytes[0] = (char)(0xC0 | (codepoint >> 6));
    utf8_bytes[1] = (char)(0x80 | (codepoint & 0x3F));
    bytes_written = 2;
  }
  else if (codepoint <= 0xFFFF)
  {
    // 3-byte UTF-8
    utf8_bytes[0] = (char)(0xE0 | (codepoint >> 12));
    utf8_bytes[1] = (char)(0x80 | ((codepoint >> 6) & 0x3F));
    utf8_bytes[2] = (char)(0x80 | (codepoint & 0x3F));
    bytes_written = 3;
  }
  else if (codepoint <= 0x10FFFF)
  {
    // 4-byte UTF-8
    utf8_bytes[0] = (char)(0xF0 | (codepoint >> 18));
    utf8_bytes[1] = (char)(0x80 | ((codepoint >> 12) & 0x3F));
    utf8_bytes[2] = (char)(0x80 | ((codepoint >> 6) & 0x3F));
    utf8_bytes[3] = (char)(0x80 | (codepoint & 0x3F));
    bytes_written = 4;
  }
  else
  {
    // Invalid codepoint
    return;
  }

  vex_string_reserve(str, bytes_written);
  vex_memcpy(str->data + str->len, utf8_bytes, bytes_written);
  str->len += bytes_written;
  str->data[str->len] = '\0';
}

/**
 * Get string length in bytes
 */
size_t vex_string_len(vex_string_t *str)
{
  return str ? str->len : 0;
}

/**
 * Get string capacity in bytes
 */
size_t vex_string_capacity(vex_string_t *str)
{
  return str ? str->capacity : 0;
}

/**
 * Get string length in UTF-8 characters
 */
size_t vex_string_char_count(vex_string_t *str)
{
  if (!str || str->len == 0)
    return 0;
  return vex_utf8_char_count(str->data);
}

/**
 * Check if string is empty
 */
bool vex_string_is_empty(vex_string_t *str)
{
  return !str || str->len == 0;
}

/**
 * Get string as C string (null-terminated)
 */
const char *vex_string_as_cstr(vex_string_t *str)
{
  return str ? str->data : "";
}

/**
 * Clear string contents
 */
void vex_string_clear(vex_string_t *str)
{
  if (!str)
    return;
  str->len = 0;
  if (str->data)
    str->data[0] = '\0';
}

/**
 * Free string memory
 */
void vex_string_free(vex_string_t *str)
{
  if (!str)
    return;
  if (str->data)
    vex_free(str->data);
  vex_free(str);
}

/**
 * Clone a string
 */
vex_string_t *vex_string_clone(vex_string_t *str)
{
  if (!str)
    return NULL;
  return vex_string_from_cstr(str->data);
}

/**
 * Get a slice view of the string (byte range)
 */
void vex_string_slice(vex_string_t *str, size_t start, size_t end, VexSlice *out_slice)
{
  if (!str || !out_slice || start > end || end > str->len)
  {
    // Invalid range - return empty slice
    out_slice->data = NULL;
    out_slice->len = 0;
    out_slice->elem_size = 1;
    return;
  }

  out_slice->data = str->data + start;
  out_slice->len = end - start;
  out_slice->elem_size = 1; // Bytes
}
