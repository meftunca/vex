#!/bin/bash
# Test runner for FS module - runs from project root to find vex-libs

# Get project root (3 levels up from fs/)
PROJECT_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$PROJECT_ROOT"

VEX="$HOME/.cargo/target/debug/vex"
FS_DIR="vex-libs/std/fs"

echo "🚀 Testing FS module from project root: $PROJECT_ROOT"
echo ""

# Clean builds
echo "🧹 Cleaning previous builds..."
rm -rf "$FS_DIR/.vex-build"

# Run tests
tests=(
    "tests/ultra_minimal.vx"
    "tests/basic_test.vx"
)

for test in "${tests[@]}"; do
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "▶ Running: $test"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    timeout 30s "$VEX" run "$FS_DIR/$test" 2>&1 | grep -v "^🔧\|^📌\|^🔹\|^🔵\|^🟢\|^📋\|^🔄\|^\[DEBUG"
    
    if [ ${PIPESTATUS[0]} -eq 0 ]; then
        echo "✅ PASSED"
    else
        echo "❌ FAILED"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ FS Module Tests Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
