# Vex Package Manager - Quick Start Guide

**Version:** 0.1.0 (MVP)  
**Status:** 🚧 In Development

---

## Installation

Package manager is built into the `vex` binary:

```bash
# Install Vex (includes package manager)
cargo install --path vex-cli

# Verify installation
vex --version
# vex 0.2.0 (std 0.2.0, pm 0.1.0)
```

---

## Quick Start

### 1. Create a New Project

```bash
vex new my-project
cd my-project
```

**Generated Structure:**

```
my-project/
├── vex.json              # Package manifest
├── src/
│   └── lib.vx            # Main entrypoint
└── tests/
    └── lib_test.vx       # Tests
```

**vex.json:**

```json
{
  "name": "my-project",
  "version": "0.1.0",
  "description": "A Vex project",
  "authors": ["Your Name <email@example.com>"],
  "license": "MIT",

  "dependencies": {},

  "vex": {
    "borrowChecker": "strict"
  }
}
```

---

### 2. Add Dependencies

```bash
# Add a package from GitHub
vex add github.com/vex-lang/json@v1.2.0

# Add latest version
vex add github.com/vex-lang/http

# Add from GitLab
vex add gitlab:company/internal-lib@v1.0.0
```

**Updated vex.json:**

```json
{
  "dependencies": {
    "github.com/vex-lang/json": "v1.2.0",
    "github.com/vex-lang/http": "latest"
  }
}
```

---

### 3. Use Dependencies

**src/lib.vx:**

```vex
import { parse, stringify } from "github.com/vex-lang/json";
import { println } from "std/io";

export fn process_data(input: string): string {
    let data = parse(input);
    println("Parsed:", data);
    return stringify(data);
}
```

---

### 4. Build & Run

```bash
# Development build
vex build

# Run
vex run

# Production build
vex build --release

# Clean
vex clean
```

---

## Platform-Specific Code

### File Naming Convention

```
{file}.testing.vx             # Test variant (mocks/fixtures)
{file}.{os}.{instruction}.vx  # Most specific
{file}.{instruction}.vx       # Instruction-specific
{file}.{os}.vx                # OS-specific
{file}.vx                     # Generic fallback
```

### Example: Cross-Platform File I/O

```
src/
├── file_io.vx           # Generic fallback
├── file_io.linux.vx     # Linux (epoll)
├── file_io.macos.vx     # macOS (kqueue)
├── file_io.windows.vx   # Windows (IOCP)
└── file_io.testing.vx   # Mock for tests
```

**Compilation:**

```bash
# Automatically selects platform file
vex build
# On Linux: uses file_io.linux.vx
# On macOS: uses file_io.macos.vx

# Test mode uses testing variant
vex test
# Uses file_io.testing.vx (with mocks)

# Cross-compile
vex build --target=windows
# Uses file_io.windows.vx
```

**Compilation:**

```bash
# Automatically selects platform file
vex build
# On Linux: uses file_io.linux.vx
# On macOS: uses file_io.macos.vx

# Cross-compile
vex build --target=windows
# Uses file_io.windows.vx
```

---

## Standard Library

No dependency needed - `std` is built-in!

```vex
import { http, json, io } from "std";

fn main(): i32 {
    let response = http.get("https://api.example.com");
    let data = json.parse(response.body);
    io.println("Result:", data);
    return 0;
}
```

**Available Modules:**

- `std/io` - I/O operations
- `std/http` - HTTP client/server
- `std/json` - JSON parsing
- `std/crypto` - Cryptography (SIMD-optimized)
- `std/net` - Networking (platform-specific)
- `std/time` - Time operations

---

## CLI Reference

### Project Management

```bash
vex new <name>              # Create new project
vex init                    # Initialize vex.json in existing dir
```

### Dependency Management

```bash
vex add <package>[@version] # Add dependency
vex remove <package>        # Remove dependency
vex update                  # Update all dependencies
vex list                    # List all dependencies
```

### Build & Run

```bash
vex build                   # Build (development)
vex build --release         # Build (production)
vex build --target=<target> # Cross-compile
vex run                     # Build and run
vex test                    # Run tests
vex clean                   # Clean build cache
```

### Supported Targets

**Instruction Sets:**

- `x64` - x86-64 (Intel, AMD)
- `arm64` - ARM64 (Apple Silicon, ARMv8)
- `wasm` - WebAssembly (browser)
- `wasi` - WASI (server-side WASM)

**Operating Systems:**

- `linux` - Linux
- `macos` - macOS
- `windows` - Windows

**Examples:**

```bash
vex build --target=x64
vex build --target=macos-arm64
vex build --target=wasm
```

---

## vex.json Reference

### Basic Structure

```json
{
  "name": "my-package",
  "version": "1.0.0",
  "description": "Description",
  "authors": ["Name <email>"],
  "license": "MIT",

  "dependencies": {
    "github.com/user/repo": "v1.0.0"
  },

  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm"]
  }
}
```

### Build Profiles

```json
{
  "profiles": {
    "development": {
      "optimizationLevel": 0,
      "debugSymbols": true
    },
    "production": {
      "optimizationLevel": 3,
      "debugSymbols": false
    }
  }
}
```

---

## Examples

### 1. Simple Library

```bash
vex new my-lib
cd my-lib
```

**src/lib.vx:**

```vex
export fn greet(name: string): string {
    return "Hello, " + name + "!";
}
```

**tests/lib_test.vx:**

```vex
import { greet } from "../src/lib";
import { assert } from "std/testing";

fn test_greet(): i32 {
    let result = greet("World");
    assert(result == "Hello, World!");
    return 0;
}
```

```bash
vex test
# ✅ test_greet passed
```

---

### 2. HTTP Server

```bash
vex new my-server
vex add github.com/vex-lang/http@v2.0.0
```

**src/lib.vx:**

```vex
import { Server } from "github.com/vex-lang/http";
import { println } from "std/io";

export fn start_server(): i32 {
    let server = Server.new("127.0.0.1:8080");

    server.get("/", |req| {
        return "Hello, Vex!";
    });

    println("Server running on http://127.0.0.1:8080");
    server.listen();

    return 0;
}
```

```bash
vex run
# Server running on http://127.0.0.1:8080
```

---

### 3. WASM Application

```bash
vex new my-wasm-app
```

**src/lib.wasm.vx:**

```vex
import { js } from "std/wasm";

export fn greet(name: string): string {
    js.console_log("Greeting: " + name);
    return "Hello from WASM!";
}
```

```bash
vex build --target=wasm
# Output: vex-builds/my-wasm-app.wasm
```

---

## Troubleshooting

### Common Issues

**1. Dependency Not Found**

```bash
Error: Package not found: github.com/user/repo@v1.0.0
```

- Verify repository exists and tag is correct
- Check git authentication for private repos

**2. Version Conflict**

```bash
Error: Version conflict for github.com/vex-lang/json
  - pkg-a requires json@v1.x
  - pkg-b requires json@v2.x
```

- Update dependencies to use compatible versions

**3. Platform File Not Found**

```bash
Warning: Platform-specific file not found: src/lib.linux.vx
Using fallback: src/lib.vx
```

- This is normal - fallback file will be used

---

## Next Steps

1. Read the [full specification](PACKAGE_MANAGER_SPEC_v1.md)
2. Explore [standard library](vex-libs/std/README.md)
3. Check [examples](examples/)
4. Join the [community](https://github.com/vex-lang/vex)

---

**Documentation:** [PACKAGE_MANAGER_SPEC_v1.md](PACKAGE_MANAGER_SPEC_v1.md)  
**Issues:** [GitHub Issues](https://github.com/vex-lang/vex/issues)
