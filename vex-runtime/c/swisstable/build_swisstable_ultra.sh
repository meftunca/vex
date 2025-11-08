#!/bin/bash
# Ultra-optimized SwissTable build
# Target: Beat Google Abseil

set -e

echo "🚀 Building SwissTable with ULTRA optimizations..."
echo ""

# Detect architecture
ARCH=$(uname -m)
SIMD_FLAGS=""

if [[ "$ARCH" == "x86_64" ]]; then
    SIMD_FLAGS="-mavx2 -msse4.2 -mbmi2 -mpopcnt"
    echo "📍 Platform: x86-64 (AVX2 + BMI2)"
elif [[ "$ARCH" == "arm64" ]] || [[ "$ARCH" == "aarch64" ]]; then
    SIMD_FLAGS="-mcpu=apple-m1"  # Or -mcpu=native
    echo "📍 Platform: ARM64 (Apple Silicon / NEON)"
fi

# ULTRA aggressive optimization flags (Clang-compatible)
ULTRA_OPT="-O3 -march=native -mtune=native \
    -flto=thin \
    -fno-plt \
    -fno-semantic-interposition \
    -funroll-loops \
    -ffast-math \
    -fomit-frame-pointer \
    -finline-functions \
    -mllvm -inline-threshold=2000 \
    -fmerge-all-constants \
    -ftree-vectorize \
    -fvectorize \
    -fslp-vectorize \
    -fno-stack-protector \
    -momit-leaf-frame-pointer \
    -fstrict-aliasing"

# Profile-guided optimization (PGO) support
USE_PGO="${USE_PGO:-0}"

echo "🔧 Compiler: $(clang --version | head -n1)"
echo "🎯 Optimization level: ULTRA"
echo "📊 PGO: $([ "$USE_PGO" == "1" ] && echo "ENABLED" || echo "DISABLED")"
echo ""

if [ "$USE_PGO" == "1" ]; then
    echo "═══ Step 1: Profile Generation Build ═══"
    clang $ULTRA_OPT $SIMD_FLAGS \
        -fprofile-generate \
        -o vex_swisstable_bench_pgo \
        vex_swisstable_bench.c vex_swisstable.c -I.
    
    echo "═══ Step 2: Running profiling workload ═══"
    ./vex_swisstable_bench_pgo > /dev/null 2>&1
    
    echo "═══ Step 3: Profile-Guided Optimized Build ═══"
    clang $ULTRA_OPT $SIMD_FLAGS \
        -fprofile-use \
        -fprofile-correction \
        -o vex_swisstable_bench_ultra \
        vex_swisstable_bench.c vex_swisstable.c -I.
    
    rm -f vex_swisstable_bench_pgo default.profdata
    echo "✅ PGO build complete"
else
    echo "═══ Standard Ultra Build ═══"
    clang $ULTRA_OPT $SIMD_FLAGS \
        -o vex_swisstable_bench_ultra \
        vex_swisstable_bench.c vex_swisstable.c -I.
    
    echo "✅ Ultra build complete"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  🎉 Build Complete!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Run benchmark:"
echo "  ./vex_swisstable_bench_ultra"
echo ""
echo "Compare with standard build:"
echo "  ./vex_swisstable_bench"
echo ""
echo "Enable PGO for even more performance:"
echo "  USE_PGO=1 ./build_swisstable_ultra.sh"
echo ""

