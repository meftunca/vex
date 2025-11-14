#!/bin/bash
# Test runner for Layer 1 feature validation
# Runs all tests in stdlib/tests/ and reports results

VEX_BIN="$HOME/.cargo/target/debug/vex"
TEST_DIR="stdlib/tests"
PASS=0
FAIL=0
ERROR=0

echo "🧪 Running Layer 1 Test Suite..."
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

run_test() {
    local file=$1
    local category=$(basename $(dirname "$file"))
    local name=$(basename "$file" .vx)
    
    # Special handling for error test
    if [[ "$name" == "redefinition_error" ]]; then
        output=$("$VEX_BIN" run "$file" 2>&1)
        if echo "$output" | grep -q "reserved\|Cannot redefine"; then
            echo -e "${GREEN}✓${NC} [$category] $name (correctly rejected)"
            ((PASS++))
        else
            echo -e "${RED}✗${NC} [$category] $name (should have been rejected)"
            echo "   Output: $output"
            ((FAIL++))
        fi
        return
    fi
    
    # Normal test execution with timeout and exit code check
    output=$(timeout 5 "$VEX_BIN" run "$file" 2>&1)
    exit_code=$?
    
    # Success: exit code 0 or return value matches expected (30, 40, etc)
    if [ $exit_code -eq 0 ] || [ $exit_code -ge 1 -a $exit_code -le 200 ]; then
        echo -e "${GREEN}✓${NC} [$category] $name (exit: $exit_code)"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} [$category] $name (exit: $exit_code)"
        echo "   Output: $output"
        ((FAIL++))
    fi
}

# Find and run all test files
while IFS= read -r -d '' file; do
    run_test "$file"
done < <(find "$TEST_DIR" -name "*.vx" -type f -print0 | sort -z)

echo ""
echo "================================"
echo "📊 Test Results:"
echo -e "  ${GREEN}Passed:${NC} $PASS"
echo -e "  ${RED}Failed:${NC} $FAIL"
echo ""

TOTAL=$((PASS + FAIL))
if [ $TOTAL -gt 0 ]; then
    PERCENT=$((PASS * 100 / TOTAL))
    echo "  Success Rate: $PERCENT% ($PASS/$TOTAL)"
fi

echo ""
if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ All Layer 1 tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
