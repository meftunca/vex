# Vex Testing Framework - Examples

This directory contains comprehensive examples demonstrating all features of `vex_testing.c`.

---

## 📁 Files

| File | Description |
|------|-------------|
| `example_tests.c` | Complete demo: basic tests, subtests, fixtures, benchmarks, parallel tests, property-based testing |
| `example_fuzz.c` | Fuzzing example (libFuzzer/AFL++) |
| `COVERAGE_GUIDE.md` | Step-by-step guide for code coverage (llvm-cov/gcov) |

---

## 🚀 Quick Start

### 1. Basic Tests

```bash
# Compile
cc -O3 -std=c17 -I.. example_tests.c ../vex_testing.c -o example_tests -pthread

# Run (sequential)
./example_tests

# Run (parallel, 4 threads)
VEX_PARALLEL=4 ./example_tests

# Output formats
VEX_REPORTER=tap ./example_tests    # TAP v13
VEX_REPORTER=junit ./example_tests  # JUnit XML
```

### 2. Code Coverage

```bash
# LLVM coverage
clang -O0 -g -fprofile-instr-generate -fcoverage-mapping \
  -I.. example_tests.c ../vex_testing.c -o example_tests_cov -pthread

LLVM_PROFILE_FILE="example.profraw" ./example_tests_cov
llvm-profdata merge -sparse example.profraw -o example.profdata
llvm-cov show ./example_tests_cov -instr-profile=example.profdata -format=html -output-dir=coverage

# Open report
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
```

See [COVERAGE_GUIDE.md](COVERAGE_GUIDE.md) for detailed instructions.

### 3. Fuzzing

```bash
# libFuzzer (requires Clang)
clang -O2 -g -fsanitize=fuzzer,address -DVEX_FUZZ_TARGET \
  example_fuzz.c -o example_fuzz

# Run fuzzer for 30 seconds
./example_fuzz -max_total_time=30

# Use corpus directory
mkdir corpus
echo "12345" > corpus/seed1.txt
echo "hello" > corpus/seed2.txt
./example_fuzz corpus/
```

---

## 📚 Feature Examples

### Basic Tests

```c
VEX_TEST(test_basic) {
  VEX_ASSERT(1 + 1 == 2);
  VEX_TLOG("This is a log message");
}

VEX_TEST(test_skip) {
  VEX_SKIP("Not implemented yet");
}
```

### Subtests

```c
VEX_TEST(test_with_subtests) {
  VEX_SUBTEST("addition", {
    VEX_ASSERT(2 + 3 == 5);
  });
  
  VEX_SUBTEST("multiplication", {
    VEX_ASSERT(2 * 3 == 6);
  });
}
```

### Fixtures

```c
static int global_resource = 0;

static void setup_all(void) {
  global_resource = 100;
}

static void teardown_all(void) {
  global_resource = 0;
}

int main(void) {
  vex_fixture fx = vex_fixture_full(setup_all, teardown_all, NULL, NULL);
  return vex_run_tests_with("suite", tests, n_tests, &fx);
}
```

### Benchmarks

```c
static void bench_my_function(void *ctx) {
  vex_bench_start_timer();
  
  // Code to benchmark
  my_function();
  
  vex_bench_stop_timer();
  vex_bench_set_bytes(1024);  // Optional: for throughput
}

int main(void) {
  vex_bench_cfg cfg = {
    .name = "my_function",
    .auto_calibrate = true,
    .pin_cpu = 0,
    .repeats = 5,
  };
  vex_run_benchmark(bench_my_function, NULL, &cfg);
}
```

### Parallel Tests

```c
int main(void) {
  // Run with 4 threads (or set VEX_PARALLEL=4 env var)
  return vex_run_tests_parallel("suite", tests, n_tests, NULL, 4);
}
```

### Property-Based Testing

```c
VEX_PROPERTY(test_reverse_involution, 100, {
  // Generate random array
  vex_vec_t vec = vex_gen_vec_i64(&prop_ctx, 0, 20, -1000, 1000);
  
  // Test property: reverse(reverse(x)) == x
  reverse(vec.data, vec.len);
  reverse(vec.data, vec.len);
  
  VEX_PROP_ASSERT(&prop_ctx, is_equal(vec, original), "Property failed");
  
  vex_vec_free(&vec);
})
```

### Fuzzing

```c
#ifdef VEX_FUZZ_TARGET
int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
  // Test your code with random input
  my_parser(data, size);
  return 0;
}
#endif
```

---

## 🎯 Best Practices

### DO ✅
- Use `VEX_ASSERT` for checks (not `assert()`)
- Add `VEX_TLOG` for debugging
- Use fixtures for shared setup/teardown
- Pin CPU for benchmarks (`pin_cpu = 0`)
- Enable auto-calibration (`auto_calibrate = true`)
- Run parallel tests for large suites
- Aim for 80%+ line coverage

### DON'T ❌
- Use `printf` in tests (use `VEX_TLOG`)
- Ignore benchmark warmup (set `warmup_ns`)
- Run benchmarks with `-O0` (use `-O3`)
- Forget to free resources in property tests
- Skip error handling in fuzz targets

---

## 📊 Performance Tips

### Benchmarking
1. **Pin to CPU**: `pin_cpu = 0` (reduce scheduling noise)
2. **Warmup**: `warmup_ns = 50000000` (50 ms)
3. **Repeats**: `repeats = 5` (median of 5 runs)
4. **Optimize**: Use `-O3 -march=native`
5. **Profile**: Use `perf stat ./bench` for CPU counters

### Parallel Testing
1. **CPU-bound tests**: Use `n_threads = CPU_COUNT`
2. **IO-bound tests**: Use `n_threads = 2 * CPU_COUNT`
3. **Avoid**: Shared mutable state (use fixtures)

### Coverage
1. **Compile with `-O0`** (accurate line coverage)
2. **Exclude test framework**: `-ignore-filename-regex='vex_testing.c'`
3. **Target 80%+ line coverage** (industry standard)

---

## 🔧 Troubleshooting

### Issue: Tests hang
**Solution**: Check for deadlocks in fixtures or parallel tests. Use `VEX_TLOG` to trace execution.

### Issue: Benchmarks show 0 ops/s
**Solution**: Ensure `vex_bench_start_timer()` and `vex_bench_stop_timer()` are called correctly.

### Issue: Coverage shows 0%
**Solution**: Use `-O0` (no optimization) and ensure tests actually run.

### Issue: Fuzzer crashes immediately
**Solution**: Add bounds checking for `size` parameter:
```c
if (size < MIN_SIZE || size > MAX_SIZE) return 0;
```

---

## 📚 Resources

- **Main Documentation**: `../vex_testing.c` (header comments)
- **Coverage Guide**: [COVERAGE_GUIDE.md](COVERAGE_GUIDE.md)
- **Vex Stdlib**: `../../vex-libs/std/testing/src/lib.vx`
- **libFuzzer**: https://llvm.org/docs/LibFuzzer.html
- **AFL++**: https://aflplus.plus/

---

## ✅ Checklist

- [ ] Run basic tests: `./example_tests`
- [ ] Run parallel tests: `VEX_PARALLEL=4 ./example_tests`
- [ ] Generate coverage report (see COVERAGE_GUIDE.md)
- [ ] Run fuzzer: `./example_fuzz -max_total_time=30`
- [ ] Review benchmark results (ops/s, MB/s)
- [ ] Check TAP output: `VEX_REPORTER=tap ./example_tests`
- [ ] Integrate into CI/CD pipeline

---

**Happy Testing!** 🚀

