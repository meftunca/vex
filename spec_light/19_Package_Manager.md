# Package Manager

**Version:** 0.1.2
**Last Updated:** November 2025

This document describes Vex's package manager (`vex-pm`), which provides dependency management, project initialization, and build coordination for Vex projects.

---

## Table of Contents

1. \1
2. \1
3. \1
4. \1
5. \1
6. \1
7. \1
8. \1

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

[10 lines code: (unknown)]

### Entry Points

- **Library**: `src/lib.vx` (default)
- **Executable**: `src/main.vx` or specified in `vex.json`
- **Custom**: Configurable via `main` field in manifest

---

## Manifest File (vex.json)

The `vex.json` file describes your project and its dependencies:

[55 lines code: ```json]

### Dependency Specification

**Current Status (v0.1.2)**: Local dependencies only. Remote Git repositories planned for future releases.

``````json
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

[10 lines code: ```json]

**Field Descriptions**:
- `sources`: C/C++ files to compile
- `libraries`: System libraries to link (e.g., `m`, `ssl`)
- `search_paths`: Library search directories
- `static_libs`: Static library files (.a)
- `cflags`: C compiler flags
- `include_dirs`: Header include directories

### Complete Field Reference

• Field — Type — Required — Description
• ------- — ------ — ---------- — -------------
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
``````json
{
  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm", "wasi"]
  }
}
```

**Profiles Structure**:
[14 lines code: ```json]

**Vex Config Structure**:
``````json
{
  "vex": {
    "borrowChecker": "strict"  // or "permissive"
  }
}
```

**Testing Config Structure**:
``````json
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

[17 lines code: ```json]

Lock files are automatically generated and should be committed to version control.

---

## CLI Commands

### Project Initialization

``````bash
# Create new project
vex new my-project

# Initialize in existing directory
vex init
```

### Dependency Management

**Current Status (v0.1.2)**: Local dependencies only. CLI commands for remote packages planned.

[12 lines code: ```bash]

### Building and Running

[17 lines code: ```bash]

### Development

[20 lines code: ```bash]

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

``````json
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
``````bash
vex test
```

**Run specific test file**:
``````bash
vex test tests/basic.test.vx
```

**Run with custom timeout**:
``````bash
vex test --timeout 60
```

**Run sequentially**:
``````bash
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

[12 lines code: ```json]

### Native Library Integration

Link with system libraries:

``````json
{
  "native": {
    "libs": ["ssl", "crypto", "z"],
    "include": ["/usr/local/include"],
    "flags": ["-O3", "-march=native"]
  }
}
```

---

**Previous**: \1

**Maintained by**: Vex Language Team</content>
<parameter name="filePath">/Users/mapletechnologies/Desktop/big_projects/vex_lang/Specifications/19_Package_Manager.md
