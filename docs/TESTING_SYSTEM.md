# Vex Testing System

> **Full Specification**: [Specifications/19_Package_Manager.md](../Specifications/19_Package_Manager.md) (Testing Section)

**Version:** 0.1.2  
**Last Updated:** November 11, 2025  
**Status:** Specification complete, implementation planned

---

## 📋 Overview

Vex's testing system provides automatic test discovery, parallel execution, and platform-specific test support.

### Key Features

- ✅ **Automatic Discovery**: Pattern-based search from project root
- ✅ **Pattern-Based**: Configurable test file patterns (default: `**/*.test.vx`)
- ✅ **Parallel Execution**: Run tests concurrently by default
- ✅ **Platform-Specific**: Support for OS/arch-specific tests
- ✅ **Timeout Control**: Configurable per-test timeouts
- ✅ **Simple CLI**: `vex test` discovers and runs all tests

---

## 🎯 Test File Naming Convention

### Required Pattern

**Default**: `*.test.vx`

All test files MUST follow this pattern for automatic discovery.

### Valid Examples

```
✅ basic.test.vx
✅ user_auth.test.vx
✅ api_integration.test.vx
✅ math_operations.test.vx
✅ string_utils.test.vx
```

### Invalid Examples

```
❌ basic_test.vx       # Missing .test before .vx
❌ test_basic.vx       # Wrong position
❌ basicTest.vx        # Wrong pattern
❌ basic.vx            # Missing .test
❌ basic.spec.vx       # Wrong keyword
```

### Custom Patterns

You can configure custom patterns in `vex.json`:

```json
{
  "testing": {
    "pattern": "*.spec.vx" // Use .spec.vx instead
  }
}
```

---

## 🔍 Test Discovery

Vex automatically discovers test files using the pattern specified in `vex.json`.

### How It Works

1. **Read Configuration**: Vex reads `vex.json` from project root
2. **Apply Pattern**: Uses glob pattern to search from project root
3. **Match Files**: Finds all files matching `**/*.test.vx` (recursive)
4. **Filter Platform**: Selects only platform-appropriate test files
5. **Execute**: Runs discovered tests in parallel (default)

### Important: Pattern Search Root

**The pattern searches from the project root** (where `vex.json` is located), NOT from the `dir` field.

**Default Configuration**:

```json
{
  "testing": {
    "dir": "tests", // Informational/organizational
    "pattern": "**/*.test.vx", // Searches from PROJECT ROOT
    "timeout": 30, // 30 seconds per test
    "parallel": true // Parallel execution enabled
  }
}
```

**Example**:

```
my-project/              ← vex.json here (PROJECT ROOT - search starts here)
├── vex.json
├── src/
│   └── lib.vx
└── tests/               ← dir field points here (organizational)
    ├── basic.test.vx           ✅ Found by pattern **/*.test.vx
    ├── integration.test.vx     ✅ Found
    └── unit/
        └── math.test.vx        ✅ Found (recursive **)
```

---

## 📁 Project Structure

### Recommended Layout

```
my-project/
├── vex.json
├── src/
│   ├── lib.vx
│   ├── math.vx
│   └── string.vx
└── tests/
    ├── basic.test.vx
    ├── math.test.vx
    └── string.test.vx
```

### Multi-Level Organization

```
my-project/
├── vex.json
├── src/
│   └── lib.vx
└── tests/
    ├── unit/
    │   ├── math.test.vx
    │   └── string.test.vx
    └── integration/
        ├── api.test.vx
        └── db.test.vx
```

### Custom Test Directory

```json
{
  "testing": {
    "dir": "test" // Use "test" instead of "tests"
  }
}
```

---

## ⚙️ Configuration

### vex.json Testing Section

```json
{
  "testing": {
    "dir": "tests", // Test directory (relative to vex.json)
    "pattern": "**/*.test.vx", // Glob pattern (searches from root)
    "timeout": 30, // Timeout per test (seconds)
    "parallel": true // Run in parallel
  }
}
```

### Field Descriptions

| Field      | Type    | Default          | Description                               |
| ---------- | ------- | ---------------- | ----------------------------------------- |
| `dir`      | string  | `"tests"`        | Test directory (relative to vex.json)     |
| `pattern`  | string  | `"**/*.test.vx"` | Glob pattern (searches from project root) |
| `timeout`  | number  | none             | Max execution time per test (seconds)     |
| `parallel` | boolean | `true`           | Run tests concurrently                    |

### Minimal Configuration

```json
{
  "name": "my-lib",
  "version": "1.0.0"
  // testing uses all defaults
}
```

Defaults to:

- Directory: `tests/`
- Pattern: `**/*.test.vx` (searches from project root)
- Parallel: `true`
- No timeout

### Custom Configuration

```json
{
  "testing": {
    "dir": "spec",
    "pattern": "**/*.spec.vx", // Searches from project root
    "timeout": 60,
    "parallel": false
  }
}
```

---

## 🚀 Running Tests

### Basic Commands

```bash
# Discover and run all tests
vex test

# Run specific test file
vex test tests/basic.test.vx

# Run tests in directory
vex test tests/unit/

# Run with custom timeout
vex test --timeout 60

# Run sequentially (disable parallel)
vex test --no-parallel

# Verbose output
vex test --verbose
```

### Test Discovery Process

1. Read `vex.json` → Get `testing.dir` and `testing.pattern`
2. Search from **project root** (where `vex.json` is) using glob pattern `**/*.test.vx`
3. Collect all matching files recursively
4. Execute tests (parallel or sequential)
5. Report results

**Note**: Pattern search starts from the directory containing `vex.json`, not from the `dir` field. The `dir` field is informational and for organization.

**Example**:

```
tests/
├── basic.test.vx           ✅ Discovered
├── math.test.vx            ✅ Discovered
├── string.test.vx          ✅ Discovered
├── helper.vx               ❌ Skipped (not *.test.vx)
└── README.md               ❌ Skipped (not .vx)
```

---

## 🎨 Test File Structure

### Basic Test

```vex
// basic.test.vx
import { assert_eq, assert } from "testing";

fn test_addition() {
    let result = 2 + 2;
    assert_eq(result, 4);
}

fn test_subtraction() {
    let result = 10 - 5;
    assert_eq(result, 5);
}

fn main(): i32 {
    test_addition();
    test_subtraction();
    return 0;
}
```

### Using Testing Framework

```vex
// math.test.vx
import { T, run_test } from "testing";

fn test_multiply(t: *T) {
    let result = 3 * 4;
    if result != 12 {
        t.error("Expected 12, got " + result.to_string());
    }
}

fn test_divide(t: *T) {
    let result = 20 / 4;
    if result != 5 {
        t.error("Expected 5, got " + result.to_string());
    }
}

fn main(): i32 {
    run_test("multiply", test_multiply);
    run_test("divide", test_divide);
    return 0;
}
```

---

## 🌍 Platform-Specific Tests

Vex supports platform-specific test files using OS and architecture suffixes.

### File Naming

Tests support platform-specific variants using the same suffix rules as regular source files:

```
tests/
├── io.test.vx                    # Generic tests (always included)
├── io.test.macos.vx              # macOS-specific tests
├── io.test.linux.vx              # Linux-specific tests
├── io.test.x64.vx                # x64-specific tests
├── io.test.macos.arm64.vx        # macOS ARM64-specific tests
└── io.test.linux.x64.vx          # Linux x64-specific tests
```

### Platform Selection Priority

When running `vex test`, the system selects test files using this priority order:

1. **OS-Architecture**: `{name}.test.{os}.{arch}.vx` (e.g., `io.test.macos.arm64.vx`)
2. **OS-Only**: `{name}.test.{os}.vx` (e.g., `io.test.macos.vx`)
3. **Architecture-Only**: `{name}.test.{arch}.vx` (e.g., `io.test.arm64.vx`)
4. **Generic**: `{name}.test.vx` (fallback)

**Only the highest-priority matching file is selected** for execution.

### Supported Platforms

| Operating System | Suffix     | Example              |
| ---------------- | ---------- | -------------------- |
| Linux            | `.linux`   | `io.test.linux.vx`   |
| macOS            | `.macos`   | `io.test.macos.vx`   |
| Windows          | `.windows` | `io.test.windows.vx` |
| FreeBSD          | `.freebsd` | `io.test.freebsd.vx` |

| Architecture | Suffix   | Example            |
| ------------ | -------- | ------------------ |
| x86-64       | `.x64`   | `io.test.x64.vx`   |
| ARM64        | `.arm64` | `io.test.arm64.vx` |
| WebAssembly  | `.wasm`  | `io.test.wasm.vx`  |

### Example: Cross-Platform I/O Tests

**Generic tests** (`io.test.vx`):

```vex
// Tests that work on all platforms
import { assert_eq } from "testing";

fn test_file_exists() {
    let path = "test_data.txt";
    let exists = File.exists(path);
    assert_eq(exists, true);
}

fn main(): i32 {
    test_file_exists();
    return 0;
}
```

**macOS-specific tests** (`io.test.macos.vx`):

```vex
// Tests using macOS-specific APIs
import { assert } from "testing";

fn test_kqueue() {
    // Test kqueue event notification mechanism
    let kq = kqueue();
    assert(kq >= 0, "Failed to create kqueue");
}

fn main(): i32 {
    test_kqueue();
    return 0;
}
```

**Linux-specific tests** (`io.test.linux.vx`):

```vex
// Tests using Linux-specific APIs
import { assert } from "testing";

fn test_epoll() {
    // Test epoll event notification mechanism
    let epfd = epoll_create1(0);
    assert(epfd >= 0, "Failed to create epoll instance");
}

fn main(): i32 {
    test_epoll();
    return 0;
}
```

### Running Platform-Specific Tests

```bash
# Automatically runs tests for current platform
vex test

# Example on macOS ARM64:
# Runs: io.test.macos.vx (if exists)
# Skips: io.test.linux.vx, io.test.windows.vx

# On Linux x64:
# Runs: io.test.linux.x64.vx (if exists), else io.test.linux.vx
# Skips: io.test.macos.vx, io.test.windows.vx
```

> **Full Specification**: See [Specifications/19_Package_Manager.md](../Specifications/19_Package_Manager.md) for complete platform-specific file selection rules.

---

## ⚡ Parallel Execution

### Default Behavior

By default, tests run in parallel:

```bash
vex test  # Runs all tests concurrently
```

**Benefits**:

- ✅ Faster execution
- ✅ Better CPU utilization
- ✅ Ideal for unit tests

### Sequential Execution

Disable parallel execution for:

- Integration tests that share state
- Tests that access same resources
- Debugging test failures

```bash
vex test --no-parallel
```

Or in `vex.json`:

```json
{
  "testing": {
    "parallel": false
  }
}
```

### Per-Test Isolation

Each test file runs in its own process:

- ✅ No shared state between tests
- ✅ Test failures are isolated
- ✅ Clean environment per test

---

## ⏱️ Timeout Control

### Global Timeout

Set maximum execution time for all tests:

```json
{
  "testing": {
    "timeout": 30 // 30 seconds per test
  }
}
```

### CLI Override

```bash
vex test --timeout 60  # 60 seconds
```

### No Timeout

```json
{
  "testing": {
    "timeout": null // No timeout
  }
}
```

Or:

```bash
vex test --no-timeout
```

---

## 📊 Test Output

### Success Output

```
Running tests...
✅ tests/basic.test.vx ... ok (12ms)
✅ tests/math.test.vx ... ok (8ms)
✅ tests/string.test.vx ... ok (15ms)

Test result: ok. 3 passed; 0 failed; 0 ignored
```

### Failure Output

```
Running tests...
✅ tests/basic.test.vx ... ok (12ms)
❌ tests/math.test.vx ... FAILED
✅ tests/string.test.vx ... ok (15ms)

Failures:

---- tests/math.test.vx ----
Expected 12, got 13
  at test_multiply (math.test.vx:5)

Test result: FAILED. 2 passed; 1 failed; 0 ignored
```

### Verbose Output

```bash
vex test --verbose
```

Shows:

- Test discovery process
- Individual test function results
- Execution times
- Platform selection details

---

## 🎯 Test Organization Best Practices

### Unit Tests

Test individual functions/modules:

```
tests/
├── math.test.vx
├── string.test.vx
└── utils.test.vx
```

### Integration Tests

Test module interactions:

```
tests/
├── api_integration.test.vx
├── db_integration.test.vx
└── workflow.test.vx
```

### Mixed Approach

```
tests/
├── unit/
│   ├── math.test.vx
│   ├── string.test.vx
│   └── utils.test.vx
└── integration/
    ├── api.test.vx
    └── db.test.vx
```

### Naming Conventions

**Good**:

```
✅ user_auth.test.vx       # Clear purpose
✅ api_get_user.test.vx    # Specific functionality
✅ db_connection.test.vx   # Module context
```

**Bad**:

```
❌ test1.test.vx           # Unclear
❌ stuff.test.vx           # Vague
❌ temp.test.vx            # Meaningless
```

---

## 🔧 Implementation Status

### Completed (v0.1.2)

- ✅ Test configuration in `vex.json`
- ✅ Manifest parsing (`TestingConfig` struct)
- ✅ Default values (dir, pattern, parallel)
- ✅ Specification documented

### Planned (Future)

- ⏳ Test discovery implementation
- ⏳ Parallel test runner
- ⏳ Timeout enforcement
- ⏳ Test result reporting
- ⏳ Platform-specific test selection
- ⏳ CLI commands (`vex test`)

---

## 📝 Example Configurations

### Stdlib Module

```json
{
  "name": "math",
  "version": "0.2.0",
  "main": "src/lib.vx",
  "testing": {
    "dir": "tests",
    "pattern": "**/*.test.vx"
  }
}
```

### Application

```json
{
  "name": "my-app",
  "version": "1.0.0",
  "main": "src/main.vx",
  "testing": {
    "dir": "tests",
    "pattern": "**/*.test.vx",
    "timeout": 60,
    "parallel": true
  }
}
```

### Library with Custom Tests

```json
{
  "name": "my-lib",
  "version": "2.0.0",
  "testing": {
    "dir": "spec",
    "pattern": "**/*.spec.vx",
    "timeout": 30,
    "parallel": false
  }
}
```

---

## 🎯 Go-Inspired Advanced Features

> **Note**: The following features are Go-inspired extensions beyond the core testing specification.
> They are planned for future implementation and are not part of the current v0.1.2 specification.
>
> **Core Testing Specification**: See [Specifications/19_Package_Manager.md](../Specifications/19_Package_Manager.md) for the current specification.

### Feature Overview

| Feature                | Status     | Description                                       |
| ---------------------- | ---------- | ------------------------------------------------- |
| **Benchmarking**       | 🔄 Planned | Performance testing with `*.bench.vx` files       |
| **Table-Driven Tests** | 🔄 Planned | Data-driven test patterns                         |
| **Subtests**           | 🔄 Planned | Hierarchical test organization with `t.run()`     |
| **Fuzzing**            | 🔄 Planned | Automated input generation for robustness testing |
| **Coverage**           | 🔄 Planned | Code coverage tracking and reporting              |
| **Test Helpers**       | 🔄 Planned | `t.Helper()` for better error reporting           |

---

### 1. Benchmarking

**Pattern**: `*.bench.vx` files

```vex
// math.bench.vx
import { B } from "testing";

fn bench_fibonacci(b: *B) {
    for b.loop() {
        fibonacci(20);
    }
}

fn bench_factorial(b: *B) {
    b.reset_timer();  // Exclude setup time
    for b.loop() {
        factorial(100);
    }
}
```

**CLI**:

```bash
vex bench                    # Run all benchmarks
vex bench --time 10s         # Run for 10 seconds
vex bench --count 5          # Run 5 times
vex bench --benchmem         # Include memory stats
```

**Output**:

```
BenchmarkFibonacci-8    1000000    1234 ns/op    512 B/op    10 allocs/op
BenchmarkFactorial-8    5000000     245 ns/op    128 B/op     2 allocs/op
```

---

### 2. Table-Driven Tests

**Pattern**: Test multiple cases with single function

```vex
// calculator.test.vx
import { T, assert_eq } from "testing";

struct TestCase {
    name: String,
    input: i32,
    expected: i32,
}

fn test_square(t: *T) {
    let cases = [
        TestCase { name: "zero", input: 0, expected: 0 },
        TestCase { name: "positive", input: 5, expected: 25 },
        TestCase { name: "negative", input: -3, expected: 9 },
        TestCase { name: "large", input: 100, expected: 10000 },
    ];

    for case in cases {
        t.run(case.name, fn(t: *T) {
            let result = square(case.input);
            assert_eq(result, case.expected);
        });
    }
}
```

**Output**:

```
=== RUN   test_square
=== RUN   test_square/zero
=== RUN   test_square/positive
=== RUN   test_square/negative
=== RUN   test_square/large
--- PASS: test_square (0.00s)
    --- PASS: test_square/zero (0.00s)
    --- PASS: test_square/positive (0.00s)
    --- PASS: test_square/negative (0.00s)
    --- PASS: test_square/large (0.00s)
```

---

### 3. Subtests (Hierarchical Tests)

**Pattern**: Nested test organization with `t.run()`

```vex
// user.test.vx
fn test_user_validation(t: *T) {
    t.run("email", fn(t: *T) {
        t.run("valid", fn(t: *T) {
            assert(validate_email("user@example.com"));
        });

        t.run("invalid", fn(t: *T) {
            assert(!validate_email("invalid-email"));
        });
    });

    t.run("password", fn(t: *T) {
        t.run("strong", fn(t: *T) {
            assert(validate_password("Str0ng!Pass"));
        });

        t.run("weak", fn(t: *T) {
            assert(!validate_password("weak"));
        });
    });
}
```

**Run specific subtests**:

```bash
vex test --run test_user_validation/email        # Only email tests
vex test --run test_user_validation/email/valid  # Only valid email test
```

---

### 4. Examples (Testable Documentation)

**Pattern**: `example_*.vx` files with output verification

```vex
// example_hello.vx
import { println } from "io";

fn example_hello() {
    println("Hello, World!");
    // Output: Hello, World!
}

fn example_greet() {
    println("Good morning");
    println("Good evening");
    // Output:
    // Good morning
    // Good evening
}

fn example_unordered() {
    let items = ["apple", "banana", "cherry"];
    for item in items {
        println(item);
    }
    // Unordered output:
    // apple
    // banana
    // cherry
}
```

**Benefits**:

- Serves as documentation
- Auto-verified by tests
- Appears in generated docs

---

### 5. Fuzzing (Property-Based Testing)

**Pattern**: `fuzz_*.vx` files

```vex
// fuzz_parser.vx
import { F } from "testing";

fn fuzz_json_parser(f: *F) {
    f.add(b"{}");              // Seed corpus
    f.add(b"{\"key\":\"val\"}");
    f.add(b"[]");

    f.fuzz(fn(t: *T, data: []byte) {
        // Parser should never crash on any input
        let result = parse_json(data);

        // If valid JSON, re-serializing should match
        if result.is_ok() {
            let serialized = serialize_json(result.unwrap());
            // Property: parse(serialize(x)) == x
        }
    });
}
```

**CLI**:

```bash
vex test --fuzz FuzzJsonParser              # Run fuzzer
vex test --fuzz FuzzJsonParser --fuzztime 1m # Fuzz for 1 minute
```

**Auto-saves crash inputs to**:

```
testdata/fuzz/FuzzJsonParser/crash-input-1
testdata/fuzz/FuzzJsonParser/crash-input-2
```

---

### 6. Test Helpers

**Pattern**: `t.helper()` marks helper functions

```vex
// helpers.vx
fn assert_user_valid(t: *T, user: User) {
    t.helper();  // Errors report caller's line, not this line

    if user.email.is_empty() {
        t.error("User email is empty");
    }
    if user.age < 0 {
        t.error("User age is negative");
    }
}

// user.test.vx
fn test_create_user(t: *T) {
    let user = create_user("test@example.com", 25);
    assert_user_valid(t, user);  // Error points here, not inside helper
}
```

---

### 7. Test Cleanup

**Pattern**: `t.cleanup()` for resource cleanup

```vex
fn test_database_operations(t: *T) {
    let db = open_database("test.db");

    t.cleanup(fn() {
        db.close();
        remove_file("test.db");
    });

    // Test operations
    db.insert("key", "value");
    assert_eq(db.get("key"), "value");

    // cleanup() runs automatically even if test fails
}
```

---

### 8. Parallel Tests

**Pattern**: `t.parallel()` for concurrent execution

```vex
fn test_concurrent_safe(t: *T) {
    t.parallel();  // Run concurrently with other parallel tests

    // Independent test that doesn't share state
    let result = expensive_computation();
    assert_eq(result, 42);
}

fn test_another_concurrent(t: *T) {
    t.parallel();
    // Runs at same time as test_concurrent_safe
}
```

---

### 9. Test Coverage

**CLI**:

```bash
vex test --coverage                    # Show coverage %
vex test --coverprofile=coverage.out   # Generate profile
vex tool cover --html=coverage.out     # HTML report
```

**Output**:

```
PASS    coverage: 85.2% of statements
ok      myproject    0.123s
```

**Coverage Modes**:

- `--covermode=set`: Line coverage (covered or not)
- `--covermode=count`: Count executions per line
- `--covermode=atomic`: Thread-safe count

---

### 10. Test Skip

**Pattern**: Skip tests conditionally

```vex
fn test_linux_only(t: *T) {
    if !is_linux() {
        t.skip("Linux-only test");
    }

    // Linux-specific test
}

fn test_slow_integration(t: *T) {
    if testing.short() {
        t.skip("Skipping slow test in short mode");
    }

    // Expensive integration test
}
```

**CLI**:

```bash
vex test --short  # Skip slow tests
```

---

### 11. TestMain (Global Setup/Teardown)

**Pattern**: Single `test_main()` per package

```vex
// main.test.vx
import { M } from "testing";

fn test_main(m: *M) {
    // Global setup
    setup_test_database();
    initialize_test_env();

    // Run all tests
    let code = m.run();

    // Global teardown
    cleanup_test_database();
    shutdown_test_env();

    return code;
}
```

---

### 12. Custom Test Output

**Pattern**: Structured test logging

```vex
fn test_with_context(t: *T) {
    t.log("Starting test with context");
    t.logf("User ID: %d", user_id);

    if result.is_err() {
        t.errorf("Operation failed: %v", result.err());
    }
}
```

---

## 📊 Configuration Summary

### vex.json Complete Testing Config

```json
{
  "testing": {
    "dir": "tests",
    "pattern": "**/*.test.vx",
    "timeout": 30,
    "parallel": true,

    "benchmark": {
      "pattern": "**/*.bench.vx",
      "time": "1s",
      "count": 1,
      "benchmem": false
    },

    "fuzz": {
      "pattern": "**/*fuzz*.vx",
      "time": "10s",
      "corpus_dir": "testdata/fuzz"
    },

    "coverage": {
      "enabled": true,
      "mode": "set",
      "min_coverage": 80.0
    },

    "examples": {
      "pattern": "**/example_*.vx"
    }
  }
}
```

---

## 🚀 Future Implementation Roadmap

### Phase 1: Core Testing (v0.1.2)

- ✅ Test discovery (`**/*.test.vx`)
- ✅ Basic assertions
- ✅ Parallel execution
- ✅ Timeout support

### Phase 2: Advanced Features (v0.2.0)

- ⏳ Benchmarking (`*.bench.vx`)
- ⏳ Table-driven tests
- ⏳ Subtests (`t.run()`)
- ⏳ Test helpers (`t.helper()`)
- ⏳ Cleanup functions (`t.cleanup()`)

### Phase 3: Documentation & Analysis (v0.3.0)

- ⏳ Examples (`example_*.vx`)
- ⏳ Coverage reporting
- ⏳ HTML coverage reports
- ⏳ TestMain support

### Phase 4: Property Testing (v0.4.0)

- ⏳ Fuzzing framework
- ⏳ Seed corpus management
- ⏳ Crash input persistence
- ⏳ Coverage-guided fuzzing

---

**Maintained by**: Vex Language Team  
**Specification**: `Specifications/19_Package_Manager.md#Testing`  
**Implementation**: `vex-pm/src/manifest.rs` (`TestingConfig`)  
**Inspired by**: Go's `testing` package (go1.25)
