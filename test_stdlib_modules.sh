#!/bin/bash
# Stdlib Module Test Runner
# Her modülü ayrı ayrı test eder

set -e

VEX_BIN="$HOME/.cargo/target/debug/vex"
STDLIB_DIR="vex-libs/std"
FAILED=0
PASSED=0

echo "╔════════════════════════════════════════╗"
echo "║  Vex Standard Library Test Runner     ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Test bir modülü
test_module() {
    local module=$1
    local test_file="$STDLIB_DIR/$module/tests/basic_test.vx"
    
    if [ ! -f "$test_file" ]; then
        echo "⚠️  $module: No test file found"
        return
    fi
    
    echo "Testing $module..."
    if $VEX_BIN run "$test_file" > /dev/null 2>&1; then
        echo "✅ $module: PASSED"
        PASSED=$((PASSED + 1))
    else
        echo "❌ $module: FAILED"
        FAILED=$((FAILED + 1))
    fi
}

# FFI ile çalışan modüller
echo "=== Testing FFI Modules ==="
test_module "io"
test_module "math"
test_module "fs"
test_module "env"
test_module "process"

echo ""
echo "╔════════════════════════════════════════╗"
echo "║  Test Results                          ║"
echo "╚════════════════════════════════════════╝"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "✅ All stdlib modules working!"
    exit 0
else
    echo "❌ Some modules failed"
    exit 1
fi
