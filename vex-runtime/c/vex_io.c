/**
 * Vex I/O Operations
 * Multi-style I/O: C-style, Go-style, Rust-style
 */

#include "vex.h"
#include <unistd.h>
#include <stdarg.h>
#include <stdio.h>
#include <ctype.h>

// ============================================================================
// STYLE 1: C-STYLE I/O (Legacy, Fastest)
// ============================================================================

void vex_print(const char *s)
{
    fputs(s, stdout); // Use buffered I/O for consistency with vex_printf
}

void vex_println(const char *s)
{
    fputs(s, stdout);
    fputc('\n', stdout);
    fflush(stdout); // Ensure newline is printed immediately
}

void vex_eprint(const char *s)
{
    size_t len = vex_strlen(s);
    write(STDERR_FILENO, s, len);
}

void vex_eprintln(const char *s)
{
    vex_eprint(s);
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
    case VEX_VALUE_I32:
        vex_printf("%d", val->as_i32);
        break;
    case VEX_VALUE_I64:
        vex_printf("%lld", (long long)val->as_i64);
        break;
    case VEX_VALUE_F32:
        vex_printf("%g", val->as_f32);
        break;
    case VEX_VALUE_F64:
        vex_printf("%g", val->as_f64);
        break;
    case VEX_VALUE_BOOL:
        vex_print(val->as_bool ? "true" : "false");
        break;
    case VEX_VALUE_STRING:
        vex_print(val->as_string);
        break;
    case VEX_VALUE_PTR:
        vex_printf("%p", val->as_ptr);
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
            vex_print(" ");
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
    vex_print("\n");
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
    case VEX_VALUE_I32:
        if (fmt_type == 'x')
        {
            vex_printf("%x", val->as_i32);
        }
        else if (fmt_type == '?')
        {
            vex_printf("i32(%d)", val->as_i32);
        }
        else
        {
            vex_printf("%d", val->as_i32);
        }
        break;

    case VEX_VALUE_I64:
        if (fmt_type == 'x')
        {
            vex_printf("%llx", (long long)val->as_i64);
        }
        else if (fmt_type == '?')
        {
            vex_printf("i64(%lld)", (long long)val->as_i64);
        }
        else
        {
            vex_printf("%lld", (long long)val->as_i64);
        }
        break;

    case VEX_VALUE_F32:
    case VEX_VALUE_F64:
    {
        double v = (val->type == VEX_VALUE_F32) ? val->as_f32 : val->as_f64;
        if (fmt_type == '?')
        {
            vex_printf("f64(%g)", v);
        }
        else if (precision >= 0)
        {
            vex_printf("%.*f", precision, v);
        }
        else
        {
            vex_printf("%g", v);
        }
        break;
    }

    case VEX_VALUE_BOOL:
        if (fmt_type == '?')
        {
            vex_printf("bool(%s)", val->as_bool ? "true" : "false");
        }
        else
        {
            vex_print(val->as_bool ? "true" : "false");
        }
        break;

    case VEX_VALUE_STRING:
        if (fmt_type == '?')
        {
            vex_printf("\"%s\"", val->as_string);
        }
        else
        {
            vex_print(val->as_string);
        }
        break;

    case VEX_VALUE_PTR:
        if (fmt_type == '?')
        {
            vex_printf("ptr(%p)", val->as_ptr);
        }
        else
        {
            vex_printf("%p", val->as_ptr);
        }
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
    vex_print("\n");
}
