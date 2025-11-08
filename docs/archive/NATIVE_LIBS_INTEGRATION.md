# Native C/C++ Library Integration

**Feature:** Link C/C++ libraries via `vex.json`  
**Status:** ✅ Complete  
**Date:** November 8, 2025

---

## 📋 Overview

Vex now supports seamless integration with native C/C++ libraries through `vex.json` configuration. No need to manually specify linker flags - just declare your dependencies!

## 🎯 Features

1. **Dynamic Libraries** - Link system libraries (OpenSSL, zlib, etc.)
2. **Static Libraries** - Include `.a` files from your project
3. **C/C++ Sources** - Compile C code alongside Vex code
4. **Search Paths** - Specify custom library directories
5. **Compiler Flags** - Control C compilation with cflags
6. **Include Directories** - Add header search paths

---

## 📝 Configuration

### Basic Example

```json
{
  "name": "my-project",
  "version": "1.0.0",
  "native": {
    "libraries": ["ssl", "crypto"],
    "search_paths": ["/opt/homebrew/lib"],
    "sources": ["native/helper.c"],
    "include_dirs": ["native"],
    "cflags": ["-O2", "-Wall"]
  }
}
```

### Full Schema

```json
{
  "native": {
    // Dynamic libraries (e.g., libssl.so, libcrypto.dylib)
    "libraries": ["library_name"],

    // Library search paths
    "search_paths": ["/path/to/libs"],

    // Static library files
    "static_libs": ["./libs/libmylib.a"],

    // C/C++ source files to compile
    "sources": ["./native/helper.c"],

    // Compiler flags
    "cflags": ["-O2", "-Wall", "-std=c11"],

    // Header include directories
    "include_dirs": ["./include", "/usr/local/include"]
  }
}
```

---

## 🚀 Examples

### Example 1: Simple C Helper

**Project Structure:**

```
my_project/
├── vex.json
├── src/
│   └── main.vx
└── native/
    └── helper.c
```

**vex.json:**

```json
{
  "name": "native-demo",
  "version": "1.0.0",
  "native": {
    "sources": ["native/helper.c"],
    "include_dirs": ["native"],
    "cflags": ["-O2"]
  }
}
```

**native/helper.c:**

```c
#include <stdio.h>

int add_numbers(int a, int b) {
    return a + b;
}
```

**src/main.vx:**

```vex
extern "C" {
    fn add_numbers(a: i32, b: i32): i32;
}

fn main(): i32 {
    let result: i32 = add_numbers(10, 32);
    return result;  // Returns 42
}
```

**Compile & Run:**

```bash
vex compile src/main.vx
./vex-builds/main
# Exit code: 42 ✅
```

---

### Example 2: OpenSSL Integration

**vex.json:**

```json
{
  "name": "ssl-demo",
  "version": "1.0.0",
  "native": {
    "libraries": ["ssl", "crypto"],
    "search_paths": ["/opt/homebrew/lib", "/usr/local/lib"],
    "include_dirs": ["/opt/homebrew/include"]
  }
}
```

**src/main.vx:**

```vex
extern "C" {
    fn SHA256(data: *u8, len: u64, output: *u8): *u8;
    fn SSL_library_init(): i32;
}

fn main(): i32 {
    SSL_library_init();
    // Use OpenSSL functions...
    return 0;
}
```

---

### Example 3: Multiple Static Libraries

**vex.json:**

```json
{
  "name": "multi-lib",
  "version": "1.0.0",
  "native": {
    "static_libs": ["./libs/libmath.a", "./libs/libutils.a"],
    "sources": ["native/wrapper.c"],
    "cflags": ["-O3", "-fPIC"]
  }
}
```

---

## 🔧 How It Works

### 1. Configuration Parsing

- Reads `vex.json` in current directory
- Parses `native` section into `NativeConfig` struct

### 2. C Compilation

- Compiles all `.c` files in `sources` array
- Uses `clang` (or `gcc` as fallback)
- Outputs `.o` files to `.vex-build/native/`
- Applies `cflags` and `include_dirs`

### 3. Linking

- Adds compiled `.o` files to linker command
- Adds `-L` flags for `search_paths`
- Adds `-l` flags for `libraries`
- Includes `static_libs` directly

### 4. Execution Flow

```
vex.json
   ↓
NativeLinker.process()
   ↓
Compile C sources → helper.o
   ↓
Generate linker args: "helper.o -lssl -lcrypto"
   ↓
clang main.o helper.o -lssl -lcrypto -o program
   ↓
./program ✅
```

---

## 📁 File Structure

**Implementation:**

```
vex-pm/
├── src/
│   ├── manifest.rs          # NativeConfig struct
│   └── native_linker.rs     # NativeLinker (NEW!)
vex-cli/
└── src/
    └── main.rs              # Integration in compile/run commands
```

**Build Artifacts:**

```
project/
├── .vex-build/
│   └── native/
│       ├── helper.o
│       └── wrapper.o
└── vex-builds/
    └── main                 # Final binary
```

---

## 🎓 Advanced Usage

### Platform-Specific Libraries

```json
{
  "native": {
    "libraries": ["pthread"],
    "search_paths": [
      "/usr/lib/x86_64-linux-gnu", // Linux
      "/opt/homebrew/lib" // macOS
    ]
  }
}
```

### Custom Compiler

Set `CC` environment variable:

```bash
CC=gcc vex compile src/main.vx
```

### Debug Builds

```json
{
  "native": {
    "sources": ["native/debug.c"],
    "cflags": ["-g", "-O0", "-DDEBUG"]
  }
}
```

---

## ⚠️ Requirements

1. **Clang/GCC** - C compiler must be in PATH
2. **Libraries** - System libraries must be installed
3. **Headers** - Include files must be accessible

---

## 🐛 Troubleshooting

### Library Not Found

```
error: library not found for -lmylibrary
```

**Solution:** Add library path to `search_paths`:

```json
"search_paths": ["/path/to/lib"]
```

### Header Not Found

```
fatal error: 'myheader.h' file not found
```

**Solution:** Add include directory:

```json
"include_dirs": ["/path/to/include"]
```

### Compilation Failed

```
⚠️  Warning: Failed to process native config: Compilation failed
```

**Solution:** Check C code syntax, add `-Wall` to `cflags` for warnings

---

## 🎯 Use Cases

✅ **Crypto:** Link OpenSSL for encryption  
✅ **Database:** Link SQLite, PostgreSQL clients  
✅ **Graphics:** Link SDL2, OpenGL  
✅ **Network:** Link libcurl, libuv  
✅ **Compression:** Link zlib, bzip2  
✅ **Custom C code:** Compile alongside Vex

---

## 📊 Comparison

### Before (Manual)

```bash
# User has to know all this!
clang main.o helper.c \
  -I/opt/homebrew/include \
  -L/opt/homebrew/lib \
  -lssl -lcrypto \
  -o program
```

### After (vex.json)

```bash
# Just compile - vex.json handles everything!
vex compile src/main.vx
```

---

## 🚀 Future Enhancements

- [ ] **pkg-config integration** - Auto-detect library paths
- [ ] **CMake integration** - Build complex C++ projects
- [ ] **Cargo-like build scripts** - `build.vx` for custom logic
- [ ] **Precompiled headers** - Speed up C compilation
- [ ] **Cross-compilation** - Target different platforms

---

## ✅ Summary

**What:** C/C++ library integration via `vex.json`  
**Why:** Seamless FFI without manual linker flags  
**How:** NativeLinker compiles sources + generates linker args  
**Status:** Production ready ✅

**Example:**

```json
{
  "native": {
    "libraries": ["ssl"],
    "sources": ["helper.c"]
  }
}
```

```bash
vex compile src/main.vx  # Just works! 🎉
```
