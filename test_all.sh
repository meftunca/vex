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
    
    # Skip borrow checker tests (they should fail intentionally)
    if [[ "$file" == *"00_borrow_checker"* ]]; then
        echo "⏭️  Skipping $name (borrow checker test)"
        continue
    fi
    
    # Skip interface tests (interface keyword deprecated)
    if [[ "$file" == *"interfaces.vx"* ]]; then
        echo "⏭️  Skipping $name (interface deprecated, use trait)"
        continue
    fi
    
    # Circular dependency tests should FAIL with error (expected behavior)
    if [[ "$file" == *"circular_dependency.vx"* ]] || [[ "$file" == *"circular_self.vx"* ]]; then
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
