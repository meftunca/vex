/**
 * Test UTF-8/UTF-16/UTF-32 validation and conversion
 * Tests functions migrated from vex_simd_utf.c to vex_string.c
 */

#include "vex.h"
#include <stdio.h>
#include <string.h>
#include <assert.h>

#define TEST(name) printf("\n[TEST] %s...\n", name)
#define PASS() printf("  ✓ PASS\n")

// ============================================================================
// UTF-16 Validation Tests
// ============================================================================

void test_utf16_validate() {
    TEST("UTF-16 Validation");
    
    // Valid UTF-16: "hello"
    uint16_t valid1[] = {0x0068, 0x0065, 0x006C, 0x006C, 0x006F};
    assert(vex_utf16_validate(valid1, 5) == true);
    
    // Valid UTF-16 with surrogate pair: "𝄞" (U+1D11E, musical symbol)
    uint16_t valid2[] = {0xD834, 0xDD1E};
    assert(vex_utf16_validate(valid2, 2) == true);
    
    // Invalid: lone high surrogate
    uint16_t invalid1[] = {0xD800};
    assert(vex_utf16_validate(invalid1, 1) == false);
    
    // Invalid: lone low surrogate
    uint16_t invalid2[] = {0xDC00};
    assert(vex_utf16_validate(invalid2, 1) == false);
    
    // Invalid: high surrogate not followed by low surrogate
    uint16_t invalid3[] = {0xD800, 0x0041};
    assert(vex_utf16_validate(invalid3, 2) == false);
    
    PASS();
}

// ============================================================================
// UTF-32 Validation Tests
// ============================================================================

void test_utf32_validate() {
    TEST("UTF-32 Validation");
    
    // Valid UTF-32: "hello"
    uint32_t valid1[] = {0x68, 0x65, 0x6C, 0x6C, 0x6F};
    assert(vex_utf32_validate(valid1, 5) == true);
    
    // Valid UTF-32: emoji "😀" (U+1F600)
    uint32_t valid2[] = {0x1F600};
    assert(vex_utf32_validate(valid2, 1) == true);
    
    // Invalid: surrogate in UTF-32 (not allowed)
    uint32_t invalid1[] = {0xD800};
    assert(vex_utf32_validate(invalid1, 1) == false);
    
    // Invalid: code point > U+10FFFF
    uint32_t invalid2[] = {0x110000};
    assert(vex_utf32_validate(invalid2, 1) == false);
    
    PASS();
}

// ============================================================================
// UTF-8 to UTF-16 Conversion Tests
// ============================================================================

void test_utf8_to_utf16() {
    TEST("UTF-8 to UTF-16 Conversion");
    
    // Test 1: ASCII "hello"
    {
        const uint8_t src[] = "hello";
        uint16_t dst[10];
        size_t n = vex_utf8_to_utf16(src, 5, dst);
        assert(n == 5);
        assert(dst[0] == 0x68 && dst[1] == 0x65 && dst[2] == 0x6C && 
               dst[3] == 0x6C && dst[4] == 0x6F);
    }
    
    // Test 2: UTF-8 with 2-byte char "café"
    {
        const uint8_t src[] = {0x63, 0x61, 0x66, 0xC3, 0xA9}; // "café"
        uint16_t dst[10];
        size_t n = vex_utf8_to_utf16(src, 5, dst);
        assert(n == 4);
        assert(dst[0] == 0x63 && dst[1] == 0x61 && dst[2] == 0x66 && dst[3] == 0xE9);
    }
    
    // Test 3: UTF-8 with 3-byte char "你好" (U+4F60, U+597D)
    {
        const uint8_t src[] = {0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD}; // "你好"
        uint16_t dst[10];
        size_t n = vex_utf8_to_utf16(src, 6, dst);
        assert(n == 2);
        assert(dst[0] == 0x4F60 && dst[1] == 0x597D);
    }
    
    // Test 4: UTF-8 with 4-byte char "😀" (U+1F600)
    {
        const uint8_t src[] = {0xF0, 0x9F, 0x98, 0x80}; // "😀"
        uint16_t dst[10];
        size_t n = vex_utf8_to_utf16(src, 4, dst);
        assert(n == 2); // Should produce surrogate pair
        assert(dst[0] == 0xD83D && dst[1] == 0xDE00);
    }
    
    // Test 5: Invalid UTF-8 (truncated)
    {
        const uint8_t src[] = {0xC3}; // Incomplete 2-byte sequence
        uint16_t dst[10];
        size_t n = vex_utf8_to_utf16(src, 1, dst);
        assert(n == (size_t)-1);
    }
    
    // Test 6: Invalid UTF-8 (overlong encoding)
    {
        const uint8_t src[] = {0xC0, 0x80}; // Overlong encoding of NULL
        uint16_t dst[10];
        size_t n = vex_utf8_to_utf16(src, 2, dst);
        assert(n == (size_t)-1);
    }
    
    PASS();
}

// ============================================================================
// UTF-8 to UTF-32 Conversion Tests
// ============================================================================

void test_utf8_to_utf32() {
    TEST("UTF-8 to UTF-32 Conversion");
    
    // Test 1: ASCII "hello"
    {
        const uint8_t src[] = "hello";
        uint32_t dst[10];
        size_t n = vex_utf8_to_utf32(src, 5, dst);
        assert(n == 5);
        assert(dst[0] == 0x68 && dst[1] == 0x65 && dst[2] == 0x6C && 
               dst[3] == 0x6C && dst[4] == 0x6F);
    }
    
    // Test 2: UTF-8 with 2-byte char "café"
    {
        const uint8_t src[] = {0x63, 0x61, 0x66, 0xC3, 0xA9}; // "café"
        uint32_t dst[10];
        size_t n = vex_utf8_to_utf32(src, 5, dst);
        assert(n == 4);
        assert(dst[0] == 0x63 && dst[1] == 0x61 && dst[2] == 0x66 && dst[3] == 0xE9);
    }
    
    // Test 3: UTF-8 with 3-byte char "你好"
    {
        const uint8_t src[] = {0xE4, 0xBD, 0xA0, 0xE5, 0xA5, 0xBD}; // "你好"
        uint32_t dst[10];
        size_t n = vex_utf8_to_utf32(src, 6, dst);
        assert(n == 2);
        assert(dst[0] == 0x4F60 && dst[1] == 0x597D);
    }
    
    // Test 4: UTF-8 with 4-byte char "😀" (U+1F600)
    {
        const uint8_t src[] = {0xF0, 0x9F, 0x98, 0x80}; // "😀"
        uint32_t dst[10];
        size_t n = vex_utf8_to_utf32(src, 4, dst);
        assert(n == 1); // Single UTF-32 code point
        assert(dst[0] == 0x1F600);
    }
    
    // Test 5: Invalid UTF-8 (truncated)
    {
        const uint8_t src[] = {0xE4, 0xBD}; // Incomplete 3-byte sequence
        uint32_t dst[10];
        size_t n = vex_utf8_to_utf32(src, 2, dst);
        assert(n == (size_t)-1);
    }
    
    // Test 6: Invalid UTF-8 (invalid continuation)
    {
        const uint8_t src[] = {0xE2, 0x28, 0xA1}; // Invalid continuation byte
        uint32_t dst[10];
        size_t n = vex_utf8_to_utf32(src, 3, dst);
        assert(n == (size_t)-1);
    }
    
    PASS();
}

// ============================================================================
// Complex Mixed Tests
// ============================================================================

void test_mixed_unicode() {
    TEST("Mixed Unicode (Latin + CJK + Emoji)");
    
    // "Hello 世界 😀" in UTF-8
    const uint8_t utf8[] = {
        0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20,           // "Hello " (6 chars)
        0xE4, 0xB8, 0x96, 0xE7, 0x95, 0x8C, 0x20,     // "世界 " (3 chars: 世, 界, space)
        0xF0, 0x9F, 0x98, 0x80                        // "😀" (1 char, 2 UTF-16 units)
    };
    size_t utf8_len = sizeof(utf8);
    
    // Convert to UTF-16
    uint16_t utf16[20];
    size_t n16 = vex_utf8_to_utf16(utf8, utf8_len, utf16);
    printf("  UTF-16 units: %zu (expected 11: 6 ASCII + 2 CJK + 1 space + 2 emoji surrogate)\n", n16);
    assert(n16 == 11); // 6 (Hello ) + 2 (CJK) + 1 (space) + 2 (emoji surrogate pair)
    
    // Convert to UTF-32
    uint32_t utf32[20];
    size_t n32 = vex_utf8_to_utf32(utf8, utf8_len, utf32);
    printf("  UTF-32 units: %zu (expected 10: 6 ASCII + 2 CJK + 1 space + 1 emoji)\n", n32);
    assert(n32 == 10); // 6 (Hello ) + 2 (CJK) + 1 (space) + 1 (emoji)
    
    // Validate UTF-16 output
    assert(vex_utf16_validate(utf16, n16) == true);
    
    // Validate UTF-32 output
    assert(vex_utf32_validate(utf32, n32) == true);
    
    PASS();
}

// ============================================================================
// Main
// ============================================================================

int main(void) {
    printf("═══════════════════════════════════════════════════════════\n");
    printf("  VEX UTF-8/UTF-16/UTF-32 CONVERSION TESTS\n");
    printf("  (Migrated from vex_simd_utf.c to vex_string.c)\n");
    printf("═══════════════════════════════════════════════════════════\n");
    
    test_utf16_validate();
    test_utf32_validate();
    test_utf8_to_utf16();
    test_utf8_to_utf32();
    test_mixed_unicode();
    
    printf("\n═══════════════════════════════════════════════════════════\n");
    printf("  ✅ ALL UTF CONVERSION TESTS PASSED!\n");
    printf("═══════════════════════════════════════════════════════════\n\n");
    printf("🎉 vex_simd_utf.c successfully migrated to vex_string.c!\n");
    printf("   All UTF-8/UTF-16/UTF-32 operations now in one place.\n\n");
    
    return 0;
}

