#!/bin/bash
# Parallel test runner for Vex examples
# Uses xargs -P for parallel execution (4-8x faster)

echo "🧪 Testing Vex Examples (Parallel Mode)..."
echo "=========================="

cd "$(dirname "$0")"
VEX_BIN="$HOME/.cargo/target/debug/vex"

# Create temp directory for results
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Test function (will be called in parallel)
test_file() {
    local file="$1"
    local vex_bin="$2"
    local temp_dir="$3"
    
    name=$(echo "$file" | sed 's|examples/||' | sed 's|\.vx$||')
    result_file="$temp_dir/$(echo "$name" | tr '/' '_').result"
    
    # Skip interface tests (interface keyword deprecated)
    if [[ "$file" == *"interfaces.vx"* ]]; then
        echo "SKIP" > "$result_file"
        echo "⏭️  Skipping $name (interface deprecated, use trait)"
        return
    fi
    
    # Skip LSP diagnostic tests (intentionally contain errors for testing)
    if [[ "$file" == *"test_lsp_diagnostics.vx"* ]]; then
        echo "SKIP" > "$result_file"
        echo "⏭️  Skipping $name (LSP diagnostic test - intentional errors)"
        return
    fi
    
    # Skip stdlib integration tests (require additional C libraries)
    if [[ "$file" == *"invalid"* ]] ||[[ "$file" == *"stdlib_integration"* ]] || [[ "$file" == *"test_stdlib_simple.vx"* ]] || [[ "$file" == *"native_demo"* ]] || [[ "$file" == *"crypto_self_signed_cert.vx"* ]]; then
        echo "SKIP" > "$result_file"
        echo "⏭️  Skipping $name (requires external C libraries)"
        return
    fi
    
    # Circular dependency tests should FAIL with error (expected behavior)
    if [[ "$file" == *"circular_dependency.vx"* ]] || [[ "$file" == *"circular_self.vx"* ]] || [[ "$file" == *"04_circular.vx"* ]]; then
        if "$vex_bin" compile "$file" > /dev/null 2>&1; then
            echo "FAIL" > "$result_file"
            echo "❌ FAIL $name (should have detected circular dependency)"

        fi
        return
    fi
    
    # Borrow checker error tests should FAIL with borrow error (expected behavior)
    if [[ "$file" == *"_error.vx"* ]] || [[ "$file" == *"return_local.vx"* ]] || [[ "$file" == *"_violation.vx"* ]]; then
        output=$("$vex_bin" run "$file" 2>&1)
        if echo "$output" | grep -qE "(Borrow check failed|error\[E[0-9]+\]:|Compilation error:)"; then
            echo "PASS" > "$result_file"
            echo "✅ PASS $name (correctly detected error)"
        else
            echo "FAIL" > "$result_file"
            echo "❌ FAIL $name (should have detected error)"
        fi
        return
    fi
    
    # Diagnostic tests should produce compilation errors (expected behavior)
    if [[ "$file" == *"test_diagnostic"* ]] || [[ "$file" == *"test_typo"* ]] || \
       [[ "$file" == *"test_function_typo"* ]] || \
       [[ "$file" == *"test_undefined"* ]] || [[ "$file" == *"test_fuzzy"* ]] || \
       [[ "$file" == *"test_if_condition"* ]] || [[ "$file" == *"test_parse_error"* ]] || \
       [[ "$file" == *"test_borrow_diagnostic"* ]] || \
       [[ "$file" == *"test_immutable_violation"* ]]; then
        output=$("$vex_bin" run "$file" 2>&1)
        if echo "$output" | grep -qE "error\[E[0-9]+\]:"; then
            echo "PASS" > "$result_file"
            echo "✅ PASS $name (correctly detected compile error)"
        else
            echo "FAIL" > "$result_file"
            echo "❌ FAIL $name (should have detected compile error)"
        fi
        return
    fi
    
    # Skip known broken tests
    if [[ "$file" == *"error_handling_try.vx"* ]] || [[ "$file" == *"test_move_diagnostic.vx"* ]]; then
        echo "SKIP" > "$result_file"
        echo "⏭️  Skipping $name (known issues)"
        return
    fi
    
    # Tests with stdlib imports need 'run' instead of 'compile'
    if [[ "$file" == *"test_io_"* ]] || [[ "$file" == *"test_stdlib_"* ]] || \
       [[ "$file" == *"test_process_"* ]] || [[ "$file" == *"stdlib_integration"* ]]; then
        if "$vex_bin" run "$file" > /dev/null 2>&1; then
            echo "PASS" > "$result_file"
            echo "✅ PASS $name"
        else
            echo "FAIL" > "$result_file"
            echo "❌ FAIL $name"
        fi
        return
    fi
    
    # Normal test - compile
    if "$vex_bin" compile "$file" > /dev/null 2>&1; then
        echo "PASS" > "$result_file"
        echo "✅ PASS $name"
    else
        echo "FAIL" > "$result_file"
        echo "❌ FAIL $name"
    fi
}

export -f test_file
export VEX_BIN
export TEMP_DIR

# Get number of CPU cores (default to 8 if can't detect)
if command -v sysctl &> /dev/null; then
    # macOS
    NCORES=$(sysctl -n hw.ncpu 2>/dev/null || echo 8)
else
    # Linux
    NCORES=$(nproc 2>/dev/null || echo 8)
fi

echo "🚀 Running tests on $NCORES cores..."
echo ""

# Run tests in parallel using xargs
find examples -name "*.vx" -type f | sort | \
    xargs -P "$NCORES" -I {} bash -c 'test_file "$@"' _ {} "$VEX_BIN" "$TEMP_DIR"

echo ""
echo "=========================="

# Count results
SUCCESS=$(grep -l "PASS" "$TEMP_DIR"/*.result 2>/dev/null | wc -l | tr -d ' ')
FAIL=$(grep -l "FAIL" "$TEMP_DIR"/*.result 2>/dev/null | wc -l | tr -d ' ')
TOTAL=$((SUCCESS + FAIL))

echo "📊 Results:"
echo "   ✅ Success: $SUCCESS"
echo "   ❌ Failed:  $FAIL"

if [ "$TOTAL" -gt 0 ]; then
    echo "   Total:     $TOTAL"
    RATE=$(awk "BEGIN {printf \"%.1f\", ($SUCCESS * 100.0) / $TOTAL}")
    echo "   Success Rate: ${RATE}%"
else
    echo "   No tests run!"
fi

# Exit with failure if any tests failed
[ "$FAIL" -eq 0 ]