// vex_fmt.c - Comprehensive formatting library for Vex
// Inspired by Go's fmt and Rust's std::fmt

#include "vex_fmt.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdarg.h>
#include <math.h>
#include <ctype.h>

// ============================================================================
// MEMORY MANAGEMENT
// ============================================================================

void *vex_fmt_malloc(size_t size)
{
    void *ptr = malloc(size);
    if (!ptr && size > 0)
    {
        fprintf(stderr, "FATAL: vex_fmt out of memory (%zu bytes)\n", size);
        abort();
    }
    return ptr;
}

void vex_fmt_free(void *ptr)
{
    if (ptr)
    {
        free(ptr);
    }
}

// ============================================================================
// BUFFER MANAGEMENT
// ============================================================================

vex_fmt_buffer_t *vex_fmt_buffer_new(size_t initial_capacity)
{
    if (initial_capacity < 16)
    {
        initial_capacity = 16;
    }

    vex_fmt_buffer_t *buf = (vex_fmt_buffer_t *)vex_fmt_malloc(sizeof(vex_fmt_buffer_t));
    buf->data = (char *)vex_fmt_malloc(initial_capacity);
    buf->len = 0;
    buf->capacity = initial_capacity;
    buf->data[0] = '\0';

    return buf;
}

void vex_fmt_buffer_free(vex_fmt_buffer_t *buf)
{
    if (buf)
    {
        vex_fmt_free(buf->data);
        vex_fmt_free(buf);
    }
}

void vex_fmt_buffer_clear(vex_fmt_buffer_t *buf)
{
    if (buf)
    {
        buf->len = 0;
        if (buf->data)
        {
            buf->data[0] = '\0';
        }
    }
}

void vex_fmt_buffer_reserve(vex_fmt_buffer_t *buf, size_t additional)
{
    if (!buf)
        return;

    size_t required = buf->len + additional + 1;
    if (required > buf->capacity)
    {
        size_t new_capacity = buf->capacity * 2;
        while (new_capacity < required)
        {
            new_capacity *= 2;
        }

        char *new_data = (char *)vex_fmt_malloc(new_capacity);
        if (buf->data && buf->len > 0)
        {
            memcpy(new_data, buf->data, buf->len);
        }
        new_data[buf->len] = '\0';

        vex_fmt_free(buf->data);
        buf->data = new_data;
        buf->capacity = new_capacity;
    }
}

void vex_fmt_buffer_append_char(vex_fmt_buffer_t *buf, char c)
{
    if (!buf)
        return;

    vex_fmt_buffer_reserve(buf, 1);
    buf->data[buf->len++] = c;
    buf->data[buf->len] = '\0';
}

void vex_fmt_buffer_append_str(vex_fmt_buffer_t *buf, const char *str, size_t len)
{
    if (!buf || !str || len == 0)
        return;

    vex_fmt_buffer_reserve(buf, len);
    memcpy(buf->data + buf->len, str, len);
    buf->len += len;
    buf->data[buf->len] = '\0';
}

char *vex_fmt_buffer_to_string(vex_fmt_buffer_t *buf)
{
    if (!buf || !buf->data)
    {
        return NULL;
    }

    char *result = (char *)vex_fmt_malloc(buf->len + 1);
    memcpy(result, buf->data, buf->len);
    result[buf->len] = '\0';

    return result;
}

// ============================================================================
// FORMAT SPEC PARSING
// ============================================================================

vex_fmt_spec_t vex_fmt_spec_default(void)
{
    vex_fmt_spec_t spec;
    spec.align = VEX_FMT_ALIGN_LEFT;
    spec.fill_char = ' ';
    spec.width = 0;
    spec.precision = -1;
    spec.sign = VEX_FMT_SIGN_MINUS;
    spec.alternate = false;
    spec.zero_pad = false;
    spec.base = VEX_FMT_BASE_DECIMAL;
    spec.uppercase = false;
    return spec;
}

bool vex_fmt_spec_parse(const char *spec_str, size_t len, vex_fmt_spec_t *spec)
{
    if (!spec_str || !spec || len == 0)
    {
        return false;
    }

    *spec = vex_fmt_spec_default();

    const char *p = spec_str;
    const char *end = spec_str + len;

    // Parse fill and align
    if (p + 1 < end && (p[1] == '<' || p[1] == '>' || p[1] == '^'))
    {
        spec->fill_char = p[0];
        p++;
    }

    if (p < end)
    {
        if (*p == '<')
        {
            spec->align = VEX_FMT_ALIGN_LEFT;
            p++;
        }
        else if (*p == '>')
        {
            spec->align = VEX_FMT_ALIGN_RIGHT;
            p++;
        }
        else if (*p == '^')
        {
            spec->align = VEX_FMT_ALIGN_CENTER;
            p++;
        }
    }

    // Parse sign
    if (p < end)
    {
        if (*p == '+')
        {
            spec->sign = VEX_FMT_SIGN_PLUS;
            p++;
        }
        else if (*p == ' ')
        {
            spec->sign = VEX_FMT_SIGN_SPACE;
            p++;
        }
        else if (*p == '-')
        {
            spec->sign = VEX_FMT_SIGN_MINUS;
            p++;
        }
    }

    // Parse alternate form
    if (p < end && *p == '#')
    {
        spec->alternate = true;
        p++;
    }

    // Parse zero padding
    if (p < end && *p == '0')
    {
        spec->zero_pad = true;
        p++;
    }

    // Parse width
    if (p < end && isdigit(*p))
    {
        spec->width = 0;
        while (p < end && isdigit(*p))
        {
            spec->width = spec->width * 10 + (*p - '0');
            p++;
        }
    }

    // Parse precision
    if (p < end && *p == '.')
    {
        p++;
        spec->precision = 0;
        while (p < end && isdigit(*p))
        {
            spec->precision = spec->precision * 10 + (*p - '0');
            p++;
        }
    }

    // Parse type
    if (p < end)
    {
        switch (*p)
        {
        case 'b':
            spec->base = VEX_FMT_BASE_BINARY;
            break;
        case 'o':
            spec->base = VEX_FMT_BASE_OCTAL;
            break;
        case 'd':
            spec->base = VEX_FMT_BASE_DECIMAL;
            break;
        case 'x':
            spec->base = VEX_FMT_BASE_HEX_LOWER;
            break;
        case 'X':
            spec->base = VEX_FMT_BASE_HEX_UPPER;
            spec->uppercase = true;
            break;
        case 'e':
        case 'E':
        case 'f':
        case 'F':
        case 'g':
        case 'G':
            // Float formatting type
            spec->uppercase = isupper(*p);
            break;
        }
    }

    return true;
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

char *vex_fmt_utoa(uint64_t value, int base, bool uppercase)
{
    if (base < 2 || base > 36)
    {
        return NULL;
    }

    static const char digits_lower[] = "0123456789abcdefghijklmnopqrstuvwxyz";
    static const char digits_upper[] = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const char *digits = uppercase ? digits_upper : digits_lower;

    char temp[65];
    int i = 64;
    temp[i] = '\0';

    if (value == 0)
    {
        temp[--i] = '0';
    }
    else
    {
        while (value > 0 && i > 0)
        {
            temp[--i] = digits[value % base];
            value /= base;
        }
    }

    size_t len = 64 - i;
    char *result = (char *)vex_fmt_malloc(len + 1);
    memcpy(result, &temp[i], len + 1);

    return result;
}

char *vex_fmt_itoa(int64_t value, int base, bool uppercase)
{
    if (value >= 0)
    {
        return vex_fmt_utoa((uint64_t)value, base, uppercase);
    }

    char *num_str = vex_fmt_utoa((uint64_t)(-value), base, uppercase);
    size_t len = strlen(num_str);
    char *result = (char *)vex_fmt_malloc(len + 2);
    result[0] = '-';
    memcpy(result + 1, num_str, len + 1);
    vex_fmt_free(num_str);

    return result;
}

char *vex_fmt_ftoa(double value, int precision)
{
    if (precision < 0)
    {
        precision = 6;
    }

    char temp[128];
    snprintf(temp, sizeof(temp), "%.*f", precision, value);

    size_t len = strlen(temp);
    char *result = (char *)vex_fmt_malloc(len + 1);
    memcpy(result, temp, len + 1);

    return result;
}

// ============================================================================
// PADDING FUNCTIONS
// ============================================================================

char *vex_fmt_pad_left(const char *str, size_t len, char fill, int width)
{
    if (!str || (int)len >= width)
    {
        char *result = (char *)vex_fmt_malloc(len + 1);
        memcpy(result, str, len);
        result[len] = '\0';
        return result;
    }

    int pad_len = width - (int)len;
    char *result = (char *)vex_fmt_malloc(width + 1);

    for (int i = 0; i < pad_len; i++)
    {
        result[i] = fill;
    }
    memcpy(result + pad_len, str, len);
    result[width] = '\0';

    return result;
}

char *vex_fmt_pad_right(const char *str, size_t len, char fill, int width)
{
    if (!str || (int)len >= width)
    {
        char *result = (char *)vex_fmt_malloc(len + 1);
        memcpy(result, str, len);
        result[len] = '\0';
        return result;
    }

    int pad_len = width - (int)len;
    char *result = (char *)vex_fmt_malloc(width + 1);

    memcpy(result, str, len);
    for (int i = 0; i < pad_len; i++)
    {
        result[len + i] = fill;
    }
    result[width] = '\0';

    return result;
}

char *vex_fmt_pad_center(const char *str, size_t len, char fill, int width)
{
    if (!str || (int)len >= width)
    {
        char *result = (char *)vex_fmt_malloc(len + 1);
        memcpy(result, str, len);
        result[len] = '\0';
        return result;
    }

    int pad_len = width - (int)len;
    int left_pad = pad_len / 2;
    int right_pad = pad_len - left_pad;

    char *result = (char *)vex_fmt_malloc(width + 1);

    for (int i = 0; i < left_pad; i++)
    {
        result[i] = fill;
    }
    memcpy(result + left_pad, str, len);
    for (int i = 0; i < right_pad; i++)
    {
        result[left_pad + len + i] = fill;
    }
    result[width] = '\0';

    return result;
}

// ============================================================================
// CORE FORMATTING FUNCTIONS
// ============================================================================

char *vex_fmt_i64(int64_t value, const vex_fmt_spec_t *spec)
{
    vex_fmt_spec_t default_spec = vex_fmt_spec_default();
    if (!spec)
    {
        spec = &default_spec;
    }

    bool is_negative = value < 0;
    uint64_t abs_value = is_negative ? (uint64_t)(-value) : (uint64_t)value;

    int base = (spec->base == VEX_FMT_BASE_HEX_UPPER) ? 16 : (spec->base == VEX_FMT_BASE_HEX_LOWER) ? 16
                                                                                                    : spec->base;
    bool uppercase = (spec->base == VEX_FMT_BASE_HEX_UPPER) || spec->uppercase;

    char *num_str = vex_fmt_utoa(abs_value, base, uppercase);
    vex_fmt_buffer_t *buf = vex_fmt_buffer_new(64);

    // Add sign
    if (is_negative)
    {
        vex_fmt_buffer_append_char(buf, '-');
    }
    else if (spec->sign == VEX_FMT_SIGN_PLUS)
    {
        vex_fmt_buffer_append_char(buf, '+');
    }
    else if (spec->sign == VEX_FMT_SIGN_SPACE)
    {
        vex_fmt_buffer_append_char(buf, ' ');
    }

    // Add prefix for alternate form
    if (spec->alternate)
    {
        if (spec->base == VEX_FMT_BASE_BINARY)
        {
            vex_fmt_buffer_append_str(buf, "0b", 2);
        }
        else if (spec->base == VEX_FMT_BASE_OCTAL && abs_value != 0)
        {
            vex_fmt_buffer_append_char(buf, '0');
        }
        else if (spec->base == VEX_FMT_BASE_HEX_LOWER || spec->base == VEX_FMT_BASE_HEX_UPPER)
        {
            vex_fmt_buffer_append_str(buf, uppercase ? "0X" : "0x", 2);
        }
    }

    vex_fmt_buffer_append_str(buf, num_str, strlen(num_str));
    vex_fmt_free(num_str);

    char *formatted = vex_fmt_buffer_to_string(buf);
    vex_fmt_buffer_free(buf);

    // Apply width and alignment
    if (spec->width > 0)
    {
        char fill = spec->zero_pad ? '0' : spec->fill_char;
        char *padded = NULL;

        switch (spec->align)
        {
        case VEX_FMT_ALIGN_RIGHT:
            padded = vex_fmt_pad_left(formatted, strlen(formatted), fill, spec->width);
            break;
        case VEX_FMT_ALIGN_CENTER:
            padded = vex_fmt_pad_center(formatted, strlen(formatted), fill, spec->width);
            break;
        default:
            padded = vex_fmt_pad_right(formatted, strlen(formatted), fill, spec->width);
            break;
        }

        vex_fmt_free(formatted);
        return padded;
    }

    return formatted;
}

char *vex_fmt_i32(int32_t value, const vex_fmt_spec_t *spec)
{
    return vex_fmt_i64((int64_t)value, spec);
}

char *vex_fmt_u64(uint64_t value, const vex_fmt_spec_t *spec)
{
    vex_fmt_spec_t default_spec = vex_fmt_spec_default();
    if (!spec)
    {
        spec = &default_spec;
    }

    int base = (spec->base == VEX_FMT_BASE_HEX_UPPER) ? 16 : (spec->base == VEX_FMT_BASE_HEX_LOWER) ? 16
                                                                                                    : spec->base;
    bool uppercase = (spec->base == VEX_FMT_BASE_HEX_UPPER) || spec->uppercase;

    char *num_str = vex_fmt_utoa(value, base, uppercase);
    vex_fmt_buffer_t *buf = vex_fmt_buffer_new(64);

    // Add prefix for alternate form
    if (spec->alternate)
    {
        if (spec->base == VEX_FMT_BASE_BINARY)
        {
            vex_fmt_buffer_append_str(buf, "0b", 2);
        }
        else if (spec->base == VEX_FMT_BASE_OCTAL && value != 0)
        {
            vex_fmt_buffer_append_char(buf, '0');
        }
        else if (spec->base == VEX_FMT_BASE_HEX_LOWER || spec->base == VEX_FMT_BASE_HEX_UPPER)
        {
            vex_fmt_buffer_append_str(buf, uppercase ? "0X" : "0x", 2);
        }
    }

    vex_fmt_buffer_append_str(buf, num_str, strlen(num_str));
    vex_fmt_free(num_str);

    char *formatted = vex_fmt_buffer_to_string(buf);
    vex_fmt_buffer_free(buf);

    if (spec->width > 0)
    {
        char fill = spec->zero_pad ? '0' : spec->fill_char;
        char *padded = NULL;

        switch (spec->align)
        {
        case VEX_FMT_ALIGN_RIGHT:
            padded = vex_fmt_pad_left(formatted, strlen(formatted), fill, spec->width);
            break;
        case VEX_FMT_ALIGN_CENTER:
            padded = vex_fmt_pad_center(formatted, strlen(formatted), fill, spec->width);
            break;
        default:
            padded = vex_fmt_pad_right(formatted, strlen(formatted), fill, spec->width);
            break;
        }

        vex_fmt_free(formatted);
        return padded;
    }

    return formatted;
}

char *vex_fmt_u32(uint32_t value, const vex_fmt_spec_t *spec)
{
    return vex_fmt_u64((uint64_t)value, spec);
}

char *vex_fmt_f64(double value, const vex_fmt_spec_t *spec)
{
    vex_fmt_spec_t default_spec = vex_fmt_spec_default();
    if (!spec)
    {
        spec = &default_spec;
    }

    int precision = (spec->precision >= 0) ? spec->precision : 6;

    char temp[128];
    snprintf(temp, sizeof(temp), "%.*f", precision, value);

    if (spec->width > 0)
    {
        char *padded = NULL;
        size_t len = strlen(temp);

        switch (spec->align)
        {
        case VEX_FMT_ALIGN_RIGHT:
            padded = vex_fmt_pad_left(temp, len, spec->fill_char, spec->width);
            break;
        case VEX_FMT_ALIGN_CENTER:
            padded = vex_fmt_pad_center(temp, len, spec->fill_char, spec->width);
            break;
        default:
            padded = vex_fmt_pad_right(temp, len, spec->fill_char, spec->width);
            break;
        }

        return padded;
    }

    size_t len = strlen(temp);
    char *result = (char *)vex_fmt_malloc(len + 1);
    memcpy(result, temp, len + 1);

    return result;
}

char *vex_fmt_f32(float value, const vex_fmt_spec_t *spec)
{
    return vex_fmt_f64((double)value, spec);
}

char *vex_fmt_string(const char *str, size_t len, const vex_fmt_spec_t *spec)
{
    if (!str)
    {
        str = "(null)";
        len = 6;
    }

    vex_fmt_spec_t default_spec = vex_fmt_spec_default();
    if (!spec)
    {
        spec = &default_spec;
    }

    // Apply precision (max length)
    if (spec->precision >= 0 && (size_t)spec->precision < len)
    {
        len = (size_t)spec->precision;
    }

    if (spec->width > 0 && spec->width > (int)len)
    {
        char *padded = NULL;

        switch (spec->align)
        {
        case VEX_FMT_ALIGN_RIGHT:
            padded = vex_fmt_pad_left(str, len, spec->fill_char, spec->width);
            break;
        case VEX_FMT_ALIGN_CENTER:
            padded = vex_fmt_pad_center(str, len, spec->fill_char, spec->width);
            break;
        default:
            padded = vex_fmt_pad_right(str, len, spec->fill_char, spec->width);
            break;
        }

        return padded;
    }

    char *result = (char *)vex_fmt_malloc(len + 1);
    memcpy(result, str, len);
    result[len] = '\0';

    return result;
}

char *vex_fmt_bool(bool value, const vex_fmt_spec_t *spec)
{
    const char *str = value ? "true" : "false";
    return vex_fmt_string(str, strlen(str), spec);
}

char *vex_fmt_pointer(const void *ptr, const vex_fmt_spec_t *spec)
{
    char temp[32];
    snprintf(temp, sizeof(temp), "%p", ptr);
    return vex_fmt_string(temp, strlen(temp), spec);
}

// ============================================================================
// STRING ESCAPING
// ============================================================================

char *vex_fmt_escape_string(const char *str, size_t len)
{
    if (!str)
        return NULL;

    vex_fmt_buffer_t *buf = vex_fmt_buffer_new(len * 2);

    for (size_t i = 0; i < len; i++)
    {
        switch (str[i])
        {
        case '\n':
            vex_fmt_buffer_append_str(buf, "\\n", 2);
            break;
        case '\r':
            vex_fmt_buffer_append_str(buf, "\\r", 2);
            break;
        case '\t':
            vex_fmt_buffer_append_str(buf, "\\t", 2);
            break;
        case '\\':
            vex_fmt_buffer_append_str(buf, "\\\\", 2);
            break;
        case '"':
            vex_fmt_buffer_append_str(buf, "\\\"", 2);
            break;
        default:
            if (isprint((unsigned char)str[i]))
            {
                vex_fmt_buffer_append_char(buf, str[i]);
            }
            else
            {
                char escaped[5];
                snprintf(escaped, sizeof(escaped), "\\x%02x", (unsigned char)str[i]);
                vex_fmt_buffer_append_str(buf, escaped, 4);
            }
            break;
        }
    }

    char *result = vex_fmt_buffer_to_string(buf);
    vex_fmt_buffer_free(buf);

    return result;
}

char *vex_fmt_debug_string(const char *str, size_t len)
{
    char *escaped = vex_fmt_escape_string(str, len);
    if (!escaped)
        return NULL;

    size_t escaped_len = strlen(escaped);
    char *result = (char *)vex_fmt_malloc(escaped_len + 3);
    result[0] = '"';
    memcpy(result + 1, escaped, escaped_len);
    result[escaped_len + 1] = '"';
    result[escaped_len + 2] = '\0';

    vex_fmt_free(escaped);
    return result;
}

// ============================================================================
// HIGH-LEVEL PRINT FUNCTIONS
// ============================================================================

void vex_fmt_print(const char *str)
{
    if (str)
    {
        fputs(str, stdout);
    }
}

void vex_fmt_println(const char *str)
{
    if (str)
    {
        fputs(str, stdout);
    }
    putchar('\n');
}

void vex_fmt_eprint(const char *str)
{
    if (str)
    {
        fputs(str, stderr);
    }
}

void vex_fmt_eprintln(const char *str)
{
    if (str)
    {
        fputs(str, stderr);
    }
    fputc('\n', stderr);
}

void vex_fmt_printf(const char *format, ...)
{
    if (!format)
        return;

    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
}

void vex_fmt_eprintf(const char *format, ...)
{
    if (!format)
        return;

    va_list args;
    va_start(args, format);
    vfprintf(stderr, format, args);
    va_end(args);
}

// ============================================================================
// SPRINTF-STYLE FORMATTING
// ============================================================================

char *vex_fmt_sprintf(const char *format, ...)
{
    if (!format)
        return NULL;

    va_list args, args_copy;
    va_start(args, format);

    // Make a copy for vsnprintf length calculation
    va_copy(args_copy, args);

    // Calculate required buffer size
    int len = vsnprintf(NULL, 0, format, args_copy);
    va_end(args_copy);

    if (len < 0)
    {
        va_end(args);
        return NULL;
    }

    // Allocate buffer
    char *result = (char *)vex_fmt_malloc(len + 1);

    // Format the string
    vsnprintf(result, len + 1, format, args);
    va_end(args);

    return result;
}

char *vex_fmt_vsprintf(const char *format, va_list args)
{
    if (!format)
        return NULL;

    va_list args_copy;
    va_copy(args_copy, args);

    // Calculate required buffer size
    int len = vsnprintf(NULL, 0, format, args_copy);
    va_end(args_copy);

    if (len < 0)
    {
        return NULL;
    }

    // Allocate buffer
    char *result = (char *)vex_fmt_malloc(len + 1);

    // Format the string
    vsnprintf(result, len + 1, format, args);

    return result;
}
