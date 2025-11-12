#include "vex.h"

// VexValue constructor helpers - non-inline versions for external linkage

VexValue vex_value_i8(int8_t val) {
    VexValue v = {.type = VEX_VALUE_I8, .as_i8 = val};
    return v;
}

VexValue vex_value_i16(int16_t val) {
    VexValue v = {.type = VEX_VALUE_I16, .as_i16 = val};
    return v;
}

VexValue vex_value_i32(int32_t val) {
    VexValue v = {.type = VEX_VALUE_I32, .as_i32 = val};
    return v;
}

VexValue vex_value_i64(int64_t val) {
    VexValue v = {.type = VEX_VALUE_I64, .as_i64 = val};
    return v;
}

VexValue vex_value_i128(__int128 val) {
    VexValue v = {.type = VEX_VALUE_I128, .as_i128 = val};
    return v;
}

VexValue vex_value_u8(uint8_t val) {
    VexValue v = {.type = VEX_VALUE_U8, .as_u8 = val};
    return v;
}

VexValue vex_value_u16(uint16_t val) {
    VexValue v = {.type = VEX_VALUE_U16, .as_u16 = val};
    return v;
}

VexValue vex_value_u32(uint32_t val) {
    VexValue v = {.type = VEX_VALUE_U32, .as_u32 = val};
    return v;
}

VexValue vex_value_u64(uint64_t val) {
    VexValue v = {.type = VEX_VALUE_U64, .as_u64 = val};
    return v;
}

VexValue vex_value_u128(unsigned __int128 val) {
    VexValue v = {.type = VEX_VALUE_U128, .as_u128 = val};
    return v;
}

VexValue vex_value_f16(_Float16 val) {
    VexValue v = {.type = VEX_VALUE_F16, .as_f16 = val};
    return v;
}

VexValue vex_value_f32(float val) {
    VexValue v = {.type = VEX_VALUE_F32, .as_f32 = val};
    return v;
}

VexValue vex_value_f64(double val) {
    VexValue v = {.type = VEX_VALUE_F64, .as_f64 = val};
    return v;
}

VexValue vex_value_bool(bool val) {
    VexValue v = {.type = VEX_VALUE_BOOL, .as_bool = val};
    return v;
}

VexValue vex_value_string(const char *val) {
    VexValue v = {.type = VEX_VALUE_STRING, .as_string = val};
    return v;
}

VexValue vex_value_ptr(void *val) {
    VexValue v = {.type = VEX_VALUE_PTR, .as_ptr = val};
    return v;
}

VexValue vex_value_error(void *val) {
    VexValue v = {.type = VEX_VALUE_ERROR, .as_error = val};
    return v;
}

VexValue vex_value_nil(void) {
    VexValue v = {.type = VEX_VALUE_NIL};
    return v;
}
