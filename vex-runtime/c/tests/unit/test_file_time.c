// test_file_time.c - Tests for file I/O, mmap, and time operations
#include "vex.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

void test_file_operations() {
    printf("\n=== Testing File Operations ===\n");
    
    const char* test_file = "test_temp.txt";
    const char* test_data = "Hello, Vex Runtime!\nLine 2\nLine 3";
    size_t data_len = strlen(test_data);
    
    // Write file
    assert(vex_file_write_all(test_file, test_data, data_len));
    printf("✓ vex_file_write_all\n");
    
    // Check existence
    assert(vex_file_exists(test_file));
    printf("✓ vex_file_exists\n");
    
    // Read file
    size_t read_size = 0;
    char* read_data = vex_file_read_all(test_file, &read_size);
    assert(read_data != NULL);
    assert(read_size == data_len);
    assert(strcmp(read_data, test_data) == 0);
    vex_free(read_data);
    printf("✓ vex_file_read_all\n");
    
    // Open, read, write
    VexFile* file = vex_file_open(test_file, "r+");
    assert(file != NULL);
    
    char buffer[100];
    size_t bytes_read = vex_file_read(file, buffer, sizeof(buffer));
    assert(bytes_read == data_len);
    buffer[bytes_read] = '\0';
    printf("✓ vex_file_open + vex_file_read\n");
    
    // Get size
    int64_t size = vex_file_size(file);
    assert(size == (int64_t)data_len);
    printf("✓ vex_file_size: %lld bytes\n", size);
    
    // Seek and tell
    assert(vex_file_seek(file, 0, 0));  // SEEK_SET
    assert(vex_file_tell(file) == 0);
    assert(vex_file_seek(file, 7, 0));  // Go to byte 7
    assert(vex_file_tell(file) == 7);
    printf("✓ vex_file_seek + vex_file_tell\n");
    
    // Write
    const char* append_data = " APPENDED";
    assert(vex_file_seek(file, 0, 2));  // SEEK_END
    size_t bytes_written = vex_file_write(file, append_data, strlen(append_data));
    assert(bytes_written == strlen(append_data));
    vex_file_flush(file);
    printf("✓ vex_file_write + vex_file_flush\n");
    
    vex_file_close(file);
    printf("✓ vex_file_close\n");
    
    // Rename
    const char* new_name = "test_temp_renamed.txt";
    assert(vex_file_rename(test_file, new_name));
    assert(!vex_file_exists(test_file));
    assert(vex_file_exists(new_name));
    printf("✓ vex_file_rename\n");
    
    // Remove
    assert(vex_file_remove(new_name));
    assert(!vex_file_exists(new_name));
    printf("✓ vex_file_remove\n");
    
    // Directory operations
    const char* test_dir = "test_temp_dir";
    assert(vex_dir_create(test_dir));
    assert(vex_dir_exists(test_dir));
    printf("✓ vex_dir_create + vex_dir_exists\n");
    
    assert(vex_dir_remove(test_dir));
    assert(!vex_dir_exists(test_dir));
    printf("✓ vex_dir_remove\n");
}

void test_mmap() {
    printf("\n=== Testing Memory Mapped Files ===\n");
    
    // Create a test file
    const char* mmap_file = "test_mmap.dat";
    const char* test_data = "Memory mapped file contents! 0123456789";
    size_t data_len = strlen(test_data);
    
    assert(vex_file_write_all(mmap_file, test_data, data_len));
    
    // Memory map for reading
    VexMmap* mapping = vex_mmap_open(mmap_file, false);
    assert(mapping != NULL);
    assert(mapping->size == data_len);
    assert(mapping->writable == false);
    printf("✓ vex_mmap_open (read-only)\n");
    
    // Read from mapping
    char* mapped_data = (char*)mapping->addr;
    assert(strncmp(mapped_data, test_data, data_len) == 0);
    printf("✓ Read from mmap: %.20s...\n", mapped_data);
    
    // Advise sequential access
    assert(vex_mmap_advise(mapping, 1));  // SEQUENTIAL
    printf("✓ vex_mmap_advise\n");
    
    vex_mmap_close(mapping);
    printf("✓ vex_mmap_close\n");
    
    // Memory map for writing
    VexMmap* writable_mapping = vex_mmap_open(mmap_file, true);
    assert(writable_mapping != NULL);
    assert(writable_mapping->writable == true);
    printf("✓ vex_mmap_open (writable)\n");
    
    // Modify mapping
    char* write_data = (char*)writable_mapping->addr;
    write_data[0] = 'M';
    write_data[1] = 'O';
    write_data[2] = 'D';
    
    // Sync to disk
    assert(vex_mmap_sync(writable_mapping));
    printf("✓ vex_mmap_sync\n");
    
    vex_mmap_close(writable_mapping);
    
    // Verify modification
    size_t verify_size = 0;
    char* verify_data = vex_file_read_all(mmap_file, &verify_size);
    assert(verify_data[0] == 'M');
    assert(verify_data[1] == 'O');
    assert(verify_data[2] == 'D');
    vex_free(verify_data);
    printf("✓ Modifications persisted\n");
    
    // Anonymous mapping
    size_t alloc_size = 1024 * 1024;  // 1 MB
    void* anon_mem = vex_mmap_alloc(alloc_size);
    assert(anon_mem != NULL);
    
    // Write to it
    char* test_ptr = (char*)anon_mem;
    test_ptr[0] = 'A';
    test_ptr[alloc_size - 1] = 'Z';
    assert(test_ptr[0] == 'A');
    assert(test_ptr[alloc_size - 1] == 'Z');
    printf("✓ vex_mmap_alloc (anonymous)\n");
    
    // Protect (make read-only)
    assert(vex_mmap_protect(anon_mem, alloc_size, 1));  // READ-ONLY
    printf("✓ vex_mmap_protect\n");
    
    vex_mmap_free(anon_mem, alloc_size);
    printf("✓ vex_mmap_free\n");
    
    // Cleanup
    vex_file_remove(mmap_file);
}

void test_time() {
    printf("\n=== Testing Time Operations ===\n");
    
    // Current time
    int64_t now_ms = vex_time_now();
    int64_t now_us = vex_time_now_micros();
    int64_t now_ns = vex_time_now_nanos();
    
    assert(now_ms > 0);
    assert(now_us >= now_ms * 1000);  // Might be equal due to timing
    assert(now_ns >= now_us * 1000);  // Might be equal due to timing
    printf("✓ vex_time_now (ms): %lld\n", now_ms);
    printf("✓ vex_time_now_micros: %lld\n", now_us);
    printf("✓ vex_time_now_nanos: %lld\n", now_ns);
    
    // Monotonic time
    int64_t mono1 = vex_time_monotonic();
    vex_time_sleep(10);  // Sleep 10ms
    int64_t mono2 = vex_time_monotonic();
    assert(mono2 > mono1);
    printf("✓ vex_time_monotonic (delta: %lld ns)\n", mono2 - mono1);
    
    // DateTime conversion
    VexDateTime* dt = vex_time_to_datetime(now_ms);
    assert(dt != NULL);
    assert(dt->year >= 2025);
    assert(dt->month >= 1 && dt->month <= 12);
    assert(dt->day >= 1 && dt->day <= 31);
    printf("✓ vex_time_to_datetime: %04d-%02d-%02d %02d:%02d:%02d\n",
           dt->year, dt->month, dt->day, dt->hour, dt->minute, dt->second);
    
    // Local time
    VexDateTime* local_dt = vex_time_to_local_datetime(now_ms);
    assert(local_dt != NULL);
    printf("✓ vex_time_to_local_datetime: %04d-%02d-%02d %02d:%02d:%02d\n",
           local_dt->year, local_dt->month, local_dt->day,
           local_dt->hour, local_dt->minute, local_dt->second);
    
    // Timestamp conversion
    int64_t timestamp = vex_datetime_to_timestamp(dt);
    assert(timestamp > 0);
    printf("✓ vex_datetime_to_timestamp: %lld\n", timestamp);
    
    // Formatting
    char* formatted = vex_time_format(dt, "%Y-%m-%d %H:%M:%S");
    assert(formatted != NULL);
    printf("✓ vex_time_format: %s\n", formatted);
    vex_free(formatted);
    
    vex_datetime_free(dt);
    vex_datetime_free(local_dt);
    
    // High-resolution timer
    VexTimer* timer = vex_timer_start();
    assert(timer != NULL);
    printf("✓ vex_timer_start\n");
    
    // Do some work
    vex_time_sleep(50);  // Sleep 50ms
    
    int64_t elapsed_ns = vex_timer_elapsed_nanos(timer);
    int64_t elapsed_us = vex_timer_elapsed_micros(timer);
    int64_t elapsed_ms = vex_timer_elapsed_millis(timer);
    double elapsed_s = vex_timer_elapsed_seconds(timer);
    
    assert(elapsed_ms >= 50);
    printf("✓ vex_timer_elapsed: %lld ms (%.3f seconds)\n", elapsed_ms, elapsed_s);
    
    // Reset and measure again
    vex_timer_reset(timer);
    vex_time_sleep(20);
    int64_t elapsed2 = vex_timer_elapsed_millis(timer);
    assert(elapsed2 >= 20 && elapsed2 < 40);
    printf("✓ vex_timer_reset: %lld ms\n", elapsed2);
    
    vex_timer_free(timer);
    printf("✓ vex_timer_free\n");
}

int main() {
    printf("╔════════════════════════════════════════╗\n");
    printf("║  File I/O, Mmap, Time Test Suite      ║\n");
    printf("╚════════════════════════════════════════╝\n");
    
    test_file_operations();
    test_mmap();
    test_time();
    
    printf("\n╔════════════════════════════════════════╗\n");
    printf("║  All Tests Passed! ✅                  ║\n");
    printf("╚════════════════════════════════════════╝\n");
    
    return 0;
}
