# Vex C Libraries

C runtime implementations for Vex standard library.

## Structure

```
vex-clibs/
├── vex_io.c         # I/O operations (logger module)
├── vex_file.c       # File operations (fs module)
├── vex_path.c       # Path operations (fs module)
├── vex_time.c       # Time operations (time module)
└── vex_testing.c    # Test utilities (testing module)
```

## Purpose

These C files provide zero-cost runtime functions that Vex stdlib modules call via LLVM IR.

**Compilation:** Statically linked into `libvex.a` during Vex build.

## Vex Packages

The corresponding Vex packages are in `vex-libs/`:

- `vex-libs/logger/` → uses `vex_io.c`
- `vex-libs/fs/` → uses `vex_file.c`, `vex_path.c`, `vex_io.c`
- `vex-libs/time/` → uses `vex_time.c`
- `vex-libs/testing/` → uses `vex_testing.c`

## Build Integration

```bash
# Compile C runtime
cd vex-runtime/c && ./build.sh

# C functions are linked automatically when:
# 1. Vex code imports stdlib: import { logger } from "std";
# 2. Compiler generates LLVM IR with external function declarations
# 3. Linker resolves from libvex.a
```

## Adding New C Functions

1. Add function to appropriate `.c` file
2. Declare in `vex-runtime/c/vex.h`
3. Register in `vex-compiler/src/codegen_ast/builtins/stdlib.rs`
4. Use in Vex code via direct function call

Example:

```c
// vex-clibs/vex_io.c
void vex_print(const char* s) {
    write(STDOUT_FILENO, s, strlen(s));
}
```

```vex
// vex-libs/logger/logger.vx
fn info(msg: string) {
    print("[INFO] ");
    println(msg);
}
```

**Result:** Zero-overhead LLVM IR call to C function, no FFI wrapper needed.
