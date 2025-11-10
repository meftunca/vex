// Vex Display Trait Runtime - Type to String Conversions
// Provides runtime functions for converting primitive types to strings

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

// Vex string representation (matches vex_string.c)
typedef struct {
    char* data;
    size_t len;
    size_t capacity;
} VexString;

// Forward declarations from vex_string.c
extern VexString* vex_string_from_cstr(const char* cstr);
extern VexString* vex_string_new(size_t capacity);

// ===== Integer to String Conversions =====

// Convert i32 to string
VexString* vex_i32_to_string(int32_t value) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%d", value);
    return vex_string_from_cstr(buffer);
}

// Convert i64 to string
VexString* vex_i64_to_string(int64_t value) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%lld", (long long)value);
    return vex_string_from_cstr(buffer);
}

// Convert i8 to string
VexString* vex_i8_to_string(int8_t value) {
    char buffer[16];
    snprintf(buffer, sizeof(buffer), "%d", value);
    return vex_string_from_cstr(buffer);
}

// Convert i16 to string
VexString* vex_i16_to_string(int16_t value) {
    char buffer[16];
    snprintf(buffer, sizeof(buffer), "%d", value);
    return vex_string_from_cstr(buffer);
}

// Convert i128 to string (using long long, may truncate)
VexString* vex_i128_to_string(__int128 value) {
    // Note: __int128 not fully supported by printf, use long long approximation
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%lld", (long long)value);
    return vex_string_from_cstr(buffer);
}

// ===== Unsigned Integer to String Conversions =====

VexString* vex_u8_to_string(uint8_t value) {
    char buffer[16];
    snprintf(buffer, sizeof(buffer), "%u", value);
    return vex_string_from_cstr(buffer);
}

VexString* vex_u16_to_string(uint16_t value) {
    char buffer[16];
    snprintf(buffer, sizeof(buffer), "%u", value);
    return vex_string_from_cstr(buffer);
}

VexString* vex_u32_to_string(uint32_t value) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%u", value);
    return vex_string_from_cstr(buffer);
}

VexString* vex_u64_to_string(uint64_t value) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%llu", (unsigned long long)value);
    return vex_string_from_cstr(buffer);
}

VexString* vex_u128_to_string(__uint128_t value) {
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%llu", (unsigned long long)value);
    return vex_string_from_cstr(buffer);
}

// ===== Float to String Conversions =====

VexString* vex_f32_to_string(float value) {
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%g", value);
    return vex_string_from_cstr(buffer);
}

VexString* vex_f64_to_string(double value) {
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%g", value);
    return vex_string_from_cstr(buffer);
}

// ===== Boolean to String Conversion =====

VexString* vex_bool_to_string(bool value) {
    return vex_string_from_cstr(value ? "true" : "false");
}

// ===== String Identity (already a string) =====

VexString* vex_string_to_string(VexString* value) {
    // String is already a string, just return it
    return value;
}

// ===== Char to String Conversion =====

VexString* vex_char_to_string(char value) {
    char buffer[2] = {value, '\0'};
    return vex_string_from_cstr(buffer);
}

VexString* vex_byte_to_string(uint8_t value) {
    return vex_u8_to_string(value);
}
