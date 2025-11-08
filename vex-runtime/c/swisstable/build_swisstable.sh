#!/bin/bash
# Build optimized Swiss Tables library
# Usage: ./build_swisstable.sh [v1|v2|v3|all]

set -e

VERSION="${1:-v2}"  # Default to V2 (recommended)

echo "🔨 Building Swiss Tables ($VERSION)..."

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
INCLUDES="-I. -I.."

build_version() {
    local ver=$1
    echo ""
    echo "Building $ver..."
    
    case $ver in
        v1)
            clang $OPT_FLAGS $SIMD_FLAGS -c vex_swisstable.c $INCLUDES -o vex_swisstable.o
            ar rcs libvex_swisstable.a vex_swisstable.o
            clang $OPT_FLAGS $SIMD_FLAGS -o vex_swisstable_test vex_swisstable_test.c vex_swisstable.c $INCLUDES
            echo "✅ V1 built: libvex_swisstable.a"
            ;;
        v2)
            clang $OPT_FLAGS $SIMD_FLAGS -c vex_swisstable_v2.c $INCLUDES -o vex_swisstable_v2.o
            ar rcs libvex_swisstable_v2.a vex_swisstable_v2.o
            clang $OPT_FLAGS $SIMD_FLAGS -o vex_swisstable_test_v2 vex_swisstable_test.c vex_swisstable_v2.c $INCLUDES -DUSE_V2
            echo "✅ V2 built: libvex_swisstable_v2.a (RECOMMENDED ⭐)"
            ;;
        v3)
            clang $OPT_FLAGS $SIMD_FLAGS -c vex_swisstable_v3.c $INCLUDES -o vex_swisstable_v3.o
            ar rcs libvex_swisstable_v3.a vex_swisstable_v3.o
            echo "⚠️  V3 built (EXPERIMENTAL - HAS BUGS)"
            ;;
        *)
            echo "❌ Unknown version: $ver"
            exit 1
            ;;
    esac
}

if [[ "$VERSION" == "all" ]]; then
    build_version v1
    build_version v2
    build_version v3
    
    # Build comparison benchmarks
    echo ""
    echo "Building benchmarks..."
    clang $OPT_FLAGS $SIMD_FLAGS -o bench_v1_vs_v2 bench_v1_vs_v2.c vex_swisstable.c vex_swisstable_v2.c $INCLUDES
    clang $OPT_FLAGS $SIMD_FLAGS -o bench_ultimate bench_ultimate.c vex_swisstable.c vex_swisstable_v2.c vex_swisstable_v3.c $INCLUDES
    echo "✅ Benchmarks built"
else
    build_version $VERSION
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  ✅ Build Complete!"
echo "═══════════════════════════════════════════════════════════"
echo ""
if [[ "$VERSION" == "v2" ]] || [[ "$VERSION" == "all" ]]; then
    echo "🧪 Run tests: ./vex_swisstable_test_v2"
    echo "📊 Run bench: make bench"
fi
if [[ "$VERSION" == "v1" ]]; then
    echo "🧪 Run tests: ./vex_swisstable_test"
fi
echo ""
