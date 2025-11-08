#!/bin/bash
# Build script for Vex Runtime Library
# Compiles C code to LLVM IR for zero-overhead integration with Vex compiler

set -e  # Exit on error

# Configuration
SRC_DIR="$(dirname "$0")"
BUILD_DIR="$SRC_DIR/build"
LLVM_IR_DIR="$SRC_DIR/llvm-ir"
OPTIMIZATION="-O3"
# Critical flags for zero-overhead:
# -emit-llvm: Generate LLVM IR instead of machine code
# -fno-discard-value-names: Keep readable names in IR
# -fPIC: Position independent code
# -flto: Enable link-time optimization
CLANG_FLAGS="-emit-llvm -fno-discard-value-names -fPIC -flto"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Building Vex Runtime Library (LLVM IR) ===${NC}"

# Create build directories
mkdir -p "$BUILD_DIR"
mkdir -p "$LLVM_IR_DIR"

# Source files
SOURCES=(
    "vex_string.c"
    "vex_memory.c"
    "vex_alloc.c"
    "vex_io.c"
    "vex_array.c"
    "vex_error.c"
    "swisstable/vex_swisstable.c"
    "vex_file.c"
    "vex_mmap.c"
    "vex_time.c"
    "vex_path.c"
    "vex_strconv.c"
    "vex_url.c"
    "vex_cpu.c"
    # Builtin Types - Phase 0 (Nov 6, 2025)
    "vex_vec.c"
    "vex_option.c"
    "vex_result.c"
    "vex_string_type.c"
    "vex_range.c"
    "vex_slice.c"
    "vex_box.c"
    "vex_set.c"
    # vex_tuple.c is documentation only, no compilation needed
)

# Detect architecture and set SIMD flags
ARCH=$(uname -m)
SIMD_FLAGS=""

if [[ "$ARCH" == "x86_64" ]]; then
    # x86_64: Enable AVX2 + SSE instructions
    SIMD_FLAGS="-mavx2 -msse4.2 -mpopcnt -mbmi2"
    echo -e "${BLUE}Architecture: x86_64 (AVX2 + SSE4.2 enabled)${NC}"
elif [[ "$ARCH" == "arm64" ]] || [[ "$ARCH" == "aarch64" ]]; then
    # ARM64: NEON is enabled by default, no extra flags needed
    SIMD_FLAGS=""
    echo -e "${BLUE}Architecture: ARM64 (NEON enabled by default)${NC}"
else
    echo -e "${BLUE}Architecture: $ARCH (no SIMD optimization)${NC}"
fi

# Step 1: Compile each C file to LLVM IR (.ll)
echo -e "${GREEN}[1/5] Compiling C to LLVM IR (.ll)...${NC}"
for src in "${SOURCES[@]}"; do
    # Extract just the filename without path and extension
    base_name=$(basename "${src%.c}")
    echo "  - $src → ${base_name}.ll"
    clang -S $OPTIMIZATION $CLANG_FLAGS $SIMD_FLAGS \
          -I"$SRC_DIR" \
          "$SRC_DIR/$src" \
          -o "$LLVM_IR_DIR/${base_name}.ll"
done

# Step 2: Link all LLVM IR files into one bitcode file
echo -e "${GREEN}[2/5] Linking LLVM IR modules...${NC}"
llvm-link "$LLVM_IR_DIR"/*.ll -o "$BUILD_DIR/vex_runtime.bc"

# Step 3: Convert to readable LLVM IR (for debugging/inspection)
echo -e "${GREEN}[3/5] Generating readable LLVM IR...${NC}"
llvm-dis "$BUILD_DIR/vex_runtime.bc" -o "$BUILD_DIR/vex_runtime.ll"

# Step 4: Optimize LLVM IR (zero-overhead inlining)
echo -e "${GREEN}[4/5] Optimizing LLVM IR (inlining, dead code elimination)...${NC}"
opt -O3 \
    "$BUILD_DIR/vex_runtime.bc" \
    -o "$BUILD_DIR/vex_runtime_opt.bc"

# Convert optimized bitcode to readable IR
llvm-dis "$BUILD_DIR/vex_runtime_opt.bc" -o "$BUILD_DIR/vex_runtime_opt.ll"

# Step 5: Create static library (for native linking)
echo -e "${GREEN}[5/5] Creating static library...${NC}"
llc -filetype=obj "$BUILD_DIR/vex_runtime_opt.bc" -o "$BUILD_DIR/vex_runtime.o"
ar rcs "$BUILD_DIR/libvex_runtime.a" "$BUILD_DIR/vex_runtime.o"

echo ""
echo -e "${GREEN}✅ Build complete!${NC}"
echo ""
echo -e "${YELLOW}Output files:${NC}"
echo "  📁 Individual IR: $LLVM_IR_DIR/*.ll"
echo "  📦 Linked IR:     $BUILD_DIR/vex_runtime.ll"
echo "  🚀 Optimized IR:  $BUILD_DIR/vex_runtime_opt.ll"
echo "  📚 Static lib:    $BUILD_DIR/libvex_runtime.a"
echo ""
echo -e "${YELLOW}Integration with Vex:${NC}"
echo "  1. Use vex_runtime_opt.ll for LLVM IR linking (zero-overhead)"
echo "  2. Or link libvex_runtime.a for native compilation"
echo ""
echo -e "${BLUE}Next step: Update vex-compiler to link vex_runtime_opt.ll${NC}"

echo -e "${BLUE}=== Build Complete ===${NC}"
echo "Outputs:"
echo "  - LLVM IR:       $BUILD_DIR/vex_runtime.ll"
echo "  - Bitcode:       $BUILD_DIR/vex_runtime.bc"
echo "  - Static lib:    $BUILD_DIR/libvex_runtime.a"
echo ""
echo "Library size: $(du -h "$BUILD_DIR/libvex_runtime.a" | cut -f1)"
