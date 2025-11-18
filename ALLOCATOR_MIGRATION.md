# Vex Allocator Migration Plan

**Goal:** Replace system `malloc/free` with pluggable allocator system (mimalloc or system)

**Status:** 🚧 IN PROGRESS  
**Started:** 18 Kasım 2025  
**Estimated Time:** 4-5 hours total

---

## 🎯 Architecture

```
┌─────────────────────────────────────────────────┐
│           Vex Runtime C Code                    │
│  (vex_alloc.c, vex_file.c, swisstable.c, etc.) │
└──────────────────┬──────────────────────────────┘
                   │ Uses vex_alloc/vex_free
                   ↓
┌─────────────────────────────────────────────────┐
│         vex_allocator.h (Abstraction)           │
│  #define vex_alloc() → mi_malloc() / malloc()   │
└──────────────────┬──────────────────────────────┘
                   │ Compile-time selection
                   ↓
┌──────────────────┬──────────────────────────────┐
│    mimalloc      │      system (libc)           │
│    (default)     │      (minimal)               │
└──────────────────┴──────────────────────────────┘
```

---

## 📦 Phase 1: Mimalloc Integration (1 hour)

### 1.1 Download mimalloc

```bash
cd vex-runtime/c
mkdir -p allocators
cd allocators

# Download mimalloc v2.1.7 (latest stable)
wget https://github.com/microsoft/mimalloc/archive/refs/tags/v2.1.7.tar.gz
tar -xzf v2.1.7.tar.gz
mv mimalloc-2.1.7 mimalloc

# Clean up
rm v2.1.7.tar.gz

# Verify structure
ls mimalloc/
# → include/ src/ CMakeLists.txt README.md
```

### 1.2 Create directory structure

```
vex-runtime/c/
├── allocators/
│   ├── mimalloc/
│   │   ├── include/mimalloc.h
│   │   └── src/static.c          # Single-file build
│   ├── jemalloc/                 # TODO: Phase 2
│   └── README.md
└── vex_allocator.h               # NEW: Abstraction layer
```

**Files created:**
- `vex-runtime/c/allocators/` (directory)
- `vex-runtime/c/allocators/README.md`
- `vex-runtime/c/vex_allocator.h`

---

## 🔧 Phase 2: Central Allocator Header (30 min)

### 2.1 Create vex_allocator.h

**File:** `vex-runtime/c/vex_allocator.h`

```c
#ifndef VEX_ALLOCATOR_H
#define VEX_ALLOCATOR_H

#include <stddef.h>

// ============================================================================
// VEX ALLOCATOR ABSTRACTION LAYER
// Centralized allocation API - all C code uses these macros
// Selected at compile time via -DVEX_ALLOCATOR_* flag
// ============================================================================

#ifdef VEX_ALLOCATOR_MIMALLOC
#include "allocators/mimalloc/include/mimalloc.h"
#define vex_alloc(size)                mi_malloc(size)
#define vex_alloc_aligned(size, align) mi_malloc_aligned(size, align)
#define vex_calloc(n, size)            mi_calloc(n, size)
#define vex_realloc(ptr, size)         mi_realloc(ptr, size)
#define vex_free(ptr)                  mi_free(ptr)

#else  // VEX_ALLOCATOR_SYSTEM (default fallback - libc)
#include <stdlib.h>
#define vex_alloc(size)                malloc(size)
#define vex_alloc_aligned(size, align) aligned_alloc(align, size)
#define vex_calloc(n, size)            calloc(n, size)
#define vex_realloc(ptr, size)         realloc(ptr, size)
#define vex_free(ptr)                  free(ptr)
#endif

// Deprecated aliases (for backward compatibility)
#define vex_malloc vex_alloc

// Stats and debugging (optional, for mimalloc)
#ifdef VEX_ALLOCATOR_MIMALLOC
#define vex_stats_print() mi_stats_print(NULL)
#else
#define vex_stats_print() ((void)0)
#endif

#endif // VEX_ALLOCATOR_H
```

### 2.2 Create allocators README

**File:** `vex-runtime/c/allocators/README.md`

```markdown
# Vex Runtime Allocators

This directory contains third-party allocator implementations.

## Supported Allocators

### mimalloc (Default)
- **Version:** 2.1.7
- **Source:** https://github.com/microsoft/mimalloc
- **License:** MIT
- **Features:** Fast, secure mode, thread-safe, pure C
- **Binary Size:** ~50KB

### system (Fallback)
- **Implementation:** Uses libc malloc/free
- **Binary Size:** 0 (OS-provided)
- **Use Case:** Minimal embedded systems

## Build Selection

```bash
# Default (mimalloc)
VEX_ALLOCATOR=mimalloc cargo build --release

# System malloc
VEX_ALLOCATOR=system cargo build --release
```
```

---

## 🔄 Phase 3: C Code Migration (2 hours)

### 3.1 Files to Update (25 files)

#### Core Runtime (Priority 1)
- [ ] `vex-runtime/c/vex_alloc.c` - Arena allocator fallback
- [ ] `vex-runtime/c/vex_file.c` - File I/O buffers
- [ ] `vex-runtime/c/vex_testing.c` - Test framework
- [ ] `vex-runtime/c/vex_cmd.c` - Command execution
- [ ] `vex-runtime/c/vex_sync.c` - Arc/Rc/Mutex

#### Collections (Priority 1)
- [ ] `vex-runtime/c/swisstable/vex_swisstable.c`
- [ ] `vex-runtime/c/swisstable/vex_swisstable_v2.c`
- [ ] `vex-runtime/c/swisstable/vex_swisstable_v3.c`
- [ ] `vex-runtime/c/swisstable/vex_swisstable_bench.c`
- [ ] `vex-runtime/c/swisstable/vex_swisstable_bench_v2.c`
- [ ] `vex-runtime/c/swisstable/vex_swisstable_bench_v3.c`

#### Async Runtime (Priority 2)
- [ ] `vex-runtime/c/async_runtime/src/lockfree_queue.c`
- [ ] `vex-runtime/c/async_runtime/src/poller_epoll.c`
- [ ] `vex-runtime/c/async_runtime/src/poller_kqueue.c`
- [ ] `vex-runtime/c/async_runtime/src/poller_iocp.c`
- [ ] `vex-runtime/c/async_runtime/src/poller_io_uring.c`
- [ ] `vex-runtime/c/async_runtime/src/poller_select.c`
- [ ] `vex-runtime/c/async_runtime/test_poller.c`
- [ ] `vex-runtime/c/async_runtime/test_queue.c`
- [ ] `vex-runtime/c/async_runtime/test_runtime.c`

#### Crypto/TLS (Priority 3)
- [ ] `vex-runtime/c/vex_openssl/src/vex_tls_openssl.c`
- [ ] `vex-runtime/c/vex_openssl/src/vex_crypto_openssl.c`

#### Headers (Update includes)
- [ ] `vex-runtime/c/vex.h`
- [ ] `vex-runtime/c/vex_macros.h`

### 3.2 Migration Pattern

**Before:**
```c
#include <stdlib.h>

void* buffer = malloc(size);
free(buffer);
```

**After:**
```c
#include "vex_allocator.h"

void* buffer = vex_alloc(size);
vex_free(buffer);
```

### 3.3 Automated Migration Script

```bash
#!/bin/bash
# migrate_allocator.sh

cd vex-runtime/c

# Backup all files
find . -name "*.c" -o -name "*.h" | xargs -I {} cp {} {}.bak

# Replace malloc/calloc/realloc/free
find . -name "*.c" | xargs sed -i '' \
  -e 's/\bmalloc(/vex_alloc(/g' \
  -e 's/\bcalloc(/vex_calloc(/g' \
  -e 's/\brealloc(/vex_realloc(/g' \
  -e 's/\bfree(/vex_free(/g'

# Replace stdlib.h with vex_allocator.h (but keep if needed for other functions)
find . -name "*.c" | while read file; do
  if grep -q "malloc\|calloc\|realloc\|free" "$file"; then
    sed -i '' '1i\
#include "vex_allocator.h"
' "$file"
  fi
done

echo "Migration complete. Review changes and commit."
```

---

## 🏗️ Phase 4: Build System Update (1 hour)

### 4.1 Update build.rs

**File:** `vex-runtime/build.rs`

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    // Select allocator (default: mimalloc)
    let allocator = env::var("VEX_ALLOCATOR")
        .unwrap_or_else(|_| "mimalloc".to_string());
    
    println!("cargo:rerun-if-env-changed=VEX_ALLOCATOR");
    println!("cargo:rustc-cfg=allocator=\"{}\"", allocator);
    println!("cargo:warning=Using allocator: {}", allocator);
    
    let mut build = cc::Build::new();
    build.include("c");
    
    // Compile selected allocator
    match allocator.as_str() {
        "mimalloc" => {
            let mimalloc_dir = PathBuf::from("c/allocators/mimalloc");
            
            cc::Build::new()
                .file(mimalloc_dir.join("src/static.c"))
                .include(mimalloc_dir.join("include"))
                .define("MI_SECURE", "1")       // Security features
                .define("MI_PADDING", "1")      // Overflow detection
                .define("MI_OVERRIDE", "0")     // Don't override system malloc
                .opt_level(3)
                .compile("mimalloc");
            
            build.define("VEX_ALLOCATOR_MIMALLOC", None);
        }
        "system" => {
            build.define("VEX_ALLOCATOR_SYSTEM", None);
        }
        _ => panic!("Unknown allocator: {}. Use: mimalloc or system", allocator),
    }
    
    // ... rest of build.rs (existing code)
}
```

### 4.2 Update Cargo.toml

**File:** `vex-runtime/Cargo.toml`

```toml
[package]
name = "vex-runtime"
version = "0.2.0"

[features]
default = ["mimalloc"]
mimalloc = []
system-alloc = []

[build-dependencies]
cc = "1.0"

# ... rest of Cargo.toml
```

---

## 🧪 Phase 5: Testing (1 hour)

### 5.1 Test with each allocator

```bash
# Test 1: mimalloc (default)
VEX_ALLOCATOR=mimalloc cargo build --release
~/.cargo/target/release/vex run examples/hello.vx

# Test 2: system malloc
VEX_ALLOCATOR=system cargo build --release
~/.cargo/target/release/vex run examples/hello.vx

# Test 3: Run test suite
VEX_ALLOCATOR=mimalloc ./test_all.sh
VEX_ALLOCATOR=system ./test_all.sh
```

### 5.2 Benchmark comparison

```bash
# Create benchmark script
cat > bench_allocators.sh << 'EOF'
#!/bin/bash

echo "=== Allocator Benchmark ==="

for alloc in mimalloc system; do
  echo ""
  echo "Testing: $alloc"
  VEX_ALLOCATOR=$alloc cargo build --release 2>/dev/null
  
  # Run allocation-heavy test
  time ~/.cargo/target/release/vex run examples/stdlib_integration_comprehensive.vx
done
EOF

chmod +x bench_allocators.sh
./bench_allocators.sh
```

### 5.3 Memory leak check (with mimalloc stats)

```c
// Add to main.rs or test
#[cfg(allocator = "mimalloc")]
extern "C" {
    fn mi_stats_print(out: *mut std::os::raw::c_void);
}

#[cfg(allocator = "mimalloc")]
fn print_allocator_stats() {
    unsafe { mi_stats_print(std::ptr::null_mut()); }
}
```

---

## 📊 Phase 6: Documentation (30 min)

### 6.1 Update README.md

Add section:
```markdown
## Memory Allocator

Vex uses **mimalloc** by default for superior performance and security.

### Changing Allocator

```bash
# Use mimalloc (default - recommended)
cargo build --release

# Use system malloc (minimal binary size)
VEX_ALLOCATOR=system cargo build --release

# Use jemalloc (long-running applications)
VEX_ALLOCATOR=jemalloc cargo build --release
```

### Allocator Comparison

| Allocator | Speed | Binary Size | Use Case |
|-----------|-------|-------------|----------|
| **mimalloc** | ⭐⭐⭐⭐⭐ | +50KB | Default (recommended) |
| **jemalloc** | ⭐⭐⭐⭐ | +200KB | Long-running servers |
| **system** | ⭐⭐⭐ | +0KB | Embedded/minimal |
```

### 6.2 Create ALLOCATOR.md guide

Detailed allocator documentation with:
- Architecture overview
- Performance benchmarks
- Security features
- Migration guide

---

## ✅ Success Criteria

- [ ] All 25 C files migrated to `vex_alloc/vex_free`
- [ ] Build succeeds with both allocators (mimalloc, system)
- [ ] All tests pass with mimalloc
- [ ] All tests pass with system allocator
- [ ] No memory leaks (verified with mimalloc stats)
- [ ] Performance benchmarks documented
- [ ] Documentation updated

---

## 🚀 Execution Order

1. ✅ Create this migration plan document
2. ✅ Download and integrate mimalloc
3. ✅ Create `vex_allocator.h`
4. ⏳ Migrate C files (automated + manual review)
5. ⏳ Update build system
6. ⏳ Test all allocators
7. ⏳ Benchmark performance
8. ⏳ Update documentation
9. ✅ Commit and deploy

---

## 📝 Notes

- **Binary size impact:** +50KB with mimalloc (acceptable for 15-30% perf gain)
- **Security:** mimalloc secure mode prevents heap exploits
- **Compatibility:** Both allocators use same API via macros
- **Pure C:** No C++ dependencies, no jemalloc complexity
- **Future:** Can add tcmalloc, snmalloc, or custom allocator if needed

**Last Updated:** 18 Kasım 2025
