// Format buffer implementation for Vex language
// Provides type-safe string formatting with dynamic buffer management

#include <stdlib.h>
#include "vex.h"
#include <string.h>
#include <stdio.h>
#include <stdint.h>

struct VexFormatBuffer
{
    char *data;
    size_t len;
    size_t capacity;
};

VexFormatBuffer *vex_fmt_buffer_new()
{
    VexFormatBuffer *buf = (VexFormatBuffer *)vex_malloc(sizeof(VexFormatBuffer));
    if (!buf)
        return NULL;

    buf->capacity = 256;
    buf->len = 0;
    buf->data = (char *)vex_malloc(buf->capacity);
    if (!buf->data)
    {
        vex_free(buf);
        return NULL;
    }
    buf->data[0] = '\0';
    return buf;
}

void vex_fmt_buffer_free(VexFormatBuffer *buf)
{
    if (!buf)
        return;
    if (buf->data)
        vex_free(buf->data);
    vex_free(buf);
}

void vex_fmt_buffer_append_str(VexFormatBuffer *buf, const char *str)
{
    if (!buf || !str)
        return;

    size_t str_len = strlen(str);
    while (buf->len + str_len >= buf->capacity)
    {
        buf->capacity *= 2;
        char *new_data = (char *)vex_realloc(buf->data, buf->capacity);
        if (!new_data)
            return;
        buf->data = new_data;
    }

    memcpy(buf->data + buf->len, str, str_len);
    buf->len += str_len;
    buf->data[buf->len] = '\0';
}

char *vex_fmt_buffer_to_string(VexFormatBuffer *buf)
{
    if (!buf)
        return NULL;

    char *result = (char *)vex_malloc(buf->len + 1);
    if (!result)
        return NULL;

    memcpy(result, buf->data, buf->len + 1);
    return result;
}

void vex_fmt_i32(VexFormatBuffer *buf, int32_t val)
{
    if (!buf)
        return;

    char tmp[32];
    snprintf(tmp, sizeof(tmp), "%d", val);
    vex_fmt_buffer_append_str(buf, tmp);
}

void vex_fmt_i64(VexFormatBuffer *buf, int64_t val)
{
    if (!buf)
        return;

    char tmp[32];
    snprintf(tmp, sizeof(tmp), "%lld", (long long)val);
    vex_fmt_buffer_append_str(buf, tmp);
}

void vex_fmt_f32(VexFormatBuffer *buf, float val)
{
    if (!buf)
        return;

    char tmp[64];
    snprintf(tmp, sizeof(tmp), "%g", val);
    vex_fmt_buffer_append_str(buf, tmp);
}

void vex_fmt_f64(VexFormatBuffer *buf, double val)
{
    if (!buf)
        return;

    char tmp[64];
    snprintf(tmp, sizeof(tmp), "%g", val);
    vex_fmt_buffer_append_str(buf, tmp);
}

void vex_fmt_bool(VexFormatBuffer *buf, int val)
{
    if (!buf)
        return;

    vex_fmt_buffer_append_str(buf, val ? "true" : "false");
}

void vex_fmt_string(VexFormatBuffer *buf, const char *str)
{
    if (!buf || !str)
        return;

    vex_fmt_buffer_append_str(buf, str);
}

void vex_fmt_char(VexFormatBuffer *buf, char c)
{
    if (!buf)
        return;

    char tmp[2] = {c, '\0'};
    vex_fmt_buffer_append_str(buf, tmp);
}

// 128-bit integers
void vex_fmt_u128(VexFormatBuffer *buf, unsigned __int128 n)
{
    if (!buf)
        return;

    if (n == 0)
    {
        vex_fmt_buffer_append_str(buf, "0");
        return;
    }

    char tmp[40];
    int i = 0;
    while (n > 0)
    {
        tmp[i++] = (n % 10) + '0';
        n /= 10;
    }

    // Reverse and append
    char rev[40];
    for (int j = 0; j < i; j++)
    {
        rev[j] = tmp[i - 1 - j];
    }
    rev[i] = '\0';
    vex_fmt_buffer_append_str(buf, rev);
}

void vex_fmt_i128(VexFormatBuffer *buf, __int128 val)
{
    if (!buf)
        return;

    if (val < 0)
    {
        vex_fmt_buffer_append_str(buf, "-");
        vex_fmt_u128(buf, (unsigned __int128)(-val));
    }
    else
    {
        vex_fmt_u128(buf, (unsigned __int128)val);
    }
}

// Float 16
void vex_fmt_f16(VexFormatBuffer *buf, uint16_t val)
{
    if (!buf)
        return;

    _Float16 f;
    memcpy(&f, &val, sizeof(uint16_t));

    char tmp[64];
    snprintf(tmp, sizeof(tmp), "%g", (double)f);
    vex_fmt_buffer_append_str(buf, tmp);
}
