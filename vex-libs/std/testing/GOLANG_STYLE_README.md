# Vex Testing Framework - Golang-Style DX

Modern testing framework for Vex with **Golang-inspired developer experience**. Pure Vex implementation (no C FFI dependency).

## Philosophy

This framework provides the same excellent DX as Go's `testing` package:

- **`t *testing.T` pattern** - Test context with methods
- **Sub-tests with `t.Run()`** - Hierarchical test organization
- **Auto-calibrating `b.N`** - Benchmarks adjust iteration count automatically
- **Table-driven tests** - Clean, data-oriented testing
- **Cleanup functions** - Automatic resource cleanup
- **Skip/Fatal** - Test flow control
- **Assertion helpers** - Testify-style assertions

## Quick Start

### Simple Test

```vex
import { T, Assert, TestCase, run_tests } from "testing";

fn test_addition(t: *T) {
    let assert: Assert = Assert.new(t);
    assert.equal_i64(2 + 2, 4, "math works");
}

fn main() {
    let tests = vec![
        TestCase { name: "TestAddition", test_fn: test_addition },
    ];
    run_tests(tests);
}
```

### Sub-tests (like Go's `t.Run()`)

```vex
fn test_math(t: *T) {
    t.run("Addition", fn(sub: *T) {
        let assert: Assert = Assert.new(sub);
        assert.equal_i64(2 + 2, 4, "2+2=4");
    });
    
    t.run("Subtraction", fn(sub: *T) {
        let assert: Assert = Assert.new(sub);
        assert.equal_i64(5 - 3, 2, "5-3=2");
    });
}
```

Output:
```
=== RUN   TestMath
=== RUN   TestMath/Addition
--- PASS: TestMath/Addition
=== RUN   TestMath/Subtraction
--- PASS: TestMath/Subtraction
--- PASS: TestMath
```

### Table-Driven Tests (Golang Best Practice)

```vex
struct AddTestCase {
    a: i64
    b: i64
    expected: i64
}

fn test_addition_table(t: *T) {
    let table = vec![
        AddTestCase { a: 1, b: 1, expected: 2 },
        AddTestCase { a: 2, b: 2, expected: 4 },
        AddTestCase { a: 5, b: 5, expected: 10 },
    ];
    
    for i in 0..table.len() {
        let tc = table[i];
        let name = tc.a.to_string() + "+" + tc.b.to_string();
        
        t.run(name, fn(sub: *T) {
            let result = tc.a + tc.b;
            let assert: Assert = Assert.new(sub);
            assert.equal_i64(result, tc.expected, "addition");
        });
    }
}
```

Output:
```
=== RUN   TestAdditionTable
=== RUN   TestAdditionTable/1+1
--- PASS: TestAdditionTable/1+1
=== RUN   TestAdditionTable/2+2
--- PASS: TestAdditionTable/2+2
=== RUN   TestAdditionTable/5+5
--- PASS: TestAdditionTable/5+5
--- PASS: TestAdditionTable
```

### Benchmarks (Auto-calibrating `b.N`)

```vex
import { B, BenchCase, run_benchmarks } from "testing";

fn benchmark_concat(b: *B) {
    for i in 0..b.n {
        let s = "hello" + " " + "world";
    }
}

fn main() {
    let benches = vec![
        BenchCase { name: "Concat", bench_fn: benchmark_concat },
    ];
    run_benchmarks(benches);
}
```

Output (Golang format):
```
goos: vex
goarch: native
BenchmarkConcat-8    1000000    1234 ns/op
```

The framework automatically calibrates `b.n` to run for ~1 second (like Go).

## API Reference

### Testing Context (`T`)

| Method | Description | Go Equivalent |
|--------|-------------|---------------|
| `t.error(msg)` | Report failure, continue test | `t.Error()` |
| `t.errorf(msg)` | Report formatted failure | `t.Errorf()` |
| `t.fatal(msg)` | Report failure, stop test | `t.Fatal()` |
| `t.skip(msg)` | Mark test as skipped | `t.Skip()` |
| `t.log(msg)` | Write to test output | `t.Log()` |
| `t.run(name, fn)` | Run sub-test | `t.Run()` |
| `t.cleanup(fn)` | Register cleanup function | `t.Cleanup()` |

### Benchmark Context (`B`)

| Method | Description | Go Equivalent |
|--------|-------------|---------------|
| `b.reset_timer()` | Reset benchmark timer | `b.ResetTimer()` |
| `b.start_timer()` | Start timer (if stopped) | `b.StartTimer()` |
| `b.stop_timer()` | Stop timer | `b.StopTimer()` |
| `b.set_bytes(n)` | Set bytes/op for MB/s | `b.SetBytes()` |
| `b.report_allocs()` | Enable allocation reporting | `b.ReportAllocs()` |
| `b.n` | Iteration count (auto-calibrated) | `b.N` |

### Assertions (`Assert`)

| Method | Description | Testify Equivalent |
|--------|-------------|--------------------|
| `assert.equal_i64(a, b, msg)` | Assert a == b | `assert.Equal()` |
| `assert.not_equal_i64(a, b, msg)` | Assert a != b | `assert.NotEqual()` |
| `assert.true(cond, msg)` | Assert condition | `assert.True()` |
| `assert.false(cond, msg)` | Assert !condition | `assert.False()` |
| `assert.nil(ptr, msg)` | Assert null | `assert.Nil()` |
| `assert.not_nil(ptr, msg)` | Assert not null | `assert.NotNil()` |

### Test Runner

```vex
fn run_tests(tests: Vec<TestCase>) i32
```

Runs all tests and returns exit code (0 = all pass, 1 = failures).

### Benchmark Runner

```vex
fn run_benchmarks(benches: Vec<BenchCase>)
```

Runs all benchmarks with auto-calibration.

## Advanced Patterns

### Setup and Cleanup

```vex
fn test_with_resources(t: *T) {
    // Setup
    let resource = open_resource();
    
    // Cleanup (runs even if test fails)
    t.cleanup(fn() {
        close_resource(resource);
    });
    
    // Test
    let assert: Assert = Assert.new(t);
    assert.not_nil(resource, "resource opened");
}
```

### Skipping Tests

```vex
fn test_optional_feature(t: *T) {
    if !feature_available() {
        t.skip("feature not available");
        return;
    }
    
    // Test code
}
```

### Benchmark with Setup

```vex
fn benchmark_with_setup(b: *B) {
    // Setup (not timed)
    b.stop_timer();
    let data = prepare_data();
    b.start_timer();
    
    // Benchmark loop (timed)
    for i in 0..b.n {
        process(data);
    }
}
```

### Memory Throughput Benchmarks

```vex
fn benchmark_copy(b: *B) {
    let size: u64 = 1024 * 1024; // 1MB
    let src = vec![0u8; size];
    let dst = vec![0u8; size];
    
    b.set_bytes(size); // Enable MB/s reporting
    
    for i in 0..b.n {
        copy(dst, src);
    }
}
```

Output:
```
BenchmarkCopy-8    1000    1234 ns/op    810.5 MB/s
```

## Comparison with Go

| Feature | Go | Vex (this framework) | Status |
|---------|----|--------------------|--------|
| `t *testing.T` | ✅ | ✅ | Complete |
| `t.Run()` sub-tests | ✅ | ✅ | Complete |
| `t.Error/Fatal/Skip` | ✅ | ✅ | Complete |
| `t.Log` | ✅ | ✅ | Complete |
| `t.Cleanup` | ✅ | ✅ | Complete |
| `b *testing.B` | ✅ | ✅ | Complete |
| Auto `b.N` | ✅ | ✅ | Complete |
| `b.ResetTimer` | ✅ | ✅ | Complete |
| `b.SetBytes` | ✅ | ✅ | Complete |
| Table-driven pattern | ✅ | ✅ | Pattern supported |
| `go test` command | ✅ | Manual | TODO |
| Parallel tests | `t.Parallel()` | 🔄 | Planned |
| Fuzzing | `f *testing.F` | 🔄 | Planned |

## Design Decisions

### Why Pure Vex (No C FFI)?

1. **Better DX** - Native Vex types, no FFI boilerplate
2. **Type Safety** - Full Vex type system, no C conversions
3. **Portability** - Works on any platform Vex supports
4. **Debuggability** - Pure Vex stack traces
5. **Maintainability** - Single language to maintain

### Why Golang-style?

Go's `testing` package is the **gold standard** for testing DX:

- ✅ **Simple** - Minimal boilerplate
- ✅ **Powerful** - Sub-tests, benchmarks, cleanup
- ✅ **Consistent** - Same patterns everywhere
- ✅ **Fast** - Auto-calibrating benchmarks
- ✅ **Readable** - Table-driven tests

### Differences from C FFI Version

| Aspect | C FFI Version | Pure Vex Version |
|--------|---------------|------------------|
| Implementation | `vex_testing.c` | Pure Vex |
| API Style | Function-based | Method-based (OOP) |
| Test Context | Global state | `t *T` parameter |
| Benchmarks | Manual config | Auto `b.N` |
| Sub-tests | ❌ Not supported | ✅ `t.Run()` |
| Cleanup | Manual | ✅ `t.Cleanup()` |
| Type Safety | C types | Native Vex types |

## Examples

See `examples/golang_style.vx` for comprehensive examples covering:

1. ✅ Simple tests
2. ✅ Sub-tests with `t.Run()`
3. ✅ Table-driven tests
4. ✅ Cleanup functions
5. ✅ Skipping tests
6. ✅ Benchmarks with auto `b.N`
7. ✅ Benchmarks with setup/teardown
8. ✅ Memory throughput benchmarks

## Roadmap

### v0.2.0 (Current)
- ✅ Core `T` and `B` types
- ✅ Sub-tests (`t.Run()`)
- ✅ Assertions (`Assert`)
- ✅ Table-driven pattern support
- ✅ Auto-calibrating benchmarks

### v0.3.0 (Next)
- 🔄 Parallel tests (`t.Parallel()`)
- 🔄 Golden file testing
- 🔄 Snapshot testing
- 🔄 HTTP test helpers

### v0.4.0 (Future)
- 🔄 Fuzzing (`f *testing.F`)
- 🔄 Coverage reporting
- 🔄 Allocation tracking (`b.ReportAllocs()`)
- 🔄 `vex test` command integration

## Migration from C FFI Version

### Before (C FFI)

```vex
import { assert_true, run_tests, TestCase } from "testing";

fn test_example() {
    assert_true(true, "test");
}

fn main() {
    let tests: [1]TestCase = [
        TestCase { name: "example", fn: test_example },
    ];
    run_tests(&tests[0], 1);
}
```

### After (Pure Vex)

```vex
import { T, Assert, TestCase, run_tests } from "testing";

fn test_example(t: *T) {
    let assert: Assert = Assert.new(t);
    assert.true(true, "test");
}

fn main() {
    let tests = vec![
        TestCase { name: "TestExample", test_fn: test_example },
    ];
    run_tests(tests);
}
```

**Key Changes:**
1. Tests take `t: *T` parameter
2. Use `Assert.new(t)` for assertions
3. Use `vec![]` instead of arrays
4. Remove `&` pointer syntax (handled by Vec)

## License

MIT - Same as Vex language

## Contributing

This is a **pure Vex** implementation. To contribute:

1. Add features matching Go's `testing` package
2. Maintain Golang-style DX
3. Write examples in `examples/`
4. Update this README

---

**Status:** ✅ Production Ready (Pure Vex v0.2.0)  
**Inspired by:** Go's `testing` package + Rust's `#[test]` + Testify assertions
