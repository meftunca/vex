#!/bin/bash
# Test runner for FS module

set -e

VEX="$HOME/.cargo/target/debug/vex"

echo "🧪 Running FS Module Tests..."
echo "================================"

# Test 1: Basic operations
echo "Test 1: Basic file operations..."
$VEX run tests/basic_test.vx

echo ""
echo "✅ All FS module tests passed!"
