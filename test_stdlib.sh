#!/bin/bash
# Comprehensive Stdlib Test Runner
# Tests all stdlib modules

set -e

VEX="$HOME/.cargo/target/debug/vex"
FAILED=0
PASSED=0

echo "🧪 Vex Standard Library Test Suite"
echo "===================================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run a test
run_test() {
    local module=$1
    local test_file=$2
    
    echo -n "Testing $module..."
    
    if $VEX run "$test_file" > /tmp/vex_test_output.txt 2>&1; then
        echo -e "${GREEN} ✓ PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED} ✗ FAIL${NC}"
        echo "  Error output:"
        cat /tmp/vex_test_output.txt | head -10
        ((FAILED++))
    fi
}

# Test modules
echo "📦 Testing Core Modules:"
echo ""

# FS module
if [ -f "vex-libs/std/fs/tests/basic_test.vx" ]; then
    run_test "fs" "vex-libs/std/fs/tests/basic_test.vx"
fi

# Math module
if [ -f "vex-libs/std/math/tests/basic_test.vx" ]; then
    run_test "math" "vex-libs/std/math/tests/basic_test.vx"
fi

# Env module
if [ -f "vex-libs/std/env/tests/basic_test.vx" ]; then
    run_test "env" "vex-libs/std/env/tests/basic_test.vx"
fi

# Process module
if [ -f "vex-libs/std/process/tests/basic_test.vx" ]; then
    run_test "process" "vex-libs/std/process/tests/basic_test.vx"
fi

echo ""
echo "📊 Integration Tests:"
echo ""

# Stdlib integration test
if [ -f "examples/stdlib_integration_comprehensive.vx" ]; then
    run_test "integration" "examples/stdlib_integration_comprehensive.vx"
fi

echo ""
echo "===================================="
echo "📋 Test Summary"
echo "===================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED${NC}"
else
    echo -e "Failed: 0"
fi
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
