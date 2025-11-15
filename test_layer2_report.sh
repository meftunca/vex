#!/bin/bash
# Quick report on Layer 2 stdlib test status

for module in cmd env process io fs time strconv memory; do
    tests_dir="vex-libs/std/$module/tests"
    if [ -d "$tests_dir" ]; then
        test_count=$(find "$tests_dir" -name "*.test.vx" 2>/dev/null | wc -l)
        echo "$module: $test_count test file(s)"
    else
        echo "$module: NO TESTS DIR"
    fi
done
