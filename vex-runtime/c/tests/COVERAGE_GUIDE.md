# Code Coverage Guide for Vex Testing

This guide explains how to measure code coverage for Vex C runtime tests using **LLVM coverage** (llvm-cov) or **GCC coverage** (gcov).

---

## 📊 Why Coverage Matters

Code coverage helps identify:
- **Untested code paths** (dead code, edge cases)
- **Critical sections** (error handling, boundary conditions)
- **Test quality** (do tests exercise the code fully?)

**Target**: Aim for **80%+ line coverage** and **70%+ branch coverage**.

---

## 🛠️ Method 1: LLVM Coverage (Recommended)

### Prerequisites
- **Clang 10+** (with llvm-cov)
- **Linux/macOS** (Windows support limited)

### Step 1: Compile with Instrumentation

```bash
cd vex-runtime/c/tests

# Compile test with coverage instrumentation
clang -O0 -g \
  -fprofile-instr-generate -fcoverage-mapping \
  -I../async_runtime/include -I.. \
  test_example.c ../vex_testing.c \
  -o test_example_cov \
  -pthread

# Note: Use -O0 for accurate coverage (no optimization)
```

### Step 2: Run Tests

```bash
# Set output file for coverage data
export LLVM_PROFILE_FILE="test_example.profraw"

# Run tests (this generates test_example.profraw)
./test_example_cov
```

### Step 3: Generate Coverage Report

```bash
# Convert .profraw to .profdata (indexed format)
llvm-profdata merge -sparse test_example.profraw -o test_example.profdata

# Generate HTML report
llvm-cov show ./test_example_cov \
  -instr-profile=test_example.profdata \
  -format=html \
  -output-dir=coverage_html \
  -Xdemangler=c++filt \
  -ignore-filename-regex='vex_testing.c'

# Open coverage report
open coverage_html/index.html  # macOS
# xdg-open coverage_html/index.html  # Linux
```

### Step 4: Generate Summary

```bash
# Console summary
llvm-cov report ./test_example_cov \
  -instr-profile=test_example.profdata \
  -ignore-filename-regex='vex_testing.c'

# Example output:
# Filename      Regions    Miss   Cover     Lines    Miss   Cover  Branches    Miss   Cover
# -----------------------------------------------------------------------------------------
# vex_string.c      123      12   90.24%      456      23   94.96%       89       8   91.01%
# -----------------------------------------------------------------------------------------
# TOTAL             123      12   90.24%      456      23   94.96%       89       8   91.01%
```

### Step 5: Export to CI/CD (Optional)

```bash
# Export to JSON (for CI/CD pipelines)
llvm-cov export ./test_example_cov \
  -instr-profile=test_example.profdata \
  -format=lcov \
  > coverage.lcov

# Upload to Codecov/Coveralls
# bash <(curl -s https://codecov.io/bash) -f coverage.lcov
```

---

## 🛠️ Method 2: GCC Coverage (gcov)

### Prerequisites
- **GCC 7+**
- **Linux/macOS**

### Step 1: Compile with Instrumentation

```bash
cd vex-runtime/c/tests

# Compile with gcov instrumentation
gcc -O0 -g --coverage \
  -I../async_runtime/include -I.. \
  test_example.c ../vex_testing.c \
  -o test_example_cov \
  -pthread

# Note: --coverage is shorthand for -fprofile-arcs -ftest-coverage
```

### Step 2: Run Tests

```bash
# Run tests (generates .gcda files)
./test_example_cov

# Verify .gcda files exist
ls -lh *.gcda
# test_example.gcda  vex_testing.gcda
```

### Step 3: Generate Coverage Report

```bash
# Generate .gcov files (per-source)
gcov test_example.c
gcov ../vex_string.c

# View coverage (text format)
cat vex_string.c.gcov | less

# Example output:
#         -:  123:static inline size_t vex_strlen(const char *s) {
#    100000:  124:  const char *p = s;
#    100000:  125:  while (*p) p++;
#    100000:  126:  return (size_t)(p - s);
#         -:  127:}
```

### Step 4: Generate HTML Report (with lcov)

```bash
# Install lcov (if not already)
# sudo apt install lcov  # Debian/Ubuntu
# brew install lcov      # macOS

# Capture coverage data
lcov --capture --directory . --output-file coverage.info

# Filter out system headers and test framework
lcov --remove coverage.info '/usr/*' '*/vex_testing.c' --output-file coverage_filtered.info

# Generate HTML report
genhtml coverage_filtered.info --output-directory coverage_html

# Open report
xdg-open coverage_html/index.html  # Linux
# open coverage_html/index.html     # macOS
```

---

## 🚀 Quick Start: All-in-One Script

### LLVM Coverage Script

```bash
#!/bin/bash
# coverage_llvm.sh - Generate LLVM coverage for Vex tests

TEST_NAME="${1:-test_string}"
TEST_SRC="../${TEST_NAME}.c"

echo "[1/5] Compiling with coverage..."
clang -O0 -g \
  -fprofile-instr-generate -fcoverage-mapping \
  -I../async_runtime/include -I.. \
  ${TEST_NAME}.c ../vex_testing.c ${TEST_SRC} \
  -o ${TEST_NAME}_cov \
  -pthread || exit 1

echo "[2/5] Running tests..."
LLVM_PROFILE_FILE="${TEST_NAME}.profraw" ./${TEST_NAME}_cov || exit 1

echo "[3/5] Processing coverage data..."
llvm-profdata merge -sparse ${TEST_NAME}.profraw -o ${TEST_NAME}.profdata || exit 1

echo "[4/5] Generating HTML report..."
llvm-cov show ./${TEST_NAME}_cov \
  -instr-profile=${TEST_NAME}.profdata \
  -format=html \
  -output-dir=coverage_${TEST_NAME} \
  -ignore-filename-regex='vex_testing.c' || exit 1

echo "[5/5] Generating summary..."
llvm-cov report ./${TEST_NAME}_cov \
  -instr-profile=${TEST_NAME}.profdata \
  -ignore-filename-regex='vex_testing.c'

echo ""
echo "✅ Coverage report: coverage_${TEST_NAME}/index.html"
```

**Usage**:
```bash
chmod +x coverage_llvm.sh
./coverage_llvm.sh test_string
```

### GCC Coverage Script

```bash
#!/bin/bash
# coverage_gcc.sh - Generate GCC coverage for Vex tests

TEST_NAME="${1:-test_string}"
TEST_SRC="../${TEST_NAME}.c"

echo "[1/5] Compiling with coverage..."
gcc -O0 -g --coverage \
  -I../async_runtime/include -I.. \
  ${TEST_NAME}.c ../vex_testing.c ${TEST_SRC} \
  -o ${TEST_NAME}_cov \
  -pthread || exit 1

echo "[2/5] Running tests..."
./${TEST_NAME}_cov || exit 1

echo "[3/5] Generating gcov files..."
gcov ${TEST_NAME}.c ${TEST_SRC} || exit 1

echo "[4/5] Generating HTML report..."
lcov --capture --directory . --output-file coverage.info
lcov --remove coverage.info '/usr/*' '*/vex_testing.c' --output-file coverage_filtered.info
genhtml coverage_filtered.info --output-directory coverage_${TEST_NAME} || exit 1

echo "[5/5] Generating summary..."
lcov --summary coverage_filtered.info

echo ""
echo "✅ Coverage report: coverage_${TEST_NAME}/index.html"
```

**Usage**:
```bash
chmod +x coverage_gcc.sh
./coverage_gcc.sh test_string
```

---

## 📈 Interpreting Results

### Line Coverage
**Formula**: `Covered Lines / Total Lines`

- **90-100%**: Excellent (production-ready)
- **80-90%**: Good (some edge cases missed)
- **70-80%**: Acceptable (needs improvement)
- **<70%**: Poor (critical gaps)

### Branch Coverage
**Formula**: `Taken Branches / Total Branches`

- **80-100%**: Excellent (all paths tested)
- **60-80%**: Good (most branches covered)
- **<60%**: Poor (missing error handling?)

### Example Report

```
Filename          Regions    Miss   Cover     Lines    Miss   Cover  Branches    Miss   Cover
-----------------------------------------------------------------------------------------------
vex_string.c          145      18   87.59%      678      45   93.36%      112      23   79.46%
vex_strconv.c         234      12   94.87%      892      34   96.19%      189       8   95.77%
vex_path.c            187      56   70.05%      543     102   81.21%      143      48   66.43%
-----------------------------------------------------------------------------------------------
TOTAL                 566      86   84.81%     2113     181   91.43%      444      79   82.21%
```

**Analysis**:
- ✅ `vex_strconv.c`: Excellent (95%+ line/branch)
- ⚠️ `vex_path.c`: Needs work (70% region, 66% branch) → Add tests for edge cases

---

## 🔧 Advanced: Coverage-Driven Development

### 1. Identify Gaps

```bash
# Find uncovered lines
llvm-cov show ./test_cov \
  -instr-profile=test.profdata \
  -show-line-counts-or-regions \
  -show-instantiation-summary \
  | grep '0|'  # Lines with 0 executions
```

### 2. Add Tests for Gaps

```c
// Example: Uncovered error path
VEX_TEST(test_error_handling) {
  // Previously untested: NULL input
  VEX_ASSERT(vex_strlen(NULL) == 0);
  
  // Previously untested: malloc failure
  vex_set_alloc_fail(true);
  char *s = vex_strdup("test");
  VEX_ASSERT(s == NULL);
}
```

### 3. Re-run Coverage

```bash
# Recompile and run
./coverage_llvm.sh test_string

# Verify improvement
llvm-cov report ./test_string_cov -instr-profile=test_string.profdata
# Lines: 93.36% → 96.82% ✅
```

---

## 🎯 Best Practices

### DO ✅
- **Run coverage regularly** (CI/CD pipeline)
- **Target 80%+ line coverage** (industry standard)
- **Focus on critical paths** (error handling, security)
- **Use `-O0`** (no optimization for accurate coverage)
- **Exclude test framework** (`vex_testing.c`)

### DON'T ❌
- **Chase 100% coverage** (diminishing returns)
- **Ignore branch coverage** (line coverage alone is insufficient)
- **Over-optimize untested code** (coverage first, then optimize)
- **Test trivial code** (getters/setters, one-liners)

---

## 🐛 Troubleshooting

### Issue: `.profraw` not generated
**Solution**: Check `LLVM_PROFILE_FILE` is set:
```bash
export LLVM_PROFILE_FILE="test.profraw"
./test_cov
ls -lh test.profraw  # Should exist
```

### Issue: `llvm-profdata: error: no profile`
**Solution**: Ensure tests actually ran (exit code 0):
```bash
./test_cov && echo "Tests passed"
```

### Issue: Coverage shows 0% for all files
**Solution**: Use `-O0` (no optimization):
```bash
clang -O0 -g -fprofile-instr-generate ...
```

### Issue: GCC: `.gcda` version mismatch
**Solution**: Clean old `.gcda` files:
```bash
rm -f *.gcda *.gcno
# Recompile and run
```

---

## 📚 Resources

- **LLVM Coverage**: https://clang.llvm.org/docs/SourceBasedCodeCoverage.html
- **GCC gcov**: https://gcc.gnu.org/onlinedocs/gcc/Gcov.html
- **lcov Manual**: https://github.com/linux-test-project/lcov
- **Codecov**: https://about.codecov.io/ (CI/CD integration)

---

## ✅ Checklist

- [ ] Install `clang` + `llvm-cov` (or `gcc` + `lcov`)
- [ ] Compile tests with `-fprofile-instr-generate -fcoverage-mapping` (LLVM) or `--coverage` (GCC)
- [ ] Run tests to generate `.profraw` (LLVM) or `.gcda` (GCC)
- [ ] Process coverage data (`llvm-profdata` / `lcov`)
- [ ] Generate HTML report (`llvm-cov show` / `genhtml`)
- [ ] Review uncovered lines and add tests
- [ ] Integrate into CI/CD pipeline (GitLab CI, GitHub Actions)

---

**Happy Testing!** 🚀

