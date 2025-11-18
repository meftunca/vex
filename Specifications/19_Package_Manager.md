# Package Manager

**Version:** 0.1.2
**Last Updated:** November 2025

This document describes Vex's package manager (`vex-pm`), which provides dependency management, project initialization, and build coordination for Vex projects.

---

## Table of Contents

1. [Overview](#overview)
2. [Project Structure](#project-structure)
3. [Manifest File (vex.json)](#manifest-file-vexjson)
4. [Lock File (vex.lock)](#lock-file-vexlock)
5. [CLI Commands](#cli-commands)
6. [Dependency Resolution](#dependency-resolution)
7. [Platform-Specific Code](#platform-specific-code)
8. [Build Integration](#build-integration)

---

## Overview

Vex's package manager is fully integrated into the `vex` command-line tool. It follows a decentralized, Git-based approach inspired by Go modules and Cargo.

### Key Features

- **Decentralized**: No central package registry - uses Git repositories directly
- **Fast**: Parallel downloads with global caching
- **Secure**: SHA-256 checksums and lock files
- **Platform-aware**: Automatic selection of platform-specific implementations
- **Simple**: Single tool for compilation, running, and package management

### Philosophy

_"Cargo'nun gücü, Go Mod'un sadeliği, Zig'in platform awareness'ı"_

---

## Project Structure

Vex projects follow a conventional directory structure:

```
my-project/
├── vex.json          # Project manifest
├── vex.lock          # Lock file (generated)
├── native/           # Native C Codes
├── src/
│   ├── lib.vx        # Library entry point
│   ├── main.vx       # Executable entry point (optional)
│   └── mod.vx        # Module declarations
├── tests/            # Test files
├── examples/         # Example code
└── vex-builds/       # Build artifacts (generated)
```

### Entry Points

- **Library**: `src/lib.vx` (default main entry)
- **Module**: `src/mod.vx` (alternative if no lib.vx)
- **Executable**: `src/main.vx` or specified in `vex.json`
- **Custom**: Configurable via `main` field in manifest

**Import Resolution**:
```vex
// Package name import uses "main" field from vex.json
import { abs } from "math";  
// → Resolves to: vex-libs/std/math/src/lib.vx (from vex.json)

// Direct file import bypasses vex.json
import { sin } from "math/native.vxc";  
// → Resolves to: vex-libs/std/math/src/native.vxc

// Relative imports (within module files)
import { helper } from "./utils.vx";
// → Resolves relative to current file
```

**Priority Order**:
1. `vex.json` → `main` field value
2. `src/lib.vx` (if exists)
3. `src/mod.vx` (if exists)
4. Error: No entry point found

---

## Manifest File (vex.json)

The `vex.json` file describes your project and its dependencies:

```json
{
  "name": "my-project",
  "version": "1.0.0",
  "description": "A Vex project",
  "authors": ["Your Name <you@example.com>"],
  "license": "MIT",
  "repository": "https://github.com/user/my-project",

  "dependencies": {
    "local-lib": "v1.2.0"
  },

  "main": "src/lib.vx",

  "bin": {
    "my-app": "src/main.vx",
    "cli-tool": "src/cli.vx"
  },

  "testing": {
    "dir": "tests",
    "pattern": "*.test.vx",
    "timeout": 30,
    "parallel": true
  },

  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm"]
  },

  "profiles": {
    "development": {
      "optimizationLevel": 0,
      "debugSymbols": true
    },
    "production": {
      "optimizationLevel": 3,
      "debugSymbols": false
    }
  },

  "native": {
    "sources": ["native/src/helper.c"],
    "libraries": ["ssl", "crypto"],
    "search_paths": ["/usr/local/lib"],
    "static_libs": ["./vendor/libmylib.a"],
    "cflags": ["-O3", "-Wall", "-fPIC"],
    "include_dirs": ["vendor/include", "../../../vex-runtime/c"]
  },

  "vex": {
    "borrowChecker": "strict"
  }
}
```

### Dependency Specification

**Current Status (v0.1.2)**: Local dependencies only. Remote Git repositories planned for future releases.

```json
{
  "dependencies": {
    "local-lib": "v1.2.3",     // Exact version (local)
    "math": "v0.2.0"          // Stdlib module
  }
}
```

**Future Support** (planned):
- `"^1.2.0"` - Compatible with 1.x (semantic versioning)
- `"1.0.0..2.0.0"` - Version range
- `"*"` - Latest version
- Git repositories: `"github.com/user/lib": "v1.2.0"`

### Native Dependencies

For C/C++ integration:

```json
{
  "native": {
    "sources": ["native/src/implementation.c"],
    "libraries": ["ssl", "zlib"],
    "search_paths": ["/usr/local/lib", "/opt/homebrew/lib"],
    "static_libs": ["./vendor/libcustom.a"],
    "cflags": ["-O3", "-Wall", "-fPIC", "-std=c11"],
    "include_dirs": ["path/to/headers", "../../../vex-runtime/c"]
  }
}
```

**Field Descriptions**:
- `sources`: C/C++ files to compile
- `libraries`: System libraries to link (e.g., `m`, `ssl`)
- `search_paths`: Library search directories
- `static_libs`: Static library files (.a)
- `cflags`: C compiler flags
- `include_dirs`: Header include directories

### Complete Field Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | ✅ | Package name |
| `version` | string | ✅ | Semantic version (e.g., "1.0.0") |
| `description` | string | ❌ | Package description |
| `authors` | array | ❌ | Author names and emails |
| `license` | string | ❌ | License identifier (e.g., "MIT") |
| `repository` | string | ❌ | Repository URL |
| `dependencies` | object | ❌ | Package dependencies |
| `main` | string | ❌ | Entry point (default: `src/lib.vx`) |
| `bin` | object | ❌ | Binary targets |
| `testing` | object | ❌ | Test configuration |
| `targets` | object | ❌ | Platform configuration |
| `profiles` | object | ❌ | Build profiles |
| `native` | object | ❌ | C/C++ integration config |
| `vex` | object | ❌ | Vex-specific settings |

**Targets Structure**:
```json
{
  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm", "wasi"]
  }
}
```

**Profiles Structure**:
```json
{
  "profiles": {
    "development": {
      "optimizationLevel": 0,
      "debugSymbols": true,
      "memProfiling": false,
      "cpuProfiling": false
    },
    "production": {
      "optimizationLevel": 3,
      "debugSymbols": false
    }
  }
}
```

**Vex Config Structure**:
```json
{
  "vex": {
    "borrowChecker": "strict"  // or "permissive"
  }
}
```

**Testing Config Structure**:
```json
{
  "testing": {
    "dir": "tests",              // Test directory (informational)
    "pattern": "**/*.test.vx",   // Glob pattern from project root (default)
    "timeout": 30,                // Test timeout in seconds (optional)
    "parallel": true              // Run tests in parallel (default: true)
  }
}
```

**Test File Naming Convention**:

- Test files MUST follow the `*.test.vx` pattern
- Examples:
  - `basic.test.vx`
  - `integration.test.vx`
  - `unit.test.vx`
- **Pattern Search**: Uses glob from project root (`**/*.test.vx`)
- Custom patterns can be specified via `testing.pattern`

---

## Lock File (vex.lock)

The `vex.lock` file ensures reproducible builds by locking exact dependency versions:

```json
{
  "version": "1.0",
  "packages": {
    "github.com/user/math-lib": {
      "version": "v1.2.3",
      "checksum": "abc123...",
      "dependencies": {}
    },
    "github.com/user/http-client": {
      "version": "v2.1.0",
      "checksum": "def456...",
      "dependencies": {
        "github.com/user/math-lib": "v1.2.3"
      }
    }
  }
}
```

Lock files are automatically generated and should be committed to version control.

---

## CLI Commands

### Project Initialization

```bash
# Create new project
vex new my-project

# Initialize in existing directory
vex init
```

### Dependency Management

**Current Status (v0.1.2)**: Local dependencies only. CLI commands for remote packages planned.

```bash
# Manual dependency management (edit vex.json)
# Add to dependencies section:
# "local-lib": "v1.2.0"

# Planned commands (future):
# vex add github.com/user/math-lib@v1.2.0
# vex remove github.com/user/math-lib
# vex update
# vex list

# Currently available:
vex clean  # Clean build cache
```

### Building and Running

```bash
# Build project
vex build

# Build with specific profile
vex build --release

# Run executable
vex run

# Run specific binary
vex run --bin my-app

# Run example
vex run --example demo

# CI build (locked dependencies)
vex build --locked
```

### Development

```bash
# Check project
vex check

# Format code
vex format

# Run tests (discovers *.test.vx files)
vex test

# Run specific test file
vex test tests/basic.test.vx

# Run tests with timeout
vex test --timeout 60

# Run tests sequentially (no parallel)
vex test --no-parallel

# Generate documentation
vex doc
```

---

## Dependency Resolution

Vex uses a Go-style flat dependency resolution:

### Algorithm

1. **Collect**: Gather all direct and transitive dependencies
2. **Resolve**: Find compatible versions for all packages
3. **Download**: Fetch packages in parallel to global cache
4. **Verify**: Check SHA-256 checksums
5. **Link**: Generate build configuration

### Version Selection

- **Semantic versioning**: `^1.2.0` allows compatible updates within major version
- **Exact pinning**: `v1.2.3` locks to specific version
- **Range specification**: `1.0.0..2.0.0` for custom ranges
- **Latest**: `*` or no version specifier

### Conflict Resolution

When version conflicts occur, Vex follows these rules:

1. Prefer already resolved versions
2. Choose highest compatible version
3. Fail with clear error message if impossible

---

## Platform-Specific Code

Vex supports platform-specific implementations using file suffixes:

### Priority Order

1. `{file}.testing.vx` (when running tests)
2. `{file}.{os}.{arch}.vx` (most specific)
3. `{file}.{arch}.vx` (architecture-specific)
4. `{file}.{os}.vx` (OS-specific)
5. `{file}.vx` (generic fallback)

### Supported Platforms

**Architectures:**

- `x64` - x86-64
- `arm64` - ARM64/AArch64
- `wasm` - WebAssembly
- `wasi` - WASI
- `riscv64` - RISC-V 64-bit

**Operating Systems:**

- `linux` - Linux
- `macos` - macOS
- `windows` - Windows
- `freebsd` - FreeBSD
- `openbsd` - OpenBSD

### Example

```
src/
├── crypto.vx           # Generic implementation
├── crypto.x64.vx       # x86-64 with SIMD
├── crypto.arm64.vx     # ARM64 with NEON
├── crypto.wasm.vx      # WebAssembly version
└── crypto.testing.vx   # Test mocks
```

---

## Testing

### Test Discovery

Vex automatically discovers test files using the pattern specified in `vex.json`.

**Default Pattern**: `*.test.vx`

**Directory Structure**:
```
my-project/
├── vex.json
├── src/
│   └── lib.vx
└── tests/
    ├── basic.test.vx
    ├── integration.test.vx
    └── unit.test.vx
```

### Test File Naming

**Required Pattern**: Files must end with `.test.vx`

**Examples**:
```
✅ basic.test.vx
✅ user_auth.test.vx
✅ api_integration.test.vx
❌ basic_test.vx       (missing .test before .vx)
❌ test_basic.vx       (wrong position)
❌ basic.vx            (missing .test)
```

### Test Configuration

```json
{
  "testing": {
    "dir": "tests",
    "pattern": "*.test.vx",
    "timeout": 30,
    "parallel": true
  }
}
```

**Fields**:
- `dir`: Directory containing test files (default: `"tests"`)
- `pattern`: Glob pattern for test files (default: `"*.test.vx"`)
- `timeout`: Maximum execution time per test in seconds (optional)
- `parallel`: Run tests in parallel (default: `true`)

### Running Tests

**Discover and run all tests**:
```bash
vex test
```

**Run specific test file**:
```bash
vex test tests/basic.test.vx
```

**Run with custom timeout**:
```bash
vex test --timeout 60
```

**Run sequentially**:
```bash
vex test --no-parallel
```

### Test Organization

**Unit Tests**: Test individual functions/modules
```
tests/
├── math.test.vx
├── string.test.vx
└── utils.test.vx
```

**Integration Tests**: Test module interactions
```
tests/
├── api_integration.test.vx
├── db_integration.test.vx
└── workflow.test.vx
```

**Mixed Structure**:
```
tests/
├── unit/
│   ├── math.test.vx
│   └── string.test.vx
└── integration/
    ├── api.test.vx
    └── db.test.vx
```

### Platform-Specific Tests

Tests can also use platform-specific files:

```
tests/
├── io.test.vx              # Generic tests
├── io.test.macos.vx        # macOS-specific tests
└── io.test.linux.vx        # Linux-specific tests
```

**Priority**:
1. `test.{os}.{arch}.vx`
2. `test.{arch}.vx`
3. `test.{os}.vx`
4. `test.vx`

---

## Build Integration

### Automatic Resolution

When building, Vex automatically:

1. **Reads** `vex.json` and `vex.lock`
2. **Downloads** dependencies to `~/.vex/cache/`
3. **Verifies** checksums
4. **Generates** build configuration
5. **Compiles** with proper include paths and linking

### Cache Location

- **Global cache**: `~/.vex/cache/`
- **Project cache**: `vex-builds/`
- **Lock file**: `vex.lock`

### Build Profiles

Configure optimization levels and flags:

```json
{
  "profiles": {
    "debug": {
      "opt-level": 0,
      "debug": true
    },
    "release": {
      "opt-level": 3,
      "lto": true
    }
  }
}
```

### Native Library Integration

Link with system libraries:

```json
{
  "native": {
    "libs": ["ssl", "crypto", "z"],
    "include": ["/usr/local/include"],
    "flags": ["-O3", "-march=native"]
  }
}
```

---

**Previous**: [18_Raw_Pointers_and_FFI.md](./18_Raw_Pointers_and_FFI.md)

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/19_Package_Manager.md
