# Vex Package Manager - Implementation Guide

**Version:** 0.1.2  
**Last Updated:** November 11, 2025  
**Status:** MVP - Core functionality complete, remote packages planned

> **Full Specification**: [Specifications/19_Package_Manager.md](../Specifications/19_Package_Manager.md)

---

## 📋 Current Implementation Status

### ✅ Completed (v0.1.2)

1. **Manifest Parsing** (`manifest.rs`)

   - Full `vex.json` parsing with serde
   - Field validation (name, version, dependencies)
   - Native configuration support
   - Profile and target configuration
   - Testing configuration support
   - Default value handling

2. **Platform Detection** (`platform.rs`)

   - OS detection (Linux, macOS, Windows, FreeBSD, OpenBSD)
   - Architecture detection (x64, arm64, wasm, riscv64)
   - Platform-specific file selection with priority chain
   - Test file support (`.testing.vx`)

3. **Native Linking** (`native_linker.rs`)

   - C/C++ source compilation
   - Static library linking
   - Dynamic library linking
   - Library search paths
   - Compiler flag handling

4. **Build Integration** (`build.rs`)

   - Dependency resolution for build system
   - Source directory collection
   - Lock file validation

5. **Project Initialization** (`cli.rs`)
   - `vex new` - Create new project
   - `vex init` - Initialize existing directory

### ⏳ Planned (Future)

- Remote Git repository support
- Dependency version resolution (semver, ranges)
- Lock file generation (`vex.lock`)
- Package cache management
- Parallel dependency downloads
- Checksum verification
- Test runner implementation

---

## 🏗️ Core Architecture

### File Structure

```
vex-pm/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── manifest.rs         # vex.json parsing ✅
│   ├── lockfile.rs         # vex.lock handling ⏳
│   ├── platform.rs         # Platform detection ✅
│   ├── native_linker.rs    # C/C++ linking ✅
│   ├── build.rs            # Build integration ✅
│   ├── cli.rs              # Project init ✅
│   ├── resolver.rs         # Dependency resolution ⏳
│   ├── cache.rs            # Package caching ⏳
│   ├── git.rs              # Git operations ⏳
│   └── commands.rs         # CLI commands ⏳
└── Cargo.toml
```

### Data Flow

```
vex.json
   ↓
Manifest::from_file()
   ↓
Validation
   ↓
Platform Detection
   ↓
Native Linker
   ↓
Build System
```

---

## 📦 vex.json Specification

### Complete Field Reference

| Field          | Type   | Required | Default      | Description                     |
| -------------- | ------ | -------- | ------------ | ------------------------------- |
| `name`         | string | ✅       | -            | Package name                    |
| `version`      | string | ✅       | -            | Semantic version                |
| `description`  | string | ❌       | null         | Package description             |
| `authors`      | array  | ❌       | null         | Author list                     |
| `license`      | string | ❌       | null         | License (MIT, Apache-2.0, etc.) |
| `repository`   | string | ❌       | null         | Repository URL                  |
| `dependencies` | object | ❌       | {}           | Package dependencies            |
| `main`         | string | ❌       | `src/lib.vx` | Entry point file                |
| `bin`          | object | ❌       | null         | Binary targets                  |
| `testing`      | object | ❌       | null         | Test configuration              |
| `targets`      | object | ❌       | null         | Platform configuration          |
| `profiles`     | object | ❌       | null         | Build profiles                  |
| `native`       | object | ❌       | null         | C/C++ integration               |
| `vex`          | object | ❌       | null         | Vex-specific settings           |

### Minimal Example

```json
{
  "name": "my-lib",
  "version": "1.0.0"
}
```

### Complete Example

```json
{
  "name": "advanced-lib",
  "version": "2.1.0",
  "description": "Advanced library with native code",
  "authors": ["Alice <alice@example.com>"],
  "license": "MIT",
  "repository": "https://github.com/user/advanced-lib",

  "dependencies": {
    "math": "v0.2.0"
  },

  "main": "src/lib.vx",

  "bin": {
    "tool": "src/main.vx"
  },

  "testing": {
    "dir": "tests",
    "pattern": "**/*.test.vx",
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
    "libraries": ["m", "ssl"],
    "search_paths": ["/usr/local/lib"],
    "static_libs": ["vendor/libcustom.a"],
    "cflags": ["-O3", "-Wall", "-fPIC"],
    "include_dirs": ["vendor/include", "../../../vex-runtime/c"]
  },

  "vex": {
    "borrowChecker": "strict"
  }
}
```

---

## 🧪 Testing Configuration

### Test Discovery

Vex automatically discovers test files using the pattern specified in `vex.json`.

**Default Pattern**: `**/*.test.vx` (searches from project root)

**Configuration**:

```json
{
  "testing": {
    "dir": "tests", // Test directory (informational)
    "pattern": "**/*.test.vx", // Glob pattern from project root
    "timeout": 30, // Test timeout in seconds (optional)
    "parallel": true // Run tests in parallel (default: true)
  }
}
```

**Test File Naming Convention**:

- Test files MUST follow the `*.test.vx` pattern
- Examples:
  - `basic.test.vx` ✅
  - `integration.test.vx` ✅
  - `basic_test.vx` ❌ (missing .test before .vx)
  - `test_basic.vx` ❌ (wrong position)

**Directory Structure**:

```
my-project/
├── vex.json
├── src/
│   └── lib.vx
└── tests/
    ├── basic.test.vx
    ├── integration.test.vx
    └── unit/
        └── math.test.vx
```

**Running Tests**:

```bash
# Discover and run all tests
vex test

# Run specific test file
vex test tests/basic.test.vx

# Run with custom timeout
vex test --timeout 60

# Run sequentially (no parallel)
vex test --no-parallel
```

> **See Also**: [TESTING_SYSTEM.md](./TESTING_SYSTEM.md) for comprehensive testing documentation.

---

## 🔧 Native Configuration

### Field Descriptions

| Field          | Type  | Description                | Example                  |
| -------------- | ----- | -------------------------- | ------------------------ |
| `sources`      | array | C/C++ files to compile     | `["native/src/impl.c"]`  |
| `libraries`    | array | System libraries to link   | `["m", "ssl", "crypto"]` |
| `search_paths` | array | Library search directories | `["/usr/local/lib"]`     |
| `static_libs`  | array | Static library files (.a)  | `["./vendor/lib.a"]`     |
| `cflags`       | array | C compiler flags           | `["-O3", "-Wall"]`       |
| `include_dirs` | array | Header include paths       | `["vendor/include"]`     |

### Common Patterns

#### Pure Vex Module (No FFI)

```json
{
  "name": "pure-vex",
  "version": "1.0.0"
}
```

#### System Library Only

```json
{
  "name": "math-wrapper",
  "version": "1.0.0",
  "native": {
    "libraries": ["m"],
    "search_paths": ["/usr/local/lib"]
  }
}
```

#### Custom C Implementation

```json
{
  "name": "custom-impl",
  "version": "1.0.0",
  "native": {
    "sources": ["native/src/vex_custom.c"],
    "cflags": ["-O3", "-Wall", "-fPIC", "-std=c11"],
    "include_dirs": ["native/src", "../../../vex-runtime/c"]
  }
}
```

#### Static Library Linking

```json
{
  "name": "vendor-lib",
  "version": "1.0.0",
  "native": {
    "static_libs": ["vendor/libvendor.a"],
    "include_dirs": ["vendor/include"]
  }
}
```

#### Complex Multi-Source

```json
{
  "name": "complex",
  "version": "1.0.0",
  "native": {
    "sources": [
      "native/src/module_a.c",
      "native/src/module_b.c",
      "native/src/utils.c"
    ],
    "libraries": ["ssl", "crypto", "z"],
    "search_paths": ["/usr/local/lib", "/opt/homebrew/lib"],
    "static_libs": ["vendor/libcustom.a"],
    "cflags": ["-O3", "-Wall", "-Werror", "-fPIC", "-std=c11", "-march=native"],
    "include_dirs": ["native/src", "vendor/include", "../../../vex-runtime/c"]
  }
}
```

---

## 🎯 Platform Detection

### Supported Platforms

**Operating Systems:**

- `linux` - Linux
- `macos` - macOS
- `windows` - Windows
- `freebsd` - FreeBSD
- `openbsd` - OpenBSD

**Architectures:**

- `x64` - x86-64
- `arm64` - ARM64/AArch64
- `wasm` - WebAssembly
- `wasi` - WASI
- `riscv64` - RISC-V 64-bit

### File Selection Priority

When resolving `src/lib.vx`, the system checks in this order:

1. `src/lib.testing.vx` (if running tests)
2. `src/lib.{os}.{arch}.vx` (e.g., `lib.macos.arm64.vx`)
3. `src/lib.{arch}.vx` (e.g., `lib.arm64.vx`)
4. `src/lib.{os}.vx` (e.g., `lib.macos.vx`)
5. `src/lib.vx` (generic fallback)

**Example:**

```
src/
├── lib.vx              # Generic (REQUIRED)
├── lib.macos.vx        # macOS-specific
├── lib.linux.vx        # Linux-specific
├── lib.x64.vx          # x64 SIMD optimizations
└── lib.testing.vx      # Test mocks
```

**Resolution on macOS ARM64:**

1. Checks `lib.testing.vx` (if test mode)
2. Checks `lib.macos.arm64.vx` ❌
3. Checks `lib.arm64.vx` ❌
4. Checks `lib.macos.vx` ✅ **FOUND**

---

## 🚀 Build Integration

### Compilation Flow

```
vex compile src/main.vx
   ↓
Read vex.json
   ↓
Parse Manifest
   ↓
Detect Platform
   ↓
Resolve Platform-Specific Files
   ↓
Process Native Config
   ↓
Compile C Sources → .o files
   ↓
Generate Linker Args
   ↓
Link Final Binary
```

### Native Linker Process

1. **Create Build Directory**: `.vex-build/native/`
2. **Compile C Sources**:
   ```bash
   cc -O3 -Wall -fPIC -c native/src/impl.c -o .vex-build/native/impl.o
   ```
3. **Collect Static Libraries**: `vendor/libcustom.a`
4. **Generate Linker Args**:
   ```
   .vex-build/native/impl.o vendor/libcustom.a -L/usr/local/lib -lm -lssl
   ```
5. **Pass to Vex Compiler**

---

## 📊 Field Validation

### Name Validation

- ✅ Non-empty string
- ✅ Any characters allowed (for now)

### Version Validation

- ✅ Must be semantic version: `X.Y.Z`
- ✅ Optional `v` prefix: `v1.2.3` or `1.2.3`
- ✅ All parts must be integers

**Valid:**

- `"1.0.0"`
- `"v2.3.1"`
- `"0.1.0"`

**Invalid:**

- `"1.0"` (missing patch)
- `"1.0.0-alpha"` (pre-release not yet supported)
- `"latest"` (reserved for dependency specs)

### Dependency Version Specs

**Currently Supported:**

- `"v1.2.3"` - Exact version
- `"1.2.3"` - Exact version (no v prefix)

**Planned:**

- `"^1.2.0"` - Compatible with 1.x
- `"~1.2.0"` - Compatible with 1.2.x
- `"1.0.0..2.0.0"` - Version range
- `"*"` or `"latest"` - Latest version

---

## 🛠️ API Usage Examples

### Parsing Manifest

```rust
use vex_pm::Manifest;

// From file
let manifest = Manifest::from_file("vex.json")?;
println!("Package: {} v{}", manifest.name, manifest.version);

// From string
let json = r#"{"name": "test", "version": "1.0.0"}"#;
let manifest = Manifest::from_str(json)?;
```

### Platform Detection

```rust
use vex_pm::{Platform, select_platform_file};

let platform = Platform::detect();
println!("Running on: {} {}", platform.os, platform.arch);

let file = select_platform_file("src/lib.vx", &platform);
// Returns: src/lib.macos.vx (if exists), else src/lib.vx
```

### Native Linking

```rust
use vex_pm::{NativeLinker, Manifest};

let manifest = Manifest::from_file("vex.json")?;
if let Some(native_config) = manifest.get_native() {
    let linker = NativeLinker::new(".");
    let linker_args = linker.process(native_config)?;
    println!("Linker args: {}", linker_args);
}
```

---

## ⚠️ Known Limitations (v0.1.2)

### Not Yet Implemented

1. **Remote Dependencies**: Only local dependencies supported
2. **Version Resolution**: No semver range support
3. **Lock File Generation**: Lock file structure exists but not auto-generated
4. **Package Cache**: No global package cache yet
5. **Dependency Graph**: Basic resolver exists but incomplete
6. **Git Integration**: Clone/checkout functions exist but not integrated
7. **Checksum Verification**: No SHA-256 validation yet

### Workarounds

**Remote Dependencies:**

```bash
# Manual workaround:
git clone https://github.com/user/lib vendor/lib
# Then use local path in vex.json
```

**Version Conflicts:**

```json
// Only exact versions for now
{
  "dependencies": {
    "lib-a": "v1.2.0", // Must be exact
    "lib-b": "v2.0.0" // No ranges yet
  }
}
```

---

## 🎯 Development Priorities

### Phase 1: Core Manifest (✅ Complete)

- [x] Parse `vex.json`
- [x] Validate fields
- [x] Platform detection
- [x] Native linking
- [x] Build integration

### Phase 2: Local Dependencies (Current)

- [ ] Resolve local package paths
- [ ] Generate `vex.lock`
- [ ] Validate lock file
- [ ] Detect circular dependencies
- [ ] Handle version conflicts

### Phase 3: Remote Packages (Future)

- [ ] Git repository cloning
- [ ] Version tag checkout
- [ ] Global package cache (`~/.vex/cache/`)
- [ ] Parallel downloads
- [ ] Checksum verification
- [ ] Authentication (SSH/HTTPS)

### Phase 4: Advanced Features (Future)

- [ ] Semantic versioning (`^1.2.0`, `~1.2.0`)
- [ ] Version range resolution
- [ ] Workspace support (monorepos)
- [ ] Private registries
- [ ] Mirror support

---

## 📝 Best Practices

### Manifest Organization

```json
{
  // Required fields first
  "name": "my-package",
  "version": "1.0.0",

  // Metadata
  "description": "...",
  "authors": ["..."],
  "license": "MIT",
  "repository": "...",

  // Dependencies
  "dependencies": {},

  // Entry points
  "main": "src/lib.vx",
  "bin": {},

  // Platform config
  "targets": {},
  "profiles": {},

  // Native integration
  "native": {},

  // Vex settings
  "vex": {}
}
```

### Version Bumping

```bash
# Patch (bug fixes): 1.0.0 → 1.0.1
# Minor (features): 1.0.0 → 1.1.0
# Major (breaking): 1.0.0 → 2.0.0

# Update vex.json manually for now
```

### Native Code Organization

```
project/
├── vex.json
├── src/
│   └── lib.vx
└── native/
    ├── src/
    │   ├── vex_module.c
    │   ├── vex_module.h
    │   └── internal.c
    └── vendor/
        ├── libcustom.a
        └── include/
```

---

**Maintained by**: Vex Language Team  
**Implementation**: `vex-pm/` crate  
**Specification**: `Specifications/19_Package_Manager.md`
