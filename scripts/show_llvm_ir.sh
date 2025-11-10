#!/usr/bin/env bash
# Generate LLVM IR for a Vex file

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <vex_file>"
    exit 1
fi

VEX_FILE="$1"
BASE_NAME=$(basename "$VEX_FILE" .vx)
OBJ_FILE="/tmp/${BASE_NAME}.o"
IR_FILE="/tmp/${BASE_NAME}.ll"

echo "🔧 Compiling to object file..."
~/.cargo/target/debug/vex compile "$VEX_FILE" --output="$OBJ_FILE"

echo "🔍 Extracting LLVM IR with llvm-dis..."
llvm-dis "$OBJ_FILE" -o "$IR_FILE" 2>/dev/null || {
    echo "⚠️  llvm-dis failed, trying with clang..."
    clang -S -emit-llvm -o "$IR_FILE" "$OBJ_FILE" 2>/dev/null || {
        echo "❌ Could not extract LLVM IR"
        exit 1
    }
}

echo "✅ LLVM IR written to: $IR_FILE"
cat "$IR_FILE"
