#!/bin/bash
set -e

echo "Building vex_time..."
cd "$(dirname "$0")"

# Clean
make clean

# Build
make

# Build stress test
make stress_test

# Test
echo ""
echo "Running full_demo..."
./examples/full_demo

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "Running stress test..."
echo "═══════════════════════════════════════════════════════════"
./stress_test

echo ""
echo "✅ vex_time build and all tests complete!"

