#!/bin/bash
# Test runner for Vex examples

echo "🧪 Testing Vex Examples..."
echo "=========================="

SUCCESS=0
FAIL=0
SKIP=0

cd "$(dirname "$0")"

for file in examples/*.vx; do
    name=$(basename "$file" .vx)
    echo -n "Testing $name... "
    
    # Compile
    cargo run --quiet --release -- compile "$file" 2>&1 | grep -q "Compilation successful"
    
    if [ $? -eq 0 ]; then
        echo "✅ PASS"
        ((SUCCESS++))
    else
        echo "❌ FAIL"
        ((FAIL++))
    fi
done

echo ""
echo "=========================="
echo "📊 Results:"
echo "   ✅ Success: $SUCCESS"
echo "   ❌ Failed:  $FAIL"
echo "   Total:     $((SUCCESS + FAIL))"
echo "   Success Rate: $(echo "scale=1; $SUCCESS * 100 / ($SUCCESS + $FAIL)" | bc)%"
