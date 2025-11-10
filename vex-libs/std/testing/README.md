# std/testing - Vex Testing Module

Comprehensive testing framework for Vex, wrapping the high-performance `vex_testing.c` C harness. Provides TAP/JUnit test reporting, fixtures, benchmarking with auto-calibration, and property-based testing.

## Features

- **TAP Reporter** - TAP version 13 output for CI/CD integration
- **JUnit Reporter** - XML output for Jenkins and similar tools
- **Fixtures** - Setup/teardown hooks (all and per-test)
- **Benchmarking** - Go-like `b.N` auto-calibration, statistical analysis
- **Property-Based Testing** - QuickCheck-style random testing
- **Parallel Execution** - Multi-threaded test runner
- **Memory Utilities** - Cache-aligned allocations
- **Optimization Helpers** - Black-box functions to prevent compiler optimization
- **CPU Control** - CPU pinning, real-time scheduling hints

## Module Status

**Version:** 0.1.2 (Syntax v0.1.2)  
**Stability:** Production-ready (FFI 100%, C compilation working)  
**C Foundation:** `vex_testing.c` (1724 lines, single-file, C17)  
**Exported Functions:** 40+

## Integration Pattern

The `std/testing` module follows the same Go-style integration pattern as `std/time`:

```
vex-libs/std/testing/
├── src/lib.vx                 # High-level Vex API (FFI bindings)
├── native/testing.c           # Symlink to vex-runtime/c/vex_testing.c
├── vex.json                   # Native C compilation config
├── tests/
│   └── smoke.vx               # Basic sanity check
└── examples/
    └── all_features.vx        # Feature demonstrations
```

## Quick Start

### Basic Assertions

```vex
import { assert_true, assert_eq_i64 } from "testing";

fn test_example() {
    assert_true(1 == 1, "one equals one");
    assert_eq_i64(42i64, 42i64, "forty-two");
}
```

### Running Tests

```vex
import { run_tests, TestCase } from "testing";

fn test_a() { /* ... */ }
fn test_b() { /* ... */ }

fn main() {
    let tests: [2]TestCase = [
        TestCase { name: "test_a", fn: test_a },
        TestCase { name: "test_b", fn: test_b },
    ];
    run_tests(&tests[0], 2);
}
```

### Fixtures (Setup/Teardown)

```vex
import { run_tests_with, fixture_full, TestCase } from "testing";

fn setup_all() { print("Global setup"); }
fn teardown_all() { print("Global teardown"); }
fn setup_each() { print("Per-test setup"); }
fn teardown_each() { print("Per-test teardown"); }

fn test_example() { /* ... */ }

fn main() {
    let tests: [1]TestCase = [
        TestCase { name: "test_example", fn: test_example },
    ];
    
    let fx: Fixture = fixture_full(setup_all, teardown_all, setup_each, teardown_each);
    run_tests_with("my_suite", &tests[0], 1, &fx);
}
```

### Benchmarking

```vex
import { BenchContext, BenchConfig, bench_run, bench_report_text } from "testing";

fn my_benchmark(ctx: *BenchContext) {
    // Code to benchmark runs ctx.iters times
    let sum: i64 = 0;
    for i in 0i64..100i64 {
        sum = sum + i;
    }
}

fn main() {
    let ctx: BenchContext = BenchContext { name: "loop", bytes_per_op: 0 };
    let cfg: BenchConfig = BenchConfig {
        name: "loop",
        iters: 0,
        time_ns: 0,
        warmup_iters: 0,
        warmup_ns: 20000000,
        pin_cpu: 0,
        repeats: 5,
        report_json: false,
        auto_calibrate: true,
        bytes_per_op: 0
    };
    
    let result = bench_run(my_benchmark, &ctx, cfg);
    bench_report_text(&result);
}
```

## API Reference

### Assertion Functions

- `assert_true(cond: bool, msg: String)` - Assert true
- `assert_false(cond: bool, msg: String)` - Assert false
- `assert_eq_i64(a: i64, b: i64, msg: String)` - Assert equal (int64)
- `assert_eq_u64(a: u64, b: u64, msg: String)` - Assert equal (uint64)
- `assert_eq_f64(a: f64, b: f64, epsilon: f64, msg: String)` - Assert approx equal
- `assert_neq_i64/u64/f64(...)` - Assert not equal variants
- `assert_lt/le/gt/ge_i64/u64(...)` - Comparison assertions
- `assert_null/not_null(ptr: *u8, msg: String)` - Pointer assertions

### Logging

- `log(msg: String)` - Write test log message

### Test Harness

- `run_tests(tests: *TestCase, count: u64) i32` - Run tests with defaults
- `run_tests_with(suite: String, tests: *TestCase, count: u64, fx: *Fixture) i32` - With fixtures
- `run_tests_parallel(suite: String, tests: *TestCase, count: u64, fx: *Fixture, threads: i32) i32` - Parallel

### Fixtures

- `fixture_all(setup_all: fn() void, teardown_all: fn() void) Fixture`
- `fixture_each(setup_each: fn() void, teardown_each: fn() void) Fixture`
- `fixture_full(setup_all, teardown_all, setup_each, teardown_each) Fixture`

### Benchmarking

- `bench_run(fn_ptr: BenchFn, ctx: *BenchContext, cfg: BenchConfig) BenchResult`
- `bench_report_text(r: *BenchResult)`
- `bench_report_json(r: *BenchResult, buf: *u8, bufsz: u64) String`

### Memory Utilities

- `aligned_alloc(alignment: u64, size: u64) *u8` - Alloc with alignment
- `aligned_free(ptr: *u8)` - Free aligned memory

### Optimization Helpers

- `black_box_i64/u64/f64/ptr(x) x` - Prevent compiler optimization

### Timing

- `monotonic_ns() u64` - Get monotonic time
- `read_cycles() u64` - Get CPU cycle count

### CPU Control

- `pin_to_cpu(cpu: i32)` - Pin thread to CPU
- `set_realtime_hint()` - Request real-time priority

### Property-Based Testing

- `gen_i64/u64/f64(seed: u64, min, max) T` - Generate random value
- `gen_bool(seed: u64) bool` - Generate random boolean

## Known Issues & Workarounds

**Issue 1: Struct Literal Scope**  
Struct literals created inline die immediately due to borrow checker.

**Workaround:** Create structs in assignment context:
```vex
let ctx: BenchContext = BenchContext { name: "test", bytes_per_op: 0 };
// Don't use: benchmark(BenchContext { ... })
```

**Issue 2: String Parameters**  
FFI functions accept String but conversion depends on Vex runtime.

**Status:** Works for literal strings; dynamic strings may need casting.

**Issue 3: Tuple Destructuring**  
Not supported in Vex v0.1.2:
```vex
let (a, b) = some_function();  // ❌ Not supported
```

**Workaround:** Access fields individually.

## Building & Testing

```bash
# Run smoke test
vex run vex-libs/std/testing/tests/smoke.vx

# Run examples
vex run vex-libs/std/testing/examples/all_features.vx

# C compilation
cd vex-libs/std/testing
clang -O3 -Wall -std=c17 -c native/testing.c
```

## Architecture

The module wraps `vex_testing.c` which provides:

- **Test Harness**: Test case registration, execution, result collection
- **Reporters**: TAP 13, JUnit XML, human-readable text
- **Fixtures**: Setup/teardown lifecycle with per-test isolation
- **Benchmarking Engine**:
  - Auto-calibration of iteration count to reach target time
  - Multiple samples (default 5) for statistical analysis
  - Percentile calculation (p90, p95, p99)
  - Memory throughput tracking (MB/s)
  - CPU cycle measurement (x86 RDTSC with serialization)
- **Property Testing**: xoroshiro128+ PRNG, range generation
- **Parallel Execution**: Cross-platform threading (Windows/POSIX)
- **Memory Management**: Cache-aligned allocations (64-byte default)
- **Optimization Tools**: Black-box functions to prevent dead code elimination

## Constants

```
VEX_TEST_ENABLE_RDTSC = 1      # Use x86 RDTSC
VEX_TEST_ENABLE_AFFINITY = 1   # CPU pinning support
VEX_TEST_MAX_SAMPLES = 100000  # Max benchmark samples
VEX_TEST_JSON_BUFSZ = 65536    # JSON output buffer
```

## Go Compatibility

This module mirrors Go's `testing` package:

| Go Feature | Vex Equivalent | Status |
|------------|----------------|--------|
| `t.Errorf` | `assert_*` | ✅ Full (40+ variants) |
| `t.Fatal` | (planned) | 🔄 Partial |
| `b.ReportAllocs` | `bytes_per_op` | ✅ Supported |
| `b.N` auto-calibration | `auto_calibrate` | ✅ Full |
| Sub-tests | (planned) | 🔄 Partial |
| `t.Parallel` | `run_tests_parallel` | ✅ Full |
| Table-driven tests | Manual | ✅ Pattern works |
| Benchmarks | `bench_run` | ✅ Full |

## Performance Notes

- Benchmarks warm up for 20ms (configurable) before measurement
- Auto-calibration typically runs 2-3 iterations
- Per-operation timing accurate to nanoseconds (x86 RDTSC) or microseconds (fallback)
- Memory throughput (MB/s) computed from `bytes_per_op`
- Results include min/median/mean/max and p90/p95/p99 percentiles

## Next Steps

Future enhancements:

1. Go-style sub-tests with hierarchical reporting
2. `t.Fatal` and panic recovery
3. Snapshot testing support
4. Coverage reporting integration
5. Fuzzing support with libFuzzer

## See Also

- `vex-runtime/c/vex_testing.c` - C implementation
- `docs/REFERENCE.md` - Language reference
- `examples/` - Feature demonstrations
