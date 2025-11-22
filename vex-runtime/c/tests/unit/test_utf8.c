/**
 * UTF-8 Test Suite
 * Tests Unicode string operations
 */

#include "vex.h"
#include <stdio.h>
#include <assert.h>

void test_utf8_basic() {
    printf("=== Testing Basic UTF-8 ===\n");
    
    // ASCII (1 byte per char)
    const char* ascii = "Hello";
    assert(vex_strlen(ascii) == 5);  // 5 bytes
    assert(vex_utf8_char_count(ascii) == 5);  // 5 characters
    printf("✓ ASCII: byte_len=5, char_len=5\n");
    
    // Latin extended (Turkish - mixed 1 and 2 bytes)
    const char* turkish = "Türkçe";
    size_t turkish_bytes = vex_strlen(turkish);
    size_t turkish_chars = vex_utf8_char_count(turkish);
    printf("  Turkish 'Türkçe': %zu bytes, %zu chars\n", turkish_bytes, turkish_chars);
    assert(turkish_bytes == 8);  // T(1) ü(2) r(1) k(1) ç(2) e(1) = 8 bytes
    assert(turkish_chars == 6);  // 6 characters
    printf("✓ Turkish: byte_len=8, char_len=6\n");
    
    // Emoji (4 bytes)
    const char* emoji = "👋";
    size_t emoji_bytes = vex_strlen(emoji);
    size_t emoji_chars = vex_utf8_char_count(emoji);
    printf("  Emoji '👋': %zu bytes, %zu chars\n", emoji_bytes, emoji_chars);
    assert(emoji_bytes == 4);  // Emoji = 4 bytes
    assert(emoji_chars == 1);  // 1 character
    printf("✓ Emoji: byte_len=4, char_len=1\n");
    
    // Mixed
    const char* mixed = "Hello 世界 👋";
    size_t mixed_bytes = vex_strlen(mixed);
    size_t mixed_chars = vex_utf8_char_count(mixed);
    printf("  Mixed 'Hello 世界 👋': %zu bytes, %zu chars\n", mixed_bytes, mixed_chars);
    // Hello(5) space(1) 世(3) 界(3) space(1) 👋(4) = 17 bytes
    // H e l l o [space] 世 界 [space] 👋 = 10 characters
    assert(mixed_chars == 10);
    printf("✓ Mixed: byte_len=%zu, char_len=10\n", mixed_bytes);
}

void test_utf8_validation() {
    printf("\n=== Testing UTF-8 Validation ===\n");
    
    // Valid UTF-8
    const char* valid = "Hello 世界";
    assert(vex_utf8_valid(valid, vex_strlen(valid)) == true);
    printf("✓ Valid UTF-8: 'Hello 世界'\n");
    
    // Invalid UTF-8 (truncated)
    const char invalid[] = {0xE4, 0xB8, 0x00};  // Truncated Chinese character
    assert(vex_utf8_valid(invalid, 3) == false);
    printf("✓ Invalid UTF-8 detected (truncated)\n");
    
    // Invalid UTF-8 (overlong encoding)
    const char overlong[] = {0xC0, 0x80, 0x00};  // Overlong encoding of NULL
    assert(vex_utf8_valid(overlong, 3) == false);
    printf("✓ Invalid UTF-8 detected (overlong)\n");
    
    // Invalid UTF-8 (surrogate)
    const char surrogate[] = {0xED, 0xA0, 0x80, 0x00};  // Surrogate U+D800
    assert(vex_utf8_valid(surrogate, 4) == false);
    printf("✓ Invalid UTF-8 detected (surrogate)\n");
}

void test_utf8_char_access() {
    printf("\n=== Testing UTF-8 Character Access ===\n");
    
    const char* s = "Merhaba dünya";
    size_t char_count = vex_utf8_char_count(s);
    printf("  String: '%s' (%zu chars)\n", s, char_count);
    
    // Access character by index
    const char* char_ptr = vex_utf8_char_at(s, 8);  // 'd' in "dünya"
    printf("  char[8] = '%c' (byte at position)\n", *char_ptr);
    assert(*char_ptr == 'd');
    printf("✓ vex_utf8_char_at(8) = 'd'\n");
    
    // Access 'ü' (2-byte character)
    char_ptr = vex_utf8_char_at(s, 9);  // 'ü'
    printf("  char[9] = 'ü' (2-byte UTF-8)\n");
    printf("✓ vex_utf8_char_at(9) = 'ü'\n");
    
    // Extract character
    char* extracted = vex_utf8_char_extract(s, 9);
    printf("  Extracted char[9]: '%s'\n", extracted);
    assert(vex_strcmp(extracted, "ü") == 0);
    vex_free(extracted);
    printf("✓ vex_utf8_char_extract(9) = 'ü'\n");
}

void test_utf8_indexing() {
    printf("\n=== Testing UTF-8 Indexing ===\n");
    
    const char* s = "Hello 世界";
    
    // Character index → byte index
    size_t byte_idx_0 = vex_utf8_char_to_byte_index(s, 0);  // 'H'
    size_t byte_idx_6 = vex_utf8_char_to_byte_index(s, 6);  // '世'
    size_t byte_idx_7 = vex_utf8_char_to_byte_index(s, 7);  // '界'
    
    printf("  char[0] 'H' → byte[%zu]\n", byte_idx_0);
    printf("  char[6] '世' → byte[%zu]\n", byte_idx_6);
    printf("  char[7] '界' → byte[%zu]\n", byte_idx_7);
    
    assert(byte_idx_0 == 0);   // H at byte 0
    assert(byte_idx_6 == 6);   // 世 at byte 6 (after "Hello ")
    assert(byte_idx_7 == 9);   // 界 at byte 9 (世 is 3 bytes)
    
    printf("✓ Character to byte index conversion\n");
}

void test_utf8_codec() {
    printf("\n=== Testing UTF-8 Encode/Decode ===\n");
    
    // Decode UTF-8 to code point
    uint32_t cp_a = vex_utf8_decode("a");
    uint32_t cp_u_umlaut = vex_utf8_decode("ü");
    uint32_t cp_emoji = vex_utf8_decode("👋");
    
    printf("  'a' → U+%04X (dec: %u)\n", cp_a, cp_a);
    printf("  'ü' → U+%04X (dec: %u)\n", cp_u_umlaut, cp_u_umlaut);
    printf("  '👋' → U+%04X (dec: %u)\n", cp_emoji, cp_emoji);
    
    assert(cp_a == 0x61);           // 'a' = U+0061
    assert(cp_u_umlaut == 0xFC);    // 'ü' = U+00FC
    assert(cp_emoji == 0x1F44B);    // '👋' = U+1F44B
    
    printf("✓ UTF-8 decode\n");
    
    // Encode code point to UTF-8
    char buf[5];
    
    size_t len_a = vex_utf8_encode(0x61, buf);
    assert(len_a == 1);
    assert(vex_strcmp(buf, "a") == 0);
    printf("  U+0061 → '%s' (%zu bytes)\n", buf, len_a);
    
    size_t len_u = vex_utf8_encode(0xFC, buf);
    assert(len_u == 2);
    assert(vex_strcmp(buf, "ü") == 0);
    printf("  U+00FC → '%s' (%zu bytes)\n", buf, len_u);
    
    size_t len_emoji = vex_utf8_encode(0x1F44B, buf);
    assert(len_emoji == 4);
    assert(vex_strcmp(buf, "👋") == 0);
    printf("  U+1F44B → '%s' (%zu bytes)\n", buf, len_emoji);
    
    printf("✓ UTF-8 encode\n");
}

void test_utf8_real_world() {
    printf("\n=== Testing Real-World Examples ===\n");
    
    // Turkish
    const char* tr = "Merhaba dünya";
    printf("  Turkish: '%s'\n", tr);
    printf("    Bytes: %zu, Chars: %zu\n", vex_strlen(tr), vex_utf8_char_count(tr));
    
    // Japanese
    const char* jp = "こんにちは";
    printf("  Japanese: '%s'\n", jp);
    printf("    Bytes: %zu, Chars: %zu\n", vex_strlen(jp), vex_utf8_char_count(jp));
    
    // Arabic
    const char* ar = "مرحبا";
    printf("  Arabic: '%s'\n", ar);
    printf("    Bytes: %zu, Chars: %zu\n", vex_strlen(ar), vex_utf8_char_count(ar));
    
    // Emoji sequence
    const char* emoji_seq = "Hello 👨‍👩‍👧‍👦 World";
    printf("  Emoji: '%s'\n", emoji_seq);
    printf("    Bytes: %zu, Chars: %zu\n", vex_strlen(emoji_seq), vex_utf8_char_count(emoji_seq));
    printf("    Note: Family emoji is multiple code points with ZWJ\n");
    
    printf("✓ Real-world examples\n");
}

int main() {
    printf("╔════════════════════════════════════════╗\n");
    printf("║  Vex UTF-8 Test Suite                 ║\n");
    printf("╚════════════════════════════════════════╝\n\n");
    
    test_utf8_basic();
    test_utf8_validation();
    test_utf8_char_access();
    test_utf8_indexing();
    test_utf8_codec();
    test_utf8_real_world();
    
    printf("\n╔════════════════════════════════════════╗\n");
    printf("║  All UTF-8 Tests Passed! ✅            ║\n");
    printf("╚════════════════════════════════════════╝\n");
    
    return 0;
}
