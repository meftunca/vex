#!/bin/bash
# Vex Standard Library Syntax Validation
# Checks all stdlib packages for deprecated v0.9 syntax

set -e

STDLIB_ROOT="vex-libs/std"
ERRORS_FOUND=0

echo "🔍 Validating Vex Standard Library Syntax (v0.9)"
echo "=================================================="
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

validate_file() {
    local file=$1
    local errors=0
    
    # Check for deprecated '->' (should use ':' for return types)
    if grep -n '\->' "$file" | grep -v '//' | grep -v '^\s*#' > /dev/null 2>&1; then
        echo -e "${RED}❌ Found '->' in $file${NC}"
        grep -n '\->' "$file" | grep -v '//' | grep -v '^\s*#' | head -3
        errors=$((errors + 1))
    fi
    
    # Check for deprecated '::' (should use '.' for member access)
    if grep -n '::' "$file" | grep -v '//' | grep -v '^\s*#' | grep -v 'extern "C"' > /dev/null 2>&1; then
        echo -e "${RED}❌ Found '::' in $file${NC}"
        grep -n '::' "$file" | grep -v '//' | grep -v '^\s*#' | grep -v 'extern "C"' | head -3
        errors=$((errors + 1))
    fi
    
    # Check for deprecated 'mut ' (should use '!' suffix)
    if grep -n '\bmut\s' "$file" | grep -v '//' | grep -v '^\s*#' > /dev/null 2>&1; then
        echo -e "${RED}❌ Found 'mut ' in $file${NC}"
        grep -n '\bmut\s' "$file" | grep -v '//' | grep -v '^\s*#' | head -3
        errors=$((errors + 1))
    fi
    
    # Check for deprecated 'interface' (should use 'trait')
    if grep -n '\binterface\s' "$file" | grep -v '//' | grep -v '^\s*#' > /dev/null 2>&1; then
        echo -e "${RED}❌ Found 'interface' in $file${NC}"
        grep -n '\binterface\s' "$file" | grep -v '//' | grep -v '^\s*#' | head -3
        errors=$((errors + 1))
    fi
    
    # Check for deprecated ':=' (should use 'let')
    if grep -n ':=' "$file" | grep -v '//' | grep -v '^\s*#' > /dev/null 2>&1; then
        echo -e "${RED}❌ Found ':=' in $file${NC}"
        grep -n ':=' "$file" | grep -v '//' | grep -v '^\s*#' | head -3
        errors=$((errors + 1))
    fi
    
    if [ $errors -eq 0 ]; then
        echo -e "${GREEN}✅ $file${NC}"
    else
        echo ""
    fi
    
    return $errors
}

# Validate all .vx files in stdlib
for module_dir in "$STDLIB_ROOT"/*/; do
    module_name=$(basename "$module_dir")
    echo "📦 Checking module: $module_name"
    
    if [ ! -d "$module_dir/src" ]; then
        echo -e "${YELLOW}⚠️  No src/ directory in $module_name${NC}"
        echo ""
        continue
    fi
    
    for vx_file in "$module_dir"/src/*.vx; do
        if [ -f "$vx_file" ]; then
            if ! validate_file "$vx_file"; then
                ERRORS_FOUND=$((ERRORS_FOUND + 1))
            fi
        fi
    done
    
    echo ""
done

echo "=================================================="
if [ $ERRORS_FOUND -eq 0 ]; then
    echo -e "${GREEN}✅ All stdlib modules passed syntax validation!${NC}"
    exit 0
else
    echo -e "${RED}❌ Found $ERRORS_FOUND file(s) with syntax errors${NC}"
    echo ""
    echo "Fix guide:"
    echo "  -> should be :   (return types)"
    echo "  :: should be .   (member access)"
    echo "  mut should be !  (mutable suffix)"
    echo "  interface → trait"
    echo "  := → let"
    exit 1
fi
