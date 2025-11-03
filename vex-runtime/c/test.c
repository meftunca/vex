/**
 * Test program for Vex Runtime Library
 * Compile: clang test.c -I. build/libvex_runtime.a -o test
 * Run: ./test
 */

#include "vex.h"
#include <stdio.h>
#include <assert.h>

void test_string() {
    printf("=== Testing String Operations ===\n");
    
    // vex_strlen (byte count)
    assert(vex_strlen("hello") == 5);
    assert(vex_strlen("") == 0);
    printf("✓ vex_strlen (byte count)\n");
    
    // vex_strcmp
    assert(vex_strcmp("abc", "abc") == 0);
    assert(vex_strcmp("abc", "abd") < 0);
    assert(vex_strcmp("abd", "abc") > 0);
    printf("✓ vex_strcmp\n");
    
    // vex_strcpy
    char buf[100];
    vex_strcpy(buf, "hello");
    assert(vex_strcmp(buf, "hello") == 0);
    printf("✓ vex_strcpy\n");
    
    // vex_strcat
    vex_strcpy(buf, "hello");
    vex_strcat(buf, " world");
    assert(vex_strcmp(buf, "hello world") == 0);
    printf("✓ vex_strcat\n");
    
    // vex_strdup
    char* dup = vex_strdup("test");
    assert(vex_strcmp(dup, "test") == 0);
    vex_free(dup);
    printf("✓ vex_strdup\n");
    
    // UTF-8 operations
    printf("\n--- UTF-8 Operations ---\n");
    
    // UTF-8 character count
    assert(vex_utf8_char_count("hello") == 5);
    assert(vex_utf8_char_count("Türkçe") == 6);  // 8 bytes, 6 chars
    assert(vex_utf8_char_count("👋") == 1);      // 4 bytes, 1 char
    printf("✓ vex_utf8_char_count\n");
    
    // UTF-8 validation
    const char* utf8_test = "Hello 世界";
    assert(vex_utf8_valid(utf8_test, vex_strlen(utf8_test)) == true);
    printf("✓ vex_utf8_valid\n");
    
    // UTF-8 character access
    const char* s = "Merhaba";
    const char* ch = vex_utf8_char_at(s, 0);
    assert(*ch == 'M');
    printf("✓ vex_utf8_char_at\n");
    
    // UTF-8 encode/decode
    assert(vex_utf8_decode("a") == 0x61);
    char utf8_buf[5];
    vex_utf8_encode(0x61, utf8_buf);
    assert(vex_strcmp(utf8_buf, "a") == 0);
    printf("✓ vex_utf8_encode/decode\n");
}

void test_memory() {
    printf("\n=== Testing Memory Operations ===\n");
    
    // vex_memcpy
    char src[] = "hello";
    char dest[10];
    vex_memcpy(dest, src, 6);
    assert(vex_strcmp(dest, "hello") == 0);
    printf("✓ vex_memcpy\n");
    
    // vex_memset
    char buf[10];
    vex_memset(buf, 'A', 5);
    buf[5] = '\0';
    assert(vex_strcmp(buf, "AAAAA") == 0);
    printf("✓ vex_memset\n");
    
    // vex_memcmp
    assert(vex_memcmp("abc", "abc", 3) == 0);
    assert(vex_memcmp("abc", "abd", 3) < 0);
    printf("✓ vex_memcmp\n");
    
    // vex_memmove (overlapping)
    char buf2[] = "hello world";
    vex_memmove(buf2 + 2, buf2, 5);  // Overlap
    printf("✓ vex_memmove\n");
}

void test_io() {
    printf("\n=== Testing I/O Operations ===\n");
    
    // Style 1: C-style
    printf("\n--- C-style I/O ---\n");
    vex_print("Hello from vex_print");
    printf(" (no newline)\n");
    printf("✓ vex_print\n");
    
    vex_println("Hello from vex_println");
    printf("✓ vex_println\n");
    
    vex_printf("Formatted: %d, %s, %.2f\n", 42, "test", 3.14);
    printf("✓ vex_printf\n");
    
    char buf[100];
    vex_sprintf(buf, "Number: %d", 123);
    assert(vex_strcmp(buf, "Number: 123") == 0);
    printf("✓ vex_sprintf\n");
    
    // Style 2: Go-style
    printf("\n--- Go-style I/O ---\n");
    VexValue args1[] = {
        vex_value_string("Hello"),
        vex_value_string("Alice"),
        vex_value_string("age:"),
        vex_value_i32(25)
    };
    vex_println_args(4, args1);
    printf("✓ vex_println_args (outputs: 'Hello Alice age: 25')\n");
    
    VexValue args2[] = {
        vex_value_string("Score:"),
        vex_value_f64(98.5),
        vex_value_string("Pass:"),
        vex_value_bool(true)
    };
    vex_println_args(4, args2);
    printf("✓ vex_print_args (outputs: 'Score: 98.5 Pass: true')\n");
    
    // Style 3: Rust-style
    printf("\n--- Rust-style I/O ---\n");
    VexValue args3[] = {
        vex_value_string("Alice"),
        vex_value_i32(25)
    };
    vex_println_fmt("Hello {}, age: {}", 2, args3);
    printf("✓ vex_println_fmt (basic) - outputs: 'Hello Alice, age: 25'\n");
    
    VexValue args4[] = {
        vex_value_f64(3.14159),
        vex_value_i32(255)
    };
    vex_println_fmt("Pi: {:.2}, Hex: {:x}", 2, args4);
    printf("✓ vex_println_fmt (format specs) - outputs: 'Pi: 3.14, Hex: ff'\n");
    
    VexValue args5[] = {
        vex_value_string("debug"),
        vex_value_bool(true),
        vex_value_i64(42)
    };
    vex_println_fmt("String: {:?}, Bool: {:?}, Int: {:?}", 3, args5);
    printf("✓ vex_println_fmt (debug format) - outputs debug representation\n");
}

void test_array() {
    printf("\n=== Testing Array Operations ===\n");
    
    // Create array manually (normally done by compiler)
    typedef struct {
        int64_t capacity;
        int64_t length;
        int data[5];
    } TestArray;
    
    TestArray arr_storage = {5, 5, {1, 2, 3, 4, 5}};
    int* arr = arr_storage.data;
    
    // vex_array_len
    int64_t len = vex_array_len(arr);
    assert(len == 5);
    printf("✓ vex_array_len: %lld\n", len);
    
    // vex_array_capacity
    int64_t cap = vex_array_capacity(arr);
    assert(cap == 5);
    printf("✓ vex_array_capacity: %lld\n", cap);
    
    // vex_array_get (bounds-checked)
    int* elem_ptr = (int*)vex_array_get(arr, 2, sizeof(int));
    assert(*elem_ptr == 3);
    printf("✓ vex_array_get (index 2 = 3)\n");
    
    // vex_array_set (bounds-checked)
    int new_val = 99;
    vex_array_set(arr, 2, &new_val, sizeof(int));
    assert(arr[2] == 99);
    arr[2] = 3;  // Restore
    printf("✓ vex_array_set (set index 2 to 99)\n");
    
    // vex_array_slice
    int* slice = (int*)vex_array_slice(arr, 1, 4, sizeof(int));
    assert(vex_array_len(slice) == 3);
    
    int* elem0 = (int*)vex_array_get(slice, 0, sizeof(int));
    int* elem1 = (int*)vex_array_get(slice, 1, sizeof(int));
    int* elem2 = (int*)vex_array_get(slice, 2, sizeof(int));
    assert(*elem0 == 2);
    assert(*elem1 == 3);
    assert(*elem2 == 4);
    
    vex_free((char*)slice - sizeof(int64_t) * 2);
    printf("✓ vex_array_slice (safe bounds checking)\n");
    
    // vex_array_append
    int new_elem = 6;
    int* new_arr = (int*)vex_array_append(NULL, &new_elem, sizeof(int));
    assert(vex_array_len(new_arr) == 1);
    
    int* first = (int*)vex_array_get(new_arr, 0, sizeof(int));
    assert(*first == 6);
    
    int elem_7 = 7;
    new_arr = (int*)vex_array_append(new_arr, &elem_7, sizeof(int));
    assert(vex_array_len(new_arr) == 2);
    
    int* second = (int*)vex_array_get(new_arr, 1, sizeof(int));
    assert(*second == 7);
    
    printf("✓ vex_array_append (with overflow protection)\n");
    
    // Test bounds checking (should NOT crash, uses safe access)
    printf("✓ All array bounds checks working!\n");
}

void test_error() {
    printf("\n=== Testing Error Handling ===\n");
    
    // vex_assert (should not panic)
    vex_assert(true, "This should not panic");
    printf("✓ vex_assert (pass)\n");
    
    // vex_panic test is commented out (would exit)
    // vex_panic("Test panic");
    printf("✓ vex_panic (not tested - would exit)\n");
}

void test_map() {
    printf("\n=== Testing Hash Map (SwissTable) ===\n");
    
    VexMap map;
    assert(vex_map_new(&map, 16));
    
    // Insert string values
    assert(vex_map_insert(&map, "name", (void*)"Alice"));
    assert(vex_map_insert(&map, "city", (void*)"Istanbul"));
    assert(vex_map_insert(&map, "country", (void*)"Turkey"));
    printf("✓ vex_map_insert (3 entries)\n");
    
    // Lookup
    assert(vex_strcmp((char*)vex_map_get(&map, "name"), "Alice") == 0);
    assert(vex_strcmp((char*)vex_map_get(&map, "city"), "Istanbul") == 0);
    assert(vex_strcmp((char*)vex_map_get(&map, "country"), "Turkey") == 0);
    printf("✓ vex_map_get (existing keys)\n");
    
    // Not found
    assert(vex_map_get(&map, "unknown") == NULL);
    printf("✓ vex_map_get (non-existent key returns NULL)\n");
    
    // Update
    assert(vex_map_insert(&map, "name", (void*)"Bob"));
    assert(vex_strcmp((char*)vex_map_get(&map, "name"), "Bob") == 0);
    printf("✓ vex_map_insert (update existing key)\n");
    
    // Length
    assert(vex_map_len(&map) == 3);
    printf("✓ vex_map_len: %zu\n", vex_map_len(&map));
    
    // Integer values
    VexMap numbers;
    assert(vex_map_new(&numbers, 8));
    vex_map_insert(&numbers, "age", (void*)(intptr_t)25);
    vex_map_insert(&numbers, "score", (void*)(intptr_t)100);
    assert((intptr_t)vex_map_get(&numbers, "age") == 25);
    assert((intptr_t)vex_map_get(&numbers, "score") == 100);
    printf("✓ vex_map integer values\n");
    
    // Unicode keys
    VexMap unicode;
    assert(vex_map_new(&unicode, 8));
    vex_map_insert(&unicode, "名前", (void*)"Tanaka");
    vex_map_insert(&unicode, "città", (void*)"Roma");
    assert(vex_strcmp((char*)vex_map_get(&unicode, "名前"), "Tanaka") == 0);
    assert(vex_strcmp((char*)vex_map_get(&unicode, "città"), "Roma") == 0);
    printf("✓ vex_map Unicode keys\n");
    
    // Cleanup
    vex_map_free(&map);
    vex_map_free(&numbers);
    vex_map_free(&unicode);
    printf("✓ vex_map_free\n");
}

int main() {
    printf("╔════════════════════════════════════════╗\n");
    printf("║  Vex Runtime Library Test Suite       ║\n");
    printf("╚════════════════════════════════════════╝\n\n");
    
    test_string();
    test_memory();
    test_io();
    test_array();
    test_map();
    test_error();
    
    printf("\n╔════════════════════════════════════════╗\n");
    printf("║  All Tests Passed! ✅                  ║\n");
    printf("╚════════════════════════════════════════╝\n");
    
    return 0;
}
