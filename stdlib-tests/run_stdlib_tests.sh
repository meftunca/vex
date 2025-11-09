#!/bin/bash
# Stdlib Test Runner
# Runs all stdlib module tests

set -e

VEX_BIN="$HOME/.cargo/target/debug/vex"
TEST_DIR="stdlib-tests"
PASSED=0
FAILED=0
PENDING=0

# Make sure we're in the project root
cd "$(dirname "$0")/.."

echo "╔════════════════════════════════════════╗"
echo "║  Vex Standard Library Test Suite      ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Helper function to run a test
run_test() {
    local test_file=$1
    local test_name=$(basename "$test_file" .vx)
    
    echo -n "Testing $test_name... "
    
    # Run the test and capture output
    output=$($VEX_BIN run "$test_file" 2>&1)
    exit_code=$?
    
    # Check for explicit PENDING markers
    if echo "$output" | grep -q "⚠️.*not yet integrated"; then
        echo "⏳ PENDING"
        PENDING=$((PENDING + 1))
        return
    fi
    
    # Check for parse errors or compilation failures
    if echo "$output" | grep -Eq "(error\[E[0-9]+\]|Parse failed|Failed to parse module)"; then
        echo "❌ FAIL (parse/compile error)"
        FAILED=$((FAILED + 1))
        return
    fi
    
    # Check exit code
    if [ $exit_code -eq 0 ]; then
        echo "✅ PASS"
        PASSED=$((PASSED + 1))
    else
        echo "❌ FAIL (exit code: $exit_code)"
        FAILED=$((FAILED + 1))
    fi
}

# Run tests
echo "=== Running Stdlib Tests ==="
echo ""

run_test "$TEST_DIR/test_io.vx"
run_test "$TEST_DIR/test_math.vx"
run_test "$TEST_DIR/test_fs.vx"
run_test "$TEST_DIR/test_core.vx"
run_test "$TEST_DIR/test_testing.vx"
run_test "$TEST_DIR/test_collections.vx"
run_test "$TEST_DIR/test_string.vx"
run_test "$TEST_DIR/test_time.vx"

echo ""
echo "╔════════════════════════════════════════╗"
echo "║  Test Results                          ║"
echo "╚════════════════════════════════════════╝"
echo "✅ Passed:  $PASSED"
echo "❌ Failed:  $FAILED"
echo "⏳ Pending: $PENDING"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All active tests passing!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
