/**
 * Optimized I/O Operations for Vex Language
 * 
 * Type-specific print functions for zero-overhead printing.
 * Replaces VexValue-based approach with direct function calls.
 * 
 * Performance: 2-3x faster than VexValue approach
 * Memory: Zero overhead (no struct allocation)
 */

#include <stdio.h>
#include <stdint.h>
#include <inttypes.h>
#include "vex.h"

// ============================================================================
// HELPER FUNCTIONS (Internal)
// ============================================================================

// Helper for 128-bit printing
static void print_u128_impl(unsigned __int128 n) {
    char buf[40];
    int i = 0;
    if (n == 0) {
        putchar('0');
        return;
    }
    while (n > 0) {
        buf[i++] = (n % 10) + '0';
        n /= 10;
    }
    for (int j = i - 1; j >= 0; j--) {
        putchar(buf[j]);
    }
}

// ============================================================================
// TYPE-SPECIFIC PRINT FUNCTIONS
// ============================================================================

// Integer types (signed)
void vex_print_i8(int8_t val) {
    printf("%d", (int)val);
}

void vex_print_i16(int16_t val) {
    printf("%d", (int)val);
}

void vex_print_i32(int32_t val) {
    printf("%" PRId32, val);
}

void vex_print_i64(int64_t val) {
    printf("%" PRId64, val);
}

void vex_print_i128(__int128 val) {
    if (val < 0) {
        putchar('-');
        print_u128_impl((unsigned __int128)-val);
    } else {
        print_u128_impl((unsigned __int128)val);
    }
}

// Integer types (unsigned)
void vex_print_u8(uint8_t val) {
    printf("%u", (unsigned int)val);
}

void vex_print_u16(uint16_t val) {
    printf("%u", (unsigned int)val);
}

void vex_print_u32(uint32_t val) {
    printf("%" PRIu32, val);
}

void vex_print_u64(uint64_t val) {
    printf("%" PRIu64, val);
}

void vex_print_u128(unsigned __int128 val) {
    print_u128_impl(val);
}

// Floating-point types
void vex_print_f32(float val) {
    // Use %g for automatic formatting (removes trailing zeros)
    printf("%g", val);
}

void vex_print_f64(double val) {
    printf("%g", val);
}

void vex_print_f16(uint16_t val) {
    // f16 is passed as u16 bits, convert to float for printing
    // TODO: Proper f16 → float conversion
    printf("f16(...)");
}

// Boolean type
void vex_print_bool(int val) {
    fputs(val ? "true" : "false", stdout);
}

// String type
void vex_print_string(const char* str) {
    if (str) {
        fputs(str, stdout);
    } else {
        fputs("(null)", stdout);
    }
}

// Pointer type
void vex_print_ptr(const void* ptr) {
    printf("%p", ptr);
}

// Nil/null type
void vex_print_nil(void) {
    fputs("nil", stdout);
}

// ============================================================================
// HELPER FUNCTIONS (Exported)
// ============================================================================

void vex_print_space(void) {
    fputc(' ', stdout);
}

void vex_print_newline(void) {
    fputc('\n', stdout);
    fflush(stdout);  // Flush to ensure immediate output
}

void vex_print_literal(const char* str) {
    if (str) {
        fputs(str, stdout);
    }
}

// ============================================================================
// FORMAT-SPECIFIC VARIANTS (for format strings)
// ============================================================================

// Hexadecimal formatting
void vex_print_i32_hex(int32_t val) {
    printf("%x", val);
}

void vex_print_i64_hex(int64_t val) {
    printf("%" PRIx64, val);
}

void vex_print_u32_hex(uint32_t val) {
    printf("%" PRIx32, val);
}

void vex_print_u64_hex(uint64_t val) {
    printf("%" PRIx64, val);
}

// Debug formatting (includes type name)
void vex_print_i32_debug(int32_t val) {
    printf("i32(%d)", val);
}

void vex_print_i64_debug(int64_t val) {
    printf("i64(%" PRId64 ")", val);
}

void vex_print_f64_debug(double val) {
    printf("f64(%g)", val);
}

void vex_print_bool_debug(int val) {
    printf("bool(%s)", val ? "true" : "false");
}

void vex_print_string_debug(const char* str) {
    if (str) {
        printf("\"%s\"", str);
    } else {
        fputs("\"(null)\"", stdout);
    }
}

// Precision formatting for floats
void vex_print_f32_precision(float val, int precision) {
    printf("%.*f", precision, val);
}

void vex_print_f64_precision(double val, int precision) {
    printf("%.*f", precision, val);
}

// Binary formatting
void vex_print_i32_bin(int32_t val) {
    printf("0b");
    for (int i = 31; i >= 0; i--) {
        printf("%d", (val >> i) & 1);
    }
}

void vex_print_u32_bin(uint32_t val) {
    printf("0b");
    for (int i = 31; i >= 0; i--) {
        printf("%d", (val >> i) & 1);
    }
}

// Octal formatting
void vex_print_i32_oct(int32_t val) {
    printf("0%o", val);
}

void vex_print_u32_oct(uint32_t val) {
    printf("0%" PRIo32, val);
}

// Scientific notation
void vex_print_f32_scientific(float val) {
    printf("%e", val);
}

void vex_print_f64_scientific(double val) {
    printf("%e", val);
}

// ============================================================================
// LEGACY / COMPATIBILITY FUNCTIONS (Restored from old vex_io.c)
// ============================================================================

#include <unistd.h>
#include <stdarg.h>
#include <ctype.h>

void vex_print(const char *ptr, uint64_t len)
{
    fwrite(ptr, 1, len, stdout);
}

void vex_println(const char *ptr, uint64_t len)
{
    fwrite(ptr, 1, len, stdout);
    fputc('\n', stdout);
    fflush(stdout); // Ensure newline is printed immediately
}

void vex_eprint(const char *ptr, uint64_t len)
{
    write(STDERR_FILENO, ptr, len);
}

void vex_eprintln(const char *ptr, uint64_t len)
{
    write(STDERR_FILENO, ptr, len);
    write(STDERR_FILENO, "\n", 1);
}

// Minimal printf implementation using libc printf
// (will be static linked with musl)
int vex_printf(const char *fmt, ...)
{
    va_list args;
    va_start(args, fmt);
    int result = vprintf(fmt, args);
    va_end(args);
    return result;
}

int vex_sprintf(char *buf, const char *fmt, ...)
{
    va_list args;
    va_start(args, fmt);
    int result = vsprintf(buf, fmt, args);
    va_end(args);
    return result;
}

// ============================================================================
// STYLE 2: GO-STYLE I/O (Variadic, Convenient)
// ============================================================================

/**
 * Print a single VexValue based on its type
 */
void vex_print_value(const VexValue *val)
{
    switch (val->type)
    {
    case VEX_VALUE_I8:
        vex_print_i8(val->as_i8);
        break;
    case VEX_VALUE_I16:
        vex_print_i16(val->as_i16);
        break;
    case VEX_VALUE_I32:
        vex_print_i32(val->as_i32);
        break;
    case VEX_VALUE_I64:
        vex_print_i64(val->as_i64);
        break;
    case VEX_VALUE_I128:
        vex_print_i128(val->as_i128);
        break;
    case VEX_VALUE_U8:
        vex_print_u8(val->as_u8);
        break;
    case VEX_VALUE_U16:
        vex_print_u16(val->as_u16);
        break;
    case VEX_VALUE_U32:
        vex_print_u32(val->as_u32);
        break;
    case VEX_VALUE_U64:
        vex_print_u64(val->as_u64);
        break;
    case VEX_VALUE_U128:
        vex_print_u128(val->as_u128);
        break;
    case VEX_VALUE_F16:
        vex_print_f16(val->as_f16);
        break;
    case VEX_VALUE_F32:
        vex_print_f32(val->as_f32);
        break;
    case VEX_VALUE_F64:
        vex_print_f64(val->as_f64);
        break;
    case VEX_VALUE_BOOL:
        vex_print_bool(val->as_bool);
        break;
    case VEX_VALUE_STRING:
        vex_print(val->as_string, vex_strlen(val->as_string));
        break;
    case VEX_VALUE_PTR:
        vex_print_ptr(val->as_ptr);
        break;
    case VEX_VALUE_ERROR:
        vex_printf("Error(%p)", val->as_error);
        break;
    case VEX_VALUE_NIL:
        vex_print("nil", 3);
        break;
    }
}

/**
 * Go-style print: print("Hello", name, "age:", 42)
 * Space-separated output, no newline
 */
void vex_print_args(int count, VexValue *args)
{
    for (int i = 0; i < count; i++)
    {
        vex_print_value(&args[i]);
        if (i < count - 1)
        {
            vex_print(" ", 1);
        }
    }
}

/**
 * Go-style println: println("Hello", name, "age:", 42)
 * Space-separated output with newline
 */
void vex_println_args(int count, VexValue *args)
{
    vex_print_args(count, args);
    vex_print("\n", 1);
}

// ============================================================================
// STYLE 3: RUST-STYLE I/O (Format Strings, Type-safe)
// ============================================================================

/**
 * Parse format specifier: {}, {:?}, {:.2}, {:x}
 * Returns number of characters consumed
 */
static int parse_format_spec(const char *spec, char *out_type, int *out_precision)
{
    const char *start = spec;
    *out_type = 0;
    *out_precision = -1;

    if (*spec != '{')
        return 0;
    spec++; // Skip '{'

    // Check for empty {}
    if (*spec == '}')
    {
        *out_type = 0; // Default format
        return 2;
    }

    // Check for :
    if (*spec == ':')
    {
        spec++;

        // Parse precision (.N)
        if (*spec == '.')
        {
            spec++;
            *out_precision = 0;
            while (isdigit(*spec))
            {
                *out_precision = (*out_precision * 10) + (*spec - '0');
                spec++;
            }
        }
        // Parse format type (?, x, etc.)
        else if (*spec != '}')
        {
            *out_type = *spec++;
        }
    }

    if (*spec != '}')
        return 0; // Invalid format
    spec++;

    return spec - start;
}

/**
 * Print value with format specifier
 */
static void vex_print_value_fmt(const VexValue *val, char fmt_type, int precision)
{
    switch (val->type)
    {
    case VEX_VALUE_I8:
    case VEX_VALUE_I16:
    case VEX_VALUE_I32:
    {
        int32_t v = (val->type == VEX_VALUE_I8) ? val->as_i8 : (val->type == VEX_VALUE_I16) ? val->as_i16
                                                                                            : val->as_i32;
        if (fmt_type == 'x') vex_print_i32_hex(v);
        else if (fmt_type == '?') vex_print_i32_debug(v);
        else if (fmt_type == 'b') vex_print_i32_bin(v);
        else if (fmt_type == 'o') vex_print_i32_oct(v);
        else vex_print_i32(v);
        break;
    }

    case VEX_VALUE_I64:
        if (fmt_type == 'x') vex_print_i64_hex(val->as_i64);
        else if (fmt_type == '?') vex_print_i64_debug(val->as_i64);
        else vex_print_i64(val->as_i64);
        break;

    case VEX_VALUE_U8:
    case VEX_VALUE_U16:
    case VEX_VALUE_U32:
    {
        uint32_t v = (val->type == VEX_VALUE_U8) ? val->as_u8 : (val->type == VEX_VALUE_U16) ? val->as_u16
                                                                                             : val->as_u32;
        if (fmt_type == 'x') vex_print_u32_hex(v);
        else if (fmt_type == 'b') vex_print_u32_bin(v);
        else if (fmt_type == 'o') vex_print_u32_oct(v);
        else vex_print_u32(v);
        break;
    }

    case VEX_VALUE_U64:
        if (fmt_type == 'x') vex_print_u64_hex(val->as_u64);
        else vex_print_u64(val->as_u64);
        break;

    case VEX_VALUE_F16:
    case VEX_VALUE_F32:
    case VEX_VALUE_F64:
    {
        double v = (val->type == VEX_VALUE_F16) ? (double)val->as_f16 : (val->type == VEX_VALUE_F32) ? val->as_f32
                                                                                                     : val->as_f64;
        if (fmt_type == '?') vex_print_f64_debug(v);
        else if (fmt_type == 'e') vex_print_f64_scientific(v);
        else if (precision >= 0) vex_print_f64_precision(v, precision);
        else vex_print_f64(v);
        break;
    }

    case VEX_VALUE_BOOL:
        if (fmt_type == '?') vex_print_bool_debug(val->as_bool);
        else vex_print_bool(val->as_bool);
        break;

    case VEX_VALUE_STRING:
        if (fmt_type == '?') vex_print_string_debug(val->as_string);
        else vex_print_string(val->as_string);
        break;

    case VEX_VALUE_PTR:
        vex_print_ptr(val->as_ptr);
        break;

    case VEX_VALUE_ERROR:
        vex_printf("Error(%p)", val->as_error);
        break;

    case VEX_VALUE_NIL:
        vex_print("nil", 3);
        break;

    case VEX_VALUE_I128:
    case VEX_VALUE_U128:
        vex_printf(val->type == VEX_VALUE_I128 ? "i128(...)" : "u128(...)");
        break;
    }
}

/**
 * Rust-style formatted print
 * Example: vex_print_fmt("Hello {}, age: {}", 2, args)
 */
void vex_print_fmt(const char *fmt, int count, VexValue *args)
{
    const char *p = fmt;
    int arg_index = 0;

    while (*p)
    {
        if (*p == '{')
        {
            // Parse format specifier
            char fmt_type;
            int precision;
            int consumed = parse_format_spec(p, &fmt_type, &precision);

            if (consumed > 0 && arg_index < count)
            {
                // Print formatted value
                vex_print_value_fmt(&args[arg_index], fmt_type, precision);
                arg_index++;
                p += consumed;
            }
            else
            {
                // Invalid format or no more args, print literal
                fputc('{', stdout);
                p++;
            }
        }
        else
        {
            // Print literal character (use putchar for buffered I/O consistency)
            putchar(*p);
            p++;
        }
    }
    fflush(stdout); // Ensure all output is flushed
}

/**
 * Rust-style formatted println
 */
void vex_println_fmt(const char *fmt, int count, VexValue *args)
{
    vex_print_fmt(fmt, count, args);
    vex_print("\n", 1);
}
