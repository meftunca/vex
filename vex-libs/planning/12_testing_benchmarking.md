# Vex Stdlib Planning - 12: Testing and Benchmarking

**Priority:** 12
**Status:** Partial (testing exists, benchmarking missing)
**Dependencies:** builtin, io, time, reflect

## 📦 Packages in This Category

### 12.1 testing (extend existing)
**Status:** ✅ Exists (comprehensive extension needed)
**Description:** Unit testing framework

#### Current Implementation
- Basic testing functions exist

#### Required Extensions
```vex
// Testing types
struct T {
    common: common,
    is_parallel: bool,
    context: context.Context,
    deadline: time.Time,
    race_errors: []raceError,
    chatty: *chattyPrinter,
}

struct B {
    common: common,
    import_path: str,
    context: context.Context,
    n: int,
    previous_n: int,
    previous_duration: time.Duration,
    benchmark_allocs: bool,
    benchmark_memory: bool,
    timer_on: bool,
    show_allocs: bool,
    result: BenchmarkResult,
}

struct F {
    common: common,
    chatty: *chattyPrinter,
}

struct common {
    output: []u8,
    err: *[]u8,
    aux: []u8,
    skipped: bool,
    failed: bool,
    chatty: bool,
    finished: bool,
    in_parallel: bool,
    race_errors: int,
    runner: str,
    ignore_race_errors: bool,
    test_name: str,
    status: *testStatus,
    indenter: *indenter,
    w: io.Writer,
    chatty_printer: *chattyPrinter,
    parent: *common,
    level: int,
    creator: []uintptr,
    start: time.Time,
    duration: time.Duration,
}

struct BenchmarkResult {
    n: int,
    t: time.Duration,
    bytes: int,
    mem_allocs: uint64,
    mem_bytes: uint64,
}

struct TestResult {
    test_name: str,
    elapsed: time.Duration,
    output: str,
    passed: bool,
    skipped: bool,
}

// Testing functions
fn run_tests(match_string: fn(pat: str, str: str): bool, tests: []InternalTest): bool
fn run_benchmarks(match_string: fn(pat: str, str: str): bool, benchmarks: []InternalBenchmark): bool
fn run_examples(match_string: fn(pat: str, str: str): bool, examples: []InternalExample): bool
fn run_fuzz_tests(match_string: fn(pat: str, str: str): bool, fuzz_tests: []InternalFuzzTarget): bool

// Test helpers
fn short(): bool
fn verbose(): bool
fn cover_mode(): str
fn benchmark_n(): int
fn allocs_per_run(runs: int, f: fn()): float64

// Coverage
fn register_cover(cover: Cover)
fn cover_mode(): str
fn cover_profile(): str
```

#### Required Types
```vex
struct InternalTest {
    name: str,
    f: fn(*T),
}

struct InternalBenchmark {
    name: str,
    f: fn(*B),
}

struct InternalExample {
    name: str,
    f: fn(),
    output: str,
}

struct InternalFuzzTarget {
    name: str,
    f: fn(*T, []u8),
}

struct Cover {
    mode: str,
    counters: Map<str, []uint32>,
    blocks: Map<str, []CoverBlock>,
}

struct CoverBlock {
    line0: uint32,
    col0: uint16,
    line1: uint32,
    col1: uint16,
    stmts: uint16,
}
```

#### Dependencies
- builtin
- io
- time
- context
- reflect

### 12.2 testing/quick
**Status:** ❌ Missing (property-based testing)
**Description:** QuickCheck-style property testing

#### Required Types
```vex
struct Generator {
    // interface for generating values
}

struct Config {
    max_count: int,
    max_count_scale: float64,
    rand: *rand.Rand,
    values: fn(values: reflect.Value, rand: *rand.Rand),
}
```

#### Required Functions
```vex
fn check(f: any, config: *Config): Result<(), Error>
fn check_equal(f: any, g: any, config: *Config): Result<(), Error>
fn check_equal_fn(f: any, g: any, config: *Config): Result<(), Error>
```

#### Dependencies
- testing
- rand
- reflect

### 12.3 testing/fuzz
**Status:** ❌ Missing (fuzz testing)
**Description:** Fuzz testing framework

#### Required Types
```vex
struct F {
    common: common,
    fuzz_target: *InternalFuzzTarget,
    data: []u8,
    cover: Cover,
    corpus: []CorpusEntry,
    types: []reflect.Type,
    vals: []reflect.Value,
    result: FuzzResult,
}

struct CorpusEntry {
    parent: str,
    path: str,
    data: []u8,
    values: []any,
    generation: int,
    is_seed: bool,
}

struct FuzzResult {
    n: int,
    t: time.Duration,
    count: int,
    crashers: int,
}
```

#### Required Functions
```vex
fn fuzz(target: any): Result<(), Error>
fn add_to_corpus(data: []u8)
fn read_corpus(dir: str, types: []reflect.Type): Result<[]CorpusEntry, Error>
```

#### Dependencies
- testing
- reflect
- io

### 12.4 testing/iotest
**Status:** ❌ Missing (I/O testing utilities)
**Description:** I/O testing helpers

#### Required Types
```vex
struct OneByteReader {
    r: io.Reader,
}

struct HalfReader {
    r: io.Reader,
}

struct DataErrReader {
    r: io.Reader,
    unwritten: []u8,
    err: Error,
}

struct TimeoutReader {
    r: io.Reader,
    timeout: time.Duration,
}
```

#### Required Functions
```vex
fn new_read_logger(w: io.Writer, r: io.Reader): io.Reader
fn new_write_logger(w: io.Writer, wr: io.Writer): io.Writer
fn one_byte_reader(r: io.Reader): io.Reader
fn half_reader(r: io.Reader): io.Reader
fn data_err_reader(r: io.Reader, unwritten: []u8, err: Error): io.Reader
fn timeout_reader(r: io.Reader, timeout: time.Duration): io.Reader
fn err_timeout(): Error
fn truncate_writer(w: io.Writer, n: i64): io.Writer
```

#### Dependencies
- testing
- io
- time

## 🎯 Implementation Priority

1. **testing extensions** - Complete testing framework
2. **testing/iotest** - I/O testing utilities
3. **testing/quick** - Property-based testing
4. **testing/fuzz** - Fuzz testing

## ⚠️ Language Feature Issues

- **Reflection:** Testing frameworks need runtime type inspection
- **Function Values:** Storing test functions in structs
- **Global State:** Test runner state management

## 📋 Missing Critical Dependencies

- **Coverage Instrumentation:** Compiler support for code coverage
- **Fuzzing Engine:** Sophisticated fuzzing algorithms
- **Property Testing:** Generic value generation

## 🚀 Next Steps

1. Extend testing package with full T/B/F types
2. Implement benchmark timing and reporting
3. Add I/O testing utilities
4. Create property-based testing
5. Implement fuzz testing framework