// vex_fs.c - File System operations for Vex FS module
// Native implementation for fs stdlib module

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include <stdint.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <errno.h>

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

static void *fs_malloc(size_t size)
{
    extern void *vex_malloc(size_t);
    void *ptr = vex_malloc(size);
    if (!ptr && size > 0)
    {
        fprintf(stderr, "FATAL: Out of memory (requested %zu bytes)\n", size);
        abort();
    }
    return ptr;
}

static void fs_free(void *ptr)
{
    if (ptr)
    {
        extern void vex_free(void *);
        vex_free(ptr);
    }
}

// ============================================================================
// FILE OPERATIONS
// ============================================================================

char *vex_file_read_all_str(const char *path, size_t *out_size)
{
    if (!path)
    {
        return NULL;
    }

    int fd = open(path, O_RDONLY);
    if (fd < 0)
    {
        if (out_size)
            *out_size = 0;
        return NULL;
    }

    // Get file size
    struct stat st;
    if (fstat(fd, &st) < 0)
    {
        close(fd);
        if (out_size)
            *out_size = 0;
        return NULL;
    }

    size_t size = (size_t)st.st_size;
    char *buffer = (char *)fs_malloc(size + 1);

    ssize_t bytes_read = read(fd, buffer, size);
    close(fd);

    if (bytes_read < 0)
    {
        fs_free(buffer);
        if (out_size)
            *out_size = 0;
        return NULL;
    }

    buffer[bytes_read] = '\0';
    if (out_size)
        *out_size = (size_t)bytes_read;

    return buffer;
}

bool vex_file_write_all_str(const char *path, const void *data, size_t size)
{
    if (!path || !data)
    {
        return false;
    }

    int fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (fd < 0)
    {
        return false;
    }

    ssize_t bytes_written = write(fd, data, size);
    close(fd);

    return bytes_written == (ssize_t)size;
}

bool vex_file_exists_str(const char *path)
{
    if (!path)
    {
        return false;
    }

    struct stat st;
    return stat(path, &st) == 0 && S_ISREG(st.st_mode);
}

bool vex_file_remove_str(const char *path)
{
    if (!path)
    {
        return false;
    }

    return unlink(path) == 0;
}

bool vex_file_rename_str(const char *old_path, const char *new_path)
{
    if (!old_path || !new_path)
    {
        return false;
    }

    return rename(old_path, new_path) == 0;
}

bool vex_file_copy_str(const char *src, const char *dst)
{
    if (!src || !dst)
    {
        return false;
    }

    size_t size;
    char *data = vex_file_read_all_str(src, &size);
    if (!data)
    {
        return false;
    }

    bool success = vex_file_write_all_str(dst, data, size);
    fs_free(data);
    return success;
}

bool vex_file_move_str(const char *src, const char *dst)
{
    if (!src || !dst)
    {
        return false;
    }

    // Try rename first (atomic on same filesystem)
    if (rename(src, dst) == 0)
    {
        return true;
    }

    // If rename fails, copy then delete
    if (!vex_file_copy_str(src, dst))
    {
        return false;
    }

    return vex_file_remove_str(src);
}

// ============================================================================
// DIRECTORY OPERATIONS
// ============================================================================

bool vex_dir_create_str(const char *path)
{
    if (!path)
    {
        return false;
    }

    return mkdir(path, 0755) == 0;
}

bool vex_dir_remove_str(const char *path)
{
    if (!path)
    {
        return false;
    }

    return rmdir(path) == 0;
}

bool vex_dir_exists_str(const char *path)
{
    if (!path)
    {
        return false;
    }

    struct stat st;
    return stat(path, &st) == 0 && S_ISDIR(st.st_mode);
}

// ============================================================================
// STRING CONVERSION HELPERS
// ============================================================================

const char *vex_str_to_cstr(const char *s)
{
    return s; // Passthrough - Vex str and C char* are compatible
}

const char *vex_cstr_to_str(const char *ptr)
{
    return ptr; // Passthrough
}

// Note: vex_strlen is provided by vex_runtime
