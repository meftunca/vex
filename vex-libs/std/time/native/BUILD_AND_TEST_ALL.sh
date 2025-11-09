#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "════════════════════════════════════════════════════════════"
echo "  vex_time - Complete Build and Test Suite"
echo "════════════════════════════════════════════════════════════"
echo ""

echo "🧹 Cleaning previous build..."
make clean

echo ""
echo "🔨 Building library..."
make

echo ""
echo "═══ 1. SWAR Optimization Test ═══"
make swar_bench
echo ""
echo "Running SWAR benchmark..."
./swar_bench

echo ""
echo "═══ 2. Stress Test ═══"
make stress_test
echo ""
echo "Running stress test..."
./stress_test

echo ""
echo "═══ 3. Layout Test (Go-style) ═══"
make layout_test
echo ""
echo "Running layout test..."
./layout_test

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  ✅ All tests completed successfully!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Summary:"
echo "  ✅ SWAR-optimized RFC3339 parsing"
echo "  ✅ Fast epoch calculation (Howard Hinnant)"
echo "  ✅ Optimized fractional second parsing"
echo "  ✅ Go-style layout support (Parse/Format)"
echo "  ✅ All standard Go layouts supported"
echo ""
echo "Performance Targets:"
echo "  Parse:  ~800-1000 ns/op (RFC3339)"
echo "  Format: <200 ns/op (RFC3339)"
echo "  Layout: ~2000-3000 ns/op (complex layouts)"
echo ""

