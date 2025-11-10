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
├── src/
│   ├── lib.vx        # Library entry point
│   ├── main.vx       # Executable entry point (optional)
│   └── mod.vx        # Module declarations
├── tests/            # Test files
├── examples/         # Example code
└── vex-builds/       # Build artifacts (generated)
```

### Entry Points

- **Library**: `src/lib.vx` (default)
- **Executable**: `src/main.vx` or specified in `vex.json`
- **Custom**: Configurable via `main` field in manifest

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

  "dependencies": {
    "github.com/user/math-lib": "v1.2.0",
    "github.com/user/http-client": "^2.0.0"
  },

  "targets": {
    "debug": {
      "opt-level": 0
    },
    "release": {
      "opt-level": 3
    }
  },

  "main": "src/main.vx",

  "bin": {
    "my-app": "src/main.vx",
    "cli-tool": "src/cli.vx"
  },

  "native": {
    "include": ["vendor/include"],
    "libs": ["ssl", "crypto"],
    "flags": ["-O3", "-march=native"]
  }
}
```

### Dependency Specification

Dependencies support multiple version formats:

```json
{
  "dependencies": {
    "github.com/user/lib": "v1.2.3", // Exact version
    "github.com/user/lib": "^1.2.0", // Compatible with 1.x
    "github.com/user/lib": "1.0.0..2.0.0", // Range
    "github.com/user/lib": "*" // Latest
  }
}
```

### Native Dependencies

For C/C++ integration:

```json
{
  "native": {
    "include": ["path/to/headers"],
    "libs": ["ssl", "zlib"],
    "flags": ["-O3", "-Wall"]
  }
}
```

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

```bash
# Add dependency
vex add github.com/user/math-lib@v1.2.0
vex add github.com/user/math-lib  # Latest version

# Remove dependency
vex remove github.com/user/math-lib

# Update dependencies
vex update

# List dependencies
vex list

# Clean cache
vex clean
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

# Run tests
vex test

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
