# Vex Runtime Library

Zero-overhead C runtime for the Vex programming language.

## 📁 Structure

```
vex-runtime/c/
├── vex.h              # Public API header
├── vex_string.c       # String operations (strlen, strcmp, strcpy, etc.)
├── vex_memory.c       # Memory operations (memcpy, memset, etc.)
├── vex_alloc.c        # Memory allocation (malloc wrapper)
├── vex_io.c           # I/O operations (print, println, printf)
├── vex_array.c        # Array operations (len, slice, append)
├── vex_error.c        # Error handling (panic, assert)
├── build.sh           # Build script (C → LLVM IR → .a)
└── build/             # Generated files
    ├── vex_runtime.ll    # LLVM IR (human-readable)
    ├── vex_runtime.bc    # LLVM bitcode
    └── libvex_runtime.a  # Static library
```

## 🚀 Building

### Prerequisites

- Clang (LLVM toolchain)
- llvm-link
- llvm-dis
- llc
- ar

### Build Command

```bash
chmod +x build.sh
./build.sh
```

This will:

1. Compile each `.c` file to LLVM IR (`.ll`)
2. Link all LLVM IR modules into one bitcode file (`.bc`)
3. Generate readable LLVM IR (`.ll`)
4. Create static library (`.a`)

## 📊 Implemented Functions (Phase 1)

### String Operations (5 functions)

- `vex_strlen()` - Get string length
- `vex_strcmp()` - Compare strings
- `vex_strcpy()` - Copy string
- `vex_strcat()` - Concatenate strings
- `vex_strdup()` - Duplicate string

### Memory Operations (4 functions)

- `vex_memcpy()` - Copy memory
- `vex_memmove()` - Move memory (overlapping-safe)
- `vex_memset()` - Set memory
- `vex_memcmp()` - Compare memory

### Memory Allocation (4 functions)

- `vex_malloc()` - Allocate memory
- `vex_calloc()` - Allocate and zero memory
- `vex_realloc()` - Reallocate memory
- `vex_free()` - Free memory

### I/O Operations (6 functions)

- `vex_print()` - Print to stdout
- `vex_println()` - Print to stdout with newline
- `vex_eprint()` - Print to stderr
- `vex_eprintln()` - Print to stderr with newline
- `vex_printf()` - Formatted print
- `vex_sprintf()` - Formatted string

### Array Operations (3 functions)

- `vex_array_len()` - Get array length
- `vex_array_slice()` - Create array slice
- `vex_array_append()` - Append element to array

### Error Handling (2 functions)

- `vex_panic()` - Panic with message and exit
- `vex_assert()` - Assert condition

**Total: 28 functions**

## 🎯 Performance

All functions are designed for **zero overhead**:

- Simple implementations that can be inlined by LLVM
- No unnecessary branching
- No hidden allocations (except where documented)
- Static linking eliminates dynamic library overhead

## 🔗 Integration with Vex Compiler

The generated `vex_runtime.ll` will be embedded into the Vex compiler:

```rust
// vex-compiler/src/codegen/mod.rs
const VEX_RUNTIME_IR: &str = include_str!("../../vex-runtime/c/build/vex_runtime.ll");

// Link at compile time
module.link_in_module(parse_ir(VEX_RUNTIME_IR))?;
```

## 📝 Usage in Vex Code

```vex
// These will compile to vex_* functions
let s = "hello";
let len = len(s);        // → vex_strlen()
print("Hello, world!");  // → vex_println()

let arr = [1, 2, 3];
let slice = arr[0..2];   // → vex_array_slice()
```

## 🚧 TODO (Phase 2+)

- [ ] File I/O operations
- [ ] HashMap implementation
- [ ] Async/concurrency primitives
- [ ] SIMD optimizations for memcpy/memset
- [ ] Type operations (sizeof, typeof)

## 📜 License

MIT License - Same as Vex Language
