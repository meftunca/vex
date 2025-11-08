# Vex Package Manager Specification (v1.0)

**Version:** 1.0  
**Last Updated:** November 8, 2025  
**Status:** ✅ Finalized + Platform-Specific Files

---

## Table of Contents

1. [Overview](#overview)
2. [Platform-Specific File Naming](#platform-specific-file-naming) ⭐ NEW
3. [Manifesto: vex.json](#manifesto-vexjson)
4. [Lock File: vex.lock](#lock-file-vexlock)
5. [CLI Commands](#cli-commands)
6. [Dependency Resolution](#dependency-resolution)
7. [Authentication](#authentication)
8. [Package Structure](#package-structure)
9. [Standard Library Integration](#standard-library-integration) ⭐ NEW
10. [Implementation Phases](#implementation-phases)

---

## Overview

### Philosophy

**"Cargo'nun gücü, Go Mod'un sadeliği, Zig'in platform awareness'ı"**

- ✅ **Decentralized**: Git-based, no central registry
- ✅ **Fast**: Parallel downloads, global cache
- ✅ **Secure**: Lock files, SHA-256 checksums
- ✅ **Simple**: Single `vex` command (compiler + package manager)
- ✅ **Platform-Aware**: Instruction set + OS-specific files ⭐ NEW

### Key Decisions

1. **Format**: JSON (not TOML)
2. **Lock File**: JSON (not custom)
3. **Authentication**: System git credentials
4. **Resolution**: Go-style flat dependency tree
5. **Entrypoint**: `src/lib.vx` (customizable via `main`)
6. **Platform Files**: Suffix-based selection (Zig-inspired) ⭐ NEW
7. **Stdlib**: Built-in module (no dependency) ⭐ NEW
8. **Nexus Mirror**: Phase 2+ (not initially supported)
9. **FFI Dependencies**: Phase 2+ (not initially supported)

---

## Platform-Specific File Naming

### Overview

Vex allows platform-specific implementations using file naming conventions, similar to Zig's approach.

**Compilation Priority:**

1. `{file}.testing.vx` (test variant, when `vex test`)
2. `{file}.{os}.{instruction}.vx` (most specific)
3. `{file}.{instruction}.vx` (instruction-specific)
4. `{file}.{os}.vx` (OS-specific)
5. `{file}.vx` (fallback/generic)

### Supported Platforms

#### Instruction Sets

- `x64` - x86-64 (AMD64, Intel 64)
- `arm64` - ARM64 (AArch64, Apple Silicon)
- `wasm` - WebAssembly (browser)
- `wasi` - WebAssembly System Interface (WASI)
- `riscv64` - RISC-V 64-bit

#### Operating Systems

- `linux` - Linux (any distro)
- `macos` - macOS (Darwin)
- `windows` - Windows
- `freebsd` - FreeBSD
- `openbsd` - OpenBSD

#### Build Variants

- `testing` - Test-specific implementations (mocks, fixtures)

### File Naming Examples

#### 1. Instruction-Specific

```
src/
├── server.vx           # Generic fallback
├── server.x64.vx       # x86-64 optimized (SIMD AVX2)
├── server.arm64.vx     # ARM64 optimized (NEON)
└── server.wasm.vx      # WebAssembly build
```

**Compilation:**

```bash
vex build --target=x64     # Uses server.x64.vx
vex build --target=arm64   # Uses server.arm64.vx
vex build --target=wasm    # Uses server.wasm.vx
vex build                  # Uses server.vx (auto-detect)
```

#### 2. OS + Instruction

```
src/
├── utils.vx                    # Generic fallback
├── utils.linux.x64.vx          # Linux x86-64 specific
├── utils.macos.arm64.vx        # macOS Apple Silicon
└── utils.windows.x64.vx        # Windows x86-64
```

**Compilation:**

```bash
vex build --target=linux-x64      # Uses utils.linux.x64.vx
vex build --target=macos-arm64    # Uses utils.macos.arm64.vx
```

#### 3. OS-Only

```
src/
├── file_io.vx           # Generic fallback
├── file_io.linux.vx     # Linux syscalls (epoll)
├── file_io.macos.vx     # macOS (kqueue)
└── file_io.windows.vx   # Windows (IOCP)
```

#### 4. WASM Variants

```
src/
├── runtime.vx           # Generic
├── runtime.wasm.vx      # Browser WebAssembly
└── runtime.wasi.vx      # WASI (server-side WASM)
```

**Usage:**

```bash
vex build --target=wasm    # Browser WASM
vex build --target=wasi    # WASI runtime
```

#### 5. Testing Variants

```
src/
├── database.vx          # Production database client
└── database.testing.vx  # Mock database for tests
```

**Usage:**

```bash
vex test                 # Uses database.testing.vx (with mocks)
vex build                # Uses database.vx (production)
```

### Selection Algorithm

```rust
fn select_platform_file(base: &str, target: Target, is_test: bool) -> PathBuf {
    let mut candidates = vec![];

    // 1. Test variant (highest priority in test mode)
    if is_test {
        candidates.push(format!("{}.testing.vx", base));
    }

    // 2. OS + arch specific
    candidates.push(format!("{}.{}.{}.vx", base, target.os, target.arch));

    // 3. Arch only
    candidates.push(format!("{}.{}.vx", base, target.arch));

    // 4. OS only
    candidates.push(format!("{}.{}.vx", base, target.os));

    // 5. Generic fallback
    candidates.push(format!("{}.vx", base));

    candidates.iter()
        .find(|path| Path::new(path).exists())
        .cloned()
        .unwrap_or_else(|| format!("{}.vx", base))
}
```

### Use Cases

**1. SIMD Optimizations**

```vex
// crypto.x64.vx - AVX2 optimized
fn sha256_hash(data: &[u8]): [u8; 32] {
    // Use AVX2 instructions
    simd_sha256(data)
}

// crypto.arm64.vx - NEON optimized
fn sha256_hash(data: &[u8]): [u8; 32] {
    // Use NEON instructions
    neon_sha256(data)
}

// crypto.vx - Generic fallback
fn sha256_hash(data: &[u8]): [u8; 32] {
    // Software implementation
    generic_sha256(data)
}
```

**2. System Calls**

```vex
// net.linux.vx
fn create_socket(): Socket {
    // Linux epoll
}

// net.macos.vx
fn create_socket(): Socket {
    // macOS kqueue
}

// net.windows.vx
fn create_socket(): Socket {
    // Windows IOCP
}
```

**3. WebAssembly**

```vex
// server.vx - Native binary
fn main(): i32 {
    let listener = TcpListener.bind("127.0.0.1:8080");
    // Native network stack
}

// server.wasm.vx - Browser WASM
fn main(): i32 {
    // Use fetch API
    js.fetch("https://api.example.com");
}

// server.wasi.vx - WASI runtime
fn main(): i32 {
    // Use WASI sockets
    wasi.tcp_listen(8080);
}
```

**4. Testing with Mocks**

```vex
// database.vx - Production implementation
fn connect(url: string): Connection {
    // Real database connection
    PostgreSQL.connect(url)
}

// database.testing.vx - Mock for tests
fn connect(url: string): Connection {
    // In-memory mock database
    MockDB.new()
}
```

**Test Usage:**

```bash
vex test  # Automatically uses database.testing.vx
# → All database calls use in-memory mocks
# → Fast, isolated tests without external dependencies
```

### Configuration in vex.json

```json
{
  "name": "my-package",
  "version": "1.0.0",

  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm", "wasi"]
  },

  "platformFiles": {
    "src/crypto": {
      "x64": "src/crypto.x64.vx",
      "arm64": "src/crypto.arm64.vx",
      "fallback": "src/crypto.vx"
    }
  }
}
```

---

## Manifesto: vex.json

### Basic Structure

```json
{
  "name": "my-package",
  "version": "0.1.0",
  "description": "High-performance Vex server",
  "authors": ["Your Name <email@example.com>"],
  "license": "MIT",

  "dependencies": {
    "github.com/vex-lang/json": "v1.2.0",
    "gitlab:company/internal-lib": "v1.5.0",
    "https://cdn.example.com/pkg.tar.gz": {
      "version": "v1.0.0",
      "headers": {
        "Authorization": "Bearer token123"
      }
    }
  },

  "vex": {
    "borrowChecker": "strict"
  },

  "profiles": {
    "development": {
      "optimizationLevel": 0,
      "debugSymbols": true
    },
    "testing": {
      "optimizationLevel": 1,
      "debugSymbols": true,
      "memProfiling": true,
      "cpuProfiling": true
    },
    "production": {
      "optimizationLevel": 3,
      "debugSymbols": false
    }
  }
}
```

### Dependency Protocols

#### 1. GitHub (Default)

```json
"github:user/repo": "v1.2.0"
```

- No prefix required
- Git clone: `https://github.com/user/repo.git`
- Tag: `v1.2.0`

#### 2. GitLab (Explicit)

```json
"gitlab:company/repo": "v1.5.0"
```

- Explicit `gitlab:` prefix
- Git clone: `https://gitlab.com/company/repo.git`
- Private repos use git credentials

#### 3. Other Git Services

```json
"bitbucket:team/project": "v2.0.0"
```

- Explicit prefix required
- Supported: `bitbucket:`, `gitea:`, etc.

#### 4. HTTP/HTTPS Direct Download

```json
"https://cdn.example.com/pkg-v1.0.0.tar.gz": {
    "version": "v1.0.0",
    "headers": {
        "Authorization": "Bearer secret_token",
        "X-Custom-Header": "value"
    }
}
```

- Full URL required
- `headers` field optional (for authentication)
- `version` field optional (usually in URL)

### Profiles

Default profiles if not specified:

```json
{
  "development": {
    "optimizationLevel": 0,
    "debugSymbols": true
  },
  "production": {
    "optimizationLevel": 3,
    "debugSymbols": false
  }
}
```

Profile selection:

- `vex build` → development profile
- `vex build --release` → production profile
- `vex build --profile=testing` → custom profile

### Optional Fields

#### Custom Entrypoint

```json
{
  "name": "my-package",
  "main": "src/custom_entry.vx"
}
```

Default: `src/lib.vx`

#### Binary Package

```json
{
  "name": "my-cli-tool",
  "main": "src/main.vx",
  "bin": {
    "my-tool": "src/main.vx"
  }
}
```

Creates executable: `vex-builds/my-tool`

---

## Lock File: vex.lock

### Format

```json
{
  "version": 1,
  "lockTime": "2025-11-07T15:30:00Z",
  "dependencies": {
    "github.com/vex-lang/json": {
      "version": "v1.2.0",
      "resolved": "https://github.com/vex-lang/json/archive/v1.2.0.tar.gz",
      "integrity": "sha256:a3f5b2c1d4e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z7",
      "dependencies": {
        "github.com/vex-lang/utf8": "v2.1.0"
      }
    },
    "github.com/vex-lang/utf8": {
      "version": "v2.1.0",
      "resolved": "https://github.com/vex-lang/utf8/archive/v2.1.0.tar.gz",
      "integrity": "sha256:b4g6c3d5e7f8g9h0i1j2k3l4m5n6o7p8q9r0s1t2u3v4w5x6y7z8"
    }
  }
}
```

### Properties

- **version**: Lock file format version (currently 1)
- **lockTime**: Timestamp of lock generation
- **integrity**: SHA-256 checksum for validation
- **resolved**: Full download URL (immutability)
- **dependencies**: Transitive dependencies (flat tree)

### Management

- ✅ Auto-generated by `vex add`, `vex update`
- ✅ Committed to Git
- ❌ Never manually edited

---

## CLI Commands

### Phase 1 Commands

```bash
# Project management
vex new <name>              # Create new project
vex init                    # Initialize vex.json in existing dir

# Dependency management
vex add <package>[@version] # Add dependency
vex remove <package>        # Remove dependency
vex update                  # Update all dependencies

# Build & run
vex build                   # Build (development profile)
vex build --release         # Build (production profile)
vex build --profile=testing # Build (custom profile)
vex run                     # Build and run
vex test                    # Run tests
vex clean                   # Clean build cache

# Information
vex list                    # List dependencies
vex tree                    # Show dependency tree
vex outdated                # Show outdated dependencies
```

### Examples

```bash
# GitHub package
vex add github.com/vex-lang/json@v1.2.0
vex add github.com/vex-lang/json  # Latest version

# GitLab package
vex add gitlab:company/internal-lib@v1.5.0

# HTTP direct
vex add https://cdn.example.com/pkg-v1.0.0.tar.gz

# Remove
vex remove json

# Update all
vex update
```

---

## Dependency Resolution

### Strategy: Go-style Flat Dependency Tree

#### Minimum Version Selection (MVS)

When multiple versions required, **highest version wins**:

```
Project dependencies:
  - github.com/vex-lang/json@v1.2.0
    └── github.com/vex-lang/utf8@v2.1.0

  - github.com/vex-lang/router@v3.0.0
    └── github.com/vex-lang/utf8@v2.0.0

Resolution:
  ✅ utf8@v2.1.0 (highest SemVer)
```

#### Conflict Resolution

SemVer incompatible versions cause **build error**:

```
Error: Version conflict for github.com/vex-lang/utf8
  - json@v1.2.0 requires utf8@v2.x
  - old-lib@v1.0.0 requires utf8@v1.x
  ❌ Cannot resolve (major version conflict)

Solution: Update old-lib or use different version
```

### Versioning Rules

- **Semantic Versioning (SemVer)**: `v1.2.3` (major.minor.patch)
- **v prefix required**: `v1.0.0` not `1.0.0` (Go convention)
- **Compatible**: `v1.2.x` compatible with `v1.3.x`
- **Incompatible**: `v1.x.x` incompatible with `v2.x.x`

---

## Authentication

### Strategy: System Git Credentials

`vex` uses existing git configuration and SSH keys.

#### GitHub Private Repos

```bash
# SSH key (recommended)
ssh-add ~/.ssh/id_rsa

# HTTPS token
git config --global credential.helper store
echo "https://username:token@github.com" >> ~/.git-credentials
```

`vex add github.com/company/private-repo@v1.0.0`:

1. Uses system git credentials
2. Prefers SSH if key available
3. Falls back to HTTPS + token
4. Errors if authentication fails

#### GitLab Private Repos

```bash
# SSH key
ssh-add ~/.ssh/id_gitlab

# Access token
export GITLAB_TOKEN=your_token  # Optional
```

`vex add gitlab:company/internal-lib@v1.0.0`:

- Uses GitLab SSH/HTTPS credentials
- Reads `GITLAB_TOKEN` if set

#### HTTP Headers (Private CDN)

```json
"https://private-cdn.com/pkg.tar.gz": {
    "headers": {
        "Authorization": "Bearer secret_token"
    }
}
```

### Advantages

- ✅ No Vex-specific configuration needed
- ✅ Works with existing git workflow
- ✅ SSH keys + credential helpers supported
- ✅ CI/CD compatible (system git config)

---

## Package Structure

### Standard Layout

```
my-package/
├── vex.json              # Package manifest
├── vex.lock              # Lock file (committed to git)
├── src/
│   ├── lib.vx            # Default entrypoint (public API)
│   ├── internal.vx       # Private module
│   └── utils.vx          # Internal utilities
├── tests/
│   ├── lib_test.vx       # Unit tests
│   └── integration_test.vx
└── examples/
    └── basic_usage.vx    # Usage examples
```

### Entrypoint Rules

**Default**: `src/lib.vx`

**Custom**: Specify in `vex.json`

```json
{
  "name": "my-package",
  "main": "src/custom_entry.vx"
}
```

### Export Rules

Public API defined in entrypoint:

```vex
// src/lib.vx
export fn parse(data: string): Result<Json, Error> {
    // Public function
}

fn internal_helper(): i32 {
    // Private function (no export)
}
```

Usage in other packages:

```vex
import { parse } from "github.com/vex-lang/json";

fn main(): i32 {
    let json = parse("{\"key\": \"value\"}");
    return 0;
}
```

### Binary vs Library

**Library Package** (default):

```json
{
  "name": "json-parser",
  "version": "1.0.0",
  "main": "src/lib.vx"
}
```

**Binary Package** (executable):

```json
{
  "name": "my-cli-tool",
  "version": "1.0.0",
  "bin": {
    "my-tool": "src/main.vx"
  }
}
```

`vex build` → creates `vex-builds/my-tool` binary

---

## Implementation Phases

### Phase 0.1: MVP Foundation (Week 1-2)

**Timeline:** 1-2 weeks  
**Priority:** 🔴 CRITICAL

**Core Infrastructure:**

- [ ] **Manifest Parser** (`vex-pm/src/manifest.rs`)
  - Parse `vex.json` to structured AST
  - Validate semver versions
  - Validate dependency URLs
- [ ] **Platform File Selector** (`vex-pm/src/platform.rs`)
  - Detect current OS + instruction set
  - Implement file selection algorithm
  - Support `--target` flag override
- [ ] **Stdlib Integration** (`vex-compiler/src/stdlib.rs`)
  - Auto-import `std` module paths
  - Platform-specific stdlib file selection
  - Version synchronization with compiler

**Commands:**

```bash
vex new my_project        # ✅ Create project with vex.json
vex init                  # ✅ Initialize vex.json in existing dir
vex build                 # ✅ Platform-aware compilation
vex build --target=wasm   # ✅ Cross-platform build
```

**File Structure:**

```
vex-lang/
├── vex-pm/              # Package manager crate (NEW)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs       # Public API
│       ├── manifest.rs  # vex.json parser
│       ├── platform.rs  # Platform file selector
│       └── cli.rs       # CLI commands (new, init)
├── vex-compiler/
│   └── src/
│       └── stdlib.rs    # Stdlib integration (NEW)
└── vex-libs/std/        # Standard library (ENHANCED)
    ├── io/
    │   ├── lib.vx
    │   ├── file.linux.vx
    │   └── file.windows.vx
    └── ...
```

---

### Phase 0.2: Dependency Resolution (Week 3)

**Timeline:** 1 week  
**Priority:** 🔴 CRITICAL

**Features:**

- [ ] **Git Integration** (`vex-pm/src/git.rs`)
  - Clone repositories
  - Checkout specific tags/commits
  - Use system git credentials
- [ ] **Global Cache** (`vex-pm/src/cache.rs`)
  - `~/.vex/cache/` management
  - SHA-256 integrity verification
  - Deduplicate dependencies
- [ ] **Dependency Resolver** (`vex-pm/src/resolver.rs`)
  - Minimum Version Selection (MVS)
  - Conflict detection
  - Flat dependency tree

**Commands:**

```bash
vex add github.com/user/pkg@v1.0.0    # ✅ Add dependency
vex remove pkg                         # ✅ Remove dependency
vex list                               # ✅ List all dependencies
```

**Data Structures:**

```rust
// vex-pm/src/resolver.rs
pub struct DependencyGraph {
    packages: HashMap<String, Package>,
    resolved: HashMap<String, Version>,
}

impl DependencyGraph {
    pub fn resolve(&mut self) -> Result<Vec<Package>, ResolutionError>;
    pub fn detect_conflicts(&self) -> Vec<Conflict>;
}
```

---

### Phase 0.3: Lock File & Build Integration (Week 4)

**Timeline:** 1 week  
**Priority:** 🟡 HIGH

**Features:**

- [ ] **Lock File Generator** (`vex-pm/src/lockfile.rs`)
  - Generate `vex.lock` with SHA-256 checksums
  - Validate existing lock files
  - Update on dependency changes
- [ ] **Build Integration** (`vex-compiler/src/build.rs`)
  - Resolve dependencies before compilation
  - Link external packages
  - Handle platform-specific imports

**Commands:**

```bash
vex build                  # ✅ Auto-resolve dependencies
vex build --locked         # ✅ Use lock file (CI mode)
vex update                 # ✅ Update dependencies + lock
vex clean                  # ✅ Clean cache + builds
```

**Lock File Format:**

```json
{
  "version": 1,
  "lockTime": "2025-11-08T12:00:00Z",
  "dependencies": {
    "github.com/vex-lang/json": {
      "version": "v1.2.0",
      "resolved": "https://github.com/vex-lang/json/archive/v1.2.0.tar.gz",
      "integrity": "sha256:abc123...",
      "platform": {
        "default": "src/lib.vx",
        "wasm": "src/lib.wasm.vx"
      }
    }
  }
}
```

---

### Phase 1: MVP (Minimum Viable Product)

**Timeline:** 2-3 weeks  
**Status:** Phase 0.1-0.3 combined

**Features:**

- ✅ `vex.json` format (JSON)
- ✅ Git-based dependencies (GitHub, GitLab)
- ✅ Semantic versioning (`v1.2.3`)
- ✅ Global cache (`~/.vex/cache/`)
- ✅ Lock file (`vex.lock` with SHA-256)
- ✅ System git authentication
- ✅ Go-style dependency resolution (MVS)
- ✅ **Platform-specific file naming** ⭐ NEW
- ✅ **Stdlib integration** ⭐ NEW
- ✅ Basic CLI commands (new, add, remove, build, run)

**Commands:**

```bash
vex new my_project
vex add github.com/user/pkg@v1.0.0
vex build
vex build --target=wasm
vex run
```

**Not Included:**

- ❌ FFI dependencies
- ❌ HTTP direct download
- ❌ Nexus mirror
- ❌ Advanced commands (tree, outdated)

---

### Phase 2: HTTP & Advanced Features

**Timeline:** 1-2 weeks after Phase 1

**Features:**

- ✅ HTTP/HTTPS direct download
- ✅ Private URL headers
- ✅ Dependency tree visualization (`vex tree`)
- ✅ Outdated package detection (`vex outdated`)
- ✅ Lock file validation

**Commands:**

```bash
vex add https://cdn.com/pkg.tar.gz
vex tree
vex outdated
vex verify  # Verify lock file integrity
```

---

### Phase 3: FFI Dependencies

**Timeline:** 2-3 weeks (after language is stable)

**Features:**

- ✅ System-installed FFI libs (pkg-config)
- ✅ `ffiDependencies` in `vex.json`
- ✅ Pre-built binary downloads (Nexus)
- ✅ Source compilation fallback

**Example:**

```json
{
  "ffiDependencies": {
    "openssl": "system",
    "zlib": "3.0.0"
  }
}
```

---

### Phase 4: Nexus Mirror & Ecosystem

**Timeline:** 1-2 months (after v1.0)

**Features:**

- ✅ Nexus CDN mirror (`nexus.vex.dev`)
- ✅ Package publishing workflow
- ✅ Package search & discovery
- ✅ Documentation hosting
- ✅ CI/CD integration guides

**Environment:**

```bash
export VEX_PROXY=https://nexus.vex.dev
vex build  # Uses mirror
```

---

## Standard Library Integration

### Built-in `std` Module

The standard library is **compiled into the Vex binary** and requires no dependency management.

**Version Synchronization:**

- `vex v1.0.0` → `std v1.0.0`
- `vex v1.1.0` → `std v1.1.0`

**Usage:**

```vex
import { http, json, io } from "std";

fn main(): i32 {
    let response = http.get("https://api.example.com");
    let data = json.parse(response.body);
    io.println("Data:", data);
    return 0;
}
```

### Standard Library Modules

Located in `vex-libs/std/`:

```
vex-libs/std/
├── io/
│   ├── lib.vx              # println, print, readLine
│   ├── file.vx             # File operations
│   ├── file.linux.vx       # Linux-specific (epoll)
│   └── file.windows.vx     # Windows-specific (IOCP)
├── http/
│   ├── lib.vx              # HTTP client/server
│   ├── client.vx           # GET, POST, etc.
│   └── server.vx           # HTTP server
├── json/
│   └── lib.vx              # JSON parse/stringify
├── time/
│   └── lib.vx              # Time operations
├── crypto/
│   ├── lib.vx              # Generic
│   ├── sha256.x64.vx       # AVX2 optimized
│   └── sha256.arm64.vx     # NEON optimized
└── net/
    ├── lib.vx              # Generic TCP/UDP
    ├── tcp.linux.vx        # Linux epoll
    ├── tcp.macos.vx        # macOS kqueue
    └── tcp.windows.vx      # Windows IOCP
```

### Stdlib Discovery

**Compiler Search Path:**

1. `$VEX_HOME/libs/std/` (if `VEX_HOME` set)
2. `./vex-libs/std/` (source tree)
3. `/usr/local/lib/vex/std/` (system install)
4. `~/.vex/std/` (user install)

**Platform File Selection:**

```bash
# Compiling on Linux x64
import { tcp } from "std/net";
# Uses: vex-libs/std/net/tcp.linux.vx (if exists)
# Falls back to: vex-libs/std/net/lib.vx
```

### No Dependency in vex.json

`std` is **implicit** - no need to declare:

```json
{
  "name": "my-app",
  "version": "1.0.0",
  "dependencies": {
    // ❌ No need for "std": "1.0.0"
    "github.com/vex-lang/json": "v1.2.0" // ✅ External dependency
  }
}
```

### Stdlib Versioning

**Guaranteed Compatibility:**

- `std` API is stable within major version
- `vex v1.x.x` → `std v1.x.x` (always compatible)
- Breaking changes only in major versions (v2.0.0)

**Version Check:**

```bash
vex --version
# vex 1.0.0 (std 1.0.0)
```

---

## Appendix: Standard Library

### `std` Module

- `std` is **built into the compiler** (not a dependency)
- `vex` version = `std` version (e.g., `vex v1.0.0` → `std v1.0.0`)
- No download required

**Usage:**

```vex
import { http } from "std";  // Always available

fn main(): i32 {
    let response = http.get("https://example.com");
    return 0;
}
```

**Advantages:**

- ✅ No dependency management needed
- ✅ Version consistency guaranteed
- ✅ Fast builds (pre-compiled std lib)

---

## Summary

| Feature              | Phase 1 (MVP) | Phase 2 | Phase 3 | Phase 4 |
| -------------------- | ------------- | ------- | ------- | ------- |
| **vex.json**         | ✅            | ✅      | ✅      | ✅      |
| **vex.lock**         | ✅            | ✅      | ✅      | ✅      |
| **Git dependencies** | ✅            | ✅      | ✅      | ✅      |
| **HTTP download**    | ❌            | ✅      | ✅      | ✅      |
| **FFI dependencies** | ❌            | ❌      | ✅      | ✅      |
| **Nexus mirror**     | ❌            | ❌      | ❌      | ✅      |
| **System auth**      | ✅            | ✅      | ✅      | ✅      |
| **MVS resolution**   | ✅            | ✅      | ✅      | ✅      |

---

## Examples

### 1. Cross-Platform HTTP Server

**Project Structure:**

```
my-server/
├── vex.json
├── src/
│   ├── lib.vx                 # Generic fallback
│   ├── lib.linux.vx           # Linux epoll
│   ├── lib.macos.vx           # macOS kqueue
│   ├── lib.windows.vx         # Windows IOCP
│   └── lib.wasm.vx            # Browser WASM
└── tests/
    └── server_test.vx
```

**vex.json:**

```json
{
  "name": "my-server",
  "version": "1.0.0",
  "description": "Cross-platform HTTP server",

  "dependencies": {
    "github.com/vex-lang/http": "v2.0.0"
  },

  "targets": {
    "default": "x64",
    "supported": ["x64", "arm64", "wasm"]
  }
}
```

**Build:**

```bash
# Native build (auto-detect)
vex build
# → Uses lib.linux.vx on Linux, lib.macos.vx on macOS

# Cross-compile for WASM
vex build --target=wasm
# → Uses lib.wasm.vx

# Production build
vex build --release --target=arm64
# → Uses lib.vx (fallback) or lib.arm64.vx if exists
```

---

### 2. SIMD-Optimized Crypto

**Project Structure:**

```
vex-crypto/
├── vex.json
└── src/
    ├── sha256.vx           # Generic fallback
    ├── sha256.x64.vx       # AVX2 optimized
    └── sha256.arm64.vx     # NEON optimized
```

**Usage:**

```vex
// Automatically uses platform-optimized version
import { sha256 } from "vex-crypto";

fn main(): i32 {
    let hash = sha256("hello world");
    // Uses sha256.x64.vx on x86-64
    // Uses sha256.arm64.vx on ARM64
    // Falls back to sha256.vx on other platforms
    return 0;
}
```

---

### 3. Standard Library with Platform Files

**Stdlib Structure:**

```
vex-libs/std/
├── io/
│   ├── lib.vx              # Generic I/O
│   ├── file.linux.vx       # Linux specific
│   ├── file.macos.vx       # macOS specific
│   └── file.windows.vx     # Windows specific
└── net/
    ├── lib.vx              # Generic TCP/UDP
    ├── tcp.linux.vx        # epoll
    ├── tcp.macos.vx        # kqueue
    └── tcp.windows.vx      # IOCP
```

**User Code:**

```vex
import { File } from "std/io";
import { TcpListener } from "std/net";

fn main(): i32 {
    // Automatically uses platform-specific implementation
    let file = File.open("/tmp/test.txt");
    let listener = TcpListener.bind("127.0.0.1:8080");

    // On Linux: uses file.linux.vx + tcp.linux.vx (epoll)
    // On macOS: uses file.macos.vx + tcp.macos.vx (kqueue)
    // On Windows: uses file.windows.vx + tcp.windows.vx (IOCP)

    return 0;
}
```

---

## Summary

| Feature                   | Phase 0.1 | Phase 0.2 | Phase 0.3 | Phase 2 | Phase 3 | Phase 4 |
| ------------------------- | --------- | --------- | --------- | ------- | ------- | ------- |
| **vex.json**              | ✅        | ✅        | ✅        | ✅      | ✅      | ✅      |
| **vex.lock**              | ❌        | ❌        | ✅        | ✅      | ✅      | ✅      |
| **Platform files**        | ✅        | ✅        | ✅        | ✅      | ✅      | ✅      |
| **Stdlib integration**    | ✅        | ✅        | ✅        | ✅      | ✅      | ✅      |
| **Git dependencies**      | ❌        | ✅        | ✅        | ✅      | ✅      | ✅      |
| **Dependency resolution** | ❌        | ✅        | ✅        | ✅      | ✅      | ✅      |
| **Global cache**          | ❌        | ✅        | ✅        | ✅      | ✅      | ✅      |
| **HTTP download**         | ❌        | ❌        | ❌        | ✅      | ✅      | ✅      |
| **FFI dependencies**      | ❌        | ❌        | ❌        | ❌      | ✅      | ✅      |
| **Nexus mirror**          | ❌        | ❌        | ❌        | ❌      | ❌      | ✅      |
| **System auth**           | ❌        | ✅        | ✅        | ✅      | ✅      | ✅      |
| **MVS resolution**        | ❌        | ✅        | ✅        | ✅      | ✅      | ✅      |

---

**Status:** ✅ Specification complete with platform-specific files + stdlib integration

**Next Steps:**

1. Implement Phase 0.1 (Platform files + Stdlib)
2. Implement Phase 0.2 (Git + Cache + Resolver)
3. Implement Phase 0.3 (Lock file + Build integration)
4. Test with real-world packages
5. Gather community feedback

---

**Maintained by:** Vex Language Team  
**Contact:** meftunca (GitHub)  
**Last Updated:** November 8, 2025
