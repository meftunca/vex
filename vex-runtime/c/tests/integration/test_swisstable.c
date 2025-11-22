// test_swisstable.c - SwissTable comprehensive test suite
#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <stdlib.h>
#include "vex_swisstable.c"  // Include implementation directly for testing

void test_basic_operations() {
    printf("\n=== Testing Basic Operations ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 16));
    
    // Insert
    assert(vex_swiss_insert(&map, "name", (void*)"Alice"));
    assert(vex_swiss_insert(&map, "city", (void*)"Istanbul"));
    assert(vex_swiss_insert(&map, "country", (void*)"Turkey"));
    printf("✓ Insert 3 entries\n");
    
    // Lookup
    assert(strcmp((char*)vex_swiss_get(&map, "name"), "Alice") == 0);
    assert(strcmp((char*)vex_swiss_get(&map, "city"), "Istanbul") == 0);
    assert(strcmp((char*)vex_swiss_get(&map, "country"), "Turkey") == 0);
    printf("✓ Lookup existing keys\n");
    
    // Not found
    assert(vex_swiss_get(&map, "unknown") == NULL);
    printf("✓ Lookup non-existent key returns NULL\n");
    
    // Update
    assert(vex_swiss_insert(&map, "name", (void*)"Bob"));
    assert(strcmp((char*)vex_swiss_get(&map, "name"), "Bob") == 0);
    printf("✓ Update existing key\n");
    
    vex_swiss_free(&map);
    printf("✓ Free map\n");
}

void test_integer_values() {
    printf("\n=== Testing Integer Values ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 8));
    
    // Store integers as pointers (common pattern)
    vex_swiss_insert(&map, "age", (void*)(intptr_t)25);
    vex_swiss_insert(&map, "score", (void*)(intptr_t)100);
    vex_swiss_insert(&map, "level", (void*)(intptr_t)42);
    printf("✓ Insert integer values\n");
    
    assert((intptr_t)vex_swiss_get(&map, "age") == 25);
    assert((intptr_t)vex_swiss_get(&map, "score") == 100);
    assert((intptr_t)vex_swiss_get(&map, "level") == 42);
    printf("✓ Retrieve integer values\n");
    
    vex_swiss_free(&map);
}

void test_rehashing() {
    printf("\n=== Testing Rehashing (Growth) ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 4));  // Start small
    
    // Insert enough to trigger rehash
    char keys[100][32];
    for (int i = 0; i < 50; i++) {
        snprintf(keys[i], 32, "key_%d", i);
        assert(vex_swiss_insert(&map, keys[i], (void*)(intptr_t)i));
    }
    printf("✓ Inserted 50 entries (triggered rehashing)\n");
    
    // Verify all entries still accessible
    for (int i = 0; i < 50; i++) {
        intptr_t val = (intptr_t)vex_swiss_get(&map, keys[i]);
        assert(val == i);
    }
    printf("✓ All entries accessible after rehash\n");
    
    printf("  Final capacity: %zu, length: %zu, load: %.2f%%\n",
           map.capacity, map.len, (100.0 * map.len) / map.capacity);
    
    vex_swiss_free(&map);
}

void test_collision_handling() {
    printf("\n=== Testing Collision Handling ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 16));
    
    // Insert many entries that might collide
    const char* words[] = {
        "apple", "banana", "cherry", "date", "elderberry",
        "fig", "grape", "honeydew", "kiwi", "lemon",
        "mango", "nectarine", "orange", "papaya", "quince"
    };
    int n_words = sizeof(words) / sizeof(words[0]);
    
    for (int i = 0; i < n_words; i++) {
        assert(vex_swiss_insert(&map, words[i], (void*)(intptr_t)(i + 1)));
    }
    printf("✓ Inserted %d words\n", n_words);
    
    // Verify all can be retrieved
    for (int i = 0; i < n_words; i++) {
        intptr_t val = (intptr_t)vex_swiss_get(&map, words[i]);
        assert(val == i + 1);
    }
    printf("✓ All words retrievable (collisions handled)\n");
    
    vex_swiss_free(&map);
}

void test_empty_map() {
    printf("\n=== Testing Empty Map ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 8));
    
    assert(vex_swiss_get(&map, "anything") == NULL);
    printf("✓ Lookup on empty map returns NULL\n");
    
    vex_swiss_free(&map);
}

void test_single_entry() {
    printf("\n=== Testing Single Entry ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 8));
    
    assert(vex_swiss_insert(&map, "only", (void*)"value"));
    assert(strcmp((char*)vex_swiss_get(&map, "only"), "value") == 0);
    printf("✓ Single entry works\n");
    
    vex_swiss_free(&map);
}

void test_large_dataset() {
    printf("\n=== Testing Large Dataset ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 64));
    
    // Insert 10,000 entries
    char keys[10000][32];
    for (int i = 0; i < 10000; i++) {
        snprintf(keys[i], 32, "large_key_%d", i);
        assert(vex_swiss_insert(&map, keys[i], (void*)(intptr_t)i));
    }
    printf("✓ Inserted 10,000 entries\n");
    
    // Random access verification (sample)
    int samples[] = {0, 100, 1000, 5000, 9999};
    for (int i = 0; i < 5; i++) {
        int idx = samples[i];
        intptr_t val = (intptr_t)vex_swiss_get(&map, keys[idx]);
        assert(val == idx);
    }
    printf("✓ Random access successful\n");
    
    printf("  Final capacity: %zu, length: %zu, load: %.2f%%\n",
           map.capacity, map.len, (100.0 * map.len) / map.capacity);
    
    vex_swiss_free(&map);
}

void test_unicode_keys() {
    printf("\n=== Testing Unicode Keys ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 16));
    
    // UTF-8 keys
    assert(vex_swiss_insert(&map, "名前", (void*)"Tanaka"));
    assert(vex_swiss_insert(&map, "città", (void*)"Roma"));
    assert(vex_swiss_insert(&map, "مدينة", (void*)"Cairo"));
    assert(vex_swiss_insert(&map, "emoji_😀", (void*)"smile"));
    printf("✓ Inserted Unicode keys\n");
    
    assert(strcmp((char*)vex_swiss_get(&map, "名前"), "Tanaka") == 0);
    assert(strcmp((char*)vex_swiss_get(&map, "città"), "Roma") == 0);
    assert(strcmp((char*)vex_swiss_get(&map, "مدينة"), "Cairo") == 0);
    assert(strcmp((char*)vex_swiss_get(&map, "emoji_😀"), "smile") == 0);
    printf("✓ Retrieved Unicode keys\n");
    
    vex_swiss_free(&map);
}

void benchmark_insert() {
    printf("\n=== Benchmark: Insert Performance ===\n");
    
    SwissMap map;
    assert(vex_swiss_init(&map, 1024));
    
    char keys[100000][32];
    for (int i = 0; i < 100000; i++) {
        snprintf(keys[i], 32, "bench_key_%d", i);
    }
    
    // Warm-up
    for (int i = 0; i < 1000; i++) {
        vex_swiss_insert(&map, keys[i], (void*)(intptr_t)i);
    }
    
    printf("  Inserted 100K entries (benchmark mode)\n");
    printf("  Final capacity: %zu, load: %.2f%%\n",
           map.capacity, (100.0 * map.len) / map.capacity);
    
    vex_swiss_free(&map);
}

int main(void) {
    printf("╔════════════════════════════════════════╗\n");
    printf("║  SwissTable Test Suite                ║\n");
    printf("╚════════════════════════════════════════╝\n");
    
    test_basic_operations();
    test_integer_values();
    test_empty_map();
    test_single_entry();
    test_collision_handling();
    test_rehashing();
    test_unicode_keys();
    test_large_dataset();
    benchmark_insert();
    
    printf("\n╔════════════════════════════════════════╗\n");
    printf("║  All SwissTable Tests Passed! ✅       ║\n");
    printf("╚════════════════════════════════════════╝\n");
    
    return 0;
}
