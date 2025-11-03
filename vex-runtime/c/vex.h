/**
 * Vex Language Runtime Library
 * Zero-overhead builtin functions
 */

#ifndef VEX_RUNTIME_H
#define VEX_RUNTIME_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

// Include LLVM intrinsics
#include "vex_intrinsics.h"

#ifdef __cplusplus
extern "C"
{
#endif

  // ============================================================================
  // FORWARD DECLARATIONS
  // ============================================================================

  typedef void *VexArray; // Opaque array type

  // ============================================================================
  // STRING OPERATIONS
  // ============================================================================

  /**
   * Get the length of a null-terminated string
   * @param s Input string
   * @return Length of the string (excluding null terminator)
   */
  size_t vex_strlen(const char *s);

  /**
   * Compare two strings
   * @param s1 First string
   * @param s2 Second string
   * @return 0 if equal, <0 if s1 < s2, >0 if s1 > s2
   */
  int vex_strcmp(const char *s1, const char *s2);

  /**
   * Copy string from src to dest
   * @param dest Destination buffer
   * @param src Source string
   * @return Pointer to dest
   */
  char *vex_strcpy(char *dest, const char *src);

  /**
   * Concatenate two strings
   * @param dest Destination buffer
   * @param src Source string to append
   * @return Pointer to dest
   */
  char *vex_strcat(char *dest, const char *src);

  /**
   * Duplicate a string (allocates memory)
   * @param s String to duplicate
   * @return Pointer to new string (must be freed)
   */
  char *vex_strdup(const char *s);

  // ============================================================================
  // UTF-8 OPERATIONS
  // ============================================================================

  /**
   * Validate UTF-8 string
   * @param s String to validate
   * @param byte_len Length in bytes
   * @return true if valid UTF-8, false otherwise
   */
  bool vex_utf8_valid(const char *s, size_t byte_len);

  /**
   * Count UTF-8 characters (not bytes) in a string
   * Panics on invalid UTF-8
   * @param s UTF-8 string
   * @return Number of characters
   */
  size_t vex_utf8_char_count(const char *s);

  /**
   * Get pointer to the Nth UTF-8 character (0-indexed)
   * Panics if index out of bounds or invalid UTF-8
   * @param s UTF-8 string
   * @param char_index Character index (not byte index)
   * @return Pointer to character (not null-terminated!)
   */
  const char *vex_utf8_char_at(const char *s, size_t char_index);

  /**
   * Convert UTF-8 character index to byte index
   * Panics if invalid UTF-8 or index out of bounds
   * @param s UTF-8 string
   * @param char_index Character index
   * @return Byte index
   */
  size_t vex_utf8_char_to_byte_index(const char *s, size_t char_index);

  /**
   * Extract a single UTF-8 character at index as new string
   * Allocates memory (must be freed)
   * Panics on invalid UTF-8 or index out of bounds
   * @param s UTF-8 string
   * @param char_index Character index
   * @return New string containing single character
   */
  char *vex_utf8_char_extract(const char *s, size_t char_index);

  /**
   * Decode UTF-8 character to Unicode code point
   * @param s Pointer to UTF-8 character
   * @return Unicode code point (0-0x10FFFF) or 0xFFFFFFFF on error
   */
  uint32_t vex_utf8_decode(const char *s);

  /**
   * Encode Unicode code point to UTF-8
   * @param code_point Unicode code point (0-0x10FFFF)
   * @param buf Buffer to write to (must have at least 5 bytes)
   * @return Number of bytes written, or 0 on error
   */
  size_t vex_utf8_encode(uint32_t code_point, char *buf);

  // ============================================================================
  // MEMORY OPERATIONS
  // ============================================================================

  /**
   * Copy memory from src to dest
   * @param dest Destination buffer
   * @param src Source buffer
   * @param n Number of bytes to copy
   * @return Pointer to dest
   */
  void *vex_memcpy(void *dest, const void *src, size_t n);

  /**
   * Move memory (handles overlapping regions)
   * @param dest Destination buffer
   * @param src Source buffer
   * @param n Number of bytes to move
   * @return Pointer to dest
   */
  void *vex_memmove(void *dest, const void *src, size_t n);

  /**
   * Set memory to a specific value
   * @param s Buffer to set
   * @param c Value to set (converted to unsigned char)
   * @param n Number of bytes to set
   * @return Pointer to s
   */
  void *vex_memset(void *s, int c, size_t n);

  /**
   * Compare two memory regions
   * @param s1 First buffer
   * @param s2 Second buffer
   * @param n Number of bytes to compare
   * @return 0 if equal, <0 if s1 < s2, >0 if s1 > s2
   */
  int vex_memcmp(const void *s1, const void *s2, size_t n);

  // ============================================================================
  // MEMORY ALLOCATION (using musl malloc via static linking)
  // ============================================================================

  void *vex_malloc(size_t size);
  void *vex_calloc(size_t nmemb, size_t size);
  void *vex_realloc(void *ptr, size_t size);
  void vex_free(void *ptr);

  // ============================================================================
  // I/O OPERATIONS - Multi-Style Support
  // ============================================================================

  // --- Style 1: C-style (Legacy, fastest) ---

  /**
   * Print string to stdout
   * @param s String to print
   */
  void vex_print(const char *s);

  /**
   * Print string to stdout with newline
   * @param s String to print
   */
  void vex_println(const char *s);

  /**
   * Print string to stderr
   * @param s String to print
   */
  void vex_eprint(const char *s);

  /**
   * Print string to stderr with newline
   * @param s String to print
   */
  void vex_eprintln(const char *s);

  /**
   * Formatted print to stdout (supports %d, %s, %f, %ld, %lf)
   * @param fmt Format string
   * @return Number of characters printed
   */
  int vex_printf(const char *fmt, ...);

  /**
   * Formatted print to buffer (supports %d, %s, %f, %ld, %lf)
   * @param buf Output buffer
   * @param fmt Format string
   * @return Number of characters written
   */
  int vex_sprintf(char *buf, const char *fmt, ...);

  // --- Style 2: Go-style (Variadic, convenient) ---

  // Value type enum for runtime polymorphism
  typedef enum
  {
    VEX_VALUE_I32,
    VEX_VALUE_I64,
    VEX_VALUE_F32,
    VEX_VALUE_F64,
    VEX_VALUE_BOOL,
    VEX_VALUE_STRING,
    VEX_VALUE_PTR,
  } VexValueType;

  // Universal value container
  typedef struct
  {
    VexValueType type;
    union
    {
      int32_t as_i32;
      int64_t as_i64;
      float as_f32;
      double as_f64;
      bool as_bool;
      const char *as_string;
      void *as_ptr;
    };
  } VexValue;

  /**
   * Print multiple values to stdout (space-separated)
   * Go-style: print("Hello", name, "age:", 42)
   * @param count Number of arguments
   * @param args Array of VexValue arguments
   */
  void vex_print_args(int count, VexValue *args);

  /**
   * Print multiple values to stdout with newline (space-separated)
   * Go-style: println("Hello", name, "age:", 42)
   * @param count Number of arguments
   * @param args Array of VexValue arguments
   */
  void vex_println_args(int count, VexValue *args);

  // --- Style 3: Rust-style (Format strings, type-safe) ---

  /**
   * Rust-style formatted print: println!("Hello {}, age: {}", name, 42)
   * Format specifiers:
   *   {}     - Default format
   *   {:?}   - Debug format
   *   {:.2}  - Precision (floats)
   *   {:x}   - Hexadecimal
   * @param fmt Format string with {} placeholders
   * @param count Number of arguments
   * @param args Array of VexValue arguments
   */
  void vex_println_fmt(const char *fmt, int count, VexValue *args);

  /**
   * Rust-style formatted print without newline
   * @param fmt Format string with {} placeholders
   * @param count Number of arguments
   * @param args Array of VexValue arguments
   */
  void vex_print_fmt(const char *fmt, int count, VexValue *args);

  // --- Helper Functions ---

  /**
   * Print a single VexValue (used internally)
   * @param val Value to print
   */
  void vex_print_value(const VexValue *val);

  /**
   * Create VexValue from i32
   */
  static inline VexValue vex_value_i32(int32_t val)
  {
    VexValue v = {.type = VEX_VALUE_I32, .as_i32 = val};
    return v;
  }

  /**
   * Create VexValue from i64
   */
  static inline VexValue vex_value_i64(int64_t val)
  {
    VexValue v = {.type = VEX_VALUE_I64, .as_i64 = val};
    return v;
  }

  /**
   * Create VexValue from f64
   */
  static inline VexValue vex_value_f64(double val)
  {
    VexValue v = {.type = VEX_VALUE_F64, .as_f64 = val};
    return v;
  }

  /**
   * Create VexValue from string
   */
  static inline VexValue vex_value_string(const char *val)
  {
    VexValue v = {.type = VEX_VALUE_STRING, .as_string = val};
    return v;
  }

  /**
   * Create VexValue from bool
   */
  static inline VexValue vex_value_bool(bool val)
  {
    VexValue v = {.type = VEX_VALUE_BOOL, .as_bool = val};
    return v;
  }

  // ============================================================================
  // ARRAY OPERATIONS
  // ============================================================================

  /**
   * Get array length (from metadata)
   * Panics if array is NULL
   * @param arr Array pointer
   * @return Length of array
   */
  int64_t vex_array_len(void *arr);

  /**
   * Get array capacity (internal use)
   * Panics if array is NULL
   * @param arr Array pointer
   * @return Capacity of array
   */
  int64_t vex_array_capacity(void *arr);

  /**
   * Bounds-checked array access
   * Returns pointer to element at index
   * Panics if index out of bounds
   * @param arr Array pointer
   * @param index Element index
   * @param elem_size Size of each element
   * @return Pointer to element
   */
  void *vex_array_get(void *arr, int64_t index, size_t elem_size);

  /**
   * Bounds-checked array set
   * Copies element to index
   * Panics if index out of bounds
   * @param arr Array pointer
   * @param index Element index
   * @param elem Element to set
   * @param elem_size Size of element
   */
  void vex_array_set(void *arr, int64_t index, void *elem, size_t elem_size);

  /**
   * Create array slice
   * Panics on invalid range or out of memory
   * @param arr Source array
   * @param start Start index
   * @param end End index (exclusive)
   * @param elem_size Size of each element
   * @return New array slice (must be freed)
   */
  void *vex_array_slice(void *arr, int64_t start, int64_t end, size_t elem_size);

  /**
   * Append element to array (reallocates if needed)
   * Panics on overflow or out of memory
   * @param arr Source array (can be NULL for new array)
   * @param elem Element to append
   * @param elem_size Size of element
   * @return New array pointer (old pointer invalidated if reallocated)
   */
  void *vex_array_append(void *arr, void *elem, size_t elem_size);

  // ============================================================================
  // ERROR HANDLING
  // ============================================================================

  /**
   * Panic with error message and exit
   * @param msg Error message
   */
  void vex_panic(const char *msg) __attribute__((noreturn));

  /**
   * Assert condition, panic if false
   * @param cond Condition to check
   * @param msg Error message
   */
  void vex_assert(bool cond, const char *msg);

  // ============================================================================
  // FILE I/O
  // ============================================================================

  /**
   * File handle structure
   */
  typedef struct
  {
    int fd;           // File descriptor
    const char *path; // File path (owned)
    bool is_open;     // Is file currently open
  } VexFile;

  /**
   * Open a file
   * @param path File path
   * @param mode Open mode: "r", "w", "a", "r+", "w+", "a+"
   * @return File handle or NULL on error
   */
  VexFile *vex_file_open(const char *path, const char *mode);

  /**
   * Close a file
   * @param file File handle to close
   */
  void vex_file_close(VexFile *file);

  /**
   * Read from file
   * @param file File handle
   * @param buffer Buffer to read into
   * @param size Number of bytes to read
   * @return Number of bytes actually read
   */
  size_t vex_file_read(VexFile *file, void *buffer, size_t size);

  /**
   * Write to file
   * @param file File handle
   * @param buffer Buffer to write from
   * @param size Number of bytes to write
   * @return Number of bytes actually written
   */
  size_t vex_file_write(VexFile *file, const void *buffer, size_t size);

  /**
   * Seek to position in file
   * @param file File handle
   * @param offset Offset from whence
   * @param whence 0=SEEK_SET, 1=SEEK_CUR, 2=SEEK_END
   * @return true on success
   */
  bool vex_file_seek(VexFile *file, int64_t offset, int whence);

  /**
   * Get current position in file
   * @param file File handle
   * @return Current position or -1 on error
   */
  int64_t vex_file_tell(VexFile *file);

  /**
   * Get file size
   * @param file File handle
   * @return File size in bytes or -1 on error
   */
  int64_t vex_file_size(VexFile *file);

  /**
   * Flush file buffers
   * @param file File handle
   * @return true on success
   */
  bool vex_file_flush(VexFile *file);

  /**
   * Read entire file into memory
   * @param path File path
   * @param out_size Optional: pointer to store file size
   * @return File contents (null-terminated) or NULL on error
   */
  char *vex_file_read_all(const char *path, size_t *out_size);

  /**
   * Write data to file
   * @param path File path
   * @param data Data to write
   * @param size Number of bytes to write
   * @return true on success
   */
  bool vex_file_write_all(const char *path, const void *data, size_t size);

  /**
   * Check if file exists
   * @param path File path
   * @return true if file exists
   */
  bool vex_file_exists(const char *path);

  /**
   * Remove/delete file
   * @param path File path
   * @return true on success
   */
  bool vex_file_remove(const char *path);

  /**
   * Rename/move file
   * @param old_path Current path
   * @param new_path New path
   * @return true on success
   */
  bool vex_file_rename(const char *old_path, const char *new_path);

  /**
   * Create directory
   * @param path Directory path
   * @return true on success
   */
  bool vex_dir_create(const char *path);

  /**
   * Remove directory
   * @param path Directory path
   * @return true on success
   */
  bool vex_dir_remove(const char *path);

  /**
   * Check if directory exists
   * @param path Directory path
   * @return true if directory exists
   */
  bool vex_dir_exists(const char *path);

  // ============================================================================
  // MEMORY MAPPED FILES
  // ============================================================================

  /**
   * Memory-mapped file structure
   */
  typedef struct
  {
    void *addr;    // Mapped address
    size_t size;   // Mapped size
    bool writable; // Is mapping writable
  } VexMmap;

  /**
   * Memory map a file
   * @param path File path
   * @param writable True for writable mapping
   * @return Memory mapping or NULL on error
   */
  VexMmap *vex_mmap_open(const char *path, bool writable);

  /**
   * Unmap and close
   * @param mapping Memory mapping to close
   */
  void vex_mmap_close(VexMmap *mapping);

  /**
   * Sync mapped memory to disk
   * @param mapping Memory mapping
   * @return true on success
   */
  bool vex_mmap_sync(VexMmap *mapping);

  /**
   * Give advice about memory access pattern
   * @param mapping Memory mapping
   * @param advice 0=NORMAL, 1=SEQUENTIAL, 2=RANDOM, 3=WILLNEED, 4=DONTNEED
   * @return true on success
   */
  bool vex_mmap_advise(VexMmap *mapping, int advice);

  /**
   * Allocate anonymous memory mapping
   * @param size Size in bytes
   * @return Allocated memory or NULL on error
   */
  void *vex_mmap_alloc(size_t size);

  /**
   * Free anonymous memory mapping
   * @param addr Address to free
   * @param size Size in bytes
   */
  void vex_mmap_free(void *addr, size_t size);

  /**
   * Change memory protection
   * @param addr Memory address
   * @param size Memory size
   * @param prot Protection flags (0=NONE, 1=READ, 2=WRITE, 4=EXEC)
   * @return true on success
   */
  bool vex_mmap_protect(void *addr, size_t size, int prot);

  // ============================================================================
  // TIME AND DATE
  // ============================================================================

  /**
   * DateTime structure
   */
  typedef struct
  {
    int year;        // Year (e.g., 2025)
    int month;       // Month (1-12)
    int day;         // Day (1-31)
    int hour;        // Hour (0-23)
    int minute;      // Minute (0-59)
    int second;      // Second (0-59)
    int millisecond; // Millisecond (0-999)
    int weekday;     // Day of week (0=Sunday, 6=Saturday)
    int yearday;     // Day of year (1-366)
  } VexDateTime;

  /**
   * High-resolution timer
   */
  typedef struct
  {
    int64_t start_ns; // Start time in nanoseconds
    bool is_running;  // Is timer running
  } VexTimer;

  /**
   * Get current Unix timestamp in milliseconds
   * @return Timestamp in milliseconds since epoch
   */
  int64_t vex_time_now();

  /**
   * Get current Unix timestamp in microseconds
   * @return Timestamp in microseconds since epoch
   */
  int64_t vex_time_now_micros();

  /**
   * Get current Unix timestamp in nanoseconds
   * @return Timestamp in nanoseconds since epoch
   */
  int64_t vex_time_now_nanos();

  /**
   * Get monotonic time (for measuring durations)
   * @return Monotonic time in nanoseconds
   */
  int64_t vex_time_monotonic();

  /**
   * Sleep for specified milliseconds
   * @param millis Milliseconds to sleep
   */
  void vex_time_sleep(int64_t millis);

  /**
   * Sleep for specified microseconds
   * @param micros Microseconds to sleep
   */
  void vex_time_sleep_micros(int64_t micros);

  /**
   * Convert timestamp to UTC datetime
   * @param timestamp_millis Timestamp in milliseconds
   * @return DateTime structure (caller must free)
   */
  VexDateTime *vex_time_to_datetime(int64_t timestamp_millis);

  /**
   * Convert timestamp to local datetime
   * @param timestamp_millis Timestamp in milliseconds
   * @return DateTime structure (caller must free)
   */
  VexDateTime *vex_time_to_local_datetime(int64_t timestamp_millis);

  /**
   * Convert datetime to timestamp
   * @param dt DateTime structure
   * @return Timestamp in milliseconds
   */
  int64_t vex_datetime_to_timestamp(const VexDateTime *dt);

  /**
   * Format datetime to string
   * @param dt DateTime structure
   * @param format strftime format string
   * @return Formatted string (caller must free)
   */
  char *vex_time_format(const VexDateTime *dt, const char *format);

  /**
   * Free datetime structure
   * @param dt DateTime to free
   */
  void vex_datetime_free(VexDateTime *dt);

  /**
   * Start high-resolution timer
   * @return Timer handle
   */
  VexTimer *vex_timer_start();

  /**
   * Get elapsed nanoseconds
   * @param timer Timer handle
   * @return Elapsed nanoseconds
   */
  int64_t vex_timer_elapsed_nanos(const VexTimer *timer);

  /**
   * Get elapsed microseconds
   * @param timer Timer handle
   * @return Elapsed microseconds
   */
  int64_t vex_timer_elapsed_micros(const VexTimer *timer);

  /**
   * Get elapsed milliseconds
   * @param timer Timer handle
   * @return Elapsed milliseconds
   */
  int64_t vex_timer_elapsed_millis(const VexTimer *timer);

  /**
   * Get elapsed seconds
   * @param timer Timer handle
   * @return Elapsed seconds
   */
  double vex_timer_elapsed_seconds(const VexTimer *timer);

  /**
   * Reset timer
   * @param timer Timer handle
   */
  void vex_timer_reset(VexTimer *timer);

  /**
   * Free timer
   * @param timer Timer to free
   */
  void vex_timer_free(VexTimer *timer);

  // ============================================================================
  // HASH MAP (SwissTable)
  // ============================================================================

  /**
   * SwissTable hash map structure
   * Features:
   * - SIMD group scanning (AVX2/SSE2/NEON)
   * - Open addressing with control bytes
   * - 1.4-1.8x faster than std::unordered_map
   */
  typedef struct
  {
    uint8_t *ctrl;   // Control bytes (capacity + GROUP_PAD)
    void **entries;  // Entry slots (hash, key, value)
    size_t capacity; // Power-of-two
    size_t len;      // Number of live entries
  } VexMap;

  /**
   * Initialize a new hash map
   * @param map Map structure to initialize
   * @param initial_capacity Initial capacity (will be rounded to power of 2)
   * @return true on success, false on allocation failure
   */
  bool vex_map_new(VexMap *map, size_t initial_capacity);

  /**
   * Insert or update a key-value pair
   * @param map Map to insert into
   * @param key Key string (must remain valid)
   * @param value Value pointer
   * @return true on success, false on allocation failure
   */
  bool vex_map_insert(VexMap *map, const char *key, void *value);

  /**
   * Get value for a key
   * @param map Map to search
   * @param key Key string to find
   * @return Value pointer or NULL if not found
   */
  void *vex_map_get(const VexMap *map, const char *key);

  /**
   * Get number of entries in map
   * @param map Map to query
   * @return Number of entries
   */
  size_t vex_map_len(const VexMap *map);

  /**
   * Free map resources
   * @param map Map to free
   */
  void vex_map_free(VexMap *map);

  // ============================================================================
  // PATH OPERATIONS
  // ============================================================================

  /**
   * Join two paths
   * @param path1 First path component
   * @param path2 Second path component
   * @return Joined path (caller must free)
   */
  char *vex_path_join(const char *path1, const char *path2);

  /**
   * Get directory name from path
   * @param path File path
   * @return Directory name (caller must free)
   */
  char *vex_path_dirname(const char *path);

  /**
   * Get base name from path
   * @param path File path
   * @return Base name (caller must free)
   */
  char *vex_path_basename(const char *path);

  /**
   * Get file extension
   * @param path File path
   * @return Extension including dot (caller must free)
   */
  char *vex_path_extension(const char *path);

  /**
   * Get absolute path
   * @param path Relative or absolute path
   * @return Absolute path or NULL on error (caller must free)
   */
  char *vex_path_absolute(const char *path);

  /**
   * Check if path is absolute
   * @param path Path to check
   * @return true if absolute
   */
  bool vex_path_is_absolute(const char *path);

  /**
   * Check if path is a directory
   * @param path Path to check
   * @return true if directory
   */
  bool vex_path_is_dir(const char *path);

  /**
   * Check if path is a file
   * @param path Path to check
   * @return true if file
   */
  bool vex_path_is_file(const char *path);

  /**
   * Glob pattern matching
   * @param pattern Glob pattern (*, ?, [...])
   * @return Array of matching paths
   */
  VexArray *vex_path_glob(const char *pattern);

  /**
   * Recursive glob
   * @param dir_path Directory to search
   * @param pattern File pattern
   * @return Array of matching paths
   */
  VexArray *vex_path_glob_recursive(const char *dir_path, const char *pattern);

  /**
   * List directory contents
   * @param dir_path Directory path
   * @return Array of VexDirEntry*
   */
  VexArray *vex_path_list_dir(const char *dir_path);

  /**
   * Copy file
   * @param src Source file
   * @param dst Destination file
   * @return true on success
   */
  bool vex_file_copy(const char *src, const char *dst);

  /**
   * Move file
   * @param src Source file
   * @param dst Destination file
   * @return true on success
   */
  bool vex_file_move(const char *src, const char *dst);

  /**
   * Create temporary file
   * @param prefix Optional prefix
   * @return Temp file path (caller must free)
   */
  char *vex_path_temp_file(const char *prefix);

  /**
   * Create temporary directory
   * @param prefix Optional prefix
   * @return Temp dir path (caller must free)
   */
  char *vex_path_temp_dir(const char *prefix);

  // ============================================================================
  // STRING CONVERSION (SIMD-accelerated)
  // ============================================================================

  /**
   * Parse string to int64
   * @param str String to parse
   * @param out Output pointer
   * @return true on success
   */
  bool vex_parse_i64(const char *str, int64_t *out);

  /**
   * Parse string to uint64
   * @param str String to parse
   * @param out Output pointer
   * @return true on success
   */
  bool vex_parse_u64(const char *str, uint64_t *out);

  /**
   * Parse string to double
   * @param str String to parse
   * @param out Output pointer
   * @return true on success
   */
  bool vex_parse_f64(const char *str, double *out);

  /**
   * Convert string to int64 (returns 0 on error)
   * @param str String to convert
   * @return Parsed value or 0
   */
  int64_t vex_str_to_i64(const char *str);

  /**
   * Convert string to uint64 (returns 0 on error)
   * @param str String to convert
   * @return Parsed value or 0
   */
  uint64_t vex_str_to_u64(const char *str);

  /**
   * Convert string to double (returns 0.0 on error)
   * @param str String to convert
   * @return Parsed value or 0.0
   */
  double vex_str_to_f64(const char *str);

  /**
   * Convert int64 to string
   * @param value Value to convert
   * @return String representation (caller must free)
   */
  char *vex_i64_to_str(int64_t value);

  /**
   * Convert uint64 to string
   * @param value Value to convert
   * @return String representation (caller must free)
   */
  char *vex_u64_to_str(uint64_t value);

  /**
   * Convert double to string
   * @param value Value to convert
   * @return String representation (caller must free)
   */
  char *vex_f64_to_str(double value);

  /**
   * Convert int64 to string with base
   * @param value Value to convert
   * @param base Base (2-36)
   * @return String representation (caller must free)
   */
  char *vex_i64_to_str_base(int64_t value, unsigned base);

  // ============================================================================
  // URL ENCODING/DECODING (SIMD-accelerated)
  // ============================================================================

  /**
   * URL encode string
   * @param str String to encode
   * @return URL-encoded string (caller must free)
   */
  char *vex_url_encode(const char *str);

  /**
   * URL decode string
   * @param str String to decode
   * @return Decoded string (caller must free)
   */
  char *vex_url_decode(const char *str);

  /**
   * URL structure
   */
  typedef struct
  {
    char *scheme;
    char *host;
    int port;
    char *path;
    char *query;
    char *fragment;
  } VexUrl;

  /**
   * Parse URL
   * @param url_str URL string
   * @return Parsed URL structure (caller must free)
   */
  VexUrl *vex_url_parse(const char *url_str);

  /**
   * Free URL structure
   * @param url URL to free
   */
  void vex_url_free(VexUrl *url);

  /**
   * Parse query string into map
   * @param query Query string
   * @return Map of parameters
   */
  VexMap *vex_url_parse_query(const char *query);

  // ============================================================================
  // CPU FEATURE DETECTION
  // ============================================================================

  /**
   * CPU features structure
   */
  typedef struct
  {
    bool sse;
    bool sse2;
    bool sse3;
    bool ssse3;
    bool sse4_1;
    bool sse4_2;
    bool avx;
    bool avx2;
    bool avx512f;
    bool fma;
    bool neon;
    bool sve;
    const char *vendor;
  } VexCpuFeatures;

  /**
   * SIMD instruction set level
   */
  typedef enum
  {
    VEX_SIMD_NONE = 0,
    VEX_SIMD_SSE2,
    VEX_SIMD_AVX,
    VEX_SIMD_AVX2,
    VEX_SIMD_AVX512,
    VEX_SIMD_NEON,
    VEX_SIMD_SVE
  } VexSimdLevel;

  /**
   * Detect CPU features
   * @return CPU features structure
   */
  const VexCpuFeatures *vex_cpu_detect();

  /**
   * Check if CPU has SSE2
   * @return true if supported
   */
  bool vex_cpu_has_sse2();

  /**
   * Check if CPU has AVX2
   * @return true if supported
   */
  bool vex_cpu_has_avx2();

  /**
   * Check if CPU has AVX-512
   * @return true if supported
   */
  bool vex_cpu_has_avx512();

  /**
   * Check if CPU has NEON
   * @return true if supported
   */
  bool vex_cpu_has_neon();

  /**
   * Get CPU vendor
   * @return Vendor string
   */
  const char *vex_cpu_vendor();

  /**
   * Get best SIMD instruction set available
   * @return SIMD level
   */
  VexSimdLevel vex_cpu_best_simd();

  /**
   * Get SIMD level name
   * @param level SIMD level
   * @return Name string
   */
  const char *vex_cpu_simd_name(VexSimdLevel level);

  /**
   * Get runtime compiler info
   * @return Compiler string
   */
  const char *vex_runtime_compiler();

  /**
   * Get runtime architecture
   * @return Architecture string
   */
  const char *vex_runtime_arch();

  /**
   * Get runtime build flags
   * @return Build flags string
   */
  const char *vex_runtime_build_flags();

  // ============================================================================
  // TYPE OPERATIONS (implemented in LLVM IR)
  // ============================================================================

  /**
   * Get size of type (compile-time constant)
   * @param type_id Type identifier
   * @return Size in bytes
   */
  size_t vex_sizeof(int type_id);

  /**
   * Get type name (RTTI)
   * @param type_id Type identifier
   * @return Type name string
   */
  const char *vex_typeof(int type_id);

#ifdef __cplusplus
}
#endif

#endif // VEX_RUNTIME_H
