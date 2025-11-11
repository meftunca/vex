#!/bin/bash
# Test runner for FS module

set -e

VEX="${VEX_BIN:-$HOME/.cargo/target/debug/vex}"

echo "🧪 Running FS Module Tests..."
echo "================================"

# Test 1: Basic operations
echo "Test 1: Basic file operations..."
timeout 30s "$VEX" run tests/basic_test.vx

# Test 2: Demo
echo ""
echo "Test 2: Running demo..."
timeout 20s "$VEX" run tests/demo.vx

echo ""
echo "✅ All FS module tests passed!"
