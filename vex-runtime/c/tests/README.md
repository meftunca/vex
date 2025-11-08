# Vex Runtime C Library - Tests

This directory contains all tests for the Vex runtime C library, organized by category.

## Directory Structure

```
tests/
├── unit/              # Unit tests for individual components
├── integration/       # Integration tests for component interaction
└── benchmarks/        # Performance benchmarks
```

## Unit Tests

Unit tests verify individual components in isolation:

- `test_builtins.c` - Built-in functions and types
- `test_intrinsics.c` - SIMD and compiler intrinsics
- `test_cpu_simd.c` - CPU-specific SIMD operations
- `test_utf8.c` - UTF-8 string handling
- `test_panic.c` - Error handling and panic
- `test_file_time.c` - File time operations
- `test_map_c.c` - Basic map operations
- `test_empty_key.c` - Empty key edge cases
- `test_wrap.c` - Value wrapping
- `test.c` - General runtime tests
- `test_vex_print_direct.c` - Direct printing functions

## Integration Tests

Integration tests verify component interactions:

- `test_collision.c` - Hash collision handling
- `test_collision_simple.c` - Simple collision test
- `test_swiss_debug.c` - SwissTable debugging
- `test_swisstable.c` - SwissTable full integration
- `test_channel_simple.c` - Channel communication
- `INTEGRATION_DEMO.c` - Full system integration demo

## Running Tests

### All Tests
```bash
make test
```

### Unit Tests Only
```bash
make test-unit
```

### Integration Tests Only
```bash
make test-integration
```

### Component-Specific Tests
```bash
# Test vex_time
make test-vextime

# Test SwissTable
make test-swisstable

# Test async runtime
make test-async
```

## Benchmarks

Performance benchmarks are located in component directories:

```bash
# SwissTable benchmarks
make bench-swisstable

# vex_time benchmarks
make bench-vextime
```

## Adding New Tests

### Unit Test
1. Create `tests/unit/test_yourfeature.c`
2. Include necessary headers: `#include "vex.h"`
3. Write test functions
4. Build with: `make test-unit`

### Integration Test
1. Create `tests/integration/test_yourintegration.c`
2. Link required components
3. Write integration scenarios
4. Build with: `make test-integration`

## Test Guidelines

1. **Keep tests focused** - One test per concept
2. **Use descriptive names** - `test_feature_scenario.c`
3. **Add error messages** - Print failures clearly
4. **Return proper exit codes** - 0 for success, non-zero for failure
5. **Clean up resources** - Free memory, close files

## CI/CD Integration

Tests are designed to run in CI/CD pipelines:

```bash
# Quick smoke test
make test-unit

# Full test suite
make test

# Performance regression check
make bench-swisstable
```

## Performance Testing

For performance-sensitive components, use:

1. **Micro-benchmarks** - Single operation timing
2. **Throughput tests** - Operations per second
3. **Latency tests** - Time per operation
4. **Memory tests** - Allocation patterns

Example:
```bash
cd swisstable
./bench_ultimate    # Full benchmark suite
```

## Debugging Tests

### Address Sanitizer
```bash
clang -fsanitize=address -g test.c -o test
./test
```

### Valgrind
```bash
valgrind --leak-check=full ./test
```

### GDB
```bash
gdb ./test
(gdb) run
(gdb) bt  # backtrace on crash
```

## Test Coverage

To generate coverage reports:

```bash
# Compile with coverage
clang -fprofile-instr-generate -fcoverage-mapping test.c -o test

# Run test
./test

# Generate report
llvm-profdata merge -sparse default.profraw -o default.profdata
llvm-cov show ./test -instr-profile=default.profdata
```

## Continuous Integration

Tests run automatically on:
- Every commit to main
- Pull requests
- Release branches

Expected pass rate: **100%**

## Contact

For test-related questions or issues, see:
- Main README: `../README.md`
- Component docs: `../vex_time/`, `../swisstable/`, etc.

