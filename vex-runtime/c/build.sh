#!/bin/bash
# Build script for Vex Runtime Library
# Compiles C code to LLVM IR and static library

set -e  # Exit on error

# Configuration
SRC_DIR="$(dirname "$0")"
BUILD_DIR="$SRC_DIR/build"
OPTIMIZATION="-O3"
CLANG_FLAGS="-fno-discard-value-names -fPIC"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Building Vex Runtime Library ===${NC}"

# Create build directory
mkdir -p "$BUILD_DIR"

# Source files
SOURCES=(
    "vex_string.c"
    "vex_memory.c"
    "vex_alloc.c"
    "vex_io.c"
    "vex_array.c"
    "vex_error.c"
    "vex_swisstable.c"
    "vex_file.c"
    "vex_mmap.c"
    "vex_time.c"
    "vex_path.c"
    "vex_strconv.c"
    "vex_url.c"
    "vex_cpu.c"
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

# Step 1: Compile each C file to LLVM IR
echo -e "${GREEN}[1/4] Compiling C to LLVM IR...${NC}"
for src in "${SOURCES[@]}"; do
    echo "  - $src"
    clang -S -emit-llvm $OPTIMIZATION $CLANG_FLAGS $SIMD_FLAGS \
          -I"$SRC_DIR" \
          "$SRC_DIR/$src" \
          -o "$BUILD_DIR/${src%.c}.ll"
done

# Step 2: Link all LLVM IR files into one
echo -e "${GREEN}[2/4] Linking LLVM IR modules...${NC}"
llvm-link "$BUILD_DIR"/*.ll -o "$BUILD_DIR/vex_runtime.bc"

# Step 3: Convert to readable LLVM IR
echo -e "${GREEN}[3/4] Generating readable LLVM IR...${NC}"
llvm-dis "$BUILD_DIR/vex_runtime.bc" -o "$BUILD_DIR/vex_runtime.ll"

# Step 4: Compile to object file and create static library
echo -e "${GREEN}[4/4] Creating static library...${NC}"
llc -filetype=obj "$BUILD_DIR/vex_runtime.bc" -o "$BUILD_DIR/vex_runtime.o"
ar rcs "$BUILD_DIR/libvex_runtime.a" "$BUILD_DIR/vex_runtime.o"

echo -e "${BLUE}=== Build Complete ===${NC}"
echo "Outputs:"
echo "  - LLVM IR:       $BUILD_DIR/vex_runtime.ll"
echo "  - Bitcode:       $BUILD_DIR/vex_runtime.bc"
echo "  - Static lib:    $BUILD_DIR/libvex_runtime.a"
echo ""
echo "Library size: $(du -h "$BUILD_DIR/libvex_runtime.a" | cut -f1)"
