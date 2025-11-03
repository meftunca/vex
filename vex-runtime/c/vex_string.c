/**
 * Vex String Operations
 * Zero-overhead, inline-friendly implementations
 */

#include "vex.h"

// ============================================================================
// STRING OPERATIONS
// ============================================================================

size_t vex_strlen(const char* s) {
    size_t len = 0;
    while (s[len]) {
        len++;
    }
    return len;
}

int vex_strcmp(const char* s1, const char* s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

char* vex_strcpy(char* dest, const char* src) {
    char* d = dest;
    while ((*d++ = *src++));
    return dest;
}

char* vex_strcat(char* dest, const char* src) {
    char* d = dest;
    // Find end of dest
    while (*d) {
        d++;
    }
    // Copy src to end of dest
    while ((*d++ = *src++));
    return dest;
}

char* vex_strdup(const char* s) {
    size_t len = vex_strlen(s) + 1;  // +1 for null terminator
    char* new_str = (char*)vex_malloc(len);
    if (new_str) {
        vex_memcpy(new_str, s, len);
    }
    return new_str;
}

// ============================================================================
// UTF-8 OPERATIONS
// ============================================================================

/**
 * Check if byte is a UTF-8 continuation byte (10xxxxxx)
 */
static inline bool vex_utf8_is_continuation(unsigned char byte) {
    return (byte & 0xC0) == 0x80;
}

/**
 * Get the length of a UTF-8 character from its first byte
 * Returns 0 for invalid UTF-8
 */
static inline size_t vex_utf8_char_len(unsigned char first_byte) {
    if ((first_byte & 0x80) == 0x00) {
        // 0xxxxxxx - 1 byte (ASCII)
        return 1;
    } else if ((first_byte & 0xE0) == 0xC0) {
        // 110xxxxx - 2 bytes
        return 2;
    } else if ((first_byte & 0xF0) == 0xE0) {
        // 1110xxxx - 3 bytes
        return 3;
    } else if ((first_byte & 0xF8) == 0xF0) {
        // 11110xxx - 4 bytes
        return 4;
    }
    // Invalid UTF-8
    return 0;
}

/**
 * Validate UTF-8 string
 * Returns true if valid UTF-8, false otherwise
 */
bool vex_utf8_valid(const char* s, size_t byte_len) {
    if (!s) return false;
    
    const unsigned char* str = (const unsigned char*)s;
    size_t i = 0;
    
    while (i < byte_len) {
        unsigned char first = str[i];
        
        // Get expected character length
        size_t char_len = vex_utf8_char_len(first);
        
        if (char_len == 0) {
            // Invalid first byte
            return false;
        }
        
        // Check if we have enough bytes
        if (i + char_len > byte_len) {
            return false;
        }
        
        // Validate continuation bytes
        for (size_t j = 1; j < char_len; j++) {
            if (!vex_utf8_is_continuation(str[i + j])) {
                return false;
            }
        }
        
        // Check for overlong encodings and invalid code points
        if (char_len == 2) {
            // Must be >= 0x80
            unsigned int code_point = ((first & 0x1F) << 6) | (str[i + 1] & 0x3F);
            if (code_point < 0x80) return false;  // Overlong
        } else if (char_len == 3) {
            // Must be >= 0x800, not surrogate (0xD800-0xDFFF)
            unsigned int code_point = ((first & 0x0F) << 12) | 
                                     ((str[i + 1] & 0x3F) << 6) | 
                                     (str[i + 2] & 0x3F);
            if (code_point < 0x800) return false;  // Overlong
            if (code_point >= 0xD800 && code_point <= 0xDFFF) return false;  // Surrogate
        } else if (char_len == 4) {
            // Must be >= 0x10000, <= 0x10FFFF
            unsigned int code_point = ((first & 0x07) << 18) | 
                                     ((str[i + 1] & 0x3F) << 12) | 
                                     ((str[i + 2] & 0x3F) << 6) | 
                                     (str[i + 3] & 0x3F);
            if (code_point < 0x10000) return false;  // Overlong
            if (code_point > 0x10FFFF) return false;  // Too large
        }
        
        i += char_len;
    }
    
    return true;
}

/**
 * Count UTF-8 characters (not bytes) in a string
 * Returns character count, or 0 if invalid UTF-8
 */
size_t vex_utf8_char_count(const char* s) {
    if (!s) return 0;
    
    const unsigned char* str = (const unsigned char*)s;
    size_t char_count = 0;
    size_t i = 0;
    
    while (str[i] != '\0') {
        size_t char_len = vex_utf8_char_len(str[i]);
        
        if (char_len == 0) {
            // Invalid UTF-8
            vex_panic("utf8_char_count: invalid UTF-8 sequence");
        }
        
        // Validate continuation bytes
        for (size_t j = 1; j < char_len; j++) {
            if (!vex_utf8_is_continuation(str[i + j])) {
                vex_panic("utf8_char_count: invalid UTF-8 continuation byte");
            }
        }
        
        char_count++;
        i += char_len;
    }
    
    return char_count;
}

/**
 * Get pointer to the Nth UTF-8 character (0-indexed)
 * Returns NULL if index out of bounds or invalid UTF-8
 */
const char* vex_utf8_char_at(const char* s, size_t char_index) {
    if (!s) {
        vex_panic("utf8_char_at: NULL string pointer");
    }
    
    const unsigned char* str = (const unsigned char*)s;
    size_t current_char = 0;
    size_t i = 0;
    
    while (str[i] != '\0') {
        if (current_char == char_index) {
            return (const char*)&str[i];
        }
        
        size_t char_len = vex_utf8_char_len(str[i]);
        
        if (char_len == 0) {
            vex_panic("utf8_char_at: invalid UTF-8 sequence");
        }
        
        // Validate continuation bytes
        for (size_t j = 1; j < char_len; j++) {
            if (!vex_utf8_is_continuation(str[i + j])) {
                vex_panic("utf8_char_at: invalid UTF-8 continuation byte");
            }
        }
        
        current_char++;
        i += char_len;
    }
    
    // Index out of bounds
    char msg[128];
    vex_sprintf(msg, "utf8_char_at: index out of bounds (index: %zu, length: %zu)", 
                char_index, current_char);
    vex_panic(msg);
    return NULL;  // Never reached
}

/**
 * Convert UTF-8 character index to byte index
 * Returns byte index, or panics if invalid
 */
size_t vex_utf8_char_to_byte_index(const char* s, size_t char_index) {
    if (!s) {
        vex_panic("utf8_char_to_byte_index: NULL string pointer");
    }
    
    const unsigned char* str = (const unsigned char*)s;
    size_t current_char = 0;
    size_t byte_index = 0;
    
    while (str[byte_index] != '\0') {
        if (current_char == char_index) {
            return byte_index;
        }
        
        size_t char_len = vex_utf8_char_len(str[byte_index]);
        
        if (char_len == 0) {
            vex_panic("utf8_char_to_byte_index: invalid UTF-8 sequence");
        }
        
        current_char++;
        byte_index += char_len;
    }
    
    // Index out of bounds
    char msg[128];
    vex_sprintf(msg, "utf8_char_to_byte_index: index out of bounds (index: %zu, length: %zu)", 
                char_index, current_char);
    vex_panic(msg);
    return 0;  // Never reached
}

/**
 * Extract a single UTF-8 character at index and return as new string
 * Allocates memory (must be freed)
 */
char* vex_utf8_char_extract(const char* s, size_t char_index) {
    const char* char_ptr = vex_utf8_char_at(s, char_index);
    size_t char_len = vex_utf8_char_len(*((unsigned char*)char_ptr));
    
    char* result = (char*)vex_malloc(char_len + 1);
    if (!result) {
        vex_panic("utf8_char_extract: out of memory");
    }
    
    vex_memcpy(result, char_ptr, char_len);
    result[char_len] = '\0';
    
    return result;
}

/**
 * Decode UTF-8 character to Unicode code point
 * Returns code point (0-0x10FFFF) or 0xFFFFFFFF on error
 */
uint32_t vex_utf8_decode(const char* s) {
    if (!s) return 0xFFFFFFFF;
    
    const unsigned char* str = (const unsigned char*)s;
    size_t char_len = vex_utf8_char_len(str[0]);
    
    if (char_len == 0) return 0xFFFFFFFF;
    
    uint32_t code_point;
    
    if (char_len == 1) {
        code_point = str[0];
    } else if (char_len == 2) {
        code_point = ((str[0] & 0x1F) << 6) | (str[1] & 0x3F);
    } else if (char_len == 3) {
        code_point = ((str[0] & 0x0F) << 12) | 
                    ((str[1] & 0x3F) << 6) | 
                    (str[2] & 0x3F);
    } else {  // char_len == 4
        code_point = ((str[0] & 0x07) << 18) | 
                    ((str[1] & 0x3F) << 12) | 
                    ((str[2] & 0x3F) << 6) | 
                    (str[3] & 0x3F);
    }
    
    return code_point;
}

/**
 * Encode Unicode code point to UTF-8
 * Writes to buf (must have at least 5 bytes)
 * Returns number of bytes written, or 0 on error
 */
size_t vex_utf8_encode(uint32_t code_point, char* buf) {
    if (!buf) return 0;
    
    if (code_point <= 0x7F) {
        // 1 byte: 0xxxxxxx
        buf[0] = (char)code_point;
        buf[1] = '\0';
        return 1;
    } else if (code_point <= 0x7FF) {
        // 2 bytes: 110xxxxx 10xxxxxx
        buf[0] = (char)(0xC0 | (code_point >> 6));
        buf[1] = (char)(0x80 | (code_point & 0x3F));
        buf[2] = '\0';
        return 2;
    } else if (code_point <= 0xFFFF) {
        // 3 bytes: 1110xxxx 10xxxxxx 10xxxxxx
        // Check for surrogates
        if (code_point >= 0xD800 && code_point <= 0xDFFF) {
            return 0;  // Invalid (surrogate range)
        }
        buf[0] = (char)(0xE0 | (code_point >> 12));
        buf[1] = (char)(0x80 | ((code_point >> 6) & 0x3F));
        buf[2] = (char)(0x80 | (code_point & 0x3F));
        buf[3] = '\0';
        return 3;
    } else if (code_point <= 0x10FFFF) {
        // 4 bytes: 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
        buf[0] = (char)(0xF0 | (code_point >> 18));
        buf[1] = (char)(0x80 | ((code_point >> 12) & 0x3F));
        buf[2] = (char)(0x80 | ((code_point >> 6) & 0x3F));
        buf[3] = (char)(0x80 | (code_point & 0x3F));
        buf[4] = '\0';
        return 4;
    }
    
    return 0;  // Invalid code point
}
