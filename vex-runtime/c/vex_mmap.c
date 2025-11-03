// vex_mmap.c - Memory-mapped file operations
#include "vex.h"
#include <sys/mman.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>

// ============================================================================
// MEMORY MAPPED FILE OPERATIONS
// ============================================================================

VexMmap* vex_mmap_open(const char* path, bool writable) {
    if (!path) {
        vex_panic("vex_mmap_open: NULL path");
    }
    
    // Open file
    int flags = writable ? O_RDWR : O_RDONLY;
    int fd = open(path, flags);
    if (fd < 0) {
        return NULL;  // Failed to open
    }
    
    // Get file size
    struct stat st;
    if (fstat(fd, &st) != 0) {
        close(fd);
        return NULL;
    }
    
    size_t size = (size_t)st.st_size;
    if (size == 0) {
        close(fd);
        return NULL;  // Cannot mmap empty file
    }
    
    // Memory map the file
    int prot = writable ? (PROT_READ | PROT_WRITE) : PROT_READ;
    void* addr = mmap(NULL, size, prot, MAP_SHARED, fd, 0);
    
    close(fd);  // Can close fd after mmap
    
    if (addr == MAP_FAILED) {
        return NULL;
    }
    
    // Create VexMmap structure
    VexMmap* mapping = (VexMmap*)vex_malloc(sizeof(VexMmap));
    mapping->addr = addr;
    mapping->size = size;
    mapping->writable = writable;
    
    return mapping;
}

void vex_mmap_close(VexMmap* mapping) {
    if (!mapping) return;
    
    if (mapping->addr) {
        munmap(mapping->addr, mapping->size);
    }
    
    vex_free(mapping);
}

bool vex_mmap_sync(VexMmap* mapping) {
    if (!mapping || !mapping->addr) {
        vex_panic("vex_mmap_sync: invalid mapping");
    }
    
    if (!mapping->writable) {
        return true;  // No-op for read-only mappings
    }
    
    return msync(mapping->addr, mapping->size, MS_SYNC) == 0;
}

bool vex_mmap_advise(VexMmap* mapping, int advice) {
    if (!mapping || !mapping->addr) {
        vex_panic("vex_mmap_advise: invalid mapping");
    }
    
    // advice: 0=NORMAL, 1=SEQUENTIAL, 2=RANDOM, 3=WILLNEED, 4=DONTNEED
    int native_advice;
    switch (advice) {
        case 0: native_advice = MADV_NORMAL; break;
        case 1: native_advice = MADV_SEQUENTIAL; break;
        case 2: native_advice = MADV_RANDOM; break;
        case 3: native_advice = MADV_WILLNEED; break;
        case 4: native_advice = MADV_DONTNEED; break;
        default:
            vex_panic("vex_mmap_advise: invalid advice");
    }
    
    return madvise(mapping->addr, mapping->size, native_advice) == 0;
}

// ============================================================================
// ANONYMOUS MEMORY MAPPING (for large allocations)
// ============================================================================

void* vex_mmap_alloc(size_t size) {
    if (size == 0) {
        vex_panic("vex_mmap_alloc: zero size");
    }
    
    void* addr = mmap(NULL, size, PROT_READ | PROT_WRITE,
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    
    if (addr == MAP_FAILED) {
        return NULL;
    }
    
    return addr;
}

void vex_mmap_free(void* addr, size_t size) {
    if (!addr || size == 0) return;
    
    munmap(addr, size);
}

bool vex_mmap_protect(void* addr, size_t size, int prot) {
    if (!addr || size == 0) {
        vex_panic("vex_mmap_protect: invalid parameters");
    }
    
    // prot: 0=NONE, 1=READ, 2=WRITE, 3=READ|WRITE, 4=EXEC
    int native_prot = 0;
    if (prot & 1) native_prot |= PROT_READ;
    if (prot & 2) native_prot |= PROT_WRITE;
    if (prot & 4) native_prot |= PROT_EXEC;
    
    return mprotect(addr, size, native_prot) == 0;
}
