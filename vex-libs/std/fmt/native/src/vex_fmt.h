// vex_fmt.h - Header for Vex Fmt native module
// Comprehensive formatting library (Go/Rust style)
#ifndef VEX_FMT_H
#define VEX_FMT_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

#ifdef __cplusplus
extern "C"
{
#endif

    // ============================================================================
    // CORE FORMATTING TYPES
    // ============================================================================

    typedef enum
    {
        VEX_FMT_ALIGN_LEFT = 0,
        VEX_FMT_ALIGN_RIGHT = 1,
        VEX_FMT_ALIGN_CENTER = 2,
    } vex_fmt_align_t;

    typedef enum
    {
        VEX_FMT_SIGN_MINUS = 0, // Only negative numbers
        VEX_FMT_SIGN_PLUS = 1,  // Always show sign
        VEX_FMT_SIGN_SPACE = 2, // Space for positive, - for negative
    } vex_fmt_sign_t;

    typedef enum
    {
        VEX_FMT_BASE_BINARY = 2,
        VEX_FMT_BASE_OCTAL = 8,
        VEX_FMT_BASE_DECIMAL = 10,
        VEX_FMT_BASE_HEX_LOWER = 16,
        VEX_FMT_BASE_HEX_UPPER = 17,
    } vex_fmt_base_t;

    typedef struct
    {
        vex_fmt_align_t align;
        char fill_char;
        int width;
        int precision;
        vex_fmt_sign_t sign;
        bool alternate; // 0x prefix for hex, 0b for binary
        bool zero_pad;  // Zero padding for numbers
        vex_fmt_base_t base;
        bool uppercase; // For hex, scientific notation
    } vex_fmt_spec_t;

    typedef struct
    {
        char *data;
        size_t len;
        size_t capacity;
    } vex_fmt_buffer_t;

    // ============================================================================
    // BUFFER MANAGEMENT
    // ============================================================================

    vex_fmt_buffer_t *vex_fmt_buffer_new(size_t initial_capacity);
    void vex_fmt_buffer_free(vex_fmt_buffer_t *buf);
    void vex_fmt_buffer_clear(vex_fmt_buffer_t *buf);
    void vex_fmt_buffer_append_str(vex_fmt_buffer_t *buf, const char *str, size_t len);
    void vex_fmt_buffer_append_char(vex_fmt_buffer_t *buf, char c);
    void vex_fmt_buffer_reserve(vex_fmt_buffer_t *buf, size_t additional);
    char *vex_fmt_buffer_to_string(vex_fmt_buffer_t *buf);

    // ============================================================================
    // FORMAT SPEC PARSING
    // ============================================================================

    vex_fmt_spec_t vex_fmt_spec_default(void);
    bool vex_fmt_spec_parse(const char *spec_str, size_t len, vex_fmt_spec_t *spec);

    // ============================================================================
    // CORE FORMATTING FUNCTIONS
    // ============================================================================

    // Integer formatting
    char *vex_fmt_i32(int32_t value, const vex_fmt_spec_t *spec);
    char *vex_fmt_i64(int64_t value, const vex_fmt_spec_t *spec);
    char *vex_fmt_u32(uint32_t value, const vex_fmt_spec_t *spec);
    char *vex_fmt_u64(uint64_t value, const vex_fmt_spec_t *spec);

    // Float formatting
    char *vex_fmt_f32(float value, const vex_fmt_spec_t *spec);
    char *vex_fmt_f64(double value, const vex_fmt_spec_t *spec);

    // String formatting
    char *vex_fmt_string(const char *str, size_t len, const vex_fmt_spec_t *spec);

    // Boolean formatting
    char *vex_fmt_bool(bool value, const vex_fmt_spec_t *spec);

    // Pointer formatting
    char *vex_fmt_pointer(const void *ptr, const vex_fmt_spec_t *spec);

    // ============================================================================
    // HIGH-LEVEL FORMATTING FUNCTIONS
    // ============================================================================

    // sprintf-style formatting
    char *vex_fmt_sprintf(const char *format, ...);
    char *vex_fmt_vsprintf(const char *format, va_list args);

    // Print to stdout/stderr
    void vex_fmt_print(const char *str);
    void vex_fmt_println(const char *str);
    void vex_fmt_eprint(const char *str);
    void vex_fmt_eprintln(const char *str);

    // Formatted print
    void vex_fmt_printf(const char *format, ...);
    void vex_fmt_eprintf(const char *format, ...);

    // ============================================================================
    // UTILITY FUNCTIONS
    // ============================================================================

    // Number to string conversions (base 2-36)
    char *vex_fmt_itoa(int64_t value, int base, bool uppercase);
    char *vex_fmt_utoa(uint64_t value, int base, bool uppercase);
    char *vex_fmt_ftoa(double value, int precision);

    // Padding helpers
    char *vex_fmt_pad_left(const char *str, size_t len, char fill, int width);
    char *vex_fmt_pad_right(const char *str, size_t len, char fill, int width);
    char *vex_fmt_pad_center(const char *str, size_t len, char fill, int width);

    // String escaping
    char *vex_fmt_escape_string(const char *str, size_t len);
    char *vex_fmt_debug_string(const char *str, size_t len);

    // Memory allocation wrapper
    void *vex_fmt_malloc(size_t size);
    void vex_fmt_free(void *ptr);

#ifdef __cplusplus
}
#endif

#endif // VEX_FMT_H
