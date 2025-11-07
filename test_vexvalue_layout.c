#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

typedef enum {
    VEX_VALUE_I32,
    VEX_VALUE_I64,
    VEX_VALUE_F32,
    VEX_VALUE_F64,
    VEX_VALUE_BOOL,
    VEX_VALUE_STRING,
    VEX_VALUE_PTR,
} VexValueType;

typedef struct {
    VexValueType type;
    union {
        int32_t as_i32;
        int64_t as_i64;
        float as_f32;
        double as_f64;
        bool as_bool;
        const char *as_string;
        void *as_ptr;
    };
} VexValue;

int main() {
    printf("sizeof(VexValue) = %zu\n", sizeof(VexValue));
    printf("offsetof(type) = %zu\n", __builtin_offsetof(VexValue, type));
    printf("offsetof(as_i32) = %zu\n", __builtin_offsetof(VexValue, as_i32));
    printf("offsetof(as_i64) = %zu\n", __builtin_offsetof(VexValue, as_i64));
    printf("offsetof(as_f64) = %zu\n", __builtin_offsetof(VexValue, as_f64));
    
    // Test actual values
    VexValue val;
    val.type = VEX_VALUE_I32;
    val.as_i32 = 42;
    printf("\nTest: type=%d, as_i32=%d\n", val.type, val.as_i32);
    
    return 0;
}
