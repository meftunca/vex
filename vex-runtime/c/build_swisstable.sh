#!/bin/bash
# Build optimized Swiss Tables library

set -e

echo "🔨 Building Swiss Tables with maximum optimization..."

# Detect architecture
ARCH=$(uname -m)
SIMD_FLAGS=""

if [[ "$ARCH" == "x86_64" ]]; then
    SIMD_FLAGS="-mavx2 -msse4.2"
    echo "📍 Platform: x86-64 (AVX2 + SSE4.2)"
elif [[ "$ARCH" == "arm64" ]] || [[ "$ARCH" == "aarch64" ]]; then
    SIMD_FLAGS=""  # NEON enabled by default
    echo "📍 Platform: ARM64 (NEON)"
fi

# Aggressive optimization flags
OPT_FLAGS="-O3 -march=native -flto -funroll-loops -ffast-math"

# Build static library
clang $OPT_FLAGS $SIMD_FLAGS -c vex_swisstable.c -I. -o vex_swisstable.o
ar rcs libvex_swisstable.a vex_swisstable.o

# Build test
clang $OPT_FLAGS $SIMD_FLAGS -o vex_swisstable_test vex_swisstable_test.c vex_swisstable.c -I.

echo "✅ Build complete: libvex_swisstable.a"
echo "🧪 Run tests: ./vex_swisstable_test"
