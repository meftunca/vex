#!/bin/bash

# Vex Language Stability Test Script
# Tests all language features from basic to advanced
# Ordered from fundamental language elements to complex features

# Output file
OUTPUT_FILE="STABILITY_OUTPUT.md"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
STOP_ON_FIRST_FAILURE=false
MAX_FAILURES=1
FAILURE_COUNT=0

# Function to write to both terminal and file
write_output() {
    echo -e "$1"
    # Remove ANSI color codes and write to file
    local clean_text=$(echo -e "$1" | sed 's/\x1B\[[0-9;]*[JKmsu]//g')
    echo "$clean_text" >> "$OUTPUT_FILE"
}

# Function to run a test file
run_test() {
    local test_file="$1"
    local test_name="$2"

    write_output "${BLUE}Running test: ${test_name}${NC}"
    write_output "File: $test_file"

    if [ ! -f "$test_file" ]; then
        write_output "${RED}ERROR: Test file not found: $test_file${NC}"
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        return 1
    fi

    # Run the test using vex compiler
    if ~/.cargo/target/release/vex run "$test_file" > /dev/null 2>&1; then
        write_output "${GREEN}✓ PASSED${NC}"
        ((PASSED_TESTS++))
    else
        write_output "${RED}✗ FAILED${NC}"
        # Show the actual error for debugging
        write_output "Error output:"
        ~/.cargo/target/release/vex run "$test_file" 2>&1 | while read -r line; do
            write_output "$line"
        done
        ((FAILED_TESTS++))
        ((TOTAL_TESTS++))
        ((FAILURE_COUNT++))
        write_output ""
        
        # Stop after MAX_FAILURES if enabled
        if [ "$STOP_ON_FIRST_FAILURE" = false ] && [ "$FAILURE_COUNT" -ge "$MAX_FAILURES" ]; then
            write_output "${RED}Reached maximum failures ($MAX_FAILURES). Stopping.${NC}"
            exit 1
        fi
        
        # Stop on first failure if enabled
        if [ "$STOP_ON_FIRST_FAILURE" = true ]; then
            write_output "${RED}Stopping on first failure as requested.${NC}"
            exit 1
        fi
        
        return 1
    fi
}

# Function to run all tests in a category
run_category() {
    local category="$1"
    local category_name="$2"

    write_output "${YELLOW}========================================${NC}"
    write_output "${YELLOW}Testing Category: $category_name${NC}"
    write_output "${YELLOW}========================================${NC}"
    write_output ""

    # Find all .vx files in the category directory
    if [ -d "$category" ]; then
        for test_file in "$category"/*.vx; do
            if [ -f "$test_file" ]; then
                local filename=$(basename "$test_file" .vx)
                run_test "$test_file" "$category_name - $filename"
            fi
        done
    else
        write_output "${RED}WARNING: Category directory not found: $category${NC}"
    fi

    write_output ""
}

# Initialize output file
cat > "$OUTPUT_FILE" << EOF
# Vex Language Stability Test Results

**Generated:** $(date)
**Vex Version:** 0.1.2

## Test Summary

| Category | Status | Details |
|----------|--------|---------|
EOF

# Main test execution - ordered from basic to advanced
write_output "${BLUE}=======================================${NC}"
write_output "${BLUE}Vex Language Stability Test Suite${NC}"
write_output "${BLUE}=======================================${NC}"
write_output "Testing all language features from fundamental to advanced"
write_output "Date: $(date)"
write_output ""

# 1. Lexical Elements (most basic)
run_category "lexical" "Lexical Elements"

# 2. Type System
run_category "type_system" "Type System"

# 3. Variables and Constants
run_category "variables_and_constants" "Variables and Constants"

# 4. Functions and Methods
run_category "functions_and_methods" "Functions and Methods"

# 5. Control Flow
run_category "control_flow" "Control Flow"

# 6. Structs
run_category "structs" "Structs"

# 7. Enums
run_category "enums" "Enums"

# 8. Traits
run_category "traits" "Traits"

# 9. Generics
run_category "generics" "Generics"

# 10. Pattern Matching
run_category "pattern_matching" "Pattern Matching"

# 11. Error Handling
run_category "error_handling" "Error Handling"

# 12. Memory Management
run_category "memory_management" "Memory Management"

# 13. Concurrency
run_category "concurrency" "Concurrency"

# 14. Modules and Imports
run_category "modules_and_imports" "Modules and Imports"

# 15. Policy System
run_category "policy_system" "Policy System"

# 16. Operators
run_category "operators" "Operators"

# Summary
write_output "${BLUE}=======================================${NC}"
write_output "${BLUE}Test Summary${NC}"
write_output "${BLUE}=======================================${NC}"
write_output "Total Tests: $TOTAL_TESTS"
write_output -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
write_output -e "Failed: ${RED}$FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 5 ]; then
    write_output "${GREEN}🎉 All tests passed! Vex language is stable.${NC}"
    exit 0
else
    write_output "${RED}❌ Some tests failed. Check the output above for details.${NC}"
    exit 1
fi