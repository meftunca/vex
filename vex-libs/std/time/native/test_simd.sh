#!/bin/bash
set -e

echo "═══════════════════════════════════════════════════════════"
echo "  vex_time SIMD Test Suite"
echo "═══════════════════════════════════════════════════════════"
echo ""

cd "$(dirname "$0")"

# Clean and build with native optimizations
echo "Building with native CPU optimizations..."
make clean > /dev/null 2>&1
make native

# Run SIMD benchmark
echo ""
echo "Running SIMD benchmarks..."
echo "───────────────────────────────────────────────────────────"
./simd_bench

# Run stress test to verify correctness
echo ""
echo "Running correctness test..."
echo "───────────────────────────────────────────────────────────"
./stress_test | grep "ALL TESTS PASSED"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  ✅ SIMD Implementation Verified!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "💡 Try different SIMD levels:"
echo "   make avx2     # Force AVX2"
echo "   make avx512   # Force AVX-512 (if supported)"
echo ""

