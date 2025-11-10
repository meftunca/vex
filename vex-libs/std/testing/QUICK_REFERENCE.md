# Quick Reference: std/testing Module

## Module Overview
- **Purpose:** Go-like testing framework with assertions, fixtures, benchmarking
- **C Foundation:** vex_testing.c (1724 lines, C17, single-file)
- **Exported Functions:** 41 C → 41 Vex API
- **Status:** ✅ Production Ready (v0.1.2)

## Basic Usage

### Import
```vex
import { assert_true, run_tests, TestCase } from "testing";
```

### Simple Test
```vex
fn test_example() {
    assert_true(2 + 2 == 4, "math works");
}

fn main() {
    let tests: [1]TestCase = [
        TestCase { name: "example", fn: test_example },
    ];
    run_tests(&tests[0], 1);
}
```

## Common Assertions
| Function | Usage |
|----------|-------|
| `assert_true(bool, String)` | Test boolean true |
| `assert_false(bool, String)` | Test boolean false |
| `assert_eq_i64(i64, i64, String)` | Test int64 equality |
| `assert_neq_i64(i64, i64, String)` | Test int64 inequality |
| `assert_lt(i64, i64, String)` | Test less-than |
| `assert_null(*u8, String)` | Test null pointer |
| `log(String)` | Write test log |

## Test Harness
```vex
// Run with defaults
run_tests(&tests[0], count: u64)

// Run with fixtures
run_tests_with(suite_name, &tests[0], count, &fixture)

// Run in parallel
run_tests_parallel(suite_name, &tests[0], count, &fixture, threads)
```

## Fixtures
```vex
fn setup() { print("setup"); }
fn teardown() { print("teardown"); }

let fx: Fixture = fixture_all(setup, teardown);
// or
let fx: Fixture = fixture_full(setup_all, teardown_all, setup_each, teardown_each);

run_tests_with("suite", &tests[0], count, &fx);
```

## Benchmarking
```vex
fn bench_code(ctx: *BenchContext) {
    // Code that runs multiple times (auto-calibrated)
    let sum: i64 = 0;
    for i in 0i64..100i64 {
        sum = sum + i;
    }
}

let cfg: BenchConfig = BenchConfig {
    name: "compute", iters: 0, time_ns: 0,
    warmup_iters: 0, warmup_ns: 20000000,
    pin_cpu: 0, repeats: 5, report_json: false,
    auto_calibrate: true, bytes_per_op: 0
};

let ctx: BenchContext = BenchContext { name: "compute", bytes_per_op: 0 };
let result = bench_run(bench_code, &ctx, cfg);
bench_report_text(&result);
```

## Memory Utilities
```vex
// Allocate cache-aligned memory
let ptr: *u8 = aligned_alloc(64, 1024);

// Use pointer...

// Free aligned memory
aligned_free(ptr);
```

## Prevent Compiler Optimization (Benchmarking)
```vex
// Black box functions prevent dead code elimination
let x: i64 = black_box_i64(42);
let y: f64 = black_box_f64(3.14);
let z: *u8 = black_box_ptr(ptr);
```

## Property-Based Testing
```vex
// Generate random values with seed
let val1: i64 = gen_i64(seed, min, max);
let val2: u64 = gen_u64(seed, min, max);
let val3: f64 = gen_f64(seed, min, max);
let val4: bool = gen_bool(seed);
```

## CPU Control
```vex
// Pin thread to specific CPU
pin_to_cpu(0);  // CPU 0

// Request real-time scheduling
set_realtime_hint();

// Get timing data
let ns: u64 = monotonic_ns();
let cycles: u64 = read_cycles();
```

## Reporters
```vex
fn pick_reporter() ReporterKind // Get current (TAP/JUNIT/TEXT)
fn set_reporter(ReporterKind)   // Set reporter
```

## Struct Types

### TestCase
```vex
struct TestCase {
    name: String
    fn: TestFn
}
```

### BenchConfig
```vex
struct BenchConfig {
    name: String
    iters: u64
    time_ns: u64
    warmup_iters: u64
    warmup_ns: u64
    pin_cpu: i32
    repeats: i32
    report_json: bool
    auto_calibrate: bool
    bytes_per_op: u64
}
```

### BenchResult
```vex
struct BenchResult {
    name: String
    ns_per_op: f64
    cycles_per_op: f64
    mb_per_s: f64
    elapsed_ns: u64
    elapsed_cycles: u64
    iters_done: u64
    samples: i32
    min_ns: f64
    median_ns: f64
    mean_ns: f64
    max_ns: f64
    p90_ns: f64
    p95_ns: f64
    p99_ns: f64
}
```

## Vex Pattern: Struct Literal Workaround
**Issue:** Inline struct literals are dropped immediately
```vex
// ❌ Won't work
run_tests_with(suite, &tests[0], count, &Fixture { ... });

// ✅ Works
let fx: Fixture = Fixture { setup_all: setup, teardown_all: teardown };
run_tests_with(suite, &tests[0], count, &fx);
```

## Full Example: Complete Test Suite
```vex
import { assert_true, assert_eq_i64, run_tests_with, fixture_full, TestCase, Fixture } from "testing";

var test_state: i64 = 0;

fn setup_all() {
    print("Global setup");
}

fn teardown_all() {
    print("Global teardown");
}

fn setup_each() {
    test_state = 0;
}

fn teardown_each() {
    print("Test cleanup");
}

fn test_basic() {
    assert_true(true, "basic test");
}

fn test_arithmetic() {
    assert_eq_i64(2 + 2, 4i64, "2+2=4");
}

fn test_state() {
    test_state = 42;
    assert_eq_i64(test_state, 42i64, "state modified");
}

fn main() {
    let tests: [3]TestCase = [
        TestCase { name: "basic", fn: test_basic },
        TestCase { name: "arithmetic", fn: test_arithmetic },
        TestCase { name: "state", fn: test_state },
    ];
    
    let fx: Fixture = fixture_full(setup_all, teardown_all, setup_each, teardown_each);
    run_tests_with("my_suite", &tests[0], 3, &fx);
}
```

## API Completeness

| Category | Count | Examples |
|----------|-------|----------|
| Assertions | 20+ | assert_true, assert_eq_i64, assert_lt_u64 |
| Test harness | 3 | run_tests, run_tests_with, run_tests_parallel |
| Fixtures | 3 | fixture_all, fixture_each, fixture_full |
| Benchmarking | 5 | bench_run, bench_report_text, bench_report_json |
| Memory | 2 | aligned_alloc, aligned_free |
| Optimization | 4 | black_box_i64, black_box_u64, black_box_f64, black_box_ptr |
| Timing | 2 | monotonic_ns, read_cycles |
| CPU | 2 | pin_to_cpu, set_realtime_hint |
| Property | 4 | gen_i64, gen_u64, gen_f64, gen_bool |
| Reporting | 2 | pick_reporter, set_reporter |
| **Total** | **41+** | |

## Documentation Files
- **README.md** - Full API documentation (520 lines)
- **INTEGRATION_REPORT.md** - Integration details (350 lines)
- **STDLIB_MODULES_COMPLETE.md** - Combined time+testing report (450 lines)

## See Also
- Full docs: `vex-libs/std/testing/README.md`
- Integration: `vex-libs/std/testing/INTEGRATION_REPORT.md`
- Smoke test: `vex-libs/std/testing/tests/smoke.vx`
- Examples: `vex-libs/std/testing/examples/all_features.vx`
