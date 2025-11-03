// vex_file.c - File I/O operations for Vex runtime
#include "vex.h"
#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <errno.h>
#include <stdio.h>

// ============================================================================
// FILE OPERATIONS
// ============================================================================

VexFile* vex_file_open(const char* path, const char* mode) {
    if (!path || !mode) {
        vex_panic("vex_file_open: NULL path or mode");
    }
    
    int flags = 0;
    int perms = 0644;  // rw-r--r--
    
    // Parse mode string (fopen-style)
    switch (mode[0]) {
        case 'r':
            flags = O_RDONLY;
            if (mode[1] == '+') flags = O_RDWR;
            break;
        case 'w':
            flags = O_WRONLY | O_CREAT | O_TRUNC;
            if (mode[1] == '+') flags = O_RDWR | O_CREAT | O_TRUNC;
            break;
        case 'a':
            flags = O_WRONLY | O_CREAT | O_APPEND;
            if (mode[1] == '+') flags = O_RDWR | O_CREAT | O_APPEND;
            break;
        default:
            vex_panic("vex_file_open: invalid mode");
    }
    
    int fd = open(path, flags, perms);
    if (fd < 0) {
        return NULL;  // Failed to open
    }
    
    VexFile* file = (VexFile*)vex_malloc(sizeof(VexFile));
    file->fd = fd;
    file->path = vex_strdup(path);
    file->is_open = true;
    
    return file;
}

void vex_file_close(VexFile* file) {
    if (!file) return;
    
    if (file->is_open && file->fd >= 0) {
        close(file->fd);
        file->is_open = false;
    }
    
    if (file->path) {
        vex_free((void*)file->path);
    }
    
    vex_free(file);
}

size_t vex_file_read(VexFile* file, void* buffer, size_t size) {
    if (!file || !file->is_open) {
        vex_panic("vex_file_read: file not open");
    }
    
    if (!buffer || size == 0) return 0;
    
    ssize_t bytes_read = read(file->fd, buffer, size);
    if (bytes_read < 0) {
        return 0;  // Error reading
    }
    
    return (size_t)bytes_read;
}

size_t vex_file_write(VexFile* file, const void* buffer, size_t size) {
    if (!file || !file->is_open) {
        vex_panic("vex_file_write: file not open");
    }
    
    if (!buffer || size == 0) return 0;
    
    ssize_t bytes_written = write(file->fd, buffer, size);
    if (bytes_written < 0) {
        return 0;  // Error writing
    }
    
    return (size_t)bytes_written;
}

bool vex_file_seek(VexFile* file, int64_t offset, int whence) {
    if (!file || !file->is_open) {
        vex_panic("vex_file_seek: file not open");
    }
    
    // whence: 0=SEEK_SET, 1=SEEK_CUR, 2=SEEK_END
    int native_whence;
    switch (whence) {
        case 0: native_whence = SEEK_SET; break;
        case 1: native_whence = SEEK_CUR; break;
        case 2: native_whence = SEEK_END; break;
        default:
            vex_panic("vex_file_seek: invalid whence");
    }
    
    off_t result = lseek(file->fd, offset, native_whence);
    return result >= 0;
}

int64_t vex_file_tell(VexFile* file) {
    if (!file || !file->is_open) {
        vex_panic("vex_file_tell: file not open");
    }
    
    off_t pos = lseek(file->fd, 0, SEEK_CUR);
    return (int64_t)pos;
}

int64_t vex_file_size(VexFile* file) {
    if (!file || !file->is_open) {
        vex_panic("vex_file_size: file not open");
    }
    
    struct stat st;
    if (fstat(file->fd, &st) != 0) {
        return -1;
    }
    
    return (int64_t)st.st_size;
}

bool vex_file_flush(VexFile* file) {
    if (!file || !file->is_open) {
        vex_panic("vex_file_flush: file not open");
    }
    
    return fsync(file->fd) == 0;
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

char* vex_file_read_all(const char* path, size_t* out_size) {
    if (!path) {
        vex_panic("vex_file_read_all: NULL path");
    }
    
    VexFile* file = vex_file_open(path, "r");
    if (!file) return NULL;
    
    int64_t size = vex_file_size(file);
    if (size < 0) {
        vex_file_close(file);
        return NULL;
    }
    
    char* buffer = (char*)vex_malloc((size_t)size + 1);  // +1 for null terminator
    size_t bytes_read = vex_file_read(file, buffer, (size_t)size);
    buffer[bytes_read] = '\0';  // Null terminate
    
    if (out_size) *out_size = bytes_read;
    
    vex_file_close(file);
    return buffer;
}

bool vex_file_write_all(const char* path, const void* data, size_t size) {
    if (!path) {
        vex_panic("vex_file_write_all: NULL path");
    }
    
    VexFile* file = vex_file_open(path, "w");
    if (!file) return false;
    
    size_t bytes_written = vex_file_write(file, data, size);
    bool success = (bytes_written == size);
    
    vex_file_close(file);
    return success;
}

bool vex_file_exists(const char* path) {
    if (!path) return false;
    
    struct stat st;
    return stat(path, &st) == 0;
}

bool vex_file_remove(const char* path) {
    if (!path) {
        vex_panic("vex_file_remove: NULL path");
    }
    
    return unlink(path) == 0;
}

bool vex_file_rename(const char* old_path, const char* new_path) {
    if (!old_path || !new_path) {
        vex_panic("vex_file_rename: NULL path");
    }
    
    return rename(old_path, new_path) == 0;
}

// ============================================================================
// DIRECTORY OPERATIONS
// ============================================================================

bool vex_dir_create(const char* path) {
    if (!path) {
        vex_panic("vex_dir_create: NULL path");
    }
    
    return mkdir(path, 0755) == 0;
}

bool vex_dir_remove(const char* path) {
    if (!path) {
        vex_panic("vex_dir_remove: NULL path");
    }
    
    return rmdir(path) == 0;
}

bool vex_dir_exists(const char* path) {
    if (!path) return false;
    
    struct stat st;
    if (stat(path, &st) != 0) return false;
    return S_ISDIR(st.st_mode);
}
