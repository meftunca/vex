#!/bin/bash
# Test runner for vex-libs/std packages
# Tests all .test.vx files in vex-libs/std

set -e

VEX_BIN="${HOME}/.cargo/target/debug/vex"
STDLIB_DIR="vex-libs/std"

echo "🧪 Testing Vex Standard Library (Layer 2)"
echo "=========================================="
echo ""

if [ ! -f "$VEX_BIN" ]; then
    echo "❌ Error: vex binary not found at $VEX_BIN"
    echo "   Run 'cargo build' first"
    exit 1
fi

# Find all test files
TEST_FILES=$(find "$STDLIB_DIR" -name "*.test.vx" | sort)
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

for test_file in $TEST_FILES; do
    TOTAL=$((TOTAL + 1))
    package=$(echo "$test_file" | sed 's|vex-libs/std/||' | sed 's|/.*||')
    test_name=$(basename "$test_file" .test.vx)
    
    # Skip known broken tests
    if [[ "$test_file" == *"io_full"* ]] || [[ "$test_file" == *"io_module"* ]]; then
        echo -e "${YELLOW}⏭️  SKIP${NC} $package/$test_name (requires C dependencies)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi
    
    # Run test
    if timeout 5 "$VEX_BIN" run "$test_file" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS${NC} $package/$test_name"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}❌ FAIL${NC} $package/$test_name"
        FAILED=$((FAILED + 1))
        
        # Show error (first 10 lines)
        if [ -n "$VERBOSE" ]; then
            "$VEX_BIN" run "$test_file" 2>&1 | head -10
            echo ""
        fi
    fi
done

echo ""
echo "=========================================="
echo "📊 Test Results:"
echo "   Total:   $TOTAL"
echo -e "   ${GREEN}Passed:  $PASSED${NC}"
echo -e "   ${RED}Failed:  $FAILED${NC}"
echo -e "   ${YELLOW}Skipped: $SKIPPED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✨ All tests passed!${NC}"
    exit 0
else
    RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED * 100.0) / $TOTAL}")
    echo -e "${YELLOW}⚠️  Success rate: ${RATE}%${NC}"
    echo ""
    echo "Run with VERBOSE=1 to see error details:"
    echo "  VERBOSE=1 $0"
    exit 1
fi
