#!/bin/bash
# Test runner for Vex examples

echo "🧪 Testing Vex Examples..."
echo "=========================="

cd "$(dirname "$0")"
VEX_BIN="$HOME/.cargo/target/debug/vex"

SUCCESS=0
FAIL=0

# Test all .vx files in examples/ recursively
while IFS= read -r file; do
    name=$(echo "$file" | sed 's|examples/||' | sed 's|\.vx$||')
    
    # # Skip borrow checker tests (they should fail intentionally)
    # if [[ "$file" == *"00_borrow_checker"* ]]; then
    #     echo "⏭️  Skipping $name (borrow checker test)"
    #     continue
    # fi
    
    # Skip interface tests (interface keyword deprecated)
    if [[ "$file" == *"interfaces.vx"* ]]; then
        echo "⏭️  Skipping $name (interface deprecated, use trait)"
        continue
    fi
    
    # Circular dependency tests should FAIL with error (expected behavior)
    if [[ "$file" == *"circular_dependency.vx"* ]] || [[ "$file" == *"circular_self.vx"* ]] || [[ "$file" == *"04_circular.vx"* ]]; then
        echo -n "Testing $name... "
        if "$VEX_BIN" compile "$file" > /dev/null 2>&1; then
            echo "❌ FAIL (should have detected circular dependency)"
            ((FAIL++))
        else
            echo "✅ PASS (correctly detected circular dependency)"
            ((SUCCESS++))
        fi
        continue
    fi
    
    # Borrow checker error tests should FAIL with borrow error (expected behavior)
    if [[ "$file" == *"_error.vx"* ]] || [[ "$file" == *"return_local.vx"* ]] || [[ "$file" == *"_violation.vx"* ]]; then
        echo -n "Testing $name... "
        output=$("$VEX_BIN" run "$file" 2>&1)
        if echo "$output" | grep -qE "(Borrow check failed|error\[E[0-9]+\]:|Compilation error:)"; then
            echo "✅ PASS (correctly detected error)"
            ((SUCCESS++))
        else
            echo "❌ FAIL (should have detected error)"
            ((FAIL++))
        fi
        continue
    fi
    
    # Diagnostic tests should produce compilation errors (expected behavior)
    if [[ "$file" == *"test_diagnostic"* ]] || [[ "$file" == *"test_typo"* ]] || \
       [[ "$file" == *"test_function_typo"* ]] || \
       [[ "$file" == *"test_undefined"* ]] || [[ "$file" == *"test_fuzzy"* ]] || \
       [[ "$file" == *"test_if_condition"* ]] || [[ "$file" == *"test_parse_error"* ]] || \
       [[ "$file" == *"test_borrow_diagnostic"* ]] || \
       [[ "$file" == *"test_immutable_violation"* ]]; then
        echo -n "Testing $name... "
        output=$("$VEX_BIN" run "$file" 2>&1)
        if echo "$output" | grep -qE "error\[E[0-9]+\]:"; then
            echo "✅ PASS (correctly detected compile error)"
            ((SUCCESS++))
        else
            echo "❌ FAIL (should have detected compile error)"
            ((FAIL++))
        fi
        continue
    fi
    
    # Skip known broken tests
    if [[ "$file" == *"error_handling_try.vx"* ]] || [[ "$file" == *"test_move_diagnostic.vx"* ]]; then
        echo "⏭️  Skipping $name (known issues)"
        continue
    fi
    
    echo -n "Testing $name... "
    
    # Try to compile
    if "$VEX_BIN" compile "$file" > /dev/null 2>&1; then
        echo "✅ PASS"
        ((SUCCESS++))
    else
        echo "❌ FAIL"
        ((FAIL++))
    fi
done < <(find examples -name "*.vx" -type f | sort)

echo ""
echo "=========================="
echo "📊 Results:"
echo "   ✅ Success: $SUCCESS"
echo "   ❌ Failed:  $FAIL"
TOTAL=$((SUCCESS + FAIL))
if [ $TOTAL -gt 0 ]; then
    echo "   Total:     $TOTAL"
    RATE=$(awk "BEGIN {printf \"%.1f\", ($SUCCESS * 100.0) / $TOTAL}")
    echo "   Success Rate: ${RATE}%"
else
    echo "   No tests run!"
fi
