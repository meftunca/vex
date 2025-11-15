#!/bin/bash
# Test Layer 2 stdlib modules
# Usage: ./test_layer2_modules.sh [module_name]

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
VEX_BIN="$PROJECT_ROOT/.cargo/target/debug/vex"

# Test a single module
test_module() {
    local module_name=$1
    local module_dir="$PROJECT_ROOT/vex-libs/std/$module_name"
    
    if [ ! -d "$module_dir/tests" ]; then
        echo "⚠️  $module_name: No tests"
        return 1
    fi
    
    echo "🧪 Testing $module_name..."
    
    # Find all test files
    local test_files=$(find "$module_dir/tests" -name "*.test.vx" 2>/dev/null)
    
    if [ -z "$test_files" ]; then
        echo "⚠️  $module_name: No *.test.vx files"
        return 1
    fi
    
    local passed=0
    local failed=0
    
    while IFS= read -r test_file; do
        local test_name=$(basename "$test_file" .test.vx)
        local temp_output="/tmp/vex_test_${module_name}_${test_name}"
        
        # Compile and run from project root
        if "$VEX_BIN" compile "$test_file" -o "$temp_output" 2>&1 | grep -q "Compilation successful" && \
           "$temp_output" >/dev/null 2>&1; then
            echo "  ✅ $test_name"
            ((passed++))
        else
            echo "  ❌ $test_name"
            ((failed++))
        fi
        rm -f "$temp_output"
    done <<< "$test_files"
    
    if [ $failed -eq 0 ]; then
        echo "✅ $module_name: $passed passed"
        return 0
    else
        echo "❌ $module_name: $passed passed, $failed failed"
        return 1
    fi
}

# Main
echo "=== Layer 2 Stdlib Module Tests ==="
echo ""

MODULES=(env process cmd io fs time)
passed=0
failed=0

if [ $# -gt 0 ]; then
    if test_module "$1"; then ((passed++)); else ((failed++)); fi
else
    for module in "${MODULES[@]}"; do
        if test_module "$module"; then ((passed++)); else ((failed++)); fi
        echo ""
    done
fi

echo "=== Summary ==="
echo "Passed: $passed"
echo "Failed: $failed"

[ $failed -eq 0 ] && exit 0 || exit 1
